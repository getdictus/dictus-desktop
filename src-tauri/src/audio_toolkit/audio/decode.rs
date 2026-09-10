//! Decoding of arbitrary user-supplied media files into the mono 16 kHz
//! `Vec<f32>` the transcription engines expect.
//!
//! This is deliberately separate from `recorder.rs`: microphone capture owns a
//! live cpal stream and already produces the right shape, while imported files
//! arrive in any container, any sample rate and any channel count. `read_wav_samples`
//! in `utils.rs` stays what it is — a fast path for the app's own managed
//! recordings, which are always 16-bit mono 16 kHz.

use super::opus_decoder::OpusDecoder;
use anyhow::Result;
use log::debug;
use once_cell::sync::Lazy;
use rubato::{FftFixedIn, Resampler};
use std::fs::File;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CodecParameters, CodecRegistry, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Sample rate every transcription engine in Dictus expects.
pub const TARGET_SAMPLE_RATE: u32 = 16_000;

/// symphonia's built-in codecs plus our libopus-backed Opus decoder.
///
/// The default registry has no Opus decoder, so `get_codecs()` would reject
/// every Telegram and WhatsApp voice note despite the Ogg demuxer reading them
/// perfectly well.
static CODECS: Lazy<CodecRegistry> = Lazy::new(|| {
    let mut registry = CodecRegistry::new();
    symphonia::default::register_enabled_codecs(&mut registry);
    registry.register_all::<OpusDecoder>();
    registry
});

/// Chunk size fed to the resampler. Matches the realtime `FrameResampler`.
const RESAMPLER_CHUNK_SIZE: usize = 1024;

/// File extensions the import pipeline advertises. Kept in sync with the
/// frontend picker filter (`SUPPORTED_AUDIO_EXTENSIONS` in
/// `src/components/file-transcription/supportedFormats.ts`).
///
/// Deliberately excludes video containers — video import is out of scope for
/// this workspace. `.opus` is a bare Ogg Opus stream, which the Ogg reader and
/// our libopus-backed decoder handle like any other `.ogg`.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "wav", "wave", "mp3", "m4a", "m4b", "aac", "flac", "ogg", "oga", "opus",
];

/// What went wrong while ingesting an imported file.
///
/// Kept coarse on purpose: each variant maps to one distinct user-facing
/// message in the frontend (`fileTranscription.errors.*`).
#[derive(Debug)]
pub enum DecodeError {
    /// The extension isn't one we advertise, or the container couldn't be read.
    UnsupportedFormat,
    /// The container was read fine, but we have no decoder for the codec inside
    /// it. Carries a human-readable codec name — telling someone their Opus
    /// `.ogg` is an "unsupported format" sends them hunting for a problem with
    /// the file instead of the codec.
    UnsupportedCodec(String),
    /// The file exists but could not be opened or read.
    Io(String),
    /// A decoder was found but the stream is damaged or truncated.
    Corrupt(String),
    /// Decoding succeeded but produced no audio at all.
    Empty,
    /// The caller flipped the cancellation flag.
    Cancelled,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedFormat => write!(f, "Unsupported audio format"),
            Self::UnsupportedCodec(name) => write!(f, "Unsupported audio codec: {}", name),
            Self::Io(detail) => write!(f, "Could not read the file: {}", detail),
            Self::Corrupt(detail) => write!(f, "Damaged or unreadable audio: {}", detail),
            Self::Empty => write!(f, "The file contains no audio"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Header-level facts about an imported file, cheap enough to read before the
/// user has committed to transcribing it.
#[derive(Debug, Clone)]
pub struct AudioFileInfo {
    /// `None` when the container doesn't declare it up front.
    pub sample_rate: Option<u32>,
    /// `None` when the container doesn't declare a channel layout. MP4 does not
    /// for AAC — the layout only appears on the first decoded packet — so this
    /// being absent says nothing about whether the file will decode.
    pub channels: Option<u16>,
    /// `None` when the container doesn't declare a frame count (some streamed
    /// MP3s and OGGs). The UI then omits the duration instead of guessing.
    pub duration_ms: Option<u64>,
}

/// True when `path` has one of the extensions we advertise.
///
/// Extension checking is a pre-filter, not the real gate — `decode_to_mono_16k`
/// still rejects anything symphonia can't build a decoder for.
pub fn has_supported_extension<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext = ext.to_ascii_lowercase();
            SUPPORTED_EXTENSIONS.contains(&ext.as_str())
        })
        .unwrap_or(false)
}

/// Read container/codec metadata without decoding any audio.
pub fn probe_audio_file<P: AsRef<Path>>(path: P) -> Result<AudioFileInfo, DecodeError> {
    let (format, track_id) = open_format(path.as_ref())?;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.id == track_id)
        .ok_or(DecodeError::UnsupportedFormat)?;

    // Everything here is advisory: containers are free to leave these out, and
    // MP4 routinely does. Probing is for showing the user what they picked, so
    // a missing field means "don't display it", never "reject the file".
    let sample_rate = track.codec_params.sample_rate.filter(|rate| *rate > 0);
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count() as u16)
        .filter(|count| *count > 0);

    let duration_ms = match (track.codec_params.n_frames, sample_rate) {
        (Some(frames), Some(rate)) => Some((frames as f64 * 1000.0 / rate as f64).round() as u64),
        _ => None,
    };

    Ok(AudioFileInfo {
        sample_rate,
        channels,
        duration_ms,
    })
}

/// Progress reported while decoding, as a fraction in `0.0..=1.0`.
///
/// `None` means the container didn't declare a total frame count, so the caller
/// must show an indeterminate state rather than invent a percentage.
pub type DecodeProgress<'a> = dyn FnMut(Option<f64>) + Send + 'a;

/// Decode `path` into mono 16 kHz `f32` samples.
///
/// The cancellation flag is checked between packets and between resampler
/// chunks, so a cancel during a long decode takes effect within a few
/// milliseconds.
pub fn decode_to_mono_16k<P: AsRef<Path>>(
    path: P,
    cancel: &AtomicBool,
    on_progress: &mut DecodeProgress<'_>,
) -> Result<Vec<f32>, DecodeError> {
    let path = path.as_ref();
    let (mut format, track_id) = open_format(path)?;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.id == track_id)
        .ok_or(DecodeError::UnsupportedFormat)?;

    let codec_params = track.codec_params.clone();
    let total_frames = codec_params.n_frames;

    // Only used to size the buffer and, if the stream itself never says
    // otherwise, as the resampling rate. The authoritative spec comes off the
    // first decoded packet — see `source_rate` below.
    let declared_rate = codec_params.sample_rate.filter(|rate| *rate > 0);

    let mut decoder = CODECS
        .make(&codec_params, &DecoderOptions::default())
        .map_err(|_| DecodeError::UnsupportedCodec(codec_name(&codec_params)))?;

    let mut mono: Vec<f32> = Vec::with_capacity(
        total_frames
            .map(|f| f as usize)
            .unwrap_or(declared_rate.unwrap_or(16_000) as usize * 8),
    );
    let mut sample_buf: Option<SampleBuffer<f32>> = None;
    // Filled in from the first packet we successfully decode. MP4 does not put
    // the channel layout in `codec_params` for AAC, so reading it from the
    // container header rejected every .m4a — which is what phones record.
    let mut source_rate: Option<u32> = None;

    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(DecodeError::Cancelled);
        }

        let packet = match format.next_packet() {
            Ok(packet) => packet,
            // Symphonia signals a clean end of stream with an IO error of kind
            // UnexpectedEof; anything else at this point is a real read failure.
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break
            }
            Err(SymphoniaError::ResetRequired) => break,
            Err(e) => return Err(DecodeError::Corrupt(e.to_string())),
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                let spec = *audio_buf.spec();
                let channels = spec.channels.count();
                if channels == 0 {
                    return Err(DecodeError::Corrupt(
                        "decoded audio reports no channels".to_string(),
                    ));
                }

                match source_rate {
                    None => source_rate = Some(spec.rate),
                    // A rate change mid-stream would silently stretch everything
                    // after it, so refuse rather than transcribe warped audio.
                    Some(rate) if rate != spec.rate => {
                        return Err(DecodeError::Corrupt(format!(
                            "sample rate changed mid-stream from {} to {}",
                            rate, spec.rate
                        )));
                    }
                    Some(_) => {}
                }

                let buf = sample_buf.get_or_insert_with(|| {
                    SampleBuffer::<f32>::new(audio_buf.capacity() as u64, spec)
                });
                buf.copy_interleaved_ref(audio_buf);
                // Per-buffer channel count: cheap, and correct even if a
                // container disagrees with what the decoder actually produced.
                downmix_into(buf.samples(), channels, &mut mono);
            }
            // Decode errors on individual packets are recoverable per the
            // symphonia contract — skip the packet and keep going.
            Err(SymphoniaError::DecodeError(e)) => {
                debug!("Skipping undecodable packet in {:?}: {}", path, e);
            }
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break
            }
            Err(e) => return Err(DecodeError::Corrupt(e.to_string())),
        }

        on_progress(total_frames.map(|total| (mono.len() as f64 / total as f64).clamp(0.0, 1.0)));
    }

    if mono.is_empty() {
        return Err(DecodeError::Empty);
    }

    // `mono` is non-empty, so at least one packet decoded and set the rate.
    let source_rate = source_rate
        .or(declared_rate)
        .ok_or_else(|| DecodeError::Corrupt("stream declares no sample rate".to_string()))?;

    resample_mono(mono, source_rate, TARGET_SAMPLE_RATE, cancel)
}

/// A name for a codec we have no decoder for.
///
/// The registry can't help here — by definition these codecs aren't registered,
/// and `CodecType`'s own `Display` prints a hex id. So the ones a user can
/// realistically reach through the extensions we advertise are named by hand,
/// and anything else falls back to that id for the log.
fn codec_name(params: &CodecParameters) -> String {
    use symphonia::core::codecs::{
        CODEC_TYPE_AC4, CODEC_TYPE_ALAC, CODEC_TYPE_ATRAC3, CODEC_TYPE_ATRAC9, CODEC_TYPE_DCA,
        CODEC_TYPE_EAC3, CODEC_TYPE_MONKEYS_AUDIO, CODEC_TYPE_MUSEPACK, CODEC_TYPE_OPUS,
        CODEC_TYPE_SPEEX, CODEC_TYPE_TTA, CODEC_TYPE_WAVPACK, CODEC_TYPE_WMA,
    };

    match params.codec {
        // Opus itself decodes now, so reaching here means a variant we don't
        // handle — say which, or the message contradicts the file that worked.
        CODEC_TYPE_OPUS => "Opus (multichannel)".to_string(),
        CODEC_TYPE_SPEEX => "Speex".to_string(),
        CODEC_TYPE_ALAC => "Apple Lossless (ALAC)".to_string(),
        CODEC_TYPE_WMA => "WMA".to_string(),
        CODEC_TYPE_EAC3 => "Enhanced AC-3".to_string(),
        CODEC_TYPE_AC4 => "AC-4".to_string(),
        CODEC_TYPE_DCA => "DTS".to_string(),
        CODEC_TYPE_ATRAC3 => "ATRAC3".to_string(),
        CODEC_TYPE_ATRAC9 => "ATRAC9".to_string(),
        CODEC_TYPE_MUSEPACK => "Musepack".to_string(),
        CODEC_TYPE_MONKEYS_AUDIO => "Monkey's Audio".to_string(),
        CODEC_TYPE_WAVPACK => "WavPack".to_string(),
        CODEC_TYPE_TTA => "TTA".to_string(),
        other => other.to_string(),
    }
}

/// Open a media file and pick its default audio track.
fn open_format(
    path: &Path,
) -> Result<(Box<dyn symphonia::core::formats::FormatReader>, u32), DecodeError> {
    if !has_supported_extension(path) {
        return Err(DecodeError::UnsupportedFormat);
    }

    let file = File::open(path).map_err(|e| DecodeError::Io(e.to_string()))?;
    let metadata = file
        .metadata()
        .map_err(|e| DecodeError::Io(e.to_string()))?;
    if metadata.len() == 0 {
        return Err(DecodeError::Empty);
    }

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| match e {
            SymphoniaError::Unsupported(_) => DecodeError::UnsupportedFormat,
            other => DecodeError::Corrupt(other.to_string()),
        })?;

    let track_id = probed
        .format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .map(|t| t.id)
        .ok_or(DecodeError::UnsupportedFormat)?;

    Ok((probed.format, track_id))
}

/// Average `channels` interleaved samples down to one mono stream.
fn downmix_into(interleaved: &[f32], channels: usize, out: &mut Vec<f32>) {
    if channels == 1 {
        out.extend_from_slice(interleaved);
        return;
    }

    let scale = 1.0 / channels as f32;
    for frame in interleaved.chunks_exact(channels) {
        out.push(frame.iter().sum::<f32>() * scale);
    }
}

/// Resample a mono buffer with the same FFT resampler the live pipeline uses.
///
/// The resampler's own group delay is trimmed from the head and the tail is cut
/// to the exact expected length, so the result lines up with the source instead
/// of drifting on long files.
fn resample_mono(
    samples: Vec<f32>,
    from_rate: u32,
    to_rate: u32,
    cancel: &AtomicBool,
) -> Result<Vec<f32>, DecodeError> {
    if from_rate == to_rate {
        return Ok(samples);
    }

    let expected_len = (samples.len() as f64 * to_rate as f64 / from_rate as f64).round() as usize;

    let mut resampler = FftFixedIn::<f32>::new(
        from_rate as usize,
        to_rate as usize,
        RESAMPLER_CHUNK_SIZE,
        1,
        1,
    )
    .map_err(|e| DecodeError::Corrupt(format!("resampler setup failed: {}", e)))?;
    let delay = resampler.output_delay();

    let mut out: Vec<f32> = Vec::with_capacity(expected_len + delay + RESAMPLER_CHUNK_SIZE);
    let mut chunk = vec![0.0f32; RESAMPLER_CHUNK_SIZE];

    // Feed full chunks, then keep feeding silence until enough output has come
    // back out to cover the resampler's delay line plus the expected length.
    let padded_len =
        samples.len() + delay * from_rate as usize / to_rate as usize + RESAMPLER_CHUNK_SIZE;
    let mut offset = 0;
    while offset < padded_len {
        if cancel.load(Ordering::Relaxed) {
            return Err(DecodeError::Cancelled);
        }

        let start = offset.min(samples.len());
        let end = (offset + RESAMPLER_CHUNK_SIZE)
            .min(samples.len())
            .max(start);
        let available = end - start;
        chunk[..available].copy_from_slice(&samples[start..end]);
        chunk[available..].fill(0.0);

        let processed = resampler
            .process(&[&chunk[..]], None)
            .map_err(|e| DecodeError::Corrupt(format!("resampling failed: {}", e)))?;
        out.extend_from_slice(&processed[0]);

        offset += RESAMPLER_CHUNK_SIZE;

        if out.len() >= expected_len + delay {
            break;
        }
    }

    if out.len() > delay {
        out.drain(..delay);
    } else {
        out.clear();
    }
    out.truncate(expected_len);

    if out.is_empty() {
        return Err(DecodeError::Empty);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::{SampleFormat, WavSpec, WavWriter};
    use std::f32::consts::TAU;
    use std::path::PathBuf;

    /// Write a WAV of `secs` seconds where every channel carries `freq` Hz.
    fn write_sine_wav(
        dir: &tempfile::TempDir,
        name: &str,
        sample_rate: u32,
        channels: u16,
        secs: f32,
        freq: f32,
    ) -> PathBuf {
        let path = dir.path().join(name);
        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(&path, spec).expect("create wav");
        let frames = (sample_rate as f32 * secs) as usize;
        for i in 0..frames {
            let value = (TAU * freq * i as f32 / sample_rate as f32).sin() * 0.5;
            for _ in 0..channels {
                writer
                    .write_sample((value * i16::MAX as f32) as i16)
                    .expect("write sample");
            }
        }
        writer.finalize().expect("finalize wav");
        path
    }

    /// Write a stereo WAV whose two channels are exact opposites, so a correct
    /// downmix must cancel to silence.
    fn write_opposed_stereo_wav(dir: &tempfile::TempDir, name: &str, sample_rate: u32) -> PathBuf {
        let path = dir.path().join(name);
        let spec = WavSpec {
            channels: 2,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(&path, spec).expect("create wav");
        for i in 0..sample_rate {
            let value = (TAU * 440.0 * i as f32 / sample_rate as f32).sin() * 0.5;
            let sample = (value * i16::MAX as f32) as i16;
            writer.write_sample(sample).expect("write left");
            writer.write_sample(-sample).expect("write right");
        }
        writer.finalize().expect("finalize wav");
        path
    }

    fn no_progress() -> Box<DecodeProgress<'static>> {
        Box::new(|_| {})
    }

    fn decode(path: &PathBuf) -> Result<Vec<f32>, DecodeError> {
        let cancel = AtomicBool::new(false);
        decode_to_mono_16k(path, &cancel, &mut *no_progress())
    }

    /// Rough peak-frequency estimate via zero crossings — enough to prove the
    /// resampler preserved pitch rather than up/down-shifting it.
    ///
    /// Measured over the voiced region only: lossy encoders pad the head of the
    /// stream with silence (AAC's encoder delay is ~1000 samples), and counting
    /// that padding as signal drags the estimate off by tens of hertz.
    fn estimated_freq(samples: &[f32], sample_rate: u32) -> f32 {
        let peak = samples.iter().fold(0.0f32, |acc, s| acc.max(s.abs()));
        assert!(peak > 0.0, "cannot estimate the pitch of silence");

        let threshold = peak * 0.5;
        let first = samples
            .iter()
            .position(|s| s.abs() > threshold)
            .expect("signal must rise above half its peak");
        let last = samples
            .iter()
            .rposition(|s| s.abs() > threshold)
            .expect("signal must rise above half its peak");
        let voiced = &samples[first..=last];

        let crossings = voiced
            .windows(2)
            .filter(|w| w[0] <= 0.0 && w[1] > 0.0)
            .count();
        crossings as f32 * sample_rate as f32 / voiced.len() as f32
    }

    #[test]
    fn decodes_mono_16k_unchanged_length() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_sine_wav(&dir, "mono16k.wav", 16_000, 1, 1.0, 440.0);

        let samples = decode(&path).expect("decode mono 16k");

        assert_eq!(samples.len(), 16_000, "16 kHz input must pass through 1:1");
        assert!((estimated_freq(&samples, 16_000) - 440.0).abs() < 5.0);
    }

    #[test]
    fn downsamples_mono_44100_to_16k() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_sine_wav(&dir, "mono44k.wav", 44_100, 1, 1.0, 440.0);

        let samples = decode(&path).expect("decode mono 44.1k");

        assert_eq!(samples.len(), 16_000);
        assert!(
            (estimated_freq(&samples, 16_000) - 440.0).abs() < 5.0,
            "resampling must preserve pitch"
        );
    }

    #[test]
    fn upsamples_mono_8k_to_16k() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_sine_wav(&dir, "mono8k.wav", 8_000, 1, 1.0, 440.0);

        let samples = decode(&path).expect("decode mono 8k");

        assert_eq!(samples.len(), 16_000);
        assert!((estimated_freq(&samples, 16_000) - 440.0).abs() < 5.0);
    }

    #[test]
    fn downmixes_stereo_48k_to_mono_16k() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_sine_wav(&dir, "stereo48k.wav", 48_000, 2, 1.0, 440.0);

        let samples = decode(&path).expect("decode stereo 48k");

        assert_eq!(
            samples.len(),
            16_000,
            "stereo must collapse to one sample per frame"
        );
        assert!((estimated_freq(&samples, 16_000) - 440.0).abs() < 5.0);
    }

    #[test]
    fn downmix_averages_channels_rather_than_concatenating() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_opposed_stereo_wav(&dir, "opposed.wav", 16_000);

        let samples = decode(&path).expect("decode opposed stereo");

        assert_eq!(samples.len(), 16_000);
        let peak = samples.iter().fold(0.0f32, |acc, s| acc.max(s.abs()));
        assert!(
            peak < 0.01,
            "L and -L must average to silence, got peak {peak}"
        );
    }

    #[test]
    fn probe_reports_rate_channels_and_duration() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_sine_wav(&dir, "probe.wav", 44_100, 2, 2.0, 440.0);

        let info = probe_audio_file(&path).expect("probe wav");

        assert_eq!(info.sample_rate, Some(44_100));
        assert_eq!(info.channels, Some(2));
        assert_eq!(info.duration_ms, Some(2_000));
    }

    #[test]
    fn rejects_unsupported_extension() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("notes.txt");
        std::fs::write(&path, b"not audio").expect("write file");

        assert!(matches!(decode(&path), Err(DecodeError::UnsupportedFormat)));
    }

    #[test]
    fn rejects_corrupt_file_with_audio_extension() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("broken.wav");
        std::fs::write(&path, b"RIFFnot-actually-a-wave-file").expect("write file");

        let err = decode(&path).expect_err("corrupt wav must not decode");
        assert!(
            matches!(
                err,
                DecodeError::UnsupportedFormat | DecodeError::Corrupt(_)
            ),
            "unexpected error for corrupt file: {err:?}"
        );
    }

    #[test]
    fn rejects_empty_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("empty.wav");
        std::fs::write(&path, b"").expect("write file");

        assert!(matches!(decode(&path), Err(DecodeError::Empty)));
    }

    #[test]
    fn rejects_missing_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("nope.wav");

        assert!(matches!(decode(&path), Err(DecodeError::Io(_))));
    }

    #[test]
    fn honours_cancellation_before_decoding() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_sine_wav(&dir, "cancel.wav", 44_100, 1, 3.0, 440.0);

        let cancel = AtomicBool::new(true);
        let result = decode_to_mono_16k(&path, &cancel, &mut *no_progress());

        assert!(matches!(result, Err(DecodeError::Cancelled)));
    }

    #[test]
    fn reports_determinate_progress_for_wav() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_sine_wav(&dir, "progress.wav", 16_000, 1, 1.0, 440.0);

        let mut seen: Vec<Option<f64>> = Vec::new();
        let cancel = AtomicBool::new(false);
        decode_to_mono_16k(&path, &cancel, &mut |p| seen.push(p)).expect("decode");

        assert!(!seen.is_empty(), "progress must be reported at least once");
        assert!(
            seen.iter()
                .all(|p| p.is_some_and(|v| (0.0..=1.0).contains(&v))),
            "WAV declares a frame count, so progress must be determinate and in range"
        );
        assert!(
            seen.last().and_then(|p| *p).is_some_and(|v| v > 0.99),
            "progress must reach ~1.0 at end of stream"
        );
    }

    /// Real container/codec samples that a synthetic WAV can never stand in for.
    /// See `src-tauri/tests/fixtures/audio/README.md`.
    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/audio")
            .join(name)
    }

    #[test]
    fn decodes_aac_in_mp4() {
        // MP4 leaves codec_params.channels empty — the channel layout only shows
        // up on the first decoded packet. Requiring it up front rejected every
        // .m4a, which is what phones record. Regression for #45.
        let samples = decode(&fixture("aac-lc-mono-44100.m4a")).expect("decode AAC in MP4");

        // AAC pads the start of the stream, so the length lands near but not
        // exactly on half a second of 16 kHz audio.
        assert!(
            (7_000..=9_500).contains(&samples.len()),
            "expected roughly 8000 samples, got {}",
            samples.len()
        );
        assert!(
            (estimated_freq(&samples, 16_000) - 440.0).abs() < 15.0,
            "resampled AAC must keep its 440 Hz tone"
        );
    }

    #[test]
    fn probes_aac_in_mp4_without_a_declared_channel_count() {
        let info = probe_audio_file(fixture("aac-lc-mono-44100.m4a")).expect("probe AAC in MP4");

        assert_eq!(info.sample_rate, Some(44_100));
        assert!(
            info.duration_ms.is_some_and(|ms| (400..=700).contains(&ms)),
            "expected about 500 ms, got {:?}",
            info.duration_ms
        );
    }

    #[test]
    fn decodes_vorbis_in_ogg() {
        let samples = decode(&fixture("vorbis-stereo-48000.ogg")).expect("decode Vorbis in Ogg");

        assert!(
            (7_500..=8_500).contains(&samples.len()),
            "expected roughly 8000 samples, got {}",
            samples.len()
        );
        assert!((estimated_freq(&samples, 16_000) - 440.0).abs() < 15.0);
    }

    #[test]
    fn decodes_opus_in_ogg() {
        // Telegram and WhatsApp send every voice note as Opus in Ogg. symphonia
        // demuxes it but has no decoder, so this only works because of the
        // libopus-backed decoder registered in CODECS.
        let samples = decode(&fixture("opus-mono-48000.ogg")).expect("decode Opus in Ogg");

        assert!(
            (7_500..=8_500).contains(&samples.len()),
            "expected roughly 8000 samples, got {}",
            samples.len()
        );
        assert!((estimated_freq(&samples, 16_000) - 440.0).abs() < 15.0);
    }

    #[test]
    fn opus_pre_skip_is_trimmed_from_the_head_of_the_stream() {
        // Every Opus stream begins with encoder priming that is not audio. Left
        // in, it puts a click at the start and shifts everything after it.
        let samples = decode(&fixture("opus-mono-48000.ogg")).expect("decode Opus in Ogg");

        let peak = samples.iter().fold(0.0f32, |acc, s| acc.max(s.abs()));
        let first_loud = samples
            .iter()
            .position(|s| s.abs() > peak * 0.5)
            .expect("signal must rise");

        // The fixture is a sine that starts immediately, so with the pre-skip
        // removed the tone must begin within a few milliseconds. 160 samples at
        // 16 kHz is 10 ms; an untrimmed pre-skip lands well past that.
        assert!(
            first_loud < 160,
            "tone should start almost immediately, first loud sample at {first_loud}"
        );
    }

    #[test]
    fn names_the_codec_when_a_stream_has_no_decoder() {
        // ALAC rides in the same MP4 container as the AAC we do support, so the
        // message has to name the codec — "unsupported format" would send the
        // user looking for a problem with their file instead.
        let err = decode(&fixture("alac-mono-44100.m4a")).expect_err("ALAC has no decoder");

        match err {
            DecodeError::UnsupportedCodec(name) => {
                assert!(
                    name.to_lowercase().contains("alac"),
                    "the message must name the codec, got {name:?}"
                );
            }
            other => panic!("expected UnsupportedCodec, got {other:?}"),
        }
    }

    #[test]
    fn decodes_a_bare_opus_extension() {
        // WhatsApp on Android names its voice notes ".opus" rather than ".ogg".
        // Same Ogg container either way, but the extension gate has to let it
        // through, so exercise that path rather than trusting the list.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("voice-note.opus");
        std::fs::copy(fixture("opus-mono-48000.ogg"), &path).expect("copy fixture");

        let samples = decode(&path).expect("decode a .opus file");

        assert!(
            (7_500..=8_500).contains(&samples.len()),
            "expected roughly 8000 samples, got {}",
            samples.len()
        );
    }

    #[test]
    fn extension_filter_is_case_insensitive() {
        assert!(has_supported_extension("/tmp/Voice Memo.M4A"));
        assert!(has_supported_extension("/tmp/take.FLAC"));
        assert!(!has_supported_extension("/tmp/clip.mkv"));
        assert!(!has_supported_extension("/tmp/no-extension"));
    }
}

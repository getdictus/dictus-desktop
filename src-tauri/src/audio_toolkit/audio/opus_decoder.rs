//! A Symphonia `Decoder` for Opus, backed by libopus.
//!
//! symphonia 0.5 reads the Ogg container and recognises its Opus mapping — the
//! demuxer hands us fully-formed Opus packets — but ships no Opus decoder. That
//! is the only missing piece, and it matters: Telegram and WhatsApp send every
//! voice note as Opus in Ogg, which is the central case for importing audio.
//!
//! libopus is the reference implementation, so decoding correctness is not in
//! question. That was the deciding factor over the pure-Rust Opus crates: Opus
//! is a hybrid SILK/CELT codec with many modes, and a decoder that is wrong in
//! one of them yields plausible-looking garbage that would be transcribed as
//! garbage, rather than failing loudly.

use opus::{Channels as OpusChannels, Decoder as LibOpusDecoder};
use std::sync::Mutex;
use symphonia::core::audio::{AsAudioBufferRef, AudioBuffer, AudioBufferRef, Signal, SignalSpec};
use symphonia::core::codecs::{
    CodecDescriptor, CodecParameters, Decoder, DecoderOptions, FinalizeResult, CODEC_TYPE_OPUS,
};
use symphonia::core::errors::{unsupported_error, Error, Result};
use symphonia::core::formats::Packet;
use symphonia::core::support_codec;

/// Opus always decodes at 48 kHz here. The codec supports 8/12/16/24/48 kHz
/// output, and decoding straight to the pipeline's 16 kHz would save a
/// resampling step — but pre-skip is defined in 48 kHz samples, so decoding at
/// the native rate lets us trim it exactly instead of scaling and rounding it.
/// The existing `resample_mono` then does 48 k → 16 k like it does for every
/// other format, which keeps one resampler for all inputs.
const OPUS_SAMPLE_RATE: u32 = 48_000;

/// Longest Opus frame is 120 ms, i.e. 5760 samples per channel at 48 kHz.
const MAX_FRAME_SAMPLES: usize = 5_760;

pub struct OpusDecoder {
    /// libopus decoder state is `Send` but not `Sync`, and symphonia requires
    /// `Decoder: Send + Sync`. The mutex supplies `Sync` without any unsafe;
    /// it is never contended, because every call that touches it takes
    /// `&mut self` and so reaches it through `get_mut` rather than locking.
    decoder: Mutex<LibOpusDecoder>,
    params: CodecParameters,
    buf: AudioBuffer<f32>,
    channel_count: usize,
    /// Samples still to be dropped from the head of the stream, at 48 kHz.
    ///
    /// Every Opus stream declares a pre-skip: encoder priming that is not part
    /// of the audio. Playing it back produces a click and shifts everything
    /// after it, so it has to go.
    pre_skip_remaining: u64,
    /// Scratch space for libopus, which writes interleaved samples.
    interleaved: Vec<f32>,
}

impl OpusDecoder {
    fn write_frames(&mut self, frames: usize) {
        // Drop whatever is left of the pre-skip before anything reaches the
        // caller. A long pre-skip can span more than one packet, hence the
        // running counter rather than a one-shot trim.
        let skipped = (self.pre_skip_remaining.min(frames as u64)) as usize;
        self.pre_skip_remaining -= skipped as u64;

        let kept = frames - skipped;
        self.buf.clear();
        self.buf.render_reserved(Some(kept));

        for channel in 0..self.channel_count {
            let dst = self.buf.chan_mut(channel);
            for (i, sample) in dst.iter_mut().enumerate() {
                *sample = self.interleaved[(skipped + i) * self.channel_count + channel];
            }
        }
    }
}

impl Decoder for OpusDecoder {
    fn try_new(params: &CodecParameters, _options: &DecoderOptions) -> Result<Self> {
        let channels = params.channels.ok_or(Error::DecodeError(
            "opus: stream declares no channel layout",
        ))?;

        // libopus' plain decoder covers mapping family 0 and the mono/stereo
        // cases of family 1. Anything wider needs the multistream decoder,
        // which we don't wire up — voice notes are never multichannel, and a
        // named error beats a wrong downmix.
        let channel_count = channels.count();
        let opus_channels = match channel_count {
            1 => OpusChannels::Mono,
            2 => OpusChannels::Stereo,
            _ => return unsupported_error("opus: multichannel streams"),
        };

        let decoder = LibOpusDecoder::new(OPUS_SAMPLE_RATE, opus_channels)
            .map_err(|_| Error::DecodeError("opus: could not initialise libopus"))?;

        // The Ogg mapper always reports 48 kHz for Opus; anything else means we
        // are being handed a stream this decoder shouldn't claim.
        if params
            .sample_rate
            .is_some_and(|rate| rate != OPUS_SAMPLE_RATE)
        {
            return unsupported_error("opus: non-48kHz stream");
        }

        let spec = SignalSpec::new(OPUS_SAMPLE_RATE, channels);
        let mut params = params.clone();
        params.with_sample_rate(OPUS_SAMPLE_RATE);

        Ok(Self {
            decoder: Mutex::new(decoder),
            pre_skip_remaining: u64::from(params.delay.unwrap_or(0)),
            params,
            buf: AudioBuffer::new(MAX_FRAME_SAMPLES as u64, spec),
            channel_count,
            interleaved: vec![0.0; MAX_FRAME_SAMPLES * channel_count],
        })
    }

    fn supported_codecs() -> &'static [CodecDescriptor] {
        &[support_codec!(CODEC_TYPE_OPUS, "opus", "Opus")]
    }

    fn reset(&mut self) {
        // Pre-skip is a property of the start of the stream, so it is not
        // re-armed here — a reset mid-stream must not chop off real audio.
        if let Ok(decoder) = self.decoder.get_mut() {
            let _ = decoder.reset_state();
        }
    }

    fn codec_params(&self) -> &CodecParameters {
        &self.params
    }

    fn decode(&mut self, packet: &Packet) -> Result<AudioBufferRef<'_>> {
        let decoded = {
            let decoder = self
                .decoder
                .get_mut()
                .map_err(|_| Error::DecodeError("opus: decoder state was poisoned"))?;
            decoder.decode_float(packet.buf(), &mut self.interleaved, false)
        };

        match decoded {
            Ok(frames) => {
                self.write_frames(frames);
                Ok(self.buf.as_audio_buffer_ref())
            }
            Err(_) => {
                // The trait requires the internal buffer be empty after an
                // error, so the caller can skip the packet and carry on.
                self.buf.clear();
                Err(Error::DecodeError("opus: packet could not be decoded"))
            }
        }
    }

    fn finalize(&mut self) -> FinalizeResult {
        FinalizeResult::default()
    }

    fn last_decoded(&self) -> AudioBufferRef<'_> {
        self.buf.as_audio_buffer_ref()
    }
}

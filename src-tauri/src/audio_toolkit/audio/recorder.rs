use std::{
    io::Error,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError, TryRecvError},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Sample, SizedSample,
};

use crate::audio_toolkit::{
    audio::{AudioVisualiser, FrameResampler},
    constants,
    vad::{self, VadFrame},
    VoiceActivityDetector,
};

/// How often the worker checks for commands when the stream delivers nothing.
/// Before #27 commands were read only after a chunk arrived, so a stream that
/// went silent (a Bluetooth headset changing route) left `Cmd::Stop` unread
/// forever.
const COMMAND_POLL: Duration = Duration::from_millis(50);

/// How long the worker waits, after a stop, for the audio callback to confirm
/// end-of-stream. A healthy callback runs every 10–100 ms; two seconds without
/// one means the stream has stalled.
const STALL_TIMEOUT: Duration = Duration::from_secs(2);

/// How long `stop()` waits for the worker's reply. It covers the worker's worst
/// healthy path (one `COMMAND_POLL`, the `STALL_TIMEOUT` drain, then resampling
/// and VAD on the tail, a few ms) with room for a loaded machine. A normal stop
/// replies in tens of milliseconds; past four seconds the worker itself is stuck
/// and the caller gets an error instead of hanging.
const STOP_REPLY_TIMEOUT: Duration = Duration::from_secs(4);

/// How long `close()` waits for the worker to exit. Dropping a stream on a
/// device that has gone away can block inside the audio backend; past this the
/// worker is left to finish on its own.
const CLOSE_TIMEOUT: Duration = Duration::from_secs(1);

/// Why `AudioRecorder::stop` returned no samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopError {
    /// The audio callback stopped delivering: the stream is dead.
    Stalled,
    /// The worker did not answer within `STOP_REPLY_TIMEOUT`.
    Unresponsive,
    /// The worker has exited (stream gone, recorder closed).
    Closed,
}

impl std::fmt::Display for StopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StopError::Stalled => write!(
                f,
                "audio stream stalled: no audio from the input device for {STALL_TIMEOUT:?}"
            ),
            StopError::Unresponsive => write!(
                f,
                "recorder worker did not answer within {STOP_REPLY_TIMEOUT:?}"
            ),
            StopError::Closed => write!(f, "recorder worker has exited"),
        }
    }
}

impl std::error::Error for StopError {}

enum Cmd {
    Start,
    Stop(mpsc::Sender<Result<Vec<f32>, StopError>>),
    Shutdown,
}

enum AudioChunk {
    Samples(Vec<f32>),
    EndOfStream,
}

pub struct AudioRecorder {
    device: Option<Device>,
    cmd_tx: Option<mpsc::Sender<Cmd>>,
    worker_handle: Option<std::thread::JoinHandle<()>>,
    vad: Option<Arc<Mutex<Box<dyn vad::VoiceActivityDetector>>>>,
    level_cb: Option<Arc<dyn Fn(Vec<f32>) + Send + Sync + 'static>>,
}

impl AudioRecorder {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(AudioRecorder {
            device: None,
            cmd_tx: None,
            worker_handle: None,
            vad: None,
            level_cb: None,
        })
    }

    pub fn with_vad(mut self, vad: Box<dyn VoiceActivityDetector>) -> Self {
        self.vad = Some(Arc::new(Mutex::new(vad)));
        self
    }

    pub fn with_level_callback<F>(mut self, cb: F) -> Self
    where
        F: Fn(Vec<f32>) + Send + Sync + 'static,
    {
        self.level_cb = Some(Arc::new(cb));
        self
    }

    pub fn open(&mut self, device: Option<Device>) -> Result<(), Box<dyn std::error::Error>> {
        if self.worker_handle.is_some() {
            return Ok(()); // already open
        }

        let (sample_tx, sample_rx) = mpsc::channel::<AudioChunk>();
        let (cmd_tx, cmd_rx) = mpsc::channel::<Cmd>();
        let (init_tx, init_rx) = mpsc::sync_channel::<Result<(), String>>(1);

        let host = crate::audio_toolkit::get_cpal_host();
        let device = match device {
            Some(dev) => dev,
            None => host
                .default_input_device()
                .ok_or_else(|| Error::new(std::io::ErrorKind::NotFound, "No input device found"))?,
        };

        let thread_device = device.clone();
        let vad = self.vad.clone();
        // Move the optional level callback into the worker thread
        let level_cb = self.level_cb.clone();

        let worker = std::thread::spawn(move || {
            let stop_flag = Arc::new(AtomicBool::new(false));
            let stop_flag_for_stream = stop_flag.clone();
            let init_result = (|| -> Result<(cpal::Stream, u32), String> {
                let config = AudioRecorder::get_preferred_config(&thread_device)
                    .map_err(|e| format!("Failed to fetch preferred config: {e}"))?;

                let sample_rate = config.sample_rate().0;
                let channels = config.channels() as usize;

                log::info!(
                    "Using device: {:?}\nSample rate: {}\nChannels: {}\nFormat: {:?}",
                    thread_device.name(),
                    sample_rate,
                    channels,
                    config.sample_format()
                );

                let stream = match config.sample_format() {
                    cpal::SampleFormat::U8 => AudioRecorder::build_stream::<u8>(
                        &thread_device,
                        &config,
                        sample_tx,
                        channels,
                        stop_flag_for_stream,
                    )
                    .map_err(|e| format!("Failed to build input stream: {e}"))?,
                    cpal::SampleFormat::I8 => AudioRecorder::build_stream::<i8>(
                        &thread_device,
                        &config,
                        sample_tx,
                        channels,
                        stop_flag_for_stream,
                    )
                    .map_err(|e| format!("Failed to build input stream: {e}"))?,
                    cpal::SampleFormat::I16 => AudioRecorder::build_stream::<i16>(
                        &thread_device,
                        &config,
                        sample_tx,
                        channels,
                        stop_flag_for_stream,
                    )
                    .map_err(|e| format!("Failed to build input stream: {e}"))?,
                    cpal::SampleFormat::I32 => AudioRecorder::build_stream::<i32>(
                        &thread_device,
                        &config,
                        sample_tx,
                        channels,
                        stop_flag_for_stream,
                    )
                    .map_err(|e| format!("Failed to build input stream: {e}"))?,
                    cpal::SampleFormat::F32 => AudioRecorder::build_stream::<f32>(
                        &thread_device,
                        &config,
                        sample_tx,
                        channels,
                        stop_flag_for_stream,
                    )
                    .map_err(|e| format!("Failed to build input stream: {e}"))?,
                    sample_format => {
                        return Err(format!("Unsupported sample format: {sample_format:?}"));
                    }
                };

                stream
                    .play()
                    .map_err(|e| format!("Failed to start microphone stream: {e}"))?;

                Ok((stream, sample_rate))
            })();

            match init_result {
                Ok((stream, sample_rate)) => {
                    let _ = init_tx.send(Ok(()));
                    // Keep the stream alive while we process samples.
                    run_consumer(sample_rate, vad, sample_rx, cmd_rx, level_cb, stop_flag);
                    drop(stream);
                }
                Err(error_message) => {
                    log::error!("{error_message}");
                    let _ = init_tx.send(Err(error_message));
                }
            }
        });

        match init_rx.recv() {
            Ok(Ok(())) => {
                self.device = Some(device);
                self.cmd_tx = Some(cmd_tx);
                self.worker_handle = Some(worker);
                Ok(())
            }
            Ok(Err(error_message)) => {
                let _ = worker.join();
                let kind = if is_microphone_access_denied(&error_message) {
                    std::io::ErrorKind::PermissionDenied
                } else {
                    std::io::ErrorKind::Other
                };
                Err(Box::new(Error::new(kind, error_message)))
            }
            Err(recv_error) => {
                let _ = worker.join();
                Err(Box::new(Error::other(format!(
                    "Failed to initialize microphone worker: {recv_error}"
                ))))
            }
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tx) = &self.cmd_tx {
            tx.send(Cmd::Start)?;
        }
        Ok(())
    }

    /// Stops recording and returns the samples. Never blocks longer than
    /// `STOP_REPLY_TIMEOUT`: on any error the stream should be closed and
    /// reopened before the next recording.
    pub fn stop(&self) -> Result<Vec<f32>, StopError> {
        self.stop_within(STOP_REPLY_TIMEOUT)
    }

    fn stop_within(&self, timeout: Duration) -> Result<Vec<f32>, StopError> {
        let (resp_tx, resp_rx) = mpsc::channel();
        let tx = self.cmd_tx.as_ref().ok_or(StopError::Closed)?;
        tx.send(Cmd::Stop(resp_tx)).map_err(|_| StopError::Closed)?;
        match resp_rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(RecvTimeoutError::Timeout) => Err(StopError::Unresponsive),
            Err(RecvTimeoutError::Disconnected) => Err(StopError::Closed),
        }
    }

    /// Shuts the worker down and waits up to `CLOSE_TIMEOUT` for it to exit.
    /// A worker still stuck after that (its stream blocked in the backend) is
    /// detached rather than joined, so closing never hangs.
    pub fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tx) = self.cmd_tx.take() {
            let _ = tx.send(Cmd::Shutdown);
        }
        if let Some(h) = self.worker_handle.take() {
            let deadline = Instant::now() + CLOSE_TIMEOUT;
            while !h.is_finished() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
            if h.is_finished() {
                let _ = h.join();
            } else {
                log::warn!(
                    "Recorder worker did not exit within {:?}; detaching it",
                    CLOSE_TIMEOUT
                );
            }
        }
        self.device = None;
        Ok(())
    }

    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::SupportedStreamConfig,
        sample_tx: mpsc::Sender<AudioChunk>,
        channels: usize,
        stop_flag: Arc<AtomicBool>,
    ) -> Result<cpal::Stream, cpal::BuildStreamError>
    where
        T: Sample + SizedSample + Send + 'static,
        f32: cpal::FromSample<T>,
    {
        let mut output_buffer = Vec::new();
        let mut eos_sent = false;

        let stream_cb = move |data: &[T], _: &cpal::InputCallbackInfo| {
            if stop_flag.load(Ordering::Relaxed) {
                if !eos_sent {
                    let _ = sample_tx.send(AudioChunk::EndOfStream);
                    eos_sent = true;
                }
                return;
            }
            eos_sent = false;

            output_buffer.clear();

            if channels == 1 {
                output_buffer.extend(data.iter().map(|&sample| sample.to_sample::<f32>()));
            } else {
                let frame_count = data.len() / channels;
                output_buffer.reserve(frame_count);

                for frame in data.chunks_exact(channels) {
                    let mono_sample = frame
                        .iter()
                        .map(|&sample| sample.to_sample::<f32>())
                        .sum::<f32>()
                        / channels as f32;
                    output_buffer.push(mono_sample);
                }
            }

            if sample_tx
                .send(AudioChunk::Samples(output_buffer.clone()))
                .is_err()
            {
                log::error!("Failed to send samples");
            }
        };

        device.build_input_stream(
            &config.clone().into(),
            stream_cb,
            |err| log::error!("Stream error: {}", err),
            None,
        )
    }

    fn get_preferred_config(
        device: &cpal::Device,
    ) -> Result<cpal::SupportedStreamConfig, Box<dyn std::error::Error>> {
        // Use the device's native/default sample rate and let the FrameResampler
        // in run_consumer() downsample to 16kHz. This avoids forcing hardware into
        // a non-native rate which can cause issues on some devices (Bluetooth
        // codecs, certain ALSA drivers, etc.).
        let default_config = device.default_input_config()?;
        let target_rate = default_config.sample_rate();

        // Try to find the best sample format at the device's default rate
        let supported_configs = match device.supported_input_configs() {
            Ok(configs) => configs,
            Err(e) => {
                log::warn!("Could not enumerate input configs ({e}), using device default");
                return Ok(default_config);
            }
        };
        let mut best_config: Option<cpal::SupportedStreamConfigRange> = None;

        for config_range in supported_configs {
            if config_range.min_sample_rate() <= target_rate
                && config_range.max_sample_rate() >= target_rate
            {
                match best_config {
                    None => best_config = Some(config_range),
                    Some(ref current) => {
                        // Prioritize F32 > I16 > I32 > others
                        let score = |fmt: cpal::SampleFormat| match fmt {
                            cpal::SampleFormat::F32 => 4,
                            cpal::SampleFormat::I16 => 3,
                            cpal::SampleFormat::I32 => 2,
                            _ => 1,
                        };

                        if score(config_range.sample_format()) > score(current.sample_format()) {
                            best_config = Some(config_range);
                        }
                    }
                }
            }
        }

        if let Some(config) = best_config {
            return Ok(config.with_sample_rate(target_rate));
        }

        // Fall back to device default if no config matched (exotic/virtual devices)
        log::warn!(
            "No supported config matched device default rate {:?}, using default config",
            target_rate
        );
        Ok(default_config)
    }
}

pub fn is_microphone_access_denied(error_message: &str) -> bool {
    let normalized = error_message.to_lowercase();
    normalized.contains("access is denied")
        || normalized.contains("permission denied")
        || normalized.contains("0x80070005")
}

pub fn is_no_input_device_error(error_message: &str) -> bool {
    let normalized = error_message.to_lowercase();
    normalized.contains("no input device found")
        || (normalized.contains("failed to fetch preferred config")
            && normalized.contains("coreaudio"))
}

fn run_consumer(
    in_sample_rate: u32,
    vad: Option<Arc<Mutex<Box<dyn vad::VoiceActivityDetector>>>>,
    sample_rx: mpsc::Receiver<AudioChunk>,
    cmd_rx: mpsc::Receiver<Cmd>,
    level_cb: Option<Arc<dyn Fn(Vec<f32>) + Send + Sync + 'static>>,
    stop_flag: Arc<AtomicBool>,
) {
    let mut frame_resampler = FrameResampler::new(
        in_sample_rate as usize,
        constants::WHISPER_SAMPLE_RATE as usize,
        Duration::from_millis(30),
    );

    let mut processed_samples = Vec::<f32>::new();
    let mut recording = false;

    // ---------- spectrum visualisation setup ---------------------------- //
    const BUCKETS: usize = 16;
    const WINDOW_SIZE: usize = 512;
    let mut visualizer = AudioVisualiser::new(
        in_sample_rate,
        WINDOW_SIZE,
        BUCKETS,
        400.0,  // vocal_min_hz
        4000.0, // vocal_max_hz
    );

    fn handle_frame(
        samples: &[f32],
        recording: bool,
        vad: &Option<Arc<Mutex<Box<dyn vad::VoiceActivityDetector>>>>,
        out_buf: &mut Vec<f32>,
    ) {
        if !recording {
            return;
        }

        if let Some(vad_arc) = vad {
            let mut det = vad_arc.lock().unwrap();
            match det.push_frame(samples).unwrap_or(VadFrame::Speech(samples)) {
                VadFrame::Speech(buf) => out_buf.extend_from_slice(buf),
                VadFrame::Noise => {}
            }
        } else {
            out_buf.extend_from_slice(samples);
        }
    }

    loop {
        // Wait for audio, but never longer than COMMAND_POLL: commands must be
        // honoured even when the stream has gone silent (#27).
        match sample_rx.recv_timeout(COMMAND_POLL) {
            Ok(AudioChunk::Samples(raw)) => {
                // ---------- spectrum processing -------------------------- //
                if let Some(buckets) = visualizer.feed(&raw) {
                    if let Some(cb) = &level_cb {
                        cb(buckets);
                    }
                }

                // ---------- existing pipeline ---------------------------- //
                frame_resampler.push(&raw, &mut |frame: &[f32]| {
                    handle_frame(frame, recording, &vad, &mut processed_samples)
                });
            }
            Ok(AudioChunk::EndOfStream) | Err(RecvTimeoutError::Timeout) => {}
            // The stream (which owns the sender) is gone. Pending and later
            // stops see the closed channel and report `StopError::Closed`.
            Err(RecvTimeoutError::Disconnected) => return,
        }

        // non-blocking check for a command
        loop {
            let cmd = match cmd_rx.try_recv() {
                Ok(cmd) => cmd,
                Err(TryRecvError::Empty) => break,
                // The recorder was dropped without close(): stop the stream.
                Err(TryRecvError::Disconnected) => {
                    stop_flag.store(true, Ordering::Relaxed);
                    return;
                }
            };
            match cmd {
                Cmd::Start => {
                    stop_flag.store(false, Ordering::Relaxed);
                    processed_samples.clear();
                    recording = true;
                    visualizer.reset();
                    if let Some(v) = &vad {
                        v.lock().unwrap().reset();
                    }
                }
                Cmd::Stop(reply_tx) => {
                    recording = false;
                    stop_flag.store(true, Ordering::Relaxed);

                    // Drain all remaining audio until the producer confirms end-of-stream.
                    // The cpal callback sees the stop flag, sends EndOfStream, and goes
                    // silent — guaranteeing every captured sample is in the channel
                    // ahead of the sentinel.
                    let mut stalled = false;
                    loop {
                        match sample_rx.recv_timeout(STALL_TIMEOUT) {
                            Ok(AudioChunk::Samples(remaining)) => {
                                frame_resampler.push(&remaining, &mut |frame: &[f32]| {
                                    handle_frame(frame, true, &vad, &mut processed_samples)
                                });
                            }
                            Ok(AudioChunk::EndOfStream) => break,
                            Err(_) => {
                                stalled = true;
                                break;
                            }
                        }
                    }

                    if stalled {
                        // The callback has stopped running: this stream is
                        // dead and the caller will reopen it. What was
                        // captured before the stall is discarded with it.
                        log::warn!(
                            "No EndOfStream from the audio callback within {:?}: stream stalled",
                            STALL_TIMEOUT
                        );
                        processed_samples.clear();
                        let _ = reply_tx.send(Err(StopError::Stalled));
                        continue;
                    }

                    frame_resampler.finish(&mut |frame: &[f32]| {
                        handle_frame(frame, true, &vad, &mut processed_samples)
                    });

                    let _ = reply_tx.send(Ok(std::mem::take(&mut processed_samples)));

                    // Resume the audio callback so the consumer loop can continue
                    // receiving chunks (important for always-on microphone mode).
                    stop_flag.store(false, Ordering::Relaxed);
                }
                Cmd::Shutdown => {
                    stop_flag.store(true, Ordering::Relaxed);
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod stall_tests {
    //! #27: a stream that stops delivering audio (a Bluetooth headset changing
    //! route) must not hang `stop()`. These drive the real worker loop with a
    //! fake audio callback in place of cpal.
    use super::*;
    use std::thread;

    /// A recorder whose worker is the real `run_consumer`, fed by `sample_rx`.
    fn recorder_on(
        sample_rx: mpsc::Receiver<AudioChunk>,
        stop_flag: Arc<AtomicBool>,
    ) -> AudioRecorder {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            run_consumer(16_000, None, sample_rx, cmd_rx, None, stop_flag);
        });
        AudioRecorder {
            device: None,
            cmd_tx: Some(cmd_tx),
            worker_handle: Some(worker),
            vad: None,
            level_cb: None,
        }
    }

    /// Mimics the cpal callback of `build_stream`: 10 ms chunks, and one
    /// EndOfStream once the stop flag is set. Ends when the worker is gone.
    fn healthy_callback(tx: mpsc::Sender<AudioChunk>, stop_flag: Arc<AtomicBool>) {
        thread::spawn(move || {
            let mut eos_sent = false;
            loop {
                let sent = if stop_flag.load(Ordering::Relaxed) {
                    if eos_sent {
                        Ok(())
                    } else {
                        eos_sent = true;
                        tx.send(AudioChunk::EndOfStream)
                    }
                } else {
                    eos_sent = false;
                    tx.send(AudioChunk::Samples(vec![0.1; 160]))
                };
                if sent.is_err() {
                    return;
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
    }

    #[test]
    fn healthy_stream_returns_the_samples() {
        let (tx, rx) = mpsc::channel();
        let flag = Arc::new(AtomicBool::new(false));
        healthy_callback(tx, flag.clone());
        let mut rec = recorder_on(rx, flag);

        rec.start().unwrap();
        thread::sleep(Duration::from_millis(300));
        let samples = rec.stop().expect("a healthy stream stops cleanly");
        assert!(!samples.is_empty());

        // And the stream keeps working for the next recording.
        rec.start().unwrap();
        thread::sleep(Duration::from_millis(100));
        assert!(!rec.stop().unwrap().is_empty());
        rec.close().unwrap();
    }

    #[test]
    fn stalled_stream_stop_returns_within_the_timeout() {
        // The callback delivers a little audio, then never runs again: the
        // sender stays alive, so the worker sees silence, not a closed stream.
        let (tx, rx) = mpsc::channel();
        let flag = Arc::new(AtomicBool::new(false));
        let mut rec = recorder_on(rx, flag);
        rec.start().unwrap();
        tx.send(AudioChunk::Samples(vec![0.1; 1600])).unwrap();
        thread::sleep(Duration::from_millis(100));

        let began = Instant::now();
        let result = rec.stop();
        let took = began.elapsed();

        assert_eq!(result, Err(StopError::Stalled));
        assert!(
            took >= STALL_TIMEOUT,
            "stopped before the stall was certain: {took:?}"
        );
        assert!(took < STOP_REPLY_TIMEOUT, "stop took {took:?}");

        // The worker is still responsive: close() joins it at once and leaves
        // the recorder closed, ready to be reopened.
        let began = Instant::now();
        rec.close().unwrap();
        assert!(began.elapsed() < CLOSE_TIMEOUT);
        assert!(rec.worker_handle.is_none() && rec.cmd_tx.is_none());
        assert_eq!(rec.stop(), Err(StopError::Closed));
        drop(tx);
    }

    #[test]
    fn stream_that_never_delivered_does_not_block_commands() {
        // Before #27 the worker read commands only after a chunk arrived, so
        // this Stop was never read and stop() waited forever.
        let (tx, rx) = mpsc::channel::<AudioChunk>();
        let mut rec = recorder_on(rx, Arc::new(AtomicBool::new(false)));
        rec.start().unwrap();
        let began = Instant::now();
        assert_eq!(rec.stop(), Err(StopError::Stalled));
        assert!(began.elapsed() < STOP_REPLY_TIMEOUT);

        // Shutdown is honoured without any audio either.
        let began = Instant::now();
        rec.close().unwrap();
        assert!(began.elapsed() < CLOSE_TIMEOUT);
        drop(tx);
    }

    #[test]
    fn unresponsive_worker_does_not_hang_stop_or_close() {
        // A worker that never reads its commands (stuck in the backend).
        let (cmd_tx, cmd_rx) = mpsc::channel::<Cmd>();
        let (release_tx, release_rx) = mpsc::channel::<()>();
        let worker = thread::spawn(move || {
            let _ = release_rx.recv();
            drop(cmd_rx);
        });
        let mut rec = AudioRecorder {
            device: None,
            cmd_tx: Some(cmd_tx),
            worker_handle: Some(worker),
            vad: None,
            level_cb: None,
        };

        let began = Instant::now();
        assert_eq!(
            rec.stop_within(Duration::from_millis(200)),
            Err(StopError::Unresponsive)
        );
        assert!(began.elapsed() < Duration::from_secs(1));

        let began = Instant::now();
        rec.close().unwrap();
        assert!(began.elapsed() < CLOSE_TIMEOUT + Duration::from_millis(500));
        assert!(rec.worker_handle.is_none());
        let _ = release_tx.send(());
    }

    #[test]
    fn stop_timeout_covers_the_worker_stall_detection() {
        // Otherwise stop() would report Unresponsive for a plain stall and the
        // worker's own detection would never be seen.
        assert!(STOP_REPLY_TIMEOUT > STALL_TIMEOUT + COMMAND_POLL + Duration::from_secs(1));
    }
}

#[cfg(test)]
mod tests {
    use super::{is_microphone_access_denied, is_no_input_device_error};

    #[test]
    fn detects_access_is_denied() {
        assert!(is_microphone_access_denied("Access is denied"));
    }

    #[test]
    fn detects_permission_denied() {
        assert!(is_microphone_access_denied("permission denied"));
    }

    #[test]
    fn detects_windows_error_code() {
        assert!(is_microphone_access_denied("WASAPI error: 0x80070005"));
    }

    #[test]
    fn does_not_match_unrelated_errors() {
        assert!(!is_microphone_access_denied("device not found"));
    }

    #[test]
    fn detects_no_input_device() {
        assert!(is_no_input_device_error("No input device found"));
    }

    #[test]
    fn detects_coreaudio_config_error() {
        assert!(is_no_input_device_error(
            "Failed to fetch preferred config: A backend-specific error has occurred: An unknown error unknown to the coreaudio-rs API occurred"
        ));
    }

    #[test]
    fn does_not_match_other_errors_for_no_device() {
        assert!(!is_no_input_device_error("permission denied"));
        assert!(!is_no_input_device_error("device not found"));
    }
}

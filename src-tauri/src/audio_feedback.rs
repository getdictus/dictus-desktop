use crate::settings::{self, AppSettings, SoundTheme};
use cpal::traits::{DeviceTrait, HostTrait};
use log::{debug, error, info, warn};
use rodio::buffer::SamplesBuffer;
use rodio::mixer::Mixer;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundType {
    /// Played when recording starts.
    Start,
    /// Played once the transcribed text has been inserted.
    End,
}

/// Where the sound for a pack lives on disk.
#[derive(Debug, PartialEq, Eq)]
enum SoundFile {
    /// Bundled with the app, relative to the resource directory.
    Resource(String),
    /// Dropped by the user into the app data folder.
    AppData(&'static str),
}

fn sound_file(theme: SoundTheme, sound: SoundType) -> SoundFile {
    let pack = match theme {
        SoundTheme::Custom => {
            return SoundFile::AppData(match sound {
                SoundType::Start => "custom_start.wav",
                SoundType::End => "custom_stop.wav",
            })
        }
        SoundTheme::Bois => "bois",
        SoundTheme::Synthe => "synthe",
        SoundTheme::Minimal => "minimal",
        SoundTheme::Marimba => "marimba",
        SoundTheme::Pop => "pop",
    };
    let name = match sound {
        SoundType::Start => "start",
        SoundType::End => "end",
    };
    SoundFile::Resource(format!("resources/sounds/{pack}/{name}.wav"))
}

fn resolve_sound_path(app: &AppHandle, theme: SoundTheme, sound: SoundType) -> Option<PathBuf> {
    let resolved = match sound_file(theme, sound) {
        SoundFile::Resource(path) => app
            .path()
            .resolve(&path, tauri::path::BaseDirectory::Resource)
            .map_err(|e| e.to_string()),
        SoundFile::AppData(name) => {
            crate::portable::resolve_app_data(app, name).map_err(|e| e.to_string())
        }
    };
    match resolved {
        Ok(path) => Some(path),
        Err(e) => {
            error!("Cannot resolve {:?} sound of {:?}: {}", sound, theme, e);
            None
        }
    }
}

/// How long the output stays open after a dictation, counted from the end of
/// its last sound. An open output stops the system from sleeping on its own
/// (macOS, Windows), so it is open only around dictations. The idle-sleep
/// timers count from the last user input, at least a minute after it on macOS
/// and Windows; the press or release that ends a dictation is such an input, so
/// a close this soon after never delays sleep. Dictations within this delay of
/// each other reuse the output and do not wait for it to open.
pub const CLOSE_AFTER: Duration = Duration::from_secs(15);

/// When the output may close: never while a dictation uses it, otherwise
/// `CLOSE_AFTER` after its last activity (release, or end of the last sound).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Lifetime {
    in_use: bool,
    last_activity: Option<Instant>,
}

impl Lifetime {
    const IDLE: Lifetime = Lifetime {
        in_use: false,
        last_activity: None,
    };

    fn begin(&mut self) {
        self.in_use = true;
    }

    fn end(&mut self, now: Instant) {
        self.in_use = false;
        self.touch(now);
    }

    /// A sound queued at `now` plays until `now + duration`.
    fn played(&mut self, now: Instant, duration: Duration) {
        self.touch(now + duration);
    }

    fn touch(&mut self, at: Instant) {
        self.last_activity = Some(self.last_activity.map_or(at, |t| t.max(at)));
    }

    /// `None` while in use; an output never used closes at once.
    fn close_at(&self, now: Instant) -> Option<Instant> {
        (!self.in_use).then(|| self.last_activity.map_or(now, |t| t + CLOSE_AFTER))
    }
}

/// A pack's two sounds, decoded once so playing one costs no disk read.
struct PackSounds {
    theme: SoundTheme,
    start: SamplesBuffer,
    end: SamplesBuffer,
}

impl PackSounds {
    fn get(&self, sound: SoundType) -> SamplesBuffer {
        match sound {
            SoundType::Start => self.start.clone(),
            SoundType::End => self.end.clone(),
        }
    }
}

/// An open output stream. `cpal::Stream` is not `Send`, so a dedicated thread
/// owns it and keeps it open until `_keep_open` is dropped.
struct Output {
    mixer: Mixer,
    /// The `selected_output_device` setting it was opened for.
    selected: Option<String>,
    /// For the default device: the device it was at opening time.
    default_name: Option<String>,
    failed: Arc<AtomicBool>,
    _keep_open: mpsc::Sender<()>,
}

impl Output {
    fn usable(&self, selected: &Option<String>) -> bool {
        !self.failed.load(Ordering::SeqCst)
            && &self.selected == selected
            && (selected.is_some() || self.default_name == default_device_name())
    }
}

/// The feedback player: the current pack's sounds in memory, and an output
/// stream open only from a dictation's press until `CLOSE_AFTER` after it.
struct Player {
    output: Option<Output>,
    pack: Option<PackSounds>,
    lifetime: Lifetime,
    /// An output is being opened, without the lock.
    opening: bool,
}

static PLAYER: Mutex<Player> = Mutex::new(Player {
    output: None,
    pack: None,
    lifetime: Lifetime::IDLE,
    opening: false,
});

fn player() -> MutexGuard<'static, Player> {
    PLAYER.lock().unwrap_or_else(|e| e.into_inner())
}

/// `None` stands for the system default device.
fn selected_device(settings: &AppSettings) -> Option<String> {
    settings
        .selected_output_device
        .clone()
        .filter(|name| name != "Default")
}

fn default_device_name() -> Option<String> {
    cpal::default_host()
        .default_output_device()
        .and_then(|d| d.name().ok())
}

impl Player {
    fn output_usable(&self, selected: &Option<String>) -> bool {
        self.output.as_ref().is_some_and(|o| o.usable(selected))
    }

    fn pack(&mut self, app: &AppHandle, theme: SoundTheme) -> Result<&PackSounds, String> {
        if self.pack.as_ref().is_none_or(|p| p.theme != theme) {
            self.pack = None;
            self.pack = Some(load_pack(app, theme)?);
        }
        Ok(self.pack.as_ref().expect("pack just loaded"))
    }

    /// Closes the output once its `close_at` has passed. Returns when to check
    /// again, if it is still open and not in use.
    fn close_if_due(&mut self, now: Instant) -> Option<Instant> {
        self.output.as_ref()?;
        let close_at = self.lifetime.close_at(now)?;
        if close_at > now {
            return Some(close_at);
        }
        self.output = None;
        info!("Feedback output closed");
        None
    }
}

fn open_output(selected: Option<String>) -> Result<Output, String> {
    let opened_at = Instant::now();
    let failed = Arc::new(AtomicBool::new(false));
    let (ready_tx, ready_rx) = mpsc::channel();
    let (keep_open, closed) = mpsc::channel::<()>();
    let on_error = {
        let failed = Arc::clone(&failed);
        move |e: cpal::StreamError| {
            if !failed.swap(true, Ordering::SeqCst) {
                warn!("Feedback output failed ({e}); the next dictation reopens it");
            }
        }
    };
    let device = selected.clone();
    thread::Builder::new()
        .name("feedback-output".into())
        .spawn(move || match open_stream(device, on_error) {
            Ok(stream) => {
                let _ = ready_tx.send(Ok(stream.mixer().clone()));
                // Returns when the `Output` holding `keep_open` is dropped.
                let _ = closed.recv();
            }
            Err(e) => {
                let _ = ready_tx.send(Err(e));
            }
        })
        .map_err(|e| e.to_string())?;
    let mixer = ready_rx.recv().map_err(|e| e.to_string())??;
    let default_name = if selected.is_none() {
        default_device_name()
    } else {
        None
    };
    info!(
        "Feedback output open on {} in {:.1} ms",
        selected
            .as_deref()
            .or(default_name.as_deref())
            .unwrap_or("default device"),
        opened_at.elapsed().as_secs_f64() * 1000.0
    );
    Ok(Output {
        mixer,
        selected,
        default_name,
        failed,
        _keep_open: keep_open,
    })
}

fn open_stream(
    selected: Option<String>,
    on_error: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<OutputStream, String> {
    let builder = match selected {
        None => OutputStreamBuilder::from_default_device(),
        Some(name) => {
            let host = crate::audio_toolkit::get_cpal_host();
            let device = host
                .output_devices()
                .map_err(|e| e.to_string())?
                .find(|d| d.name().is_ok_and(|n| n == name));
            match device {
                Some(device) => OutputStreamBuilder::from_device(device),
                None => {
                    warn!("Device '{}' not found, using default device", name);
                    OutputStreamBuilder::from_default_device()
                }
            }
        }
    };
    builder
        .map_err(|e| e.to_string())?
        .with_error_callback(on_error)
        .open_stream()
        .map_err(|e| e.to_string())
}

fn decode(path: &Path) -> Result<SamplesBuffer, String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let decoder =
        Decoder::new(BufReader::new(file)).map_err(|e| format!("{}: {e}", path.display()))?;
    let (channels, rate) = (decoder.channels(), decoder.sample_rate());
    Ok(SamplesBuffer::new(
        channels,
        rate,
        decoder.collect::<Vec<_>>(),
    ))
}

fn load_pack(app: &AppHandle, theme: SoundTheme) -> Result<PackSounds, String> {
    let load = |sound| {
        let path = resolve_sound_path(app, theme, sound)
            .ok_or_else(|| format!("no {sound:?} sound for {theme:?}"))?;
        decode(&path)
    };
    Ok(PackSounds {
        theme,
        start: load(SoundType::Start)?,
        end: load(SoundType::End)?,
    })
}

/// Brings the player in line with the settings, off the calling thread: the
/// selected pack decoded in memory, the output closed if audio feedback is
/// off. Never opens the output. Called at startup and when a sound setting
/// changes.
pub fn sync_with_settings(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || {
        let settings = settings::get_settings(&app);
        let mut player = player();
        if !settings.audio_feedback {
            player.output = None;
            return;
        }
        if player
            .output
            .as_ref()
            .is_some_and(|o| o.selected != selected_device(&settings))
        {
            player.output = None;
        }
        if let Err(e) = player.pack(&app, settings.sound_theme) {
            error!("Cannot load the feedback sounds: {}", e);
        }
    });
}

/// Called at a dictation's press: marks the output in use and, if it is
/// closed, opens it in the background. Returns `true` once it is open, `false`
/// if it is not open after `max_wait` (it keeps opening, for the end sound) or
/// cannot open.
pub fn begin_session(app: &AppHandle, max_wait: Duration) -> bool {
    let selected = selected_device(&settings::get_settings(app));
    let (tx, rx) = mpsc::channel();
    {
        let mut player = player();
        player.lifetime.begin();
        if player.output_usable(&selected) {
            return true;
        }
        if !player.opening {
            player.opening = true;
            thread::spawn(move || open_for_session(selected, tx));
        }
    }
    rx.recv_timeout(max_wait).unwrap_or_else(|_| {
        // Timed out, or another press is already opening it (its sender was
        // not ours): the pill then shows without the start sound.
        let ready = player().output.is_some();
        if !ready {
            warn!("Feedback output not open after {} ms", max_wait.as_millis());
        }
        ready
    })
}

fn open_for_session(selected: Option<String>, ready: mpsc::Sender<bool>) {
    // Opening can take hundreds of ms: not under the lock, so a sound or the
    // end of a dictation never waits for it.
    let opened = open_output(selected);
    let mut player = player();
    player.opening = false;
    let ok = match opened {
        Ok(output) => {
            player.output = Some(output);
            true
        }
        Err(e) => {
            error!("Cannot open the feedback output: {}", e);
            false
        }
    };
    // The dictation may have ended while the output opened.
    if let Some(check_at) = player.close_if_due(Instant::now()) {
        schedule_close(check_at);
    }
    let _ = ready.send(ok);
}

/// Called when a dictation is over, whatever its outcome: the output closes
/// `CLOSE_AFTER` after its last sound ends.
pub fn end_session() {
    let now = Instant::now();
    let mut player = player();
    player.lifetime.end(now);
    if let Some(check_at) = player.close_if_due(now) {
        schedule_close(check_at);
    }
}

fn schedule_close(mut check_at: Instant) {
    thread::spawn(move || loop {
        thread::sleep(check_at.saturating_duration_since(Instant::now()));
        match player().close_if_due(Instant::now()) {
            Some(next) => check_at = next,
            None => break,
        }
    });
}

/// Queues a recording sound on the open feedback output and returns its
/// length, if audio feedback is on. Never opens the output (see
/// `begin_session`) and never waits for the sound to play.
pub fn play_feedback_sound(app: &AppHandle, sound: SoundType) -> Option<Duration> {
    let settings = settings::get_settings(app);
    if !settings.audio_feedback {
        return None;
    }
    queue_sound(app, &settings, settings.sound_theme, sound)
}

fn queue_sound(
    app: &AppHandle,
    settings: &AppSettings,
    theme: SoundTheme,
    sound: SoundType,
) -> Option<Duration> {
    // The slider stops at 1; a stored value above it must not amplify.
    let volume = settings.audio_feedback_volume.clamp(0.0, 1.0);
    let mut player = player();
    let queued = player
        .pack(app, theme)
        .map(|p| p.get(sound))
        .and_then(|buffer| {
            let output = player
                .output
                .as_ref()
                .ok_or_else(|| "the output is not open".to_string())?;
            let duration = buffer.total_duration().unwrap_or_default();
            output.mixer.add(buffer.amplify(volume));
            player.lifetime.played(Instant::now(), duration);
            Ok(duration)
        });
    match queued {
        Ok(duration) => {
            debug!(
                "Queued {:?} sound of {:?} ({} ms, volume {:.2})",
                sound,
                theme,
                duration.as_millis(),
                volume
            );
            Some(duration)
        }
        Err(e) => {
            error!("Failed to play {:?} sound of {:?}: {}", sound, theme, e);
            None
        }
    }
}

/// Plays a pack's start sound then its end sound, whatever the audio feedback
/// toggle says, so the user can hear a pack before choosing it. Returns once
/// both have played.
pub fn preview_sound_pack(app: &AppHandle, theme: SoundTheme) {
    let settings = settings::get_settings(app);
    if begin_session(app, Duration::from_secs(5)) {
        for sound in [SoundType::Start, SoundType::End] {
            if let Some(duration) = queue_sound(app, &settings, theme, sound) {
                thread::sleep(duration);
            }
        }
    }
    end_session();
    sync_with_settings(app);
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUNDLED: [SoundTheme; 5] = [
        SoundTheme::Bois,
        SoundTheme::Synthe,
        SoundTheme::Minimal,
        SoundTheme::Marimba,
        SoundTheme::Pop,
    ];

    fn at(t0: Instant, ms: u64) -> Instant {
        t0 + Duration::from_millis(ms)
    }

    #[test]
    fn output_in_use_never_closes() {
        let t0 = Instant::now();
        let mut life = Lifetime::IDLE;
        life.begin();
        life.played(t0, Duration::from_millis(450));
        assert_eq!(life.close_at(at(t0, 600_000)), None);
    }

    #[test]
    fn output_closes_close_after_past_the_end_sound() {
        let t0 = Instant::now();
        let mut life = Lifetime::IDLE;
        life.begin();
        life.played(at(t0, 5_000), Duration::from_millis(600));
        life.end(at(t0, 5_001));
        assert_eq!(
            life.close_at(at(t0, 5_001)),
            Some(at(t0, 5_600) + CLOSE_AFTER)
        );
    }

    #[test]
    fn end_sound_queued_after_the_release_pushes_the_close_back() {
        let t0 = Instant::now();
        let mut life = Lifetime::IDLE;
        life.begin();
        life.end(t0);
        life.played(at(t0, 200), Duration::from_millis(600));
        assert_eq!(life.close_at(at(t0, 200)), Some(at(t0, 800) + CLOSE_AFTER));
    }

    #[test]
    fn a_press_before_the_close_keeps_the_output_open() {
        let t0 = Instant::now();
        let mut life = Lifetime::IDLE;
        life.begin();
        life.end(t0);
        life.begin();
        assert_eq!(life.close_at(t0 + CLOSE_AFTER), None);
    }

    #[test]
    fn an_output_never_used_closes_at_once() {
        let t0 = Instant::now();
        assert_eq!(Lifetime::IDLE.close_at(t0), Some(t0));
    }

    #[test]
    fn close_after_stays_well_under_the_shortest_idle_sleep_timer() {
        // macOS and Windows let a user set idle sleep down to one minute.
        assert!(CLOSE_AFTER <= Duration::from_secs(30));
    }

    #[test]
    fn bundled_packs_resolve_to_their_folder() {
        assert_eq!(
            sound_file(SoundTheme::Bois, SoundType::Start),
            SoundFile::Resource("resources/sounds/bois/start.wav".into())
        );
        assert_eq!(
            sound_file(SoundTheme::Synthe, SoundType::End),
            SoundFile::Resource("resources/sounds/synthe/end.wav".into())
        );
    }

    #[test]
    fn custom_pack_reads_the_user_files() {
        assert_eq!(
            sound_file(SoundTheme::Custom, SoundType::Start),
            SoundFile::AppData("custom_start.wav")
        );
        assert_eq!(
            sound_file(SoundTheme::Custom, SoundType::End),
            SoundFile::AppData("custom_stop.wav")
        );
    }

    #[test]
    fn every_bundled_sound_exists_and_decodes() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        for theme in BUNDLED {
            for sound in [SoundType::Start, SoundType::End] {
                let SoundFile::Resource(rel) = sound_file(theme, sound) else {
                    panic!("{theme:?} must be bundled");
                };
                let file = File::open(root.join(&rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
                let decoder = Decoder::new(BufReader::new(file))
                    .unwrap_or_else(|e| panic!("{rel} does not decode: {e}"));
                let duration = decoder.total_duration().expect("WAV has a length");
                assert!(
                    duration > Duration::ZERO && duration < Duration::from_secs(1),
                    "{rel} lasts {duration:?}"
                );
            }
        }
    }

    #[test]
    fn new_packs_start_without_a_click_and_peak_under_minus_16_dbfs() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let peak_max = 10f32.powf(-16.0 / 20.0) + 1e-4;
        for theme in [SoundTheme::Bois, SoundTheme::Synthe, SoundTheme::Minimal] {
            for sound in [SoundType::Start, SoundType::End] {
                let SoundFile::Resource(rel) = sound_file(theme, sound) else {
                    panic!("{theme:?} must be bundled");
                };
                let buffer = decode(&root.join(&rel)).expect("decodes");
                let first_ms = (buffer.sample_rate() / 1000 * buffer.channels() as u32) as usize;
                let samples: Vec<f32> = buffer.collect();
                let peak = |s: &[f32]| s.iter().fold(0f32, |m, v| m.max(v.abs()));
                assert!(
                    peak(&samples[..first_ms]) < 0.02,
                    "{rel} starts with a step"
                );
                assert!(peak(&samples) <= peak_max, "{rel} peaks above -16 dBFS");
            }
        }
    }
}

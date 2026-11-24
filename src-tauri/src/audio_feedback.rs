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

/// The feedback player: one output stream, kept open while audio feedback is
/// on, and the current pack's sounds in memory. Opening a stream takes a
/// variable time (hundreds of ms on CoreAudio), which made the sounds lag the
/// overlay when it was done for every sound.
struct Player {
    output: Option<Output>,
    pack: Option<PackSounds>,
}

static PLAYER: Mutex<Player> = Mutex::new(Player {
    output: None,
    pack: None,
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
    /// The mixer of an output open on `selected`, opening one if needed.
    fn mixer(&mut self, app: &AppHandle, selected: &Option<String>) -> Result<Mixer, String> {
        let usable = self
            .output
            .as_ref()
            .is_some_and(|o| !o.failed.load(Ordering::SeqCst) && &o.selected == selected);
        if !usable {
            self.output = None;
            self.output = Some(open_output(app, selected.clone())?);
        }
        Ok(self
            .output
            .as_ref()
            .expect("output just opened")
            .mixer
            .clone())
    }

    fn pack(&mut self, app: &AppHandle, theme: SoundTheme) -> Result<&PackSounds, String> {
        if self.pack.as_ref().is_none_or(|p| p.theme != theme) {
            self.pack = None;
            self.pack = Some(load_pack(app, theme)?);
        }
        Ok(self.pack.as_ref().expect("pack just loaded"))
    }

    /// Brings the player in line with the settings: open on the selected
    /// device with the current pack loaded while audio feedback is on, closed
    /// otherwise. Reopens when the default device has changed under it.
    fn sync(&mut self, app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
        if !settings.audio_feedback {
            self.output = None;
            self.pack = None;
            return Ok(());
        }
        let selected = selected_device(settings);
        if let Some(output) = &self.output {
            if selected.is_none()
                && output.selected.is_none()
                && output.default_name != default_device_name()
            {
                info!("Default output device changed, reopening the feedback output");
                self.output = None;
            }
        }
        self.mixer(app, &selected)?;
        self.pack(app, settings.sound_theme)?;
        Ok(())
    }
}

fn open_output(app: &AppHandle, selected: Option<String>) -> Result<Output, String> {
    let opened_at = Instant::now();
    let failed = Arc::new(AtomicBool::new(false));
    let (ready_tx, ready_rx) = mpsc::channel();
    let (keep_open, closed) = mpsc::channel::<()>();
    let on_error = {
        let failed = Arc::clone(&failed);
        let app = app.clone();
        move |e: cpal::StreamError| {
            if !failed.swap(true, Ordering::SeqCst) {
                warn!("Feedback output failed ({e}), reopening it");
                sync_with_settings(&app);
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
        "Feedback output open on {} in {} ms",
        selected
            .as_deref()
            .or(default_name.as_deref())
            .unwrap_or("default device"),
        opened_at.elapsed().as_millis()
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

/// Opens or closes the feedback output to match the settings, off the
/// calling thread. Called at startup and whenever a sound setting changes,
/// so the first sound after that does not wait for the device.
pub fn sync_with_settings(app: &AppHandle) {
    let app = app.clone();
    thread::spawn(move || {
        let settings = settings::get_settings(&app);
        if let Err(e) = player().sync(&app, &settings) {
            error!("Cannot prepare the feedback sounds: {}", e);
        }
    });
}

/// Queues a recording sound on the feedback output and returns its length,
/// if audio feedback is on. Does not wait for the sound to play.
pub fn play_feedback_sound(app: &AppHandle, sound: SoundType) -> Option<Duration> {
    let settings = settings::get_settings(app);
    if !settings.audio_feedback {
        return None;
    }
    let played = queue_sound(app, &settings, settings.sound_theme, sound);
    if let Some(duration) = played {
        // Picks up a change of default device for the next sound, once this
        // one has finished: reopening the output now would cut it.
        let app = app.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            sync_with_settings(&app);
        });
    }
    played
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
        .mixer(app, &selected_device(settings))
        .and_then(|mixer| {
            let buffer = player.pack(app, theme)?.get(sound);
            let duration = buffer.total_duration().unwrap_or_default();
            mixer.add(buffer.amplify(volume));
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
    for sound in [SoundType::Start, SoundType::End] {
        if let Some(duration) = queue_sound(app, &settings, theme, sound) {
            thread::sleep(duration);
        }
    }
    // Back to the selected pack, and closed again if audio feedback is off.
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

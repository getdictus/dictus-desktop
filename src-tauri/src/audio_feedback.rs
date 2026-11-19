use crate::settings::{self, SoundTheme};
use cpal::traits::{DeviceTrait, HostTrait};
use log::{debug, error, info, warn};
use rodio::{Decoder, OutputStreamBuilder, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
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

/// Plays a recording sound on its own thread, if audio feedback is on.
pub fn play_feedback_sound(app: &AppHandle, sound: SoundType) {
    let app = app.clone();
    thread::spawn(move || play_feedback_sound_blocking(&app, sound, |_| {}));
}

/// Plays a recording sound and returns when it has finished, if audio feedback
/// is on. `on_playback` runs just before the first sample is queued, with the
/// sound's length.
pub fn play_feedback_sound_blocking(
    app: &AppHandle,
    sound: SoundType,
    on_playback: impl FnOnce(Duration),
) {
    let settings = settings::get_settings(app);
    if !settings.audio_feedback {
        return;
    }
    play_theme_sound(app, settings.sound_theme, sound, on_playback);
}

/// Plays a pack's start sound then its end sound, whatever the audio feedback
/// toggle says, so the user can hear a pack before choosing it.
pub fn preview_sound_pack(app: &AppHandle, theme: SoundTheme) {
    play_theme_sound(app, theme, SoundType::Start, |_| {});
    play_theme_sound(app, theme, SoundType::End, |_| {});
}

fn play_theme_sound(
    app: &AppHandle,
    theme: SoundTheme,
    sound: SoundType,
    on_playback: impl FnOnce(Duration),
) {
    let Some(path) = resolve_sound_path(app, theme, sound) else {
        return;
    };
    let settings = settings::get_settings(app);
    let volume = settings.audio_feedback_volume;
    match play_audio_file(&path, settings.selected_output_device, volume, on_playback) {
        Ok(duration) => info!(
            "Played {:?} sound of {:?} ({}, {} ms, volume {:.2})",
            sound,
            theme,
            path.display(),
            duration.as_millis(),
            volume
        ),
        Err(e) => error!("Failed to play sound '{}': {}", path.display(), e),
    }
}

fn play_audio_file(
    path: &std::path::Path,
    selected_device: Option<String>,
    volume: f32,
    on_playback: impl FnOnce(Duration),
) -> Result<Duration, Box<dyn std::error::Error>> {
    let stream_builder = if let Some(device_name) = selected_device {
        if device_name == "Default" {
            debug!("Using default device");
            OutputStreamBuilder::from_default_device()?
        } else {
            let host = crate::audio_toolkit::get_cpal_host();
            let devices = host.output_devices()?;

            let mut found_device = None;
            for device in devices {
                if device.name()? == device_name {
                    found_device = Some(device);
                    break;
                }
            }

            match found_device {
                Some(device) => OutputStreamBuilder::from_device(device)?,
                None => {
                    warn!("Device '{}' not found, using default device", device_name);
                    OutputStreamBuilder::from_default_device()?
                }
            }
        }
    } else {
        debug!("Using default device");
        OutputStreamBuilder::from_default_device()?
    };

    let stream_handle = stream_builder.open_stream()?;
    let source = Decoder::new(BufReader::new(File::open(path)?))?;
    let duration = source.total_duration().unwrap_or_default();

    let sink = Sink::connect_new(stream_handle.mixer());
    sink.set_volume(volume);
    on_playback(duration);
    sink.append(source);
    sink.sleep_until_end();

    Ok(duration)
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
}

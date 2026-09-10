// Re-export all audio components
mod decode;
mod device;
mod recorder;
mod resampler;
mod utils;
mod visualizer;

pub use decode::{
    decode_to_mono_16k, has_supported_extension, probe_audio_file, AudioFileInfo, DecodeError,
    DecodeProgress, SUPPORTED_EXTENSIONS, TARGET_SAMPLE_RATE,
};
pub use device::{list_input_devices, list_output_devices, CpalDeviceInfo};
pub use recorder::{is_microphone_access_denied, is_no_input_device_error, AudioRecorder};
pub use resampler::FrameResampler;
pub use utils::{read_wav_samples, save_wav_file, verify_wav_file};
pub use visualizer::AudioVisualiser;

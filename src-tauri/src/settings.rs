use log::{debug, warn};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

pub const APPLE_INTELLIGENCE_PROVIDER_ID: &str = "apple_intelligence";
pub const APPLE_INTELLIGENCE_DEFAULT_MODEL_ID: &str = "Apple Intelligence";

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

// Custom deserializer to handle both old numeric format (1-5) and new string format ("trace", "debug", etc.)
impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LogLevelVisitor;

        impl<'de> Visitor<'de> for LogLevelVisitor {
            type Value = LogLevel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or integer representing log level")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<LogLevel, E> {
                match value.to_lowercase().as_str() {
                    "trace" => Ok(LogLevel::Trace),
                    "debug" => Ok(LogLevel::Debug),
                    "info" => Ok(LogLevel::Info),
                    "warn" => Ok(LogLevel::Warn),
                    "error" => Ok(LogLevel::Error),
                    _ => Err(E::unknown_variant(
                        value,
                        &["trace", "debug", "info", "warn", "error"],
                    )),
                }
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<LogLevel, E> {
                match value {
                    1 => Ok(LogLevel::Trace),
                    2 => Ok(LogLevel::Debug),
                    3 => Ok(LogLevel::Info),
                    4 => Ok(LogLevel::Warn),
                    5 => Ok(LogLevel::Error),
                    _ => Err(E::invalid_value(de::Unexpected::Unsigned(value), &"1-5")),
                }
            }
        }

        deserializer.deserialize_any(LogLevelVisitor)
    }
}

impl From<LogLevel> for tauri_plugin_log::LogLevel {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => tauri_plugin_log::LogLevel::Trace,
            LogLevel::Debug => tauri_plugin_log::LogLevel::Debug,
            LogLevel::Info => tauri_plugin_log::LogLevel::Info,
            LogLevel::Warn => tauri_plugin_log::LogLevel::Warn,
            LogLevel::Error => tauri_plugin_log::LogLevel::Error,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ShortcutBinding {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_binding: String,
    pub current_binding: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct LLMPrompt {
    pub id: String,
    pub name: String,
    pub prompt: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum SmartModeKind {
    #[default]
    Rewrite,
    Translation,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct TargetLanguage {
    pub code: String,
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct SmartMode {
    pub id: String,
    pub name: String,
    pub kind: SmartModeKind,
    pub prompt: String,
    #[serde(default)]
    pub target_language: Option<TargetLanguage>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct PostProcessProvider {
    pub id: String,
    pub label: String,
    pub base_url: String,
    #[serde(default)]
    pub allow_base_url_edit: bool,
    #[serde(default)]
    pub models_endpoint: Option<String>,
    #[serde(default)]
    pub supports_structured_output: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "lowercase")]
pub enum OverlayPosition {
    None,
    Top,
    Bottom,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModelUnloadTimeout {
    Never,
    Immediately,
    Min2,
    #[default]
    Min5,
    Min10,
    Min15,
    Hour1,
    Sec15, // Debug mode only
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum PasteMethod {
    CtrlV,
    Direct,
    None,
    ShiftInsert,
    CtrlShiftV,
    ExternalScript,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardHandling {
    #[default]
    DontModify,
    CopyToClipboard,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum AutoSubmitKey {
    #[default]
    Enter,
    CtrlEnter,
    CmdEnter,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum RecordingRetentionPeriod {
    Never,
    PreserveLimit,
    Days3,
    Weeks2,
    Months3,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum KeyboardImplementation {
    Tauri,
    HandyKeys,
}

impl Default for KeyboardImplementation {
    fn default() -> Self {
        #[cfg(target_os = "linux")]
        return KeyboardImplementation::Tauri;
        #[cfg(not(target_os = "linux"))]
        return KeyboardImplementation::HandyKeys;
    }
}

impl Default for PasteMethod {
    fn default() -> Self {
        // Default to CtrlV for macOS and Windows, Direct for Linux
        #[cfg(target_os = "linux")]
        return PasteMethod::Direct;
        #[cfg(not(target_os = "linux"))]
        return PasteMethod::CtrlV;
    }
}

impl ModelUnloadTimeout {
    pub fn to_minutes(self) -> Option<u64> {
        match self {
            ModelUnloadTimeout::Never => None,
            ModelUnloadTimeout::Immediately => Some(0), // Special case for immediate unloading
            ModelUnloadTimeout::Min2 => Some(2),
            ModelUnloadTimeout::Min5 => Some(5),
            ModelUnloadTimeout::Min10 => Some(10),
            ModelUnloadTimeout::Min15 => Some(15),
            ModelUnloadTimeout::Hour1 => Some(60),
            ModelUnloadTimeout::Sec15 => Some(0), // Special case for debug - handled separately
        }
    }

    pub fn to_seconds(self) -> Option<u64> {
        match self {
            ModelUnloadTimeout::Never => None,
            ModelUnloadTimeout::Immediately => Some(0), // Special case for immediate unloading
            ModelUnloadTimeout::Sec15 => Some(15),
            _ => self.to_minutes().map(|m| m * 60),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum SoundTheme {
    Marimba,
    Pop,
    Custom,
}

impl SoundTheme {
    fn as_str(&self) -> &'static str {
        match self {
            SoundTheme::Marimba => "marimba",
            SoundTheme::Pop => "pop",
            SoundTheme::Custom => "custom",
        }
    }

    pub fn to_start_path(self) -> String {
        format!("resources/{}_start.wav", self.as_str())
    }

    pub fn to_stop_path(self) -> String {
        format!("resources/{}_stop.wav", self.as_str())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum TypingTool {
    #[default]
    Auto,
    Wtype,
    Kwtype,
    Dotool,
    Ydotool,
    Xdotool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum WhisperAcceleratorSetting {
    #[default]
    Auto,
    Cpu,
    Gpu,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum OrtAcceleratorSetting {
    #[default]
    Auto,
    Cpu,
    Cuda,
    #[serde(rename = "directml")]
    DirectMl,
    Rocm,
}

#[derive(Clone, Serialize, Deserialize, Type)]
#[serde(transparent)]
pub(crate) struct SecretMap(HashMap<String, String>);

impl fmt::Debug for SecretMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted: HashMap<&String, &str> = self
            .0
            .iter()
            .map(|(k, v)| (k, if v.is_empty() { "" } else { "[REDACTED]" }))
            .collect();
        redacted.fmt(f)
    }
}

impl std::ops::Deref for SecretMap {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SecretMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/* still handy for composing the initial JSON in the store ------------- */
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AppSettings {
    pub bindings: HashMap<String, ShortcutBinding>,
    pub push_to_talk: bool,
    pub audio_feedback: bool,
    #[serde(default = "default_audio_feedback_volume")]
    pub audio_feedback_volume: f32,
    #[serde(default = "default_sound_theme")]
    pub sound_theme: SoundTheme,
    #[serde(default = "default_start_hidden")]
    pub start_hidden: bool,
    #[serde(default = "default_autostart_enabled")]
    pub autostart_enabled: bool,
    #[serde(default = "default_update_checks_enabled")]
    pub update_checks_enabled: bool,
    #[serde(default = "default_model")]
    pub selected_model: String,
    #[serde(default = "default_always_on_microphone")]
    pub always_on_microphone: bool,
    #[serde(default)]
    pub selected_microphone: Option<String>,
    #[serde(default)]
    pub clamshell_microphone: Option<String>,
    #[serde(default)]
    pub selected_output_device: Option<String>,
    #[serde(default = "default_translate_to_english")]
    pub translate_to_english: bool,
    #[serde(default = "default_selected_language")]
    pub selected_language: String,
    #[serde(default = "default_overlay_position")]
    pub overlay_position: OverlayPosition,
    #[serde(default = "default_debug_mode")]
    pub debug_mode: bool,
    #[serde(default = "default_log_level")]
    pub log_level: LogLevel,
    #[serde(default)]
    pub custom_words: Vec<String>,
    #[serde(default)]
    pub model_unload_timeout: ModelUnloadTimeout,
    #[serde(default = "default_word_correction_threshold")]
    pub word_correction_threshold: f64,
    #[serde(default = "default_history_limit")]
    pub history_limit: usize,
    #[serde(default = "default_recording_retention_period")]
    pub recording_retention_period: RecordingRetentionPeriod,
    #[serde(default)]
    pub paste_method: PasteMethod,
    #[serde(default)]
    pub clipboard_handling: ClipboardHandling,
    #[serde(default = "default_auto_submit")]
    pub auto_submit: bool,
    #[serde(default)]
    pub auto_submit_key: AutoSubmitKey,
    #[serde(default = "default_post_process_enabled")]
    pub post_process_enabled: bool,
    #[serde(default = "default_post_process_provider_id")]
    pub post_process_provider_id: String,
    #[serde(default = "default_post_process_providers")]
    pub post_process_providers: Vec<PostProcessProvider>,
    #[serde(default = "default_post_process_api_keys")]
    pub post_process_api_keys: SecretMap,
    #[serde(default = "default_post_process_models")]
    pub post_process_models: HashMap<String, String>,
    #[serde(default = "default_post_process_prompts")]
    pub post_process_prompts: Vec<LLMPrompt>,
    #[serde(default)]
    pub post_process_selected_prompt_id: Option<String>,
    #[serde(default)]
    pub settings_schema_version: u32,
    #[serde(default = "default_smart_modes")]
    pub smart_modes: Vec<SmartMode>,
    #[serde(default)]
    pub smart_mode_active_id: Option<String>,
    #[serde(default)]
    pub mute_while_recording: bool,
    #[serde(default)]
    pub append_trailing_space: bool,
    #[serde(default = "default_app_language")]
    pub app_language: String,
    #[serde(default)]
    pub experimental_enabled: bool,
    #[serde(default = "default_enable_cloud_providers")]
    pub enable_cloud_providers: bool,
    #[serde(default)]
    pub lazy_stream_close: bool,
    #[serde(default)]
    pub keyboard_implementation: KeyboardImplementation,
    #[serde(default = "default_show_tray_icon")]
    pub show_tray_icon: bool,
    #[serde(default = "default_paste_delay_ms")]
    pub paste_delay_ms: u64,
    #[serde(default = "default_typing_tool")]
    pub typing_tool: TypingTool,
    pub external_script_path: Option<String>,
    #[serde(default)]
    pub custom_filler_words: Option<Vec<String>>,
    #[serde(default)]
    pub whisper_accelerator: WhisperAcceleratorSetting,
    #[serde(default)]
    pub ort_accelerator: OrtAcceleratorSetting,
    #[serde(default = "default_whisper_gpu_device")]
    pub whisper_gpu_device: i32,
    #[serde(default)]
    pub extra_recording_buffer_ms: u64,
    #[serde(default)]
    pub active_llm_model_id: Option<String>,
    #[serde(default)]
    pub llm_unload_timeout: ModelUnloadTimeout,
}

fn default_model() -> String {
    "".to_string()
}

fn default_always_on_microphone() -> bool {
    false
}

fn default_translate_to_english() -> bool {
    false
}

fn default_start_hidden() -> bool {
    false
}

fn default_autostart_enabled() -> bool {
    false
}

fn default_update_checks_enabled() -> bool {
    true
}

fn default_selected_language() -> String {
    "auto".to_string()
}

fn default_overlay_position() -> OverlayPosition {
    #[cfg(target_os = "linux")]
    return OverlayPosition::None;
    #[cfg(not(target_os = "linux"))]
    return OverlayPosition::Bottom;
}

fn default_debug_mode() -> bool {
    false
}

fn default_log_level() -> LogLevel {
    LogLevel::Debug
}

fn default_word_correction_threshold() -> f64 {
    0.18
}

fn default_paste_delay_ms() -> u64 {
    60
}

fn default_auto_submit() -> bool {
    false
}

fn default_history_limit() -> usize {
    5
}

fn default_recording_retention_period() -> RecordingRetentionPeriod {
    RecordingRetentionPeriod::PreserveLimit
}

fn default_audio_feedback_volume() -> f32 {
    1.0
}

fn default_sound_theme() -> SoundTheme {
    SoundTheme::Marimba
}

fn default_post_process_enabled() -> bool {
    false
}

fn default_enable_cloud_providers() -> bool {
    false
}

fn default_app_language() -> String {
    tauri_plugin_os::locale()
        .map(|l| l.replace('_', "-"))
        .unwrap_or_else(|| "en".to_string())
}

fn default_show_tray_icon() -> bool {
    true
}

fn default_post_process_provider_id() -> String {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        APPLE_INTELLIGENCE_PROVIDER_ID.to_string()
    }
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    {
        "custom".to_string()
    }
}

fn default_post_process_providers() -> Vec<PostProcessProvider> {
    let mut providers = vec![
        PostProcessProvider {
            id: "openai".to_string(),
            label: "OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
            supports_structured_output: true,
        },
        PostProcessProvider {
            id: "zai".to_string(),
            label: "Z.AI".to_string(),
            base_url: "https://api.z.ai/api/paas/v4".to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
            supports_structured_output: true,
        },
        PostProcessProvider {
            id: "openrouter".to_string(),
            label: "OpenRouter".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
            supports_structured_output: true,
        },
        PostProcessProvider {
            id: "anthropic".to_string(),
            label: "Anthropic".to_string(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
            supports_structured_output: false,
        },
        PostProcessProvider {
            id: "groq".to_string(),
            label: "Groq".to_string(),
            base_url: "https://api.groq.com/openai/v1".to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
            supports_structured_output: false,
        },
        PostProcessProvider {
            id: "cerebras".to_string(),
            label: "Cerebras".to_string(),
            base_url: "https://api.cerebras.ai/v1".to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
            supports_structured_output: true,
        },
    ];

    // Note: We always include Apple Intelligence on macOS ARM64 without checking availability
    // at startup. The availability check is deferred to when the user actually tries to use it
    // (in actions.rs). This prevents crashes on macOS 26.x beta where accessing
    // SystemLanguageModel.default during early app initialization causes SIGABRT.
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        providers.push(PostProcessProvider {
            id: APPLE_INTELLIGENCE_PROVIDER_ID.to_string(),
            label: "Apple Intelligence".to_string(),
            base_url: "apple-intelligence://local".to_string(),
            allow_base_url_edit: false,
            models_endpoint: None,
            supports_structured_output: true,
        });
    }

    // Custom provider always comes last
    providers.push(PostProcessProvider {
        id: "custom".to_string(),
        label: "Custom (local)".to_string(),
        base_url: "http://localhost:11434/v1".to_string(),
        allow_base_url_edit: true,
        models_endpoint: Some("/models".to_string()),
        supports_structured_output: false,
    });

    providers
}

fn default_post_process_api_keys() -> SecretMap {
    let mut map = HashMap::new();
    for provider in default_post_process_providers() {
        map.insert(provider.id, String::new());
    }
    SecretMap(map)
}

fn default_model_for_provider(provider_id: &str) -> String {
    if provider_id == APPLE_INTELLIGENCE_PROVIDER_ID {
        return APPLE_INTELLIGENCE_DEFAULT_MODEL_ID.to_string();
    }
    String::new()
}

fn default_post_process_models() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for provider in default_post_process_providers() {
        map.insert(
            provider.id.clone(),
            default_model_for_provider(&provider.id),
        );
    }
    map
}

fn default_post_process_prompts() -> Vec<LLMPrompt> {
    vec![LLMPrompt {
        id: "default_improve_transcriptions".to_string(),
        name: "Improve Transcriptions".to_string(),
        prompt: "Clean this transcript:\n1. Fix spelling, capitalization, and punctuation errors\n2. Convert number words to digits (twenty-five → 25, ten percent → 10%, five dollars → $5)\n3. Replace spoken punctuation with symbols (period → ., comma → ,, question mark → ?)\n4. Remove filler words (um, uh, like as filler)\n5. Keep the language in the original version (if it was french, keep it in french for example)\n\nPreserve exact meaning and word order. Do not paraphrase or reorder content.\n\nReturn only the cleaned transcript.\n\nTranscript:\n${output}".to_string(),
    }]
}

pub const CLEAN_UP_MODE_ID: &str = "mode_clean_up";

/// Returns the full predefined template catalogue (10 modes).
/// This is the source for the create-picker; it is NOT the first-run seed.
pub fn smart_mode_templates() -> Vec<SmartMode> {
    vec![
        SmartMode {
            id: "mode_clean_up".to_string(),
            name: "Clean Up".to_string(),
            kind: SmartModeKind::Rewrite,
            prompt: "Clean this transcript:\n1. Fix spelling, capitalization, and punctuation errors\n2. ALWAYS write numbers and quantities as digits, never spelled out. Convert every single one: vingt-cinq mille euros → 25 000 €, dix pour cent → 10 %, quatorze heures → 14 h, trois → 3, twenty-five → 25, ten percent → 10%\n3. Replace spoken punctuation with symbols (point/period → ., virgule/comma → ,, point d'interrogation/question mark → ?)\n4. Remove filler words (euh, um, uh, like as filler)\n5. Keep the language in the original version (if it was french, keep it in french for example)\n6. Never output two punctuation marks in a row (for example \".,\" or \",.\")\n\nPreserve exact meaning and word order. Do not paraphrase or reorder content.\n\nReturn only the cleaned transcript.\n\nTranscript:\n${output}".to_string(),
            target_language: None,
        },
        SmartMode {
            id: "mode_make_formal".to_string(),
            name: "Make Formal".to_string(),
            kind: SmartModeKind::Rewrite,
            prompt: "Rewrite this text in a formal, professional tone while preserving its exact meaning and language. Return only the rewritten text.\n\nText:\n${output}".to_string(),
            target_language: None,
        },
        SmartMode {
            id: "mode_make_casual".to_string(),
            name: "Make Casual".to_string(),
            kind: SmartModeKind::Rewrite,
            prompt: "Rewrite this text in a relaxed, casual, conversational tone while preserving its exact meaning and language. Return only the rewritten text.\n\nText:\n${output}".to_string(),
            target_language: None,
        },
        SmartMode {
            id: "mode_email".to_string(),
            name: "Write as Email".to_string(),
            kind: SmartModeKind::Rewrite,
            prompt: "Reformat this text as an email WITHOUT changing its tone, register, or wording more than necessary. Keep exactly the same level of familiarity or formality as the input: if it is casual and friendly, the email stays casual and friendly; if it is formal (vouvoiement), it stays formal. Apply ONLY email structure:\n- Start with exactly ONE greeting line, then a blank line. Use the same greeting word the user actually spoke and do not change it: if the text starts with \"salut\", the greeting must start with \"Salut\"; if it starts with \"bonjour\", it must start with \"Bonjour\". Add the recipient's name and fix capitalization. If the input has no greeting at all, use \"Bonjour,\". Never write a second greeting.\n- Then the message body, fixing capitalization and punctuation and splitting the run-on dictation into proper sentences (add periods and question marks where needed). Keep the user's own words and phrasing; do NOT make requests more polite and do NOT add words the user did not say (for example, never add \"s'il te plaît\" or \"please\").\n- Closing: ONLY if the input ends with a closing phrase (such as \"cordialement\", \"bien cordialement\", \"à plus\", \"bien à vous\", \"merci\") followed by a name, you MUST move it OUT of the body into a separate signature block: end the body sentence, then a blank line, then the closing word on its own line, then the name on the next line. Example — input body \"...votre email ? Bien cordialement, Pierre.\" must become:\n...votre email ?\n\nBien cordialement,\nPierre\nIf the input contains no such closing, do NOT add one.\nDo NOT add a subject line. Do NOT invent pleasantries such as \"I hope you are well\". Do not add, remove, or reword the actual content. Preserve the meaning and language. Return only the email.\n\nText:\n${output}".to_string(),
            target_language: None,
        },
        SmartMode {
            id: "mode_bullet_points".to_string(),
            name: "Bullet Points".to_string(),
            kind: SmartModeKind::Rewrite,
            prompt: "Restructure this text as a concise bulleted list, one idea per bullet, preserving meaning and language. Use a hyphen \"- \" as the bullet marker for every item. Return only the bullet list.\n\nText:\n${output}".to_string(),
            target_language: None,
        },
        SmartMode {
            id: "mode_summarize".to_string(),
            name: "Summarize".to_string(),
            kind: SmartModeKind::Rewrite,
            prompt: "Summarize this text concisely, preserving the key points and language. Return only the summary.\n\nText:\n${output}".to_string(),
            target_language: None,
        },
        SmartMode {
            id: "mode_translate_en".to_string(),
            name: "Translate \u{2192} English".to_string(),
            kind: SmartModeKind::Translation,
            prompt: String::new(),
            target_language: Some(TargetLanguage {
                code: "en".to_string(),
                label: "English".to_string(),
            }),
        },
        SmartMode {
            id: "mode_translate_es".to_string(),
            name: "Translate \u{2192} Spanish".to_string(),
            kind: SmartModeKind::Translation,
            prompt: String::new(),
            target_language: Some(TargetLanguage {
                code: "es".to_string(),
                label: "Spanish".to_string(),
            }),
        },
        SmartMode {
            id: "mode_translate_fr".to_string(),
            name: "Translate \u{2192} French".to_string(),
            kind: SmartModeKind::Translation,
            prompt: String::new(),
            target_language: Some(TargetLanguage {
                code: "fr".to_string(),
                label: "French".to_string(),
            }),
        },
        SmartMode {
            id: "mode_translate_zh".to_string(),
            name: "Translate \u{2192} Chinese".to_string(),
            kind: SmartModeKind::Translation,
            prompt: String::new(),
            target_language: Some(TargetLanguage {
                code: "zh".to_string(),
                label: "Chinese".to_string(),
            }),
        },
    ]
}

/// Returns the first-run seed: only Clean Up.
/// The full catalogue is available via `smart_mode_templates()`.
fn default_smart_modes() -> Vec<SmartMode> {
    smart_mode_templates()
        .into_iter()
        .filter(|m| m.id == CLEAN_UP_MODE_ID)
        .collect()
}

fn default_whisper_gpu_device() -> i32 {
    -1 // auto
}

fn default_typing_tool() -> TypingTool {
    TypingTool::Auto
}

fn ensure_post_process_defaults(settings: &mut AppSettings) -> bool {
    let mut changed = false;
    for provider in default_post_process_providers() {
        // Use match to do a single lookup - either sync existing or add new
        match settings
            .post_process_providers
            .iter_mut()
            .find(|p| p.id == provider.id)
        {
            Some(existing) => {
                // Sync supports_structured_output field for existing providers (migration)
                if existing.supports_structured_output != provider.supports_structured_output {
                    debug!(
                        "Updating supports_structured_output for provider '{}' from {} to {}",
                        provider.id,
                        existing.supports_structured_output,
                        provider.supports_structured_output
                    );
                    existing.supports_structured_output = provider.supports_structured_output;
                    changed = true;
                }
            }
            None => {
                // Provider doesn't exist, add it
                settings.post_process_providers.push(provider.clone());
                changed = true;
            }
        }

        use std::collections::hash_map::Entry;
        if let Entry::Vacant(e) = settings.post_process_api_keys.entry(provider.id.clone()) {
            e.insert(String::new());
            changed = true;
        }

        let default_model = default_model_for_provider(&provider.id);
        match settings.post_process_models.get_mut(&provider.id) {
            Some(existing) => {
                if existing.is_empty() && !default_model.is_empty() {
                    *existing = default_model.clone();
                    changed = true;
                }
            }
            None => {
                settings
                    .post_process_models
                    .insert(provider.id.clone(), default_model);
                changed = true;
            }
        }
    }

    changed
}

fn migrate_settings_if_needed(settings: &mut AppSettings) -> bool {
    const CURRENT_VERSION: u32 = 1;
    if settings.settings_schema_version >= CURRENT_VERSION {
        return false;
    }

    // Build new modes list atomically — do not mutate settings.smart_modes until we are done
    let mut new_modes: Vec<SmartMode> = default_smart_modes();
    let seeded_ids: HashSet<String> = new_modes.iter().map(|m| m.id.clone()).collect();

    const PRISTINE_IMPROVE_ID: &str = "default_improve_transcriptions";
    let pristine_prompt = default_post_process_prompts()
        .into_iter()
        .next()
        .map(|p| p.prompt)
        .unwrap_or_default();

    for p in &settings.post_process_prompts {
        if seeded_ids.contains(&p.id) {
            // Already seeded — skip to avoid duplicates
            continue;
        }
        if p.id == PRISTINE_IMPROVE_ID && p.prompt == pristine_prompt {
            // Unmodified "Improve Transcriptions" — replaced by Clean Up; drop it
            continue;
        }
        // Custom prompt or edited Improve Transcriptions — preserve as Rewrite mode
        new_modes.push(SmartMode {
            id: p.id.clone(),
            name: p.name.clone(),
            kind: SmartModeKind::Rewrite,
            prompt: p.prompt.clone(),
            target_language: None,
        });
    }

    // Determine active id
    let new_active_id: Option<String> = match &settings.post_process_selected_prompt_id {
        Some(sel) if sel == PRISTINE_IMPROVE_ID => {
            // Check if it was pristine (replaced by Clean Up) or edited (kept with same id)
            let was_pristine = settings
                .post_process_prompts
                .iter()
                .find(|p| &p.id == sel)
                .map(|p| p.prompt == pristine_prompt)
                .unwrap_or(true); // if not found, treat as pristine → Clean Up
            if was_pristine {
                Some(CLEAN_UP_MODE_ID.to_string())
            } else {
                // Edited version kept with original id
                Some(sel.clone())
            }
        }
        Some(sel) => {
            // Check if this id exists in the migrated modes
            if new_modes.iter().any(|m| &m.id == sel) {
                Some(sel.clone())
            } else {
                Some(CLEAN_UP_MODE_ID.to_string())
            }
        }
        None => Some(CLEAN_UP_MODE_ID.to_string()),
    };

    // Combo transfer: move transcribe_with_post_process binding to smart_mode_{active_id}
    if let Some(ref active_id) = new_active_id {
        if let Some(old_binding) = settings.bindings.remove("transcribe_with_post_process") {
            if !old_binding.current_binding.is_empty() {
                let active_name = new_modes
                    .iter()
                    .find(|m| &m.id == active_id)
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| "Smart Mode".to_string());
                let key = format!("smart_mode_{}", active_id);
                settings.bindings.insert(
                    key.clone(),
                    ShortcutBinding {
                        id: key,
                        name: active_name,
                        description: "Smart Mode shortcut".to_string(),
                        default_binding: old_binding.current_binding.clone(),
                        current_binding: old_binding.current_binding,
                    },
                );
            }
        }
    }

    // Atomic commit
    settings.smart_modes = new_modes;
    settings.smart_mode_active_id = new_active_id;
    settings.settings_schema_version = CURRENT_VERSION;

    true
}

/// Remove any `bindings["smart_mode_*"]` whose mode id is not present in
/// `smart_modes`. This prevents orphaned smart-mode shortcuts (e.g. from a
/// reduced seed or a deleted mode that crashed before cleanup) from registering
/// and firing invisibly after the next launch.
///
/// Returns `true` if any binding was removed (settings mutated).
/// Called from `load_or_create_app_settings` AFTER `migrate_settings_if_needed`
/// so it also catches seed-reduction orphans introduced by intermediary builds.
pub fn reconcile_dangling_smart_mode_bindings(settings: &mut AppSettings) -> bool {
    let live_ids: std::collections::HashSet<String> =
        settings.smart_modes.iter().map(|m| m.id.clone()).collect();
    let before = settings.bindings.len();
    settings.bindings.retain(|key, _| {
        match key.strip_prefix("smart_mode_") {
            Some(mode_id) => live_ids.contains(mode_id),
            None => true, // not a smart-mode binding — keep
        }
    });
    settings.bindings.len() != before
}

pub const SETTINGS_STORE_PATH: &str = "settings_store.json";

pub fn get_default_settings() -> AppSettings {
    #[cfg(target_os = "windows")]
    let default_shortcut = "ctrl+space";
    #[cfg(target_os = "macos")]
    let default_shortcut = "option+space";
    #[cfg(target_os = "linux")]
    let default_shortcut = "ctrl+space";
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let default_shortcut = "alt+space";

    let mut bindings = HashMap::new();
    bindings.insert(
        "transcribe".to_string(),
        ShortcutBinding {
            id: "transcribe".to_string(),
            name: "Transcribe".to_string(),
            description: "Converts your speech into text.".to_string(),
            default_binding: default_shortcut.to_string(),
            current_binding: default_shortcut.to_string(),
        },
    );
    bindings.insert(
        "cancel".to_string(),
        ShortcutBinding {
            id: "cancel".to_string(),
            name: "Cancel".to_string(),
            description: "Cancels the current recording.".to_string(),
            default_binding: "escape".to_string(),
            current_binding: "escape".to_string(),
        },
    );

    AppSettings {
        bindings,
        push_to_talk: true,
        audio_feedback: false,
        audio_feedback_volume: default_audio_feedback_volume(),
        sound_theme: default_sound_theme(),
        start_hidden: default_start_hidden(),
        autostart_enabled: default_autostart_enabled(),
        update_checks_enabled: default_update_checks_enabled(),
        selected_model: "".to_string(),
        always_on_microphone: false,
        selected_microphone: None,
        clamshell_microphone: None,
        selected_output_device: None,
        translate_to_english: false,
        selected_language: "auto".to_string(),
        overlay_position: default_overlay_position(),
        debug_mode: false,
        log_level: default_log_level(),
        custom_words: Vec::new(),
        model_unload_timeout: ModelUnloadTimeout::default(),
        word_correction_threshold: default_word_correction_threshold(),
        history_limit: default_history_limit(),
        recording_retention_period: default_recording_retention_period(),
        paste_method: PasteMethod::default(),
        clipboard_handling: ClipboardHandling::default(),
        auto_submit: default_auto_submit(),
        auto_submit_key: AutoSubmitKey::default(),
        post_process_enabled: default_post_process_enabled(),
        post_process_provider_id: default_post_process_provider_id(),
        post_process_providers: default_post_process_providers(),
        post_process_api_keys: default_post_process_api_keys(),
        post_process_models: default_post_process_models(),
        post_process_prompts: default_post_process_prompts(),
        post_process_selected_prompt_id: None,
        settings_schema_version: 1,
        smart_modes: default_smart_modes(),
        smart_mode_active_id: default_smart_modes().first().map(|m| m.id.clone()),
        mute_while_recording: false,
        append_trailing_space: false,
        app_language: default_app_language(),
        experimental_enabled: false,
        enable_cloud_providers: default_enable_cloud_providers(),
        lazy_stream_close: false,
        keyboard_implementation: KeyboardImplementation::default(),
        show_tray_icon: default_show_tray_icon(),
        paste_delay_ms: default_paste_delay_ms(),
        typing_tool: default_typing_tool(),
        external_script_path: None,
        custom_filler_words: None,
        whisper_accelerator: WhisperAcceleratorSetting::default(),
        ort_accelerator: OrtAcceleratorSetting::default(),
        whisper_gpu_device: default_whisper_gpu_device(),
        extra_recording_buffer_ms: 0,
        active_llm_model_id: None,
        llm_unload_timeout: ModelUnloadTimeout::default(),
    }
}

impl AppSettings {
    pub fn active_post_process_provider(&self) -> Option<&PostProcessProvider> {
        self.post_process_providers
            .iter()
            .find(|provider| provider.id == self.post_process_provider_id)
    }

    pub fn post_process_provider(&self, provider_id: &str) -> Option<&PostProcessProvider> {
        self.post_process_providers
            .iter()
            .find(|provider| provider.id == provider_id)
    }

    pub fn post_process_provider_mut(
        &mut self,
        provider_id: &str,
    ) -> Option<&mut PostProcessProvider> {
        self.post_process_providers
            .iter_mut()
            .find(|provider| provider.id == provider_id)
    }
}

pub fn load_or_create_app_settings(app: &AppHandle) -> AppSettings {
    // Initialize store
    let store = app
        .store(crate::portable::store_path(SETTINGS_STORE_PATH))
        .expect("Failed to initialize store");

    let mut settings = if let Some(settings_value) = store.get("settings") {
        // Parse the entire settings object
        match serde_json::from_value::<AppSettings>(settings_value) {
            Ok(mut settings) => {
                debug!("Found existing settings: {:?}", settings);
                let default_settings = get_default_settings();
                let mut updated = false;

                // Merge default bindings into existing settings
                for (key, value) in default_settings.bindings {
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        settings.bindings.entry(key.clone())
                    {
                        debug!("Adding missing binding: {}", key);
                        e.insert(value);
                        updated = true;
                    }
                }

                if updated {
                    debug!("Settings updated with new bindings");
                    store.set("settings", serde_json::to_value(&settings).unwrap());
                }

                settings
            }
            Err(e) => {
                warn!("Failed to parse settings: {}", e);
                // Fall back to default settings if parsing fails
                let default_settings = get_default_settings();
                store.set("settings", serde_json::to_value(&default_settings).unwrap());
                default_settings
            }
        }
    } else {
        let default_settings = get_default_settings();
        store.set("settings", serde_json::to_value(&default_settings).unwrap());
        default_settings
    };

    if ensure_post_process_defaults(&mut settings) {
        store.set("settings", serde_json::to_value(&settings).unwrap());
    }

    let migrated = migrate_settings_if_needed(&mut settings);
    // Run reconciliation after migration so seed-reduction orphans and any
    // binding left by a pre-cleanup build are removed before init_shortcuts
    // iterates bindings. If either step mutated settings, persist once.
    let reconciled = reconcile_dangling_smart_mode_bindings(&mut settings);
    if migrated || reconciled {
        store.set("settings", serde_json::to_value(&settings).unwrap());
    }

    settings
}

pub fn get_settings(app: &AppHandle) -> AppSettings {
    let store = app
        .store(crate::portable::store_path(SETTINGS_STORE_PATH))
        .expect("Failed to initialize store");

    let mut settings = if let Some(settings_value) = store.get("settings") {
        serde_json::from_value::<AppSettings>(settings_value).unwrap_or_else(|_| {
            let default_settings = get_default_settings();
            store.set("settings", serde_json::to_value(&default_settings).unwrap());
            default_settings
        })
    } else {
        let default_settings = get_default_settings();
        store.set("settings", serde_json::to_value(&default_settings).unwrap());
        default_settings
    };

    if ensure_post_process_defaults(&mut settings) {
        store.set("settings", serde_json::to_value(&settings).unwrap());
    }

    if migrate_settings_if_needed(&mut settings) {
        store.set("settings", serde_json::to_value(&settings).unwrap());
    }

    settings
}

pub fn write_settings(app: &AppHandle, settings: AppSettings) {
    let store = app
        .store(crate::portable::store_path(SETTINGS_STORE_PATH))
        .expect("Failed to initialize store");

    store.set("settings", serde_json::to_value(&settings).unwrap());
}

pub fn get_bindings(app: &AppHandle) -> HashMap<String, ShortcutBinding> {
    let settings = get_settings(app);

    settings.bindings
}

pub fn get_stored_binding(app: &AppHandle, id: &str) -> ShortcutBinding {
    let bindings = get_bindings(app);

    let binding = bindings.get(id).unwrap().clone();

    binding
}

pub fn get_history_limit(app: &AppHandle) -> usize {
    let settings = get_settings(app);
    settings.history_limit
}

pub fn get_recording_retention_period(app: &AppHandle) -> RecordingRetentionPeriod {
    let settings = get_settings(app);
    settings.recording_retention_period
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_disable_auto_submit() {
        let settings = get_default_settings();
        assert!(!settings.auto_submit);
        assert_eq!(settings.auto_submit_key, AutoSubmitKey::Enter);
    }

    #[test]
    fn debug_output_redacts_api_keys() {
        let mut settings = get_default_settings();
        settings
            .post_process_api_keys
            .insert("openai".to_string(), "sk-proj-secret-key-12345".to_string());
        settings.post_process_api_keys.insert(
            "anthropic".to_string(),
            "sk-ant-secret-key-67890".to_string(),
        );
        settings
            .post_process_api_keys
            .insert("empty_provider".to_string(), "".to_string());

        let debug_output = format!("{:?}", settings);

        assert!(!debug_output.contains("sk-proj-secret-key-12345"));
        assert!(!debug_output.contains("sk-ant-secret-key-67890"));
        assert!(debug_output.contains("[REDACTED]"));
    }

    #[test]
    fn secret_map_debug_redacts_values() {
        let map = SecretMap(HashMap::from([("key".into(), "secret".into())]));
        let out = format!("{:?}", map);
        assert!(!out.contains("secret"));
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn default_post_process_provider_id_returns_apple_on_macos_arm64() {
        let id = default_post_process_provider_id();
        if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            assert_eq!(id, APPLE_INTELLIGENCE_PROVIDER_ID);
        } else {
            assert_eq!(id, "custom");
        }
    }

    #[test]
    fn default_post_process_providers_includes_custom_with_stable_id() {
        let providers = default_post_process_providers();
        let custom = providers
            .iter()
            .find(|p| p.id == "custom")
            .expect("custom provider must exist");
        assert_eq!(custom.id, "custom", "Persisted id MUST remain stable");
        assert_eq!(custom.label, "Custom (local)");
        assert_eq!(custom.base_url, "http://localhost:11434/v1");
    }

    #[test]
    fn default_enable_cloud_providers_is_false() {
        assert!(!default_enable_cloud_providers());
    }

    #[test]
    fn default_settings_have_cloud_providers_disabled() {
        let settings = get_default_settings();
        assert!(!settings.enable_cloud_providers);
    }

    // ── Smart Modes defaults tests ────────────────────────────────────────────

    #[test]
    fn smart_mode_templates_count_is_ten() {
        assert_eq!(smart_mode_templates().len(), 10);
    }

    #[test]
    fn default_smart_modes_seeds_only_clean_up() {
        let modes = default_smart_modes();
        assert_eq!(modes.len(), 1, "first-run seed must be exactly 1 mode");
        assert_eq!(
            modes[0].id, CLEAN_UP_MODE_ID,
            "first-run seed must be Clean Up"
        );
    }

    #[test]
    fn default_smart_modes_order() {
        let modes = smart_mode_templates();
        assert_eq!(modes[0].name, "Clean Up");
        for (i, mode) in modes.iter().enumerate().take(6) {
            assert_eq!(
                mode.kind,
                SmartModeKind::Rewrite,
                "index {} should be Rewrite",
                i
            );
        }
        for (i, mode) in modes.iter().enumerate().skip(6) {
            assert_eq!(
                mode.kind,
                SmartModeKind::Translation,
                "index {} should be Translation",
                i
            );
        }
    }

    #[test]
    fn default_translation_modes_have_target_language() {
        let modes = smart_mode_templates();
        let expected_codes = ["en", "es", "fr", "zh"];
        for (i, code) in expected_codes.iter().enumerate() {
            let mode = &modes[6 + i];
            assert_eq!(mode.kind, SmartModeKind::Translation);
            let tl = mode
                .target_language
                .as_ref()
                .expect("translation mode must have target_language");
            assert_eq!(&tl.code, code, "index {} code mismatch", 6 + i);
            assert!(!tl.label.is_empty(), "index {} label is empty", 6 + i);
        }
    }

    #[test]
    fn smart_mode_kind_serializes_snake_case() {
        let rewrite = serde_json::to_string(&SmartModeKind::Rewrite).unwrap();
        let translation = serde_json::to_string(&SmartModeKind::Translation).unwrap();
        assert_eq!(rewrite, "\"rewrite\"");
        assert_eq!(translation, "\"translation\"");
    }

    #[test]
    fn v12_settings_deserializes_without_smart_modes_fields() {
        // A minimal v1.2-shaped JSON lacking the new fields should deserialize without error.
        let v12_json = serde_json::json!({
            "bindings": {},
            "push_to_talk": true,
            "audio_feedback": false,
            "selected_model": "",
            "always_on_microphone": false,
            "translate_to_english": false,
            "selected_language": "auto",
            "overlay_position": "none",
            "debug_mode": false,
            "log_level": "info",
            "post_process_enabled": false,
            "post_process_provider_id": "custom",
            "post_process_providers": [],
            "post_process_api_keys": {},
            "post_process_models": {},
            "post_process_prompts": [],
            "external_script_path": null
        });
        let settings: AppSettings = serde_json::from_value(v12_json).unwrap();
        assert_eq!(settings.settings_schema_version, 0);
        assert!(settings.smart_mode_active_id.is_none());
        // smart_modes gets the default via serde default — now only Clean Up
        assert_eq!(settings.smart_modes.len(), 1);
    }

    // ── Migration tests ───────────────────────────────────────────────────────

    fn make_v12_fixture(
        prompt_id: &str,
        prompt_name: &str,
        prompt_text: &str,
        selected_id: Option<&str>,
        post_process_combo: &str,
    ) -> AppSettings {
        let v12_json = serde_json::json!({
            "bindings": {
                "transcribe": {
                    "id": "transcribe",
                    "name": "Transcribe",
                    "description": "Converts your speech into text.",
                    "default_binding": "option+space",
                    "current_binding": "option+space"
                },
                "transcribe_with_post_process": {
                    "id": "transcribe_with_post_process",
                    "name": "Transcribe with Post-Processing",
                    "description": "Converts your speech into text and applies AI post-processing.",
                    "default_binding": post_process_combo,
                    "current_binding": post_process_combo
                },
                "cancel": {
                    "id": "cancel",
                    "name": "Cancel",
                    "description": "Cancels the current recording.",
                    "default_binding": "escape",
                    "current_binding": "escape"
                }
            },
            "push_to_talk": true,
            "audio_feedback": false,
            "selected_model": "",
            "always_on_microphone": false,
            "translate_to_english": false,
            "selected_language": "auto",
            "overlay_position": "none",
            "debug_mode": false,
            "log_level": "info",
            "post_process_enabled": true,
            "post_process_provider_id": "custom",
            "post_process_providers": [],
            "post_process_api_keys": {},
            "post_process_models": {},
            "post_process_prompts": [
                {
                    "id": prompt_id,
                    "name": prompt_name,
                    "prompt": prompt_text
                }
            ],
            "post_process_selected_prompt_id": selected_id,
            "external_script_path": null
            // NOTE: no settings_schema_version → deserializes as 0
        });
        serde_json::from_value(v12_json).expect("v12 fixture must deserialize")
    }

    #[test]
    fn migration_v12_to_v13_preserves_custom_prompt() {
        let mut settings = make_v12_fixture(
            "prompt_custom_123",
            "My Custom Prompt",
            "Do something with: ${output}",
            Some("prompt_custom_123"),
            "option+shift+space",
        );
        assert_eq!(settings.settings_schema_version, 0);

        let migrated = migrate_settings_if_needed(&mut settings);
        assert!(migrated, "first call must return true");

        // Custom mode preserved
        assert!(
            settings
                .smart_modes
                .iter()
                .any(|m| m.id == "prompt_custom_123" && m.name == "My Custom Prompt"),
            "custom prompt must be preserved as a SmartMode"
        );
        // Clean Up exists
        assert!(
            settings
                .smart_modes
                .iter()
                .any(|m| m.id == CLEAN_UP_MODE_ID),
            "Clean Up must exist"
        );
        // Active id is set
        assert!(settings.smart_mode_active_id.is_some());
        assert_eq!(settings.settings_schema_version, 1);
    }

    #[test]
    fn migration_v12_pristine_improve_transcriptions_replaced_by_clean_up() {
        let pristine_prompt = default_post_process_prompts()
            .into_iter()
            .next()
            .unwrap()
            .prompt;
        let mut settings = make_v12_fixture(
            "default_improve_transcriptions",
            "Improve Transcriptions",
            &pristine_prompt,
            Some("default_improve_transcriptions"),
            "option+shift+space",
        );

        migrate_settings_if_needed(&mut settings);

        // No mode named "Improve Transcriptions"
        assert!(
            !settings
                .smart_modes
                .iter()
                .any(|m| m.name == "Improve Transcriptions"),
            "pristine Improve Transcriptions must not survive migration"
        );
        // Clean Up exists
        assert!(
            settings
                .smart_modes
                .iter()
                .any(|m| m.id == CLEAN_UP_MODE_ID),
            "Clean Up must exist"
        );
        // Active id points to Clean Up
        assert_eq!(
            settings.smart_mode_active_id.as_deref(),
            Some(CLEAN_UP_MODE_ID)
        );
    }

    #[test]
    fn migration_v12_edited_improve_transcriptions_kept_plus_clean_up() {
        let mut settings = make_v12_fixture(
            "default_improve_transcriptions",
            "Improve Transcriptions",
            "My edited prompt: ${output}", // different from pristine
            Some("default_improve_transcriptions"),
            "option+shift+space",
        );

        migrate_settings_if_needed(&mut settings);

        // The edited mode is preserved (id = "default_improve_transcriptions")
        assert!(
            settings
                .smart_modes
                .iter()
                .any(|m| m.id == "default_improve_transcriptions"),
            "edited Improve Transcriptions mode must be preserved"
        );
        // Clean Up also exists
        assert!(
            settings
                .smart_modes
                .iter()
                .any(|m| m.id == CLEAN_UP_MODE_ID),
            "Clean Up must also exist"
        );
    }

    #[test]
    fn migration_idempotent() {
        let mut settings = make_v12_fixture(
            "prompt_custom_123",
            "My Custom Prompt",
            "Do something with: ${output}",
            Some("prompt_custom_123"),
            "option+shift+space",
        );

        let first = migrate_settings_if_needed(&mut settings);
        assert!(first, "first call must return true");

        let len_after_first = settings.smart_modes.len();
        let second = migrate_settings_if_needed(&mut settings);
        assert!(!second, "second call must return false");
        assert_eq!(
            settings.smart_modes.len(),
            len_after_first,
            "smart_modes must not grow on second call"
        );
    }

    #[test]
    fn migration_stamps_version() {
        let mut settings = make_v12_fixture(
            "default_improve_transcriptions",
            "Improve Transcriptions",
            &default_post_process_prompts()[0].prompt,
            None,
            "option+shift+space",
        );
        assert_eq!(settings.settings_schema_version, 0);
        migrate_settings_if_needed(&mut settings);
        assert_eq!(settings.settings_schema_version, 1);
    }

    #[test]
    fn migration_active_selection() {
        let mut settings = make_v12_fixture(
            "prompt_custom_123",
            "My Custom Prompt",
            "Do something: ${output}",
            Some("prompt_custom_123"),
            "option+shift+space",
        );
        migrate_settings_if_needed(&mut settings);
        // The selected prompt id was "prompt_custom_123" — that mode is migrated with same id
        assert_eq!(
            settings.smart_mode_active_id.as_deref(),
            Some("prompt_custom_123")
        );
    }

    #[test]
    fn migration_transfers_post_process_combo() {
        let combo = "option+shift+space";
        let mut settings = make_v12_fixture(
            "default_improve_transcriptions",
            "Improve Transcriptions",
            &default_post_process_prompts()[0].prompt,
            Some("default_improve_transcriptions"),
            combo,
        );
        migrate_settings_if_needed(&mut settings);

        // transcribe_with_post_process must be removed
        assert!(
            !settings
                .bindings
                .contains_key("transcribe_with_post_process"),
            "transcribe_with_post_process binding must be retired after migration"
        );
        // A smart_mode_ prefixed key must exist with the transferred combo
        let active_id = settings.smart_mode_active_id.as_deref().unwrap();
        let key = format!("smart_mode_{}", active_id);
        let binding = settings
            .bindings
            .get(&key)
            .expect("smart_mode_ binding must exist after migration");
        assert_eq!(
            binding.current_binding, combo,
            "transferred combo must match original"
        );
    }

    // ── reconcile_dangling_smart_mode_bindings tests ─────────────────────────

    fn make_settings_with_modes_and_bindings(
        modes: Vec<SmartMode>,
        binding_keys: &[&str],
    ) -> AppSettings {
        let mut settings = get_default_settings();
        settings.smart_modes = modes;
        for key in binding_keys {
            settings.bindings.insert(
                key.to_string(),
                ShortcutBinding {
                    id: key.to_string(),
                    name: key.to_string(),
                    description: String::new(),
                    default_binding: String::new(),
                    current_binding: "option+shift+x".to_string(),
                },
            );
        }
        settings
    }

    #[test]
    fn reconcile_removes_orphaned_smart_mode_binding() {
        // "ghost" has no corresponding mode in smart_modes
        let modes = vec![SmartMode {
            id: "mode_clean_up".to_string(),
            name: "Clean Up".to_string(),
            kind: SmartModeKind::Rewrite,
            prompt: "p".to_string(),
            target_language: None,
        }];
        let mut settings = make_settings_with_modes_and_bindings(
            modes,
            &["smart_mode_ghost", "smart_mode_mode_clean_up"],
        );

        let changed = reconcile_dangling_smart_mode_bindings(&mut settings);

        assert!(changed, "must return true when a binding was removed");
        assert!(
            !settings.bindings.contains_key("smart_mode_ghost"),
            "orphaned smart_mode_ghost must be removed"
        );
        assert!(
            settings.bindings.contains_key("smart_mode_mode_clean_up"),
            "live smart_mode_mode_clean_up must be kept"
        );
    }

    #[test]
    fn reconcile_keeps_non_smart_mode_bindings() {
        let modes = vec![SmartMode {
            id: "mode_clean_up".to_string(),
            name: "Clean Up".to_string(),
            kind: SmartModeKind::Rewrite,
            prompt: "p".to_string(),
            target_language: None,
        }];
        let mut settings = make_settings_with_modes_and_bindings(
            modes,
            &["transcribe", "smart_mode_mode_clean_up"],
        );

        let changed = reconcile_dangling_smart_mode_bindings(&mut settings);

        assert!(!changed, "must return false when nothing was removed");
        assert!(
            settings.bindings.contains_key("transcribe"),
            "non-smart-mode binding must be kept"
        );
        assert!(
            settings.bindings.contains_key("smart_mode_mode_clean_up"),
            "live smart_mode binding must be kept"
        );
    }

    #[test]
    fn reconcile_returns_false_when_nothing_to_remove() {
        let modes = vec![SmartMode {
            id: "mode_clean_up".to_string(),
            name: "Clean Up".to_string(),
            kind: SmartModeKind::Rewrite,
            prompt: "p".to_string(),
            target_language: None,
        }];
        let mut settings = make_settings_with_modes_and_bindings(modes, &["transcribe"]);

        let changed = reconcile_dangling_smart_mode_bindings(&mut settings);
        assert!(
            !changed,
            "must return false when no smart_mode_* bindings present"
        );
    }
}

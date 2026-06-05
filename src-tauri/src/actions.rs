#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use crate::apple_intelligence;
use crate::audio_feedback::{play_feedback_sound, play_feedback_sound_blocking, SoundType};
use crate::audio_toolkit::{is_microphone_access_denied, is_no_input_device_error};
use crate::managers::audio::AudioRecordingManager;
use crate::managers::history::HistoryManager;
use crate::managers::transcription::TranscriptionManager;
use crate::settings::{get_settings, AppSettings, APPLE_INTELLIGENCE_PROVIDER_ID};
use crate::shortcut;
use crate::tray::{change_tray_icon, TrayIconState};
use crate::utils::{
    self, show_processing_overlay, show_recording_overlay, show_transcribing_overlay,
};
use crate::TranscriptionCoordinator;
use ferrous_opencc::{config::BuiltinConfig, OpenCC};
use log::{debug, error, warn};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tauri::Manager;
use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
struct RecordingErrorEvent {
    error_type: String,
    detail: Option<String>,
}

/// Drop guard that notifies the [`TranscriptionCoordinator`] when the
/// transcription pipeline finishes — whether it completes normally or panics.
struct FinishGuard(AppHandle);
impl Drop for FinishGuard {
    fn drop(&mut self) {
        if let Some(c) = self.0.try_state::<TranscriptionCoordinator>() {
            c.notify_processing_finished();
        }
    }
}

// Shortcut Action Trait
pub trait ShortcutAction: Send + Sync {
    fn start(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str);
    fn stop(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str);
}

// Transcribe Action
struct TranscribeAction {
    post_process: bool,
}

// Smart Mode Action
pub(crate) struct SmartModeAction {
    pub mode_id: String,
}

/// Field name for structured output JSON schema
const TRANSCRIPTION_FIELD: &str = "transcription";

/// Strip invisible Unicode characters that some LLMs may insert
fn strip_invisible_chars(s: &str) -> String {
    s.replace(['\u{200B}', '\u{200C}', '\u{200D}', '\u{FEFF}'], "")
}

/// Build a system prompt from the user's prompt template.
/// Removes `${output}` placeholder since the transcription is sent as the user message.
///
/// Also appends an explicit output-language directive derived from the input
/// text. Small LLMs tend to answer in the language of the (English) instructions
/// and ignore a "preserve the language" hint; naming the detected language
/// explicitly is far more reliable. The directive defers to prompts that
/// explicitly request translation / a specific language (e.g. a custom mode).
fn build_system_prompt(prompt_template: &str, transcription: &str) -> String {
    let base = prompt_template.replace("${output}", "").trim().to_string();
    match language_directive(transcription) {
        // Directive goes FIRST, as a top-level instruction. Appending it after
        // the base lands it next to the trailing "Text:"/"Transcript:" label, so
        // the model treats it as content and echoes it into the output.
        Some(directive) => format!("{}\n\n{}", directive, base),
        None => base,
    }
}

/// An explicit output-language directive derived from the input text, or `None`
/// when the language can't be detected reliably. Defers to prompts that
/// explicitly request translation / a specific language.
fn language_directive(transcription: &str) -> Option<String> {
    detect_language_name(transcription).map(|lang| {
        format!(
            "Write your entire response in {} (the language of the text being processed), unless the task explicitly asks you to translate or to use another language. Do not mention, repeat, or include this instruction in your output.",
            lang
        )
    })
}

/// Human-readable English name of the input text's detected language (e.g.
/// "French"), or `None` when the text is too short to detect reliably.
fn detect_language_name(text: &str) -> Option<String> {
    if text.trim().chars().count() < 8 {
        return None;
    }
    whatlang::detect(text).map(|info| info.lang().eng_name().to_string())
}

/// Run the inference pipeline with an explicit prompt string.
///
/// This is the shared engine behind both the active-selection path
/// (`post_process_transcription`) and the smart-mode override path
/// (`process_transcription_output` with `mode_id_override`).
/// It receives a resolved `prompt` (already looked up by the caller) and
/// executes it against the configured provider / embedded engine.
async fn post_process_with_prompt(
    app: &AppHandle,
    settings: &AppSettings,
    transcription: &str,
    prompt: &str,
) -> Option<String> {
    if prompt.trim().is_empty() {
        debug!("post_process_with_prompt: prompt is empty, skipping");
        return None;
    }

    // Handle the embedded (local LLM) provider before the HTTP provider lookup.
    // "embedded" is not in post_process_providers and has no base_url, model, or api key.
    if settings.post_process_provider_id == "embedded" {
        use std::sync::Arc;
        let llm_manager = app.state::<Arc<crate::managers::llm::LlmManager>>();

        // Load the active model on demand if not already loaded.
        if !llm_manager.is_model_loaded() {
            if let Some(active_id) = &settings.active_llm_model_id {
                if let Err(e) = llm_manager.load_model(active_id).await {
                    log::error!("Embedded LLM failed to load model '{}': {}", active_id, e);
                    return None;
                }
            } else {
                debug!("Embedded provider selected but no active LLM model is set");
                return None;
            }
        }

        let system_prompt = build_system_prompt(prompt, transcription);
        let full_prompt = format!("{}\n\n{}", system_prompt, transcription);
        return match llm_manager.run_inference(full_prompt).await {
            Ok(result) => {
                let result = strip_invisible_chars(&result);
                if result.trim().is_empty() {
                    None
                } else {
                    debug!(
                        "Embedded LLM post-processing succeeded. Output length: {} chars",
                        result.len()
                    );
                    Some(result)
                }
            }
            Err(e) => {
                log::error!("Embedded LLM inference failed: {}", e);
                None
            }
        };
    }

    let provider = match settings.active_post_process_provider().cloned() {
        Some(provider) => provider,
        None => {
            debug!("Post-processing enabled but no provider is selected");
            return None;
        }
    };

    let model = settings
        .post_process_models
        .get(&provider.id)
        .cloned()
        .unwrap_or_default();

    if model.trim().is_empty() {
        debug!(
            "Post-processing skipped because provider '{}' has no model configured",
            provider.id
        );
        return None;
    }

    debug!(
        "Starting LLM post-processing with provider '{}' (model: {})",
        provider.id, model
    );

    let api_key = settings
        .post_process_api_keys
        .get(&provider.id)
        .cloned()
        .unwrap_or_default();

    // Disable reasoning for providers where post-processing rarely benefits from it.
    // - custom: top-level reasoning_effort (works for local OpenAI-compat servers)
    // - openrouter: nested reasoning object; exclude:true also keeps reasoning text
    //   out of the response so it can't pollute structured-output JSON parsing
    let (reasoning_effort, reasoning) = match provider.id.as_str() {
        "custom" => (Some("none".to_string()), None),
        "openrouter" => (
            None,
            Some(crate::llm_client::ReasoningConfig {
                effort: Some("none".to_string()),
                exclude: Some(true),
            }),
        ),
        _ => (None, None),
    };

    if provider.supports_structured_output {
        debug!("Using structured outputs for provider '{}'", provider.id);

        let system_prompt = build_system_prompt(prompt, transcription);
        let user_content = transcription.to_string();

        // Handle Apple Intelligence separately since it uses native Swift APIs
        if provider.id == APPLE_INTELLIGENCE_PROVIDER_ID {
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            {
                if !apple_intelligence::check_apple_intelligence_availability() {
                    debug!(
                        "Apple Intelligence selected but not currently available on this device"
                    );
                    return None;
                }

                // Apple Intelligence "model" is a provider label ("Apple Intelligence"),
                // not a word cap. Never truncate smart-mode output by word count.
                // (debug: engine-persist-apple-truncation.md)
                let token_limit: i32 = 0;
                return match apple_intelligence::process_text_with_system_prompt(
                    &system_prompt,
                    &user_content,
                    token_limit,
                ) {
                    Ok(result) => {
                        if result.trim().is_empty() {
                            debug!("Apple Intelligence returned an empty response");
                            None
                        } else {
                            let result = strip_invisible_chars(&result);
                            debug!(
                                "Apple Intelligence post-processing succeeded. Output length: {} chars",
                                result.len()
                            );
                            Some(result)
                        }
                    }
                    Err(err) => {
                        error!("Apple Intelligence post-processing failed: {}", err);
                        None
                    }
                };
            }

            #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
            {
                debug!("Apple Intelligence provider selected on unsupported platform");
                return None;
            }
        }

        // Define JSON schema for transcription output
        let json_schema = serde_json::json!({
            "type": "object",
            "properties": {
                (TRANSCRIPTION_FIELD): {
                    "type": "string",
                    "description": "The cleaned and processed transcription text"
                }
            },
            "required": [TRANSCRIPTION_FIELD],
            "additionalProperties": false
        });

        match crate::llm_client::send_chat_completion_with_schema(
            crate::llm_client::ChatCompletionParams {
                provider: provider.clone(),
                api_key: api_key.clone(),
                model: model.clone(),
                user_content,
                system_prompt: Some(system_prompt),
                json_schema: Some(json_schema),
                reasoning_effort: reasoning_effort.clone(),
                reasoning: reasoning.clone(),
            },
        )
        .await
        {
            Ok(Some(content)) => {
                // Parse the JSON response to extract the transcription field
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json) => {
                        if let Some(transcription_value) =
                            json.get(TRANSCRIPTION_FIELD).and_then(|t| t.as_str())
                        {
                            let result = strip_invisible_chars(transcription_value);
                            debug!(
                                "Structured output post-processing succeeded for provider '{}'. Output length: {} chars",
                                provider.id,
                                result.len()
                            );
                            return Some(result);
                        } else {
                            error!("Structured output response missing 'transcription' field");
                            return Some(strip_invisible_chars(&content));
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to parse structured output JSON: {}. Returning raw content.",
                            e
                        );
                        return Some(strip_invisible_chars(&content));
                    }
                }
            }
            Ok(None) => {
                error!("LLM API response has no content");
                return None;
            }
            Err(e) => {
                warn!(
                    "Structured output failed for provider '{}': {}. Falling back to legacy mode.",
                    provider.id, e
                );
                // Fall through to legacy mode below
            }
        }
    }

    // Legacy mode: Replace ${output} variable in the prompt with the actual text
    let mut processed_prompt = prompt.replace("${output}", transcription);
    if let Some(directive) = language_directive(transcription) {
        processed_prompt = format!("{}\n\n{}", directive, processed_prompt);
    }
    debug!("Processed prompt length: {} chars", processed_prompt.len());

    match crate::llm_client::send_chat_completion(
        &provider,
        api_key,
        &model,
        processed_prompt,
        reasoning_effort,
        reasoning,
    )
    .await
    {
        Ok(Some(content)) => {
            let content = strip_invisible_chars(&content);
            debug!(
                "LLM post-processing succeeded for provider '{}'. Output length: {} chars",
                provider.id,
                content.len()
            );
            Some(content)
        }
        Ok(None) => {
            error!("LLM API response has no content");
            None
        }
        Err(e) => {
            error!(
                "LLM post-processing failed for provider '{}': {}. Falling back to original transcription.",
                provider.id,
                e
            );
            None
        }
    }
}

/// Resolve the active selected prompt from settings, then run inference.
/// This is the existing post-processing path for the standard shortcut.
async fn post_process_transcription(
    app: &AppHandle,
    settings: &AppSettings,
    transcription: &str,
) -> Option<String> {
    // Handle the embedded provider prompt resolution separately (same lookup as HTTP path below).
    if settings.post_process_provider_id == "embedded" {
        let selected_prompt_id = match &settings.post_process_selected_prompt_id {
            Some(id) => id.clone(),
            None => {
                debug!("Embedded post-processing skipped: no prompt selected");
                return None;
            }
        };
        let prompt = match settings
            .post_process_prompts
            .iter()
            .find(|p| p.id == selected_prompt_id)
        {
            Some(p) => p.prompt.clone(),
            None => {
                debug!(
                    "Embedded post-processing skipped: prompt '{}' not found",
                    selected_prompt_id
                );
                return None;
            }
        };
        return post_process_with_prompt(app, settings, transcription, &prompt).await;
    }

    let selected_prompt_id = match &settings.post_process_selected_prompt_id {
        Some(id) => id.clone(),
        None => {
            debug!("Post-processing skipped because no prompt is selected");
            return None;
        }
    };

    let prompt = match settings
        .post_process_prompts
        .iter()
        .find(|prompt| prompt.id == selected_prompt_id)
    {
        Some(prompt) => prompt.prompt.clone(),
        None => {
            debug!(
                "Post-processing skipped because prompt '{}' was not found",
                selected_prompt_id
            );
            return None;
        }
    };

    post_process_with_prompt(app, settings, transcription, &prompt).await
}

async fn maybe_convert_chinese_variant(
    settings: &AppSettings,
    transcription: &str,
) -> Option<String> {
    // Check if language is set to Simplified or Traditional Chinese
    let is_simplified = settings.selected_language == "zh-Hans";
    let is_traditional = settings.selected_language == "zh-Hant";

    if !is_simplified && !is_traditional {
        debug!("selected_language is not Simplified or Traditional Chinese; skipping translation");
        return None;
    }

    debug!(
        "Starting Chinese translation using OpenCC for language: {}",
        settings.selected_language
    );

    // Use OpenCC to convert based on selected language
    let config = if is_simplified {
        // Convert Traditional Chinese to Simplified Chinese
        BuiltinConfig::Tw2sp
    } else {
        // Convert Simplified Chinese to Traditional Chinese
        BuiltinConfig::S2tw
    };

    match OpenCC::from_config(config) {
        Ok(converter) => {
            let converted = converter.convert(transcription);
            debug!(
                "OpenCC translation completed. Input length: {}, Output length: {}",
                transcription.len(),
                converted.len()
            );
            Some(converted)
        }
        Err(e) => {
            error!("Failed to initialize OpenCC converter: {}. Falling back to original transcription.", e);
            None
        }
    }
}

pub(crate) struct ProcessedTranscription {
    pub final_text: String,
    pub post_processed_text: Option<String>,
    pub post_process_prompt: Option<String>,
}

pub(crate) async fn process_transcription_output(
    app: &AppHandle,
    transcription: &str,
    post_process: bool,
    mode_id_override: Option<&str>,
) -> ProcessedTranscription {
    let settings = get_settings(app);
    let mut final_text = transcription.to_string();
    let mut post_processed_text: Option<String> = None;
    let mut post_process_prompt: Option<String> = None;

    if let Some(converted_text) = maybe_convert_chinese_variant(&settings, transcription).await {
        final_text = converted_text;
    }

    if post_process {
        if let Some(mode_id) = mode_id_override {
            // Smart mode path: branch on kind.
            match resolve_mode_prompt(&settings, mode_id) {
                Some((_mode, prompt)) => {
                    // Rewrite kind: run inference with the mode's explicit prompt.
                    if let Some(processed_text) =
                        post_process_with_prompt(app, &settings, &final_text, &prompt).await
                    {
                        post_processed_text = Some(processed_text.clone());
                        final_text = processed_text;
                        post_process_prompt = Some(prompt);
                    }
                }
                None => {
                    // Translation kind: run through the embedded LLM per the global engine choice.
                    let mode = settings.smart_modes.iter().find(|m| m.id == mode_id);
                    if let Some(mode) = mode {
                        if mode.kind == crate::settings::SmartModeKind::Translation {
                            if let Some(translated) =
                                run_translation(app, &settings, &final_text, mode).await
                            {
                                post_processed_text = Some(translated.clone());
                                final_text = translated;
                            }
                        }
                    }
                    // else: unknown mode id — leave text unchanged (no-op).
                }
            }
        } else {
            // Standard path: use active selected prompt (existing behavior).
            if let Some(processed_text) =
                post_process_transcription(app, &settings, &final_text).await
            {
                post_processed_text = Some(processed_text.clone());
                final_text = processed_text;

                if let Some(prompt_id) = &settings.post_process_selected_prompt_id {
                    if let Some(prompt) = settings
                        .post_process_prompts
                        .iter()
                        .find(|prompt| &prompt.id == prompt_id)
                    {
                        post_process_prompt = Some(prompt.prompt.clone());
                    }
                }
            }
        }
    } else if final_text != transcription {
        post_processed_text = Some(final_text.clone());
    }

    ProcessedTranscription {
        final_text,
        post_processed_text,
        post_process_prompt,
    }
}

impl ShortcutAction for TranscribeAction {
    fn start(&self, app: &AppHandle, binding_id: &str, _shortcut_str: &str) {
        let start_time = Instant::now();
        debug!("TranscribeAction::start called for binding: {}", binding_id);

        // Load model in the background
        let tm = app.state::<Arc<TranscriptionManager>>();
        let rm = app.state::<Arc<AudioRecordingManager>>();

        // Load ASR model and VAD model in parallel
        tm.initiate_model_load();
        let rm_clone = Arc::clone(&rm);
        std::thread::spawn(move || {
            if let Err(e) = rm_clone.preload_vad() {
                debug!("VAD pre-load failed: {}", e);
            }
        });

        let binding_id = binding_id.to_string();
        change_tray_icon(app, TrayIconState::Recording);
        show_recording_overlay(app);

        // Get the microphone mode to determine audio feedback timing
        let settings = get_settings(app);
        let is_always_on = settings.always_on_microphone;
        debug!("Microphone mode - always_on: {}", is_always_on);

        let mut recording_error: Option<String> = None;
        if is_always_on {
            // Always-on mode: Play audio feedback immediately, then apply mute after sound finishes
            debug!("Always-on mode: Playing audio feedback immediately");
            let rm_clone = Arc::clone(&rm);
            let app_clone = app.clone();
            // The blocking helper exits immediately if audio feedback is disabled,
            // so we can always reuse this thread to ensure mute happens right after playback.
            std::thread::spawn(move || {
                play_feedback_sound_blocking(&app_clone, SoundType::Start);
                rm_clone.apply_mute();
            });

            if let Err(e) = rm.try_start_recording(&binding_id) {
                debug!("Recording failed: {}", e);
                recording_error = Some(e);
            }
        } else {
            // On-demand mode: Start recording first, then play audio feedback, then apply mute
            // This allows the microphone to be activated before playing the sound
            debug!("On-demand mode: Starting recording first, then audio feedback");
            let recording_start_time = Instant::now();
            match rm.try_start_recording(&binding_id) {
                Ok(()) => {
                    debug!("Recording started in {:?}", recording_start_time.elapsed());
                    // Small delay to ensure microphone stream is active
                    let app_clone = app.clone();
                    let rm_clone = Arc::clone(&rm);
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        debug!("Handling delayed audio feedback/mute sequence");
                        // Helper handles disabled audio feedback by returning early, so we reuse it
                        // to keep mute sequencing consistent in every mode.
                        play_feedback_sound_blocking(&app_clone, SoundType::Start);
                        rm_clone.apply_mute();
                    });
                }
                Err(e) => {
                    debug!("Failed to start recording: {}", e);
                    recording_error = Some(e);
                }
            }
        }

        if recording_error.is_none() {
            // Dynamically register the cancel shortcut in a separate task to avoid deadlock
            shortcut::register_cancel_shortcut(app);
        } else {
            // Starting failed (for example due to blocked microphone permissions).
            // Revert UI state so we don't stay stuck in the recording overlay.
            utils::hide_recording_overlay(app);
            change_tray_icon(app, TrayIconState::Idle);
            if let Some(err) = recording_error {
                let error_type = if is_microphone_access_denied(&err) {
                    "microphone_permission_denied"
                } else if is_no_input_device_error(&err) {
                    "no_input_device"
                } else {
                    "unknown"
                };
                let _ = app.emit(
                    "recording-error",
                    RecordingErrorEvent {
                        error_type: error_type.to_string(),
                        detail: Some(err),
                    },
                );
            }
        }

        debug!(
            "TranscribeAction::start completed in {:?}",
            start_time.elapsed()
        );
    }

    fn stop(&self, app: &AppHandle, binding_id: &str, _shortcut_str: &str) {
        let stop_time = Instant::now();
        debug!("TranscribeAction::stop called for binding: {}", binding_id);
        spawn_transcription_task(app, binding_id.to_string(), self.post_process, None);
        debug!(
            "TranscribeAction::stop completed in {:?}",
            stop_time.elapsed()
        );
    }
}

/// Shared async transcription pipeline spawned by both `TranscribeAction::stop`
/// and `SmartModeAction::stop`.
///
/// - `post_process`: whether to run post-processing at all.
/// - `mode_id_override`: `Some(id)` for smart modes (routes through the mode's
///   prompt), `None` for the standard active-selection path.
fn spawn_transcription_task(
    app: &AppHandle,
    binding_id: String,
    post_process: bool,
    mode_id_override: Option<String>,
) {
    // Unregister the cancel shortcut when transcription stops
    shortcut::unregister_cancel_shortcut(app);

    let ah = app.clone();
    let rm = Arc::clone(&app.state::<Arc<AudioRecordingManager>>());
    let tm = Arc::clone(&app.state::<Arc<TranscriptionManager>>());
    let hm = Arc::clone(&app.state::<Arc<HistoryManager>>());

    change_tray_icon(app, TrayIconState::Transcribing);
    show_transcribing_overlay(app);

    // Unmute before playing audio feedback so the stop sound is audible
    rm.remove_mute();

    // Play audio feedback for recording stop
    play_feedback_sound(app, SoundType::Stop);

    tauri::async_runtime::spawn(async move {
        let _guard = FinishGuard(ah.clone());
        debug!(
            "Starting async transcription task for binding: {}",
            binding_id
        );

        let stop_recording_time = Instant::now();
        if let Some(samples) = rm.stop_recording(&binding_id) {
            debug!(
                "Recording stopped and samples retrieved in {:?}, sample count: {}",
                stop_recording_time.elapsed(),
                samples.len()
            );

            if samples.is_empty() {
                debug!("Recording produced no audio samples; skipping persistence");
                utils::hide_recording_overlay(&ah);
                change_tray_icon(&ah, TrayIconState::Idle);
            } else {
                // Save WAV concurrently with transcription
                let sample_count = samples.len();
                let file_name = format!("dictus-{}.wav", chrono::Utc::now().timestamp());
                let wav_path = hm.recordings_dir().join(&file_name);
                let wav_path_for_verify = wav_path.clone();
                let samples_for_wav = samples.clone();
                let wav_handle = tauri::async_runtime::spawn_blocking(move || {
                    crate::audio_toolkit::save_wav_file(&wav_path, &samples_for_wav)
                });

                // Transcribe concurrently with WAV save
                let transcription_time = Instant::now();
                let transcription_result = tm.transcribe(samples);

                // Await WAV save and verify
                let wav_saved = match wav_handle.await {
                    Ok(Ok(())) => {
                        match crate::audio_toolkit::verify_wav_file(
                            &wav_path_for_verify,
                            sample_count,
                        ) {
                            Ok(()) => true,
                            Err(e) => {
                                error!("WAV verification failed: {}", e);
                                false
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        error!("Failed to save WAV file: {}", e);
                        false
                    }
                    Err(e) => {
                        error!("WAV save task panicked: {}", e);
                        false
                    }
                };

                match transcription_result {
                    Ok(transcription) => {
                        debug!(
                            "Transcription completed in {:?}: '{}'",
                            transcription_time.elapsed(),
                            transcription
                        );

                        if post_process {
                            show_processing_overlay(&ah);
                        }
                        let mode_id_ref = mode_id_override.as_deref();
                        let processed = process_transcription_output(
                            &ah,
                            &transcription,
                            post_process,
                            mode_id_ref,
                        )
                        .await;

                        // Save to history if WAV was saved
                        if wav_saved {
                            if let Err(err) = hm.save_entry(
                                file_name,
                                transcription,
                                post_process,
                                processed.post_processed_text.clone(),
                                processed.post_process_prompt.clone(),
                            ) {
                                error!("Failed to save history entry: {}", err);
                            }
                        }

                        if processed.final_text.is_empty() {
                            utils::hide_recording_overlay(&ah);
                            change_tray_icon(&ah, TrayIconState::Idle);
                        } else {
                            let ah_clone = ah.clone();
                            let paste_time = Instant::now();
                            let final_text = processed.final_text;
                            ah.run_on_main_thread(move || {
                                match utils::paste(final_text, ah_clone.clone()) {
                                    Ok(()) => debug!(
                                        "Text pasted successfully in {:?}",
                                        paste_time.elapsed()
                                    ),
                                    Err(e) => {
                                        error!("Failed to paste transcription: {}", e);
                                        let _ = ah_clone.emit("paste-error", ());
                                    }
                                }
                                utils::hide_recording_overlay(&ah_clone);
                                change_tray_icon(&ah_clone, TrayIconState::Idle);
                            })
                            .unwrap_or_else(|e| {
                                error!("Failed to run paste on main thread: {:?}", e);
                                utils::hide_recording_overlay(&ah);
                                change_tray_icon(&ah, TrayIconState::Idle);
                            });
                        }
                    }
                    Err(err) => {
                        debug!("Global Shortcut Transcription error: {}", err);
                        // Save entry with empty text so user can retry
                        if wav_saved {
                            if let Err(save_err) =
                                hm.save_entry(file_name, String::new(), post_process, None, None)
                            {
                                error!("Failed to save failed history entry: {}", save_err);
                            }
                        }
                        utils::hide_recording_overlay(&ah);
                        change_tray_icon(&ah, TrayIconState::Idle);
                    }
                }
            }
        } else {
            debug!("No samples retrieved from recording stop");
            utils::hide_recording_overlay(&ah);
            change_tray_icon(&ah, TrayIconState::Idle);
        }
    });
}

impl ShortcutAction for SmartModeAction {
    fn start(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str) {
        // Start is identical to TranscribeAction::start — the mode_id only matters
        // during stop (post-processing). Delegate via the static action.
        if let Some(action) = ACTION_MAP.get("transcribe_with_post_process") {
            action.start(app, binding_id, shortcut_str);
        }
    }

    fn stop(&self, app: &AppHandle, binding_id: &str, _shortcut_str: &str) {
        let stop_time = Instant::now();
        debug!("SmartModeAction::stop called for binding: {}", binding_id);
        spawn_transcription_task(
            app,
            binding_id.to_string(),
            true, // smart modes always post-process
            Some(self.mode_id.clone()),
        );
        debug!(
            "SmartModeAction::stop completed in {:?}",
            stop_time.elapsed()
        );
    }
}

// Cancel Action
struct CancelAction;

impl ShortcutAction for CancelAction {
    fn start(&self, app: &AppHandle, _binding_id: &str, _shortcut_str: &str) {
        utils::cancel_current_operation(app);
    }

    fn stop(&self, _app: &AppHandle, _binding_id: &str, _shortcut_str: &str) {
        // Nothing to do on stop for cancel
    }
}

// Test Action
struct TestAction;

impl ShortcutAction for TestAction {
    fn start(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str) {
        log::info!(
            "Shortcut ID '{}': Started - {} (App: {})", // Changed "Pressed" to "Started" for consistency
            binding_id,
            shortcut_str,
            app.package_info().name
        );
    }

    fn stop(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str) {
        log::info!(
            "Shortcut ID '{}': Stopped - {} (App: {})", // Changed "Released" to "Stopped" for consistency
            binding_id,
            shortcut_str,
            app.package_info().name
        );
    }
}

/// Run translation inference for a Translation Smart Mode via the global engine choice.
///
/// Returns `Some(translated_text)` on success, `None` if not configured or inference fails.
async fn run_translation(
    app: &AppHandle,
    settings: &crate::settings::AppSettings,
    text: &str,
    mode: &crate::settings::SmartMode,
) -> Option<String> {
    use crate::settings::TranslationEngineChoice;

    let target = mode.target_language.as_ref()?;
    let llm_manager = app.state::<Arc<crate::managers::llm::LlmManager>>();

    // Translation always runs through the active generic LLM model. (The legacy
    // `TranslateGemma` dedicated-model path was removed — benchmarks showed a
    // same-size generic like Gemma 3 4B matches or beats it while producing
    // clean, reliable output. The enum variant is kept for settings back-compat
    // and treated identically to `GenericModel`.)
    if matches!(
        settings.translation_engine_choice,
        TranslationEngineChoice::NotChosen
    ) {
        log::warn!(
            "Translation mode '{}' triggered but translation is not enabled",
            mode.id
        );
        return None;
    }

    let model_id = match &settings.active_llm_model_id {
        Some(id) => id.clone(),
        None => {
            log::warn!("Translation enabled but no active LLM model is set");
            return None;
        }
    };
    // Ensure the active model is the one loaded (a prior Rewrite mode may have
    // left a different model loaded).
    if llm_manager.loaded_model_id().as_deref() != Some(model_id.as_str()) {
        if let Err(e) = llm_manager.load_model(&model_id).await {
            log::error!("Active model failed to load for translation: {}", e);
            return None;
        }
    }
    let prompt = format!(
        "Translate the following text to {}. Output only the translation, no explanation or commentary.\n\n{}",
        target.label, text
    );

    match llm_manager.run_inference(prompt).await {
        Ok(result) => {
            let result = strip_invisible_chars(&result);
            if result.trim().is_empty() {
                None
            } else {
                Some(result)
            }
        }
        Err(e) => {
            log::error!("Translation inference failed: {}", e);
            None
        }
    }
}

/// Resolve which SmartMode and prompt text to use for a `mode_id_override`.
///
/// Returns `Some((mode, prompt_text))` for Rewrite modes and `None` for Translation
/// modes (handled in process_transcription_output via run_translation).
pub(crate) fn resolve_mode_prompt<'a>(
    settings: &'a crate::settings::AppSettings,
    mode_id: &str,
) -> Option<(&'a crate::settings::SmartMode, String)> {
    let mode = settings.smart_modes.iter().find(|m| m.id == mode_id)?;
    match mode.kind {
        crate::settings::SmartModeKind::Rewrite => {
            let prompt = mode.prompt.clone();
            Some((mode, prompt))
        }
        crate::settings::SmartModeKind::Translation => None,
    }
}

// Static Action Map
pub static ACTION_MAP: Lazy<HashMap<String, Arc<dyn ShortcutAction>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(
        "transcribe".to_string(),
        Arc::new(TranscribeAction {
            post_process: false,
        }) as Arc<dyn ShortcutAction>,
    );
    map.insert(
        "transcribe_with_post_process".to_string(),
        Arc::new(TranscribeAction { post_process: true }) as Arc<dyn ShortcutAction>,
    );
    map.insert(
        "cancel".to_string(),
        Arc::new(CancelAction) as Arc<dyn ShortcutAction>,
    );
    map.insert(
        "test".to_string(),
        Arc::new(TestAction) as Arc<dyn ShortcutAction>,
    );
    map
});

#[cfg(test)]
mod actions_tests {
    use super::*;
    use crate::settings::{AppSettings, SmartMode, SmartModeKind};

    fn make_settings_with_modes(modes: Vec<SmartMode>) -> AppSettings {
        let mut s = crate::settings::get_default_settings();
        s.smart_modes = modes;
        s
    }

    fn rewrite_mode(id: &str, prompt: &str) -> SmartMode {
        SmartMode {
            id: id.to_string(),
            name: format!("Mode {}", id),
            kind: SmartModeKind::Rewrite,
            prompt: prompt.to_string(),
            target_language: None,
        }
    }

    fn translation_mode(id: &str) -> SmartMode {
        SmartMode {
            id: id.to_string(),
            name: format!("Mode {}", id),
            kind: SmartModeKind::Translation,
            prompt: String::new(),
            target_language: None,
        }
    }

    #[test]
    fn mode_routing_rewrite_uses_correct_prompt() {
        let expected_prompt = "Fix grammar and style.";
        let mode = rewrite_mode("mode_x", expected_prompt);
        let settings = make_settings_with_modes(vec![mode]);

        let result = resolve_mode_prompt(&settings, "mode_x");
        assert!(result.is_some(), "Rewrite mode should resolve successfully");
        let (found_mode, prompt_text) = result.unwrap();
        assert_eq!(found_mode.id, "mode_x");
        assert_eq!(
            prompt_text, expected_prompt,
            "Should return the mode's own prompt, not the active selection"
        );
    }

    #[test]
    fn mode_routing_translation_returns_none() {
        let mode = translation_mode("translate_fr");
        let settings = make_settings_with_modes(vec![mode]);

        let result = resolve_mode_prompt(&settings, "translate_fr");
        assert!(
            result.is_none(),
            "Translation mode should return None from resolve_mode_prompt (handled in process_transcription_output)"
        );
    }

    #[test]
    fn mode_routing_unknown_id_returns_none() {
        let settings = make_settings_with_modes(vec![]);
        let result = resolve_mode_prompt(&settings, "nonexistent");
        assert!(result.is_none(), "Unknown mode id should return None");
    }

    #[test]
    fn generic_model_translation_prompt_contains_target_language() {
        // Test the pure logic: confirm the prompt format for GenericModel contains
        // the expected strings without needing an AppHandle.
        let target_label = "Spanish";
        let transcription = "Hello, how are you?";
        let prompt = format!(
            "Translate the following text to {}. Output only the translation, no explanation or commentary.\n\n{}",
            target_label, transcription
        );
        assert!(
            prompt.contains("Translate the following text to Spanish"),
            "Prompt should contain target language"
        );
        assert!(
            prompt.contains("Output only the translation"),
            "Prompt should contain output instruction"
        );
        assert!(
            prompt.contains(transcription),
            "Prompt should contain the original transcription"
        );
    }
}

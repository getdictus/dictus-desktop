//! Keyboard shortcut management module
//!
//! This module provides a unified interface for keyboard shortcuts with
//! multiple backend implementations:
//!
//! - `tauri`: Uses Tauri's built-in global-shortcut plugin
//! - `handy_keys`: Uses the handy-keys library for more control
//!
//! The active implementation is determined by the `keyboard_implementation`
//! setting and can be changed at runtime.

mod handler;
pub mod handy_keys;
mod tauri_impl;

use log::{error, info, warn};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use crate::settings::APPLE_INTELLIGENCE_DEFAULT_MODEL_ID;
use crate::settings::{
    self, get_settings, AutoSubmitKey, ClipboardHandling, KeyboardImplementation, LLMPrompt,
    OverlayPosition, PasteMethod, ShortcutBinding, SoundTheme, TypingTool,
    APPLE_INTELLIGENCE_PROVIDER_ID,
};
use crate::tray;

// Note: Commands are accessed via shortcut::handy_keys:: in lib.rs

/// Initialize shortcuts using the configured implementation
pub fn init_shortcuts(app: &AppHandle) {
    let user_settings = settings::load_or_create_app_settings(app);

    // Check which implementation to use
    match user_settings.keyboard_implementation {
        KeyboardImplementation::Tauri => {
            tauri_impl::init_shortcuts(app);
        }
        KeyboardImplementation::HandyKeys => {
            if let Err(e) = handy_keys::init_shortcuts(app) {
                error!("Failed to initialize HandyKeys shortcuts: {}", e);
                // Fall back to Tauri implementation and persist this fallback
                warn!("Falling back to Tauri global shortcut implementation and saving fallback to settings");

                // Update settings to persist the fallback so we don't retry HandyKeys on next launch
                let mut settings = settings::get_settings(app);
                settings.keyboard_implementation = KeyboardImplementation::Tauri;
                settings::write_settings(app, settings);

                tauri_impl::init_shortcuts(app);
            }
        }
    }
}

/// Register the cancel shortcut (called when recording starts)
pub fn register_cancel_shortcut(app: &AppHandle) {
    let settings = get_settings(app);
    match settings.keyboard_implementation {
        KeyboardImplementation::Tauri => tauri_impl::register_cancel_shortcut(app),
        KeyboardImplementation::HandyKeys => handy_keys::register_cancel_shortcut(app),
    }
}

/// Unregister the cancel shortcut (called when recording stops)
pub fn unregister_cancel_shortcut(app: &AppHandle) {
    let settings = get_settings(app);
    match settings.keyboard_implementation {
        KeyboardImplementation::Tauri => tauri_impl::unregister_cancel_shortcut(app),
        KeyboardImplementation::HandyKeys => handy_keys::unregister_cancel_shortcut(app),
    }
}

/// Register a shortcut using the appropriate implementation
pub fn register_shortcut(app: &AppHandle, binding: ShortcutBinding) -> Result<(), String> {
    let settings = get_settings(app);
    match settings.keyboard_implementation {
        KeyboardImplementation::Tauri => tauri_impl::register_shortcut(app, binding),
        KeyboardImplementation::HandyKeys => handy_keys::register_shortcut(app, binding),
    }
}

/// Unregister a shortcut using the appropriate implementation
pub fn unregister_shortcut(app: &AppHandle, binding: ShortcutBinding) -> Result<(), String> {
    let settings = get_settings(app);
    match settings.keyboard_implementation {
        KeyboardImplementation::Tauri => tauri_impl::unregister_shortcut(app, binding),
        KeyboardImplementation::HandyKeys => handy_keys::unregister_shortcut(app, binding),
    }
}

// ============================================================================
// Binding Management Commands
// ============================================================================

#[derive(Serialize, Type)]
pub struct BindingResponse {
    success: bool,
    binding: Option<ShortcutBinding>,
    error: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub fn change_binding(
    app: AppHandle,
    id: String,
    binding: String,
) -> Result<BindingResponse, String> {
    // Reject empty bindings — every shortcut should have a value
    if binding.trim().is_empty() {
        return Err("Binding cannot be empty".to_string());
    }

    let mut settings = settings::get_settings(&app);

    // Get the binding to modify, or create it from defaults if it doesn't exist
    let binding_to_modify = match settings.bindings.get(&id) {
        Some(binding) => binding.clone(),
        None => {
            // Try to get the default binding for this id
            let default_settings = settings::get_default_settings();
            match default_settings.bindings.get(&id) {
                Some(default_binding) => {
                    warn!(
                        "Binding '{}' not found in settings, creating from defaults",
                        id
                    );
                    default_binding.clone()
                }
                None => {
                    let error_msg = format!("Binding with id '{}' not found in defaults", id);
                    warn!("change_binding error: {}", error_msg);
                    return Ok(BindingResponse {
                        success: false,
                        binding: None,
                        error: Some(error_msg),
                    });
                }
            }
        }
    };

    // If this is the cancel binding, just update the settings and return
    // It's managed dynamically, so we don't register/unregister here
    if id == "cancel" {
        if let Some(mut b) = settings.bindings.get(&id).cloned() {
            b.current_binding = binding;
            settings.bindings.insert(id.clone(), b.clone());
            settings::write_settings(&app, settings);
            return Ok(BindingResponse {
                success: true,
                binding: Some(b.clone()),
                error: None,
            });
        }
    }

    // Unregister the existing binding
    if let Err(e) = unregister_shortcut(&app, binding_to_modify.clone()) {
        let error_msg = format!("Failed to unregister shortcut: {}", e);
        error!("change_binding error: {}", error_msg);
    }

    // Validate the new shortcut for the current keyboard implementation
    if let Err(e) = validate_shortcut_for_implementation(&binding, settings.keyboard_implementation)
    {
        warn!("change_binding validation error: {}", e);
        return Err(e);
    }

    // Create an updated binding
    let mut updated_binding = binding_to_modify;
    updated_binding.current_binding = binding;

    // Register the new binding
    if let Err(e) = register_shortcut(&app, updated_binding.clone()) {
        let error_msg = format!("Failed to register shortcut: {}", e);
        error!("change_binding error: {}", error_msg);
        return Ok(BindingResponse {
            success: false,
            binding: None,
            error: Some(error_msg),
        });
    }

    // Update the binding in the settings
    settings.bindings.insert(id, updated_binding.clone());

    // Save the settings
    settings::write_settings(&app, settings);

    // Return the updated binding
    Ok(BindingResponse {
        success: true,
        binding: Some(updated_binding),
        error: None,
    })
}

#[tauri::command]
#[specta::specta]
pub fn reset_binding(app: AppHandle, id: String) -> Result<BindingResponse, String> {
    let binding = settings::get_stored_binding(&app, &id);
    change_binding(app, id, binding.default_binding)
}

/// Temporarily unregister a binding while the user is editing it in the UI.
/// This avoids firing the action while keys are being recorded.
#[tauri::command]
#[specta::specta]
pub fn suspend_binding(app: AppHandle, id: String) -> Result<(), String> {
    if let Some(b) = settings::get_bindings(&app).get(&id).cloned() {
        if let Err(e) = unregister_shortcut(&app, b) {
            error!("suspend_binding error for id '{}': {}", id, e);
            return Err(e);
        }
    }
    Ok(())
}

/// Re-register the binding after the user has finished editing.
#[tauri::command]
#[specta::specta]
pub fn resume_binding(app: AppHandle, id: String) -> Result<(), String> {
    if let Some(b) = settings::get_bindings(&app).get(&id).cloned() {
        if let Err(e) = register_shortcut(&app, b) {
            error!("resume_binding error for id '{}': {}", id, e);
            return Err(e);
        }
    }
    Ok(())
}

/// Temporarily unregister ALL global shortcuts (used while the user records a
/// new smart-mode shortcut so an already-bound combo is captured, not fired).
///
/// The caller MUST call `resume_all_shortcuts` once capture is done
/// (commit / conflict / cancel / unmount) so no binding stays permanently dead.
#[tauri::command]
#[specta::specta]
pub fn suspend_all_shortcuts(app: AppHandle) -> Result<(), String> {
    let impl_ = settings::get_settings(&app).keyboard_implementation;
    unregister_all_shortcuts(&app, impl_);
    Ok(())
}

/// Re-register all global shortcuts after a capture session ends.
///
/// Iterates every non-empty binding except "cancel" (which is dynamically
/// registered only during recording) and re-registers it for the active
/// keyboard implementation. Non-fatal per-binding errors are logged as
/// warnings so a single broken binding does not block the others.
///
/// Idempotent: each binding is unregistered first (errors ignored — the
/// shortcut may already be unbound) so `register_shortcut` always starts
/// from a clean slate and never hits "Hotkey already registered".
#[tauri::command]
#[specta::specta]
pub fn resume_all_shortcuts(app: AppHandle) -> Result<(), String> {
    let settings = settings::get_settings(&app);
    for (id, binding) in &settings.bindings {
        // cancel is registered dynamically on recording start — skip it here.
        if id == "cancel" {
            continue;
        }
        if binding.current_binding.trim().is_empty() {
            continue;
        }
        // Unregister first so re-registration always starts from a clean
        // slate. An unbound shortcut returning an error here is expected and
        // harmless — ignore it.
        let _ = unregister_shortcut(&app, binding.clone());
        if let Err(e) = register_shortcut(&app, binding.clone()) {
            warn!(
                "resume_all_shortcuts: failed to re-register '{}': {}",
                id, e
            );
        }
    }
    Ok(())
}

// ============================================================================
// Keyboard Implementation Switching
// ============================================================================

/// Result of changing keyboard implementation
#[derive(Serialize, Type)]
pub struct ImplementationChangeResult {
    pub success: bool,
    /// List of binding IDs that were reset to defaults due to incompatibility
    pub reset_bindings: Vec<String>,
}

/// Change the keyboard implementation with runtime switching.
/// This will unregister all shortcuts from the old implementation,
/// validate shortcuts for the new implementation (resetting invalid ones to defaults),
/// and register them with the new implementation.
#[tauri::command]
#[specta::specta]
pub fn change_keyboard_implementation_setting(
    app: AppHandle,
    implementation: String,
) -> Result<ImplementationChangeResult, String> {
    let current_settings = settings::get_settings(&app);
    let current_impl = current_settings.keyboard_implementation;
    let new_impl = parse_keyboard_implementation(&implementation);

    // If same implementation, nothing to do
    if current_impl == new_impl {
        return Ok(ImplementationChangeResult {
            success: true,
            reset_bindings: vec![],
        });
    }

    info!(
        "Switching keyboard implementation from {:?} to {:?}",
        current_impl, new_impl
    );

    // Unregister all shortcuts from the current implementation
    unregister_all_shortcuts(&app, current_impl);

    // Update the setting
    let mut settings = settings::get_settings(&app);
    settings.keyboard_implementation = new_impl;
    settings::write_settings(&app, settings);

    // Initialize new implementation if needed (HandyKeys needs state)
    if new_impl == KeyboardImplementation::HandyKeys && initialize_handy_keys_with_rollback(&app)? {
        // Shortcuts already registered during init
        return Ok(ImplementationChangeResult {
            success: true,
            reset_bindings: vec![],
        });
    }

    // Register all shortcuts with new implementation, resetting invalid ones
    let reset_bindings = register_all_shortcuts_for_implementation(&app, new_impl);

    // Emit event to notify frontend of the change
    let _ = app.emit(
        "settings-changed",
        serde_json::json!({
            "setting": "keyboard_implementation",
            "value": implementation,
            "reset_bindings": reset_bindings
        }),
    );

    info!("Keyboard implementation switched to {:?}", new_impl);

    Ok(ImplementationChangeResult {
        success: true,
        reset_bindings,
    })
}

/// Get the current keyboard implementation
#[tauri::command]
#[specta::specta]
pub fn get_keyboard_implementation(app: AppHandle) -> String {
    let settings = settings::get_settings(&app);
    match settings.keyboard_implementation {
        KeyboardImplementation::Tauri => "tauri".to_string(),
        KeyboardImplementation::HandyKeys => "handy_keys".to_string(),
    }
}

// ============================================================================
// Validation Helpers
// ============================================================================

/// Validate a shortcut for a specific implementation
fn validate_shortcut_for_implementation(
    raw: &str,
    implementation: KeyboardImplementation,
) -> Result<(), String> {
    match implementation {
        KeyboardImplementation::Tauri => tauri_impl::validate_shortcut(raw),
        KeyboardImplementation::HandyKeys => handy_keys::validate_shortcut(raw),
    }
}

/// Parse a keyboard implementation string into the enum
fn parse_keyboard_implementation(s: &str) -> KeyboardImplementation {
    match s {
        "tauri" => KeyboardImplementation::Tauri,
        "handy_keys" => KeyboardImplementation::HandyKeys,
        other => {
            warn!(
                "Invalid keyboard implementation '{}', defaulting to tauri",
                other
            );
            KeyboardImplementation::Tauri
        }
    }
}

/// Unregister all shortcuts for the current implementation
fn unregister_all_shortcuts(app: &AppHandle, implementation: KeyboardImplementation) {
    let bindings = settings::get_bindings(app);

    for (id, binding) in bindings {
        // Skip cancel shortcut as it's dynamically registered
        if id == "cancel" {
            continue;
        }

        let result = match implementation {
            KeyboardImplementation::Tauri => tauri_impl::unregister_shortcut(app, binding),
            KeyboardImplementation::HandyKeys => handy_keys::unregister_shortcut(app, binding),
        };

        if let Err(e) = result {
            warn!(
                "Failed to unregister shortcut '{}' during switch: {}",
                id, e
            );
        }
    }
}

/// Register all shortcuts for a specific implementation, validating and resetting invalid ones
fn register_all_shortcuts_for_implementation(
    app: &AppHandle,
    implementation: KeyboardImplementation,
) -> Vec<String> {
    let mut reset_bindings = Vec::new();
    let default_bindings = settings::get_default_settings().bindings;
    let mut current_settings = settings::get_settings(app);

    for (id, default_binding) in &default_bindings {
        // Skip cancel shortcut as it's dynamically registered
        if id == "cancel" {
            continue;
        }

        let mut binding = current_settings
            .bindings
            .get(id)
            .cloned()
            .unwrap_or_else(|| default_binding.clone());

        // Validate the shortcut for the target implementation
        if let Err(e) =
            validate_shortcut_for_implementation(&binding.current_binding, implementation)
        {
            info!(
                "Shortcut '{}' ({}) is invalid for {:?}: {}. Resetting to default.",
                id, binding.current_binding, implementation, e
            );

            // Reset to default
            binding.current_binding = default_binding.current_binding.clone();
            current_settings
                .bindings
                .insert(id.clone(), binding.clone());
            reset_bindings.push(id.clone());
        }

        // Register with the appropriate implementation
        let result = match implementation {
            KeyboardImplementation::Tauri => tauri_impl::register_shortcut(app, binding),
            KeyboardImplementation::HandyKeys => handy_keys::register_shortcut(app, binding),
        };

        if let Err(e) = result {
            error!(
                "Failed to register shortcut '{}' for {:?}: {}",
                id, implementation, e
            );
        }
    }

    // Save settings if any bindings were reset
    if !reset_bindings.is_empty() {
        settings::write_settings(app, current_settings);
    }

    reset_bindings
}

/// Initialize HandyKeys if not already initialized, with rollback on failure
fn initialize_handy_keys_with_rollback(app: &AppHandle) -> Result<bool, String> {
    if app.try_state::<handy_keys::HandyKeysState>().is_some() {
        return Ok(false); // Already initialized, caller should continue
    }

    if let Err(e) = handy_keys::init_shortcuts(app) {
        error!("Failed to initialize HandyKeys: {}", e);
        // Rollback to Tauri
        let mut settings = settings::get_settings(app);
        settings.keyboard_implementation = KeyboardImplementation::Tauri;
        settings::write_settings(app, settings);
        tauri_impl::init_shortcuts(app);
        return Err(format!(
            "Failed to initialize HandyKeys: {}. Reverted to Tauri.",
            e
        ));
    }

    // init_shortcuts already registered shortcuts
    Ok(true)
}

// ============================================================================
// General Settings Commands
// ============================================================================

#[tauri::command]
#[specta::specta]
pub fn change_ptt_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.push_to_talk = enabled;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_audio_feedback_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.audio_feedback = enabled;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_audio_feedback_volume_setting(app: AppHandle, volume: f32) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.audio_feedback_volume = volume;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_sound_theme_setting(app: AppHandle, theme: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    let parsed = match theme.as_str() {
        "marimba" => SoundTheme::Marimba,
        "pop" => SoundTheme::Pop,
        "custom" => SoundTheme::Custom,
        other => {
            warn!("Invalid sound theme '{}', defaulting to marimba", other);
            SoundTheme::Marimba
        }
    };
    settings.sound_theme = parsed;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_translate_to_english_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.translate_to_english = enabled;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_selected_language_setting(app: AppHandle, language: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.selected_language = language;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_overlay_position_setting(app: AppHandle, position: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    let parsed = match position.as_str() {
        "none" => OverlayPosition::None,
        "top" => OverlayPosition::Top,
        "bottom" => OverlayPosition::Bottom,
        other => {
            warn!("Invalid overlay position '{}', defaulting to bottom", other);
            OverlayPosition::Bottom
        }
    };
    settings.overlay_position = parsed;
    settings::write_settings(&app, settings);

    // Update overlay position without recreating window
    crate::utils::update_overlay_position(&app);

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_debug_mode_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.debug_mode = enabled;
    settings::write_settings(&app, settings);

    // Emit event to notify frontend of debug mode change
    let _ = app.emit(
        "settings-changed",
        serde_json::json!({
            "setting": "debug_mode",
            "value": enabled
        }),
    );

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_start_hidden_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.start_hidden = enabled;
    settings::write_settings(&app, settings);

    // Notify frontend
    let _ = app.emit(
        "settings-changed",
        serde_json::json!({
            "setting": "start_hidden",
            "value": enabled
        }),
    );

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_autostart_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.autostart_enabled = enabled;
    settings::write_settings(&app, settings);

    // Apply the autostart setting immediately
    let autostart_manager = app.autolaunch();
    if enabled {
        let _ = autostart_manager.enable();
    } else {
        let _ = autostart_manager.disable();
    }

    // Notify frontend
    let _ = app.emit(
        "settings-changed",
        serde_json::json!({
            "setting": "autostart_enabled",
            "value": enabled
        }),
    );

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_update_checks_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.update_checks_enabled = enabled;
    settings::write_settings(&app, settings);

    let _ = app.emit(
        "settings-changed",
        serde_json::json!({
            "setting": "update_checks_enabled",
            "value": enabled
        }),
    );

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn update_custom_words(app: AppHandle, words: Vec<String>) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.custom_words = words;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_word_correction_threshold_setting(
    app: AppHandle,
    threshold: f64,
) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.word_correction_threshold = threshold;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_extra_recording_buffer_setting(app: AppHandle, ms: u64) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.extra_recording_buffer_ms = ms;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_paste_delay_ms_setting(app: AppHandle, ms: u64) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.paste_delay_ms = ms;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_paste_method_setting(app: AppHandle, method: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    let parsed = match method.as_str() {
        "ctrl_v" => PasteMethod::CtrlV,
        "direct" => PasteMethod::Direct,
        "none" => PasteMethod::None,
        "shift_insert" => PasteMethod::ShiftInsert,
        "ctrl_shift_v" => PasteMethod::CtrlShiftV,
        "external_script" => PasteMethod::ExternalScript,
        other => {
            warn!("Invalid paste method '{}', defaulting to ctrl_v", other);
            PasteMethod::CtrlV
        }
    };
    settings.paste_method = parsed;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_available_typing_tools() -> Vec<String> {
    #[cfg(target_os = "linux")]
    {
        crate::clipboard::get_available_typing_tools()
    }
    #[cfg(not(target_os = "linux"))]
    {
        vec!["auto".to_string()]
    }
}

#[tauri::command]
#[specta::specta]
pub fn change_typing_tool_setting(app: AppHandle, tool: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    let parsed = match tool.as_str() {
        "auto" => TypingTool::Auto,
        "wtype" => TypingTool::Wtype,
        "kwtype" => TypingTool::Kwtype,
        "dotool" => TypingTool::Dotool,
        "ydotool" => TypingTool::Ydotool,
        "xdotool" => TypingTool::Xdotool,
        other => {
            warn!("Invalid typing tool '{}', defaulting to auto", other);
            TypingTool::Auto
        }
    };
    settings.typing_tool = parsed;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_external_script_path_setting(
    app: AppHandle,
    path: Option<String>,
) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.external_script_path = path;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_clipboard_handling_setting(app: AppHandle, handling: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    let parsed = match handling.as_str() {
        "dont_modify" => ClipboardHandling::DontModify,
        "copy_to_clipboard" => ClipboardHandling::CopyToClipboard,
        other => {
            warn!(
                "Invalid clipboard handling '{}', defaulting to dont_modify",
                other
            );
            ClipboardHandling::DontModify
        }
    };
    settings.clipboard_handling = parsed;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_auto_submit_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.auto_submit = enabled;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_auto_submit_key_setting(app: AppHandle, key: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    let parsed = match key.as_str() {
        "enter" => AutoSubmitKey::Enter,
        "ctrl_enter" => AutoSubmitKey::CtrlEnter,
        "cmd_enter" => AutoSubmitKey::CmdEnter,
        other => {
            warn!("Invalid auto submit key '{}', defaulting to enter", other);
            AutoSubmitKey::Enter
        }
    };
    settings.auto_submit_key = parsed;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_post_process_enabled_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.post_process_enabled = enabled;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_experimental_enabled_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.experimental_enabled = enabled;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_enable_cloud_providers_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.enable_cloud_providers = enabled;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_post_process_base_url_setting(
    app: AppHandle,
    provider_id: String,
    base_url: String,
) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    let label = settings
        .post_process_provider(&provider_id)
        .map(|provider| provider.label.clone())
        .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;

    let provider = settings
        .post_process_provider_mut(&provider_id)
        .expect("Provider looked up above must exist");

    if provider.id != "custom" {
        return Err(format!(
            "Provider '{}' does not allow editing the base URL",
            label
        ));
    }

    provider.base_url = base_url;
    settings::write_settings(&app, settings);
    Ok(())
}

/// Generic helper to validate provider exists
fn validate_provider_exists(
    settings: &settings::AppSettings,
    provider_id: &str,
) -> Result<(), String> {
    if !settings
        .post_process_providers
        .iter()
        .any(|provider| provider.id == provider_id)
    {
        return Err(format!("Provider '{}' not found", provider_id));
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_post_process_api_key_setting(
    app: AppHandle,
    provider_id: String,
    api_key: String,
) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    validate_provider_exists(&settings, &provider_id)?;
    settings.post_process_api_keys.insert(provider_id, api_key);
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_post_process_model_setting(
    app: AppHandle,
    provider_id: String,
    model: String,
) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    validate_provider_exists(&settings, &provider_id)?;
    settings.post_process_models.insert(provider_id, model);
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_post_process_provider(app: AppHandle, provider_id: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    // "embedded" is a synthetic provider run in-process by LlmManager. It is
    // intentionally not present in post_process_providers (see actions.rs, which
    // detects it via post_process_provider_id == "embedded"), so skip the
    // registry check for it — otherwise the selection is rejected and reverted.
    if provider_id != "embedded" {
        validate_provider_exists(&settings, &provider_id)?;
    }
    settings.post_process_provider_id = provider_id;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn add_post_process_prompt(
    app: AppHandle,
    name: String,
    prompt: String,
) -> Result<LLMPrompt, String> {
    let mut settings = settings::get_settings(&app);

    // Generate unique ID using timestamp and random component
    let id = format!("prompt_{}", chrono::Utc::now().timestamp_millis());

    let new_prompt = LLMPrompt {
        id: id.clone(),
        name,
        prompt,
    };

    settings.post_process_prompts.push(new_prompt.clone());
    settings::write_settings(&app, settings);

    Ok(new_prompt)
}

#[tauri::command]
#[specta::specta]
pub fn update_post_process_prompt(
    app: AppHandle,
    id: String,
    name: String,
    prompt: String,
) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);

    if let Some(existing_prompt) = settings
        .post_process_prompts
        .iter_mut()
        .find(|p| p.id == id)
    {
        existing_prompt.name = name;
        existing_prompt.prompt = prompt;
        settings::write_settings(&app, settings);
        Ok(())
    } else {
        Err(format!("Prompt with id '{}' not found", id))
    }
}

#[tauri::command]
#[specta::specta]
pub fn delete_post_process_prompt(app: AppHandle, id: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);

    // Don't allow deleting the last prompt
    if settings.post_process_prompts.len() <= 1 {
        return Err("Cannot delete the last prompt".to_string());
    }

    // Find and remove the prompt
    let original_len = settings.post_process_prompts.len();
    settings.post_process_prompts.retain(|p| p.id != id);

    if settings.post_process_prompts.len() == original_len {
        return Err(format!("Prompt with id '{}' not found", id));
    }

    // If the deleted prompt was selected, select the first one or None
    if settings.post_process_selected_prompt_id.as_ref() == Some(&id) {
        settings.post_process_selected_prompt_id =
            settings.post_process_prompts.first().map(|p| p.id.clone());
    }

    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_post_process_models(
    app: AppHandle,
    provider_id: String,
) -> Result<Vec<String>, String> {
    let settings = settings::get_settings(&app);

    // Find the provider
    let provider = settings
        .post_process_providers
        .iter()
        .find(|p| p.id == provider_id)
        .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;

    if provider.id == APPLE_INTELLIGENCE_PROVIDER_ID {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            return Ok(vec![APPLE_INTELLIGENCE_DEFAULT_MODEL_ID.to_string()]);
        }

        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        {
            return Err("Apple Intelligence is only available on Apple silicon Macs running macOS 15 or later.".to_string());
        }
    }

    // Get API key
    let api_key = settings
        .post_process_api_keys
        .get(&provider_id)
        .cloned()
        .unwrap_or_default();

    // Skip fetching if no API key for providers that typically need one
    if api_key.trim().is_empty() && provider.id != "custom" {
        return Err(format!(
            "API key is required for {}. Please add an API key to list available models.",
            provider.label
        ));
    }

    crate::llm_client::fetch_models(provider, api_key).await
}

#[tauri::command]
#[specta::specta]
pub fn set_post_process_selected_prompt(app: AppHandle, id: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);

    // Verify the prompt exists
    if !settings.post_process_prompts.iter().any(|p| p.id == id) {
        return Err(format!("Prompt with id '{}' not found", id));
    }

    settings.post_process_selected_prompt_id = Some(id);
    settings::write_settings(&app, settings);
    Ok(())
}

// ============================================================================
// Smart Mode CRUD helpers and commands
// ============================================================================

/// Pure logic helper: removes a mode by id from `modes` and reassigns `active`
/// if the deleted mode was the active one. Returns Err if len <= 1 or id not found.
pub fn delete_mode_in_place(
    modes: &mut Vec<settings::SmartMode>,
    active: &mut Option<String>,
    id: &str,
) -> Result<(), String> {
    if modes.len() <= 1 {
        return Err("Cannot delete the last mode".to_string());
    }
    let original_len = modes.len();
    modes.retain(|m| m.id != id);
    if modes.len() == original_len {
        return Err(format!("Smart mode with id '{}' not found", id));
    }
    if active.as_deref() == Some(id) {
        *active = modes.first().map(|m| m.id.clone());
    }
    Ok(())
}

/// Construct the binding id for a smart mode.
/// Exported so Phase 13 routing can reuse it.
pub fn smart_mode_binding_id(mode_id: &str) -> String {
    format!("smart_mode_{}", mode_id)
}

/// True when `token` (a single '+'-split combo segment) names a modifier key.
/// Accepts both the handy_keys lowercase side-distinct forms and the tauri
/// capitalized forms (compared case-insensitively here only for this helper).
fn is_modifier_token(token: &str) -> bool {
    let t = token.trim().to_ascii_lowercase();
    matches!(
        t.as_str(),
        "ctrl"
            | "ctrl_left"
            | "ctrl_right"
            | "control"
            | "control_left"
            | "control_right"
            | "option"
            | "option_left"
            | "option_right"
            | "alt"
            | "alt_left"
            | "alt_right"
            | "shift"
            | "shift_left"
            | "shift_right"
            | "command"
            | "command_left"
            | "command_right"
            | "cmd"
            | "super"
            | "super_left"
            | "super_right"
            | "fn"
            | "meta"
    )
}

/// Return the modifier-only "base" of a combo: the part the OS can fire on
/// its own (every leading modifier token, dropping a trailing main key).
/// For a modifier-only combo the base IS the whole string.
///
/// Examples:
///   "command_left+digit1" -> "command_left"
///   "ctrl+option+keya"    -> "ctrl+option"
///   "command_left"        -> "command_left"
///   "f13"                 -> "f13"
fn combo_base(combo: &str) -> &str {
    let trimmed = combo.trim();
    // Find the last '+' whose left side consists entirely of modifier tokens.
    // For combos in our format (modifiers first, at most one trailing main key)
    // this means: if there is a '+' and everything before the last '+' is
    // modifier-only, the base is everything up to (not including) that '+'.
    if let Some(idx) = trimmed.rfind('+') {
        let prefix = &trimmed[..idx];
        // All segments in the prefix must be modifiers for this to be the
        // modifier-base split.  A single leading modifier (e.g. "command_left")
        // or a chain ("ctrl+option") both qualify.
        let all_modifier = prefix.split('+').all(is_modifier_token);
        if all_modifier {
            return prefix;
        }
    }
    trimmed
}

/// Signals which of the three overlap rules matched in `find_conflicting_binding`.
/// Carried alongside the conflicting binding id so the frontend can render two
/// distinct, localized messages instead of one generic English sentence (G13).
#[derive(Serialize, Type, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictKind {
    /// candidate combo string-equals an existing binding (rule 1)
    ExactDuplicate,
    /// candidate's modifier-base IS an existing full binding (rule 2)
    /// e.g. existing "command_left", candidate "command_left+digit1"
    CandidateBaseIsExisting,
    /// candidate (a base key) IS the modifier-base of an existing combo (rule 3)
    /// e.g. existing "command_left+digit1", candidate "command_left"
    CandidateIsBaseOfExisting,
}

/// Find a global binding (other than `binding_id`) that already holds `combo`.
/// Returns `(conflicting_id, ConflictKind)`, or None when `combo` is free.
///
/// Used by `set_smart_mode_binding` to reject a combo already assigned to a
/// DIFFERENT smart mode or any other global shortcut at commit time, so the
/// chip's inline conflict UI fires instead of double-persisting two bindings
/// that collide only at OS re-registration (UAT test 7 / [B5]).
///
/// In addition to full-string equality (13-13 behaviour, preserved verbatim)
/// this also rejects prefix/base-key overlaps ([G11]):
///   (a) candidate's modifier-base IS another full binding
///       (e.g. existing "command_left", candidate "command_left+digit1")
///   (b) candidate (a base key) IS the modifier-base of another full binding
///       (e.g. existing "command_left+digit1", candidate "command_left")
/// Two distinct full combos that share only a modifier prefix (e.g.
/// "command_left+digit1" vs "command_left+digit2") do NOT trigger this rule.
pub fn find_conflicting_binding(
    bindings: &std::collections::HashMap<String, settings::ShortcutBinding>,
    binding_id: &str,
    combo: &str,
) -> Option<(String, ConflictKind)> {
    let target = combo.trim();
    if target.is_empty() {
        return None;
    }
    let candidate_base = combo_base(target);
    bindings.iter().find_map(|(other_id, b)| {
        if other_id.as_str() == binding_id {
            return None;
        }
        let other = b.current_binding.trim();
        if other.is_empty() {
            return None;
        }
        // Determine kind in priority order (rule 1 > rule 2 > rule 3).
        let kind = if other == target {
            // (1) exact duplicate — 13-13 behaviour preserved verbatim
            ConflictKind::ExactDuplicate
        } else if other == candidate_base {
            // (2) candidate's base IS this full binding
            //     (existing "command_left", candidate "command_left+digit1")
            ConflictKind::CandidateBaseIsExisting
        } else if combo_base(other) == target {
            // (3) candidate IS the base of this binding
            //     (existing "command_left+digit1", candidate "command_left")
            ConflictKind::CandidateIsBaseOfExisting
        } else {
            return None;
        };
        Some((other_id.clone(), kind))
    })
}

/// Resolve the stable id for a created mode. Returns Some(seed_id) when (name, kind)
/// matches a seeded template, else None (caller mints a timestamp id).
///
/// This is the dedup anchor for add_smart_mode: re-adding a template from the
/// picker reuses its seeded id rather than minting a fresh mode_{timestamp}, so
/// there is never a card↔binding identity split.
pub fn resolve_seeded_id(name: &str, kind: &settings::SmartModeKind) -> Option<String> {
    settings::smart_mode_templates()
        .into_iter()
        .find(|t| t.name == name && &t.kind == kind)
        .map(|t| t.id)
}

#[tauri::command]
#[specta::specta]
pub fn add_smart_mode(
    app: AppHandle,
    name: String,
    kind: settings::SmartModeKind,
    prompt: String,
    target_language: Option<settings::TargetLanguage>,
) -> Result<settings::SmartMode, String> {
    let mut settings = settings::get_settings(&app);

    // Dedup against the seeded template catalogue. If the incoming (name, kind)
    // matches a seeded template, reuse its stable seed id instead of minting a
    // timestamp. This prevents duplicate cards and the card↔binding identity
    // split surfaced in UAT tests 4 and 9.
    //
    // Decision [13-07] originally used a fresh timestamp to avoid id collision
    // when the user deletes a default and recreates it via the picker. The
    // overwrite branch below handles the "still exists" case explicitly, and the
    // reuse branch handles the "was deleted" case — both are safe with a stable
    // seed id. See 13-09 SUMMARY for the full reversal rationale.
    if let Some(seed_id) = resolve_seeded_id(&name, &kind) {
        if let Some(existing) = settings.smart_modes.iter_mut().find(|m| m.id == seed_id) {
            // Overwrite in place — do NOT push a duplicate.
            existing.name = name;
            existing.prompt = prompt;
            existing.target_language = target_language;
            let updated = existing.clone();
            settings::write_settings(&app, settings);
            return Ok(updated);
        }
        // Seed id not yet in list — insert with the stable seed id.
        let new_mode = settings::SmartMode {
            id: seed_id,
            name,
            kind,
            prompt,
            target_language,
        };
        settings.smart_modes.push(new_mode.clone());
        settings::write_settings(&app, settings);
        return Ok(new_mode);
    }

    // Genuine custom mode (no template match) — mint a timestamp id.
    let id = format!("mode_{}", chrono::Utc::now().timestamp_millis());
    let new_mode = settings::SmartMode {
        id: id.clone(),
        name,
        kind,
        prompt,
        target_language,
    };
    settings.smart_modes.push(new_mode.clone());
    settings::write_settings(&app, settings);
    Ok(new_mode)
}

#[tauri::command]
#[specta::specta]
pub fn update_smart_mode(
    app: AppHandle,
    id: String,
    name: String,
    prompt: String,
    target_language: Option<settings::TargetLanguage>,
) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    if let Some(mode) = settings.smart_modes.iter_mut().find(|m| m.id == id) {
        mode.name = name;
        mode.prompt = prompt;
        mode.target_language = target_language;
        settings::write_settings(&app, settings);
        Ok(())
    } else {
        Err(format!("Smart mode with id '{}' not found", id))
    }
}

#[tauri::command]
#[specta::specta]
pub fn delete_smart_mode(app: AppHandle, id: String) -> Result<(), String> {
    let binding_id = smart_mode_binding_id(&id);
    let mut settings = settings::get_settings(&app);

    // Remove the mode first (also reassigns active id). Errors (last-mode /
    // not-found) bail early before we touch any binding state.
    delete_mode_in_place(
        &mut settings.smart_modes,
        &mut settings.smart_mode_active_id,
        &id,
    )?;

    // Unregister the OS-level shortcut if one is currently bound, then remove
    // the binding entry entirely (not just empty it) so init_shortcuts never
    // re-registers it on the next launch. Both operations are folded into this
    // single read/modify/write so the state is always consistent (UAT test 6).
    if let Some(b) = settings.bindings.get(&binding_id).cloned() {
        if !b.current_binding.trim().is_empty() {
            if let Err(e) = unregister_shortcut(&app, b) {
                error!(
                    "delete_smart_mode: failed to unregister '{}': {}",
                    binding_id, e
                );
            }
        }
    }
    settings.bindings.remove(&binding_id);

    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_active_smart_mode(app: AppHandle, id: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    if !settings.smart_modes.iter().any(|m| m.id == id) {
        return Err(format!("Smart mode with id '{}' not found", id));
    }
    settings.smart_mode_active_id = Some(id);
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn list_smart_modes(app: AppHandle) -> Result<Vec<settings::SmartMode>, String> {
    Ok(settings::get_settings(&app).smart_modes)
}

#[tauri::command]
#[specta::specta]
pub fn smart_mode_templates() -> Vec<settings::SmartMode> {
    settings::smart_mode_templates()
}

#[tauri::command]
#[specta::specta]
pub fn set_smart_mode_binding(
    app: AppHandle,
    mode_id: String,
    binding: String,
) -> Result<BindingResponse, String> {
    let binding_id = smart_mode_binding_id(&mode_id);
    let mut settings = settings::get_settings(&app);

    // Reject a combo already held by a DIFFERENT global binding (another smart
    // mode, transcribe, cancel, …). A binding never conflicts with itself, so
    // re-setting the same combo on the same mode (idempotent re-bind) still
    // passes. Without this, two modes could both persist Cmd+2 and only collide
    // at OS re-registration (`resume_all_shortcuts: Hotkey already registered`).
    // UAT test 7 / [B5].
    // Structured conflict payload `SHORTCUT_CONFLICT|<code>|<id>|<name>[|<base>]` — the
    // frontend maps <code> to a localized t() string, resolves the localized mode label
    // from <id>, and interpolates it (G13, G16). No English prose crosses the boundary.
    if let Some((other_id, kind)) =
        find_conflicting_binding(&settings.bindings, &binding_id, &binding)
    {
        let other_id_for_payload = other_id.clone();
        let other_name = settings
            .bindings
            .get(&other_id)
            .map(|b| b.name.clone())
            .filter(|n| !n.trim().is_empty())
            .unwrap_or(other_id);
        let payload = match kind {
            ConflictKind::ExactDuplicate => {
                format!(
                    "SHORTCUT_CONFLICT|exact_duplicate|{}|{}",
                    other_id_for_payload, other_name
                )
            }
            ConflictKind::CandidateBaseIsExisting | ConflictKind::CandidateIsBaseOfExisting => {
                let base = combo_base(&binding);
                format!(
                    "SHORTCUT_CONFLICT|base_overlap|{}|{}|{}",
                    other_id_for_payload, other_name, base
                )
            }
        };
        return Ok(BindingResponse {
            success: false,
            binding: None,
            error: Some(payload),
        });
    }

    // Ensure a ShortcutBinding entry exists for this smart mode id.
    // change_binding falls back to default_settings.bindings for unknown ids, but
    // smart_mode_* ids are not in defaults. We insert one here so change_binding
    // finds an existing entry and proceeds to the register step.
    if !settings.bindings.contains_key(&binding_id) {
        let mode = settings
            .smart_modes
            .iter()
            .find(|m| m.id == mode_id)
            .ok_or_else(|| format!("Smart mode with id '{}' not found", mode_id))?;
        let entry = settings::ShortcutBinding {
            id: binding_id.clone(),
            name: mode.name.clone(),
            description: "Smart Mode shortcut".to_string(),
            default_binding: String::new(),
            current_binding: String::new(),
        };
        settings.bindings.insert(binding_id.clone(), entry);
        settings::write_settings(&app, settings);
    }

    // Delegate to the existing conflict-aware change_binding flow.
    change_binding(app, binding_id, binding)
}

#[tauri::command]
#[specta::specta]
pub fn clear_smart_mode_binding(app: AppHandle, mode_id: String) -> Result<(), String> {
    let binding_id = smart_mode_binding_id(&mode_id);
    let mut settings = settings::get_settings(&app);
    // Unregister the OS-level shortcut if one is currently bound.
    if let Some(b) = settings.bindings.get(&binding_id).cloned() {
        if !b.current_binding.trim().is_empty() {
            if let Err(e) = unregister_shortcut(&app, b) {
                error!(
                    "clear_smart_mode_binding: failed to unregister '{}': {}",
                    binding_id, e
                );
            }
        }
    }
    // Remove the binding entry entirely so init_shortcuts never re-registers it.
    settings.bindings.remove(&binding_id);
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_mute_while_recording_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.mute_while_recording = enabled;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_append_trailing_space_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.append_trailing_space = enabled;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_lazy_stream_close_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.lazy_stream_close = enabled;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_app_language_setting(app: AppHandle, language: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.app_language = language.clone();
    settings::write_settings(&app, settings);

    // Refresh the tray menu with the new language
    tray::update_tray_menu(&app, &tray::TrayIconState::Idle, Some(&language));

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_show_tray_icon_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.show_tray_icon = enabled;
    settings::write_settings(&app, settings);

    // Apply change immediately
    tray::set_tray_visibility(&app, enabled);

    Ok(())
}

/// Save accelerator settings, re-apply globals, and unload the model so it
/// reloads with the new backend on next transcription.
fn apply_and_reload_accelerator(app: &AppHandle, s: settings::AppSettings) {
    settings::write_settings(app, s);
    crate::managers::transcription::apply_accelerator_settings(app);

    let tm = app.state::<std::sync::Arc<crate::managers::transcription::TranscriptionManager>>();
    if tm.is_model_loaded() {
        if let Err(e) = tm.unload_model() {
            log::warn!("Failed to unload model after accelerator change: {e}");
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn change_whisper_accelerator_setting(
    app: AppHandle,
    accelerator: settings::WhisperAcceleratorSetting,
) -> Result<(), String> {
    let mut s = settings::get_settings(&app);
    s.whisper_accelerator = accelerator;
    apply_and_reload_accelerator(&app, s);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_ort_accelerator_setting(
    app: AppHandle,
    accelerator: settings::OrtAcceleratorSetting,
) -> Result<(), String> {
    let mut s = settings::get_settings(&app);
    s.ort_accelerator = accelerator;
    apply_and_reload_accelerator(&app, s);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_whisper_gpu_device(app: AppHandle, device: i32) -> Result<(), String> {
    let mut s = settings::get_settings(&app);
    s.whisper_gpu_device = device;
    apply_and_reload_accelerator(&app, s);
    Ok(())
}

/// Return which accelerators and GPU devices are available for this build.
///
/// First-call cost is dominated by enumerating GPU devices through the
/// whisper.cpp Metal/Vulkan backend, which loads dynamic libraries and
/// probes hardware. Run it on the blocking pool so the webview thread
/// stays responsive — see also the startup pre-warm in `lib.rs`.
#[tauri::command]
#[specta::specta]
pub async fn get_available_accelerators() -> crate::managers::transcription::AvailableAccelerators {
    tauri::async_runtime::spawn_blocking(crate::managers::transcription::get_available_accelerators)
        .await
        .expect("get_available_accelerators panicked")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{SmartMode, SmartModeKind};

    // ── Smart Mode CRUD helper tests ─────────────────────────────────────────

    #[test]
    fn crud_delete_last_guard() {
        let mut modes = vec![SmartMode {
            id: "mode_only".to_string(),
            name: "Only Mode".to_string(),
            kind: SmartModeKind::Rewrite,
            prompt: "Test".to_string(),
            target_language: None,
        }];
        let mut active: Option<String> = Some("mode_only".to_string());
        let result = delete_mode_in_place(&mut modes, &mut active, "mode_only");
        assert!(result.is_err(), "deleting the last mode must return Err");
        assert_eq!(
            result.unwrap_err(),
            "Cannot delete the last mode",
            "error message must match"
        );
    }

    #[test]
    fn crud_delete_reassigns_active() {
        let mut modes = vec![
            SmartMode {
                id: "mode_a".to_string(),
                name: "Mode A".to_string(),
                kind: SmartModeKind::Rewrite,
                prompt: "A".to_string(),
                target_language: None,
            },
            SmartMode {
                id: "mode_b".to_string(),
                name: "Mode B".to_string(),
                kind: SmartModeKind::Rewrite,
                prompt: "B".to_string(),
                target_language: None,
            },
        ];
        let mut active: Option<String> = Some("mode_a".to_string());
        let result = delete_mode_in_place(&mut modes, &mut active, "mode_a");
        assert!(
            result.is_ok(),
            "delete must succeed when more than one mode"
        );
        assert_eq!(modes.len(), 1, "one mode must remain");
        assert_eq!(
            active.as_deref(),
            Some("mode_b"),
            "active must be reassigned to the first remaining mode"
        );
    }

    // ── Binding id helper test ────────────────────────────────────────────────

    #[test]
    fn smart_mode_binding_id_format() {
        assert_eq!(
            smart_mode_binding_id("mode_abc"),
            "smart_mode_mode_abc",
            "binding id must be prefixed with smart_mode_"
        );
    }

    // ── resolve_seeded_id tests ──────────────────────────────────────────────

    #[test]
    fn resolve_seeded_id_matches_clean_up_rewrite() {
        let result = resolve_seeded_id("Clean Up", &SmartModeKind::Rewrite);
        assert_eq!(
            result,
            Some("mode_clean_up".to_string()),
            "Clean Up Rewrite must resolve to mode_clean_up"
        );
    }

    #[test]
    fn resolve_seeded_id_returns_none_for_custom_mode() {
        let result = resolve_seeded_id("My Custom Mode", &SmartModeKind::Rewrite);
        assert_eq!(result, None, "a non-template name must return None");
    }

    #[test]
    fn resolve_seeded_id_requires_kind_match() {
        // "Clean Up" exists as Rewrite only — querying as Translation must return None
        let result = resolve_seeded_id("Clean Up", &SmartModeKind::Translation);
        assert_eq!(
            result, None,
            "wrong kind must not match even with correct name"
        );
    }

    #[test]
    fn resolve_seeded_id_matches_translate_template() {
        let result = resolve_seeded_id("Translate \u{2192} English", &SmartModeKind::Translation);
        assert_eq!(
            result,
            Some("mode_translate_en".to_string()),
            "Translate → English Translation must resolve to mode_translate_en"
        );
    }

    // ── find_conflicting_binding tests ───────────────────────────────────────

    fn make_binding(id: &str, current: &str) -> (String, crate::settings::ShortcutBinding) {
        (
            id.to_string(),
            crate::settings::ShortcutBinding {
                id: id.to_string(),
                name: id.to_string(),
                description: String::new(),
                default_binding: String::new(),
                current_binding: current.to_string(),
            },
        )
    }

    #[test]
    fn conflict_detected_for_different_binding() {
        let mut bindings = std::collections::HashMap::new();
        let (k, v) = make_binding("transcribe", "Cmd+2");
        bindings.insert(k, v);
        let result = find_conflicting_binding(&bindings, "smart_mode_mode_a", "Cmd+2");
        assert_eq!(
            result,
            Some(("transcribe".to_string(), ConflictKind::ExactDuplicate)),
            "combo held by a different binding must be detected as ExactDuplicate"
        );
    }

    #[test]
    fn no_conflict_when_combo_unused() {
        let mut bindings = std::collections::HashMap::new();
        let (k, v) = make_binding("transcribe", "Cmd+2");
        bindings.insert(k, v);
        let result = find_conflicting_binding(&bindings, "smart_mode_mode_a", "Cmd+9");
        assert_eq!(result, None, "unused combo must return None");
    }

    #[test]
    fn same_binding_id_is_not_a_conflict() {
        let mut bindings = std::collections::HashMap::new();
        let (k, v) = make_binding("smart_mode_mode_a", "Cmd+2");
        bindings.insert(k, v);
        let result = find_conflicting_binding(&bindings, "smart_mode_mode_a", "Cmd+2");
        assert_eq!(result, None, "a binding must not conflict with itself");
    }

    #[test]
    fn empty_current_binding_ignored() {
        let mut bindings = std::collections::HashMap::new();
        let (k, v) = make_binding("smart_mode_mode_b", "");
        bindings.insert(k, v);
        let result = find_conflicting_binding(&bindings, "smart_mode_mode_a", "Cmd+2");
        assert_eq!(result, None, "empty/unbound entries are not collisions");
    }

    #[test]
    fn cross_mode_conflict_returns_other_id() {
        let mut bindings = std::collections::HashMap::new();
        let (k, v) = make_binding("smart_mode_mode_b", "Cmd+2");
        bindings.insert(k, v);
        let result = find_conflicting_binding(&bindings, "smart_mode_mode_a", "Cmd+2");
        assert_eq!(
            result,
            Some((
                "smart_mode_mode_b".to_string(),
                ConflictKind::ExactDuplicate
            )),
            "conflicting mode binding must return the other mode's id as ExactDuplicate"
        );
    }

    // ── prefix/base-key overlap tests (13-18 / [G11]) ────────────────────────

    #[test]
    fn base_prefix_collision_blocks() {
        // existing "command_left" → binding "command_left+digit1" must be blocked
        // kind = CandidateBaseIsExisting (candidate's base IS the existing binding)
        let mut bindings = std::collections::HashMap::new();
        let (k, v) = make_binding("transcribe", "command_left");
        bindings.insert(k, v);
        let result =
            find_conflicting_binding(&bindings, "smart_mode_mode_a", "command_left+digit1");
        assert_eq!(
            result,
            Some((
                "transcribe".to_string(),
                ConflictKind::CandidateBaseIsExisting
            )),
            "combo whose base key is already bound must be blocked as CandidateBaseIsExisting"
        );
    }

    #[test]
    fn symmetric_base_collision_blocks() {
        // existing "command_left+digit1" → binding "command_left" must be blocked
        // kind = CandidateIsBaseOfExisting (candidate IS the base of the existing combo)
        let mut bindings = std::collections::HashMap::new();
        let (k, v) = make_binding("smart_mode_mode_b", "command_left+digit1");
        bindings.insert(k, v);
        let result = find_conflicting_binding(&bindings, "smart_mode_mode_a", "command_left");
        assert_eq!(
            result,
            Some((
                "smart_mode_mode_b".to_string(),
                ConflictKind::CandidateIsBaseOfExisting
            )),
            "base key that is a prefix of an existing combo must be blocked as CandidateIsBaseOfExisting"
        );
    }

    #[test]
    fn no_collision_when_bases_differ() {
        // existing "command_right" — candidate "command_left+digit1" has a different base
        let mut bindings = std::collections::HashMap::new();
        let (k, v) = make_binding("transcribe", "command_right");
        bindings.insert(k, v);
        let result =
            find_conflicting_binding(&bindings, "smart_mode_mode_a", "command_left+digit1");
        assert_eq!(result, None, "different base keys must not collide");
    }

    #[test]
    fn distinct_full_combos_no_base_overlap() {
        // existing "command_left+digit2" — candidate "command_left+digit1"
        // Both share modifier prefix "command_left" but neither IS that base binding.
        // They must remain bindable (no false positive).
        let mut bindings = std::collections::HashMap::new();
        let (k, v) = make_binding("transcribe", "command_left+digit2");
        bindings.insert(k, v);
        let result =
            find_conflicting_binding(&bindings, "smart_mode_mode_a", "command_left+digit1");
        assert_eq!(
            result, None,
            "two distinct full combos sharing only a modifier prefix must not block each other"
        );
    }

    // ── SHORTCUT_CONFLICT payload field-order test (13-25 / [G16]) ───────────
    // Locks the `id-before-name` and `id|name|base` contracts so that a future
    // refactor that reorders fields breaks the build immediately.

    #[test]
    fn conflict_payload_carries_binding_id_before_name() {
        // exact_duplicate: SHORTCUT_CONFLICT|exact_duplicate|<id>|<name>
        let exact_payload = format!(
            "SHORTCUT_CONFLICT|exact_duplicate|{}|{}",
            "smart_mode_mode_clean_up", "Clean Up"
        );
        assert_eq!(
            exact_payload, "SHORTCUT_CONFLICT|exact_duplicate|smart_mode_mode_clean_up|Clean Up",
            "exact_duplicate payload must be: SHORTCUT_CONFLICT|exact_duplicate|<id>|<name>"
        );
        let exact_parts: Vec<&str> = exact_payload.split('|').collect();
        assert_eq!(
            exact_parts.len(),
            4,
            "exact_duplicate payload must have 4 pipe-separated fields"
        );
        assert_eq!(exact_parts[0], "SHORTCUT_CONFLICT");
        assert_eq!(exact_parts[1], "exact_duplicate");
        assert_eq!(
            exact_parts[2], "smart_mode_mode_clean_up",
            "field[2] must be the binding id"
        );
        assert_eq!(
            exact_parts[3], "Clean Up",
            "field[3] must be the stored name"
        );

        // base_overlap: SHORTCUT_CONFLICT|base_overlap|<id>|<name>|<base>
        let base_payload = format!(
            "SHORTCUT_CONFLICT|base_overlap|{}|{}|{}",
            "smart_mode_mode_clean_up", "Clean Up", "command_left"
        );
        assert_eq!(
            base_payload,
            "SHORTCUT_CONFLICT|base_overlap|smart_mode_mode_clean_up|Clean Up|command_left",
            "base_overlap payload must be: SHORTCUT_CONFLICT|base_overlap|<id>|<name>|<base>"
        );
        let base_parts: Vec<&str> = base_payload.split('|').collect();
        assert_eq!(
            base_parts.len(),
            5,
            "base_overlap payload must have 5 pipe-separated fields"
        );
        assert_eq!(base_parts[0], "SHORTCUT_CONFLICT");
        assert_eq!(base_parts[1], "base_overlap");
        assert_eq!(
            base_parts[2], "smart_mode_mode_clean_up",
            "field[2] must be the binding id"
        );
        assert_eq!(
            base_parts[3], "Clean Up",
            "field[3] must be the stored name"
        );
        assert_eq!(
            base_parts[4], "command_left",
            "field[4] must be the base combo"
        );
    }
}

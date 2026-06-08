use crate::managers::llm::{LlmManager, LlmModelInfo};
use crate::settings::{get_settings, write_settings};
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
#[specta::specta]
pub async fn get_llm_models(
    llm_manager: State<'_, Arc<LlmManager>>,
) -> Result<Vec<LlmModelInfo>, String> {
    Ok(llm_manager.get_models())
}

#[tauri::command]
#[specta::specta]
pub async fn download_llm_model(
    llm_manager: State<'_, Arc<LlmManager>>,
    model_id: String,
) -> Result<(), String> {
    llm_manager
        .download_llm_model(&model_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn cancel_llm_download(
    llm_manager: State<'_, Arc<LlmManager>>,
    model_id: String,
) -> Result<(), String> {
    llm_manager
        .cancel_download(&model_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_llm_model(
    llm_manager: State<'_, Arc<LlmManager>>,
    model_id: String,
) -> Result<(), String> {
    llm_manager
        .delete_llm_model(&model_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn set_active_llm_model(
    app_handle: AppHandle,
    llm_manager: State<'_, Arc<LlmManager>>,
    model_id: String,
) -> Result<(), String> {
    llm_manager
        .load_model(&model_id)
        .await
        .map_err(|e| e.to_string())?;

    let mut settings = get_settings(&app_handle);
    settings.active_llm_model_id = Some(model_id);
    write_settings(&app_handle, settings);

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_active_llm_model(app_handle: AppHandle) -> Result<Option<String>, String> {
    let settings = get_settings(&app_handle);
    Ok(settings.active_llm_model_id)
}

#[tauri::command]
#[specta::specta]
pub async fn set_translation_engine_choice(
    app_handle: AppHandle,
    choice: crate::settings::TranslationEngineChoice,
) -> Result<(), String> {
    let mut settings = get_settings(&app_handle);
    settings.translation_engine_choice = choice;
    write_settings(&app_handle, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn set_translation_engine_to_embedded(
    app_handle: AppHandle,
    llm_manager: State<'_, Arc<LlmManager>>,
    model_id: String,
) -> Result<(), String> {
    // Load the GGUF model first (same as set_active_llm_model).
    llm_manager
        .load_model(&model_id)
        .await
        .map_err(|e| e.to_string())?;
    let mut settings = get_settings(&app_handle);
    // Remember the prior real provider ONLY if we are not already on embedded,
    // so a second Gemma click does not clobber the saved provider with "embedded".
    if settings.post_process_provider_id != "embedded" {
        settings.previous_post_process_provider_id =
            Some(settings.post_process_provider_id.clone());
    }
    settings.post_process_provider_id = "embedded".to_string();
    settings.active_llm_model_id = Some(model_id);
    settings.translation_engine_choice = crate::settings::TranslationEngineChoice::GenericModel;
    write_settings(&app_handle, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn restore_translation_engine_provider(app_handle: AppHandle) -> Result<(), String> {
    let mut settings = get_settings(&app_handle);
    // Only restore if we previously switched away from a real provider into embedded.
    if settings.post_process_provider_id == "embedded" {
        if let Some(prev) = settings.previous_post_process_provider_id.take() {
            settings.post_process_provider_id = prev;
        }
    }
    settings.translation_engine_choice = crate::settings::TranslationEngineChoice::GenericModel;
    write_settings(&app_handle, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn import_custom_llm_model(
    _app_handle: AppHandle,
    llm_manager: State<'_, Arc<LlmManager>>,
    path: String,
) -> Result<LlmModelInfo, String> {
    llm_manager.import_custom_gguf(Path::new(&path))
}

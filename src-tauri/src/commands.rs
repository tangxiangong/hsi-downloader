use crate::state::AppState;
use std::path::PathBuf;
use tauri::State;
use yushi_core::{
    AppConfig, CompletedTask, DownloadHistory, Task, TorrentFileInfo, types::AddTaskOptions,
};

#[tauri::command]
pub async fn get_tasks(state: State<'_, AppState>) -> Result<Vec<Task>, String> {
    Ok(state.queue.get_all_tasks().await)
}

#[tauri::command]
pub async fn add_task(
    state: State<'_, AppState>,
    options: AddTaskOptions,
) -> Result<String, String> {
    let task_id = state
        .queue
        .add_task_with_options(options)
        .await
        .map_err(|e| e.to_string())?;
    Ok(task_id)
}

#[tauri::command]
pub async fn pause_task(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    state
        .queue
        .pause_task(&task_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_task(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    state
        .queue
        .resume_task(&task_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_task(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    state
        .queue
        .cancel_task(&task_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_task(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    state
        .queue
        .remove_task(&task_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_task_with_file(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), String> {
    state
        .queue
        .remove_task_with_file(&task_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_completed(state: State<'_, AppState>) -> Result<(), String> {
    state
        .queue
        .clear_completed()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_history(state: State<'_, AppState>) -> Result<Vec<CompletedTask>, String> {
    let history = state.history.read().await;
    Ok(history.get_all().to_vec())
}

#[tauri::command]
pub async fn remove_history(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    let (new_history, _) = DownloadHistory::remove_from_file(&state.history_path, &task_id)
        .await
        .map_err(|e| e.to_string())?;
    *state.history.write().await = new_history;
    Ok(())
}

#[tauri::command]
pub async fn remove_history_with_file(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), String> {
    let (new_history, _) =
        DownloadHistory::remove_entry_and_file_from_file(&state.history_path, &task_id)
            .await
            .map_err(|e| e.to_string())?;
    *state.history.write().await = new_history;
    Ok(())
}

#[tauri::command]
pub async fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    let new_history = DownloadHistory::clear_file(&state.history_path)
        .await
        .map_err(|e| e.to_string())?;
    *state.history.write().await = new_history;
    Ok(())
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(state.config.read().await.clone())
}

#[tauri::command]
pub async fn update_config(state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    config.validate().map_err(|e| e.to_string())?;
    config
        .save(&state.config_path)
        .await
        .map_err(|e| e.to_string())?;

    state
        .queue
        .apply_runtime_config(config.downloader_config(), config.max_concurrent_tasks)
        .await
        .map_err(|e| e.to_string())?;

    *state.config.write().await = config;
    Ok(())
}

#[tauri::command]
pub async fn list_torrent_files(
    state: State<'_, AppState>,
    uri: String,
) -> Result<Vec<TorrentFileInfo>, String> {
    state
        .queue
        .list_torrent_files(&uri)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn infer_destination(
    state: State<'_, AppState>,
    url: String,
    directory: PathBuf,
) -> Result<PathBuf, String> {
    Ok(state.queue.infer_destination_in_dir(&url, directory).await)
}

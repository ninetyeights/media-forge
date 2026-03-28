use crate::services::watcher::{self, WatchFolderConfig, WatcherHolder};
use tauri::State;

#[tauri::command]
pub async fn start_folder_watcher(
    app: tauri::AppHandle,
    holder: State<'_, WatcherHolder>,
    folder: WatchFolderConfig,
) -> Result<(), String> {
    watcher::start_folder(app, folder, holder.inner().clone())
}

#[tauri::command]
pub async fn stop_folder_watcher(
    holder: State<'_, WatcherHolder>,
    path: String,
) -> Result<(), String> {
    watcher::stop_folder(&path, holder.inner())
}

#[tauri::command]
pub async fn get_active_watchers(
    holder: State<'_, WatcherHolder>,
) -> Result<Vec<String>, String> {
    Ok(watcher::active_paths(holder.inner()))
}

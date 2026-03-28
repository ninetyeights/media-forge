use crate::models::media_file::{ImageInfo, ProcessResult};
use crate::models::settings::ImageProcessSettings;
use crate::services::image_processor;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn get_image_info(path: String) -> Result<ImageInfo, String> {
    tokio::task::spawn_blocking(move || {
        image_processor::get_image_info(&path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Clone, serde::Serialize)]
struct ProgressPayload {
    #[serde(rename = "fileId")]
    file_id: String,
    progress: u8,
    status: String,
}

#[tauri::command]
pub async fn process_image(
    app: AppHandle,
    file_id: String,
    file_path: String,
    output_path: String,
    settings: ImageProcessSettings,
) -> Result<ProcessResult, String> {
    let fid = file_id.clone();
    let app_clone = app.clone();

    let on_progress = move |progress: u8, status: &str| {
        let _ = app_clone.emit("processing-progress", ProgressPayload {
            file_id: fid.clone(),
            progress,
            status: status.to_string(),
        });
    };

    tokio::task::spawn_blocking(move || {
        image_processor::process_image(&file_path, &output_path, &settings, &on_progress)
    })
    .await
    .map_err(|e| e.to_string())?
}

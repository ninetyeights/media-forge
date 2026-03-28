use crate::models::media_file::{ImageInfo, ProcessResult};
use crate::models::settings::ImageProcessSettings;
use crate::services::image_processor;

#[tauri::command]
pub async fn get_image_info(path: String) -> Result<ImageInfo, String> {
    tokio::task::spawn_blocking(move || {
        image_processor::get_image_info(&path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn process_image(
    file_path: String,
    output_path: String,
    settings: ImageProcessSettings,
) -> Result<ProcessResult, String> {
    tokio::task::spawn_blocking(move || {
        image_processor::process_image(&file_path, &output_path, &settings)
    })
    .await
    .map_err(|e| e.to_string())?
}

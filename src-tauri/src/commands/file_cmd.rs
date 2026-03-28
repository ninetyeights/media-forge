use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;

const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "avif", "gif", "bmp", "tiff", "tif"];
const VIDEO_EXTS: &[&str] = &["mp4", "webm", "mov", "avi", "mkv", "flv", "wmv"];
const AUDIO_EXTS: &[&str] = &["mp3", "aac", "ogg", "wav", "flac", "m4a", "wma"];

#[derive(Debug, Serialize)]
pub struct ScannedFile {
    pub path: String,
    pub base_dir: String,
}

/// Takes a list of paths (files and/or directories), recursively expands directories,
/// and returns all media files with their base directory for structure preservation.
#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    std::fs::remove_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn scan_paths(paths: Vec<String>, media_type: String) -> Vec<ScannedFile> {
    tokio::task::spawn_blocking(move || scan_paths_sync(paths, media_type))
        .await
        .unwrap_or_default()
}

fn scan_paths_sync(paths: Vec<String>, media_type: String) -> Vec<ScannedFile> {
    let exts: HashSet<&str> = match media_type.as_str() {
        "image" => IMAGE_EXTS.iter().copied().collect(),
        "video" => VIDEO_EXTS.iter().copied().collect(),
        "audio" => AUDIO_EXTS.iter().copied().collect(),
        _ => IMAGE_EXTS.iter().chain(VIDEO_EXTS.iter()).chain(AUDIO_EXTS.iter()).copied().collect(),
    };

    let mut result = Vec::new();
    for path_str in &paths {
        let path = Path::new(path_str);
        if path.is_dir() {
            // base_dir = parent of the dropped directory, so the folder name itself is preserved
            let base = path.parent()
                .and_then(|p| p.to_str())
                .unwrap_or(path_str);
            collect_files(path, base, &exts, &mut result);
        } else if is_matching_file(path, &exts) {
            // base_dir = parent of the file
            let base = path.parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();
            result.push(ScannedFile { path: path_str.clone(), base_dir: base });
        }
    }
    result
}

fn collect_files(dir: &Path, base_dir: &str, exts: &HashSet<&str>, result: &mut Vec<ScannedFile>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, base_dir, exts, result);
        } else if is_matching_file(&path, exts) {
            if let Some(s) = path.to_str() {
                result.push(ScannedFile {
                    path: s.to_string(),
                    base_dir: base_dir.to_string(),
                });
            }
        }
    }
}

fn is_matching_file(path: &Path, exts: &HashSet<&str>) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| exts.contains(e.to_lowercase().as_str()))
        .unwrap_or(false)
}

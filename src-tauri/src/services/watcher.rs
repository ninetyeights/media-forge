use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchFolderConfig {
    pub path: String,
    pub exts: Vec<String>,
    pub recursive: bool,
}

pub struct FolderWatcher {
    _debouncer: notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
}

/// Maps folder path -> active watcher
pub type WatcherHolder = Arc<Mutex<HashMap<String, FolderWatcher>>>;

pub fn create_holder() -> WatcherHolder {
    Arc::new(Mutex::new(HashMap::new()))
}

fn file_ext_lower(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
}

#[derive(Clone, serde::Serialize)]
pub struct WatcherEvent {
    pub path: String,
    pub base_dir: String,
}

/// Wait until a file's size stops changing, then return true.
/// Returns false if the file disappears or exceeds max wait time.
fn wait_until_stable(path: &Path) -> bool {
    let max_attempts = 20; // 20 × 300ms = 6 seconds max wait
    let interval = Duration::from_millis(300);

    let mut last_size: Option<u64> = None;
    let mut stable_count = 0;

    for _ in 0..max_attempts {
        let current_size = std::fs::metadata(path).ok().map(|m| m.len());

        match (current_size, last_size) {
            (Some(cur), Some(prev)) if cur == prev && cur > 0 => {
                stable_count += 1;
                if stable_count >= 2 {
                    return true; // Size unchanged for 2 consecutive checks
                }
            }
            (None, _) => return false, // File disappeared
            _ => {
                stable_count = 0;
            }
        }

        last_size = current_size;
        std::thread::sleep(interval);
    }

    // Timed out but file exists — proceed anyway
    last_size.is_some()
}

pub fn start_folder(
    app: AppHandle,
    config: WatchFolderConfig,
    holder: WatcherHolder,
) -> Result<(), String> {
    // Stop existing watcher for this path if any
    {
        let mut guard = holder.lock().map_err(|e| e.to_string())?;
        guard.remove(&config.path);
    }

    let exts: HashSet<String> = config.exts.iter().map(|e| e.to_lowercase()).collect();
    let base_dir = config.path.clone();
    let app_clone = app.clone();

    let mut debouncer = new_debouncer(
        Duration::from_millis(200),
        move |result: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            let events = match result {
                Ok(evts) => evts,
                Err(_) => return,
            };

            for event in events {
                if event.kind != DebouncedEventKind::Any {
                    continue;
                }
                let path = event.path.clone();
                if !path.is_file() {
                    continue;
                }
                let Some(path_str) = path.to_str().map(|s| s.to_string()) else { continue };
                if let Some(ext) = file_ext_lower(&path) {
                    if exts.contains(&ext) {
                        let app_ref = app_clone.clone();
                        let base = base_dir.clone();
                        // Spawn stability check on a separate thread to avoid blocking the watcher
                        std::thread::spawn(move || {
                            if wait_until_stable(&path) {
                                let _ = app_ref.emit(
                                    "watcher:new-file",
                                    WatcherEvent {
                                        path: path_str,
                                        base_dir: base,
                                    },
                                );
                            }
                        });
                    }
                }
            }
        },
    )
    .map_err(|e| format!("Failed to create watcher: {}", e))?;

    let mode = if config.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    debouncer
        .watcher()
        .watch(Path::new(&config.path), mode)
        .map_err(|e| format!("Failed to watch '{}': {}", config.path, e))?;

    let watcher = FolderWatcher {
        _debouncer: debouncer,
    };

    let mut guard = holder.lock().map_err(|e| e.to_string())?;
    guard.insert(config.path.clone(), watcher);

    Ok(())
}

pub fn stop_folder(path: &str, holder: &WatcherHolder) -> Result<(), String> {
    let mut guard = holder.lock().map_err(|e| e.to_string())?;
    guard.remove(path);
    Ok(())
}

pub fn active_paths(holder: &WatcherHolder) -> Vec<String> {
    let guard = holder.lock().unwrap_or_else(|e| e.into_inner());
    guard.keys().cloned().collect()
}

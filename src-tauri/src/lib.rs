mod commands;
mod models;
mod services;

use commands::{file_cmd, image_cmd, tray_cmd, watcher_cmd};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "macos")]
            {
                use window_vibrancy::{
                    apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState,
                };

                window.set_theme(Some(tauri::Theme::Dark)).ok();
                window.set_decorations(true).ok();
                window.set_title_bar_style(tauri::TitleBarStyle::Overlay).ok();

                apply_vibrancy(
                    &window,
                    NSVisualEffectMaterial::UnderWindowBackground,
                    Some(NSVisualEffectState::Active),
                    None,
                )
                .expect("Failed to apply vibrancy");
            }

            #[cfg(target_os = "windows")]
            {
                use window_vibrancy::apply_mica;
                let _ = apply_mica(&window, None);
            }

            Ok(())
        })
        .manage(services::watcher::create_holder())
        .invoke_handler(tauri::generate_handler![
            image_cmd::get_image_info,
            image_cmd::process_image,
            file_cmd::scan_paths,
            file_cmd::delete_file,
            watcher_cmd::start_folder_watcher,
            watcher_cmd::stop_folder_watcher,
            watcher_cmd::get_active_watchers,
            tray_cmd::update_tray,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

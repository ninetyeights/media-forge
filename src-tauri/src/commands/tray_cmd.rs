use serde::Deserialize;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex};
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

static TRAY_CREATED: AtomicBool = AtomicBool::new(false);
static FOLDER_PATHS: std::sync::LazyLock<Arc<Mutex<Vec<String>>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

#[derive(Debug, Deserialize)]
pub struct TrayFolder {
    pub name: String,
    pub path: String,
    pub media_type: String,
    pub auto_process: bool,
    pub active: bool,
}

#[tauri::command]
pub async fn update_tray(
    app: AppHandle,
    folders: Vec<TrayFolder>,
) -> Result<(), String> {
    fn media_label(t: &str) -> &'static str {
        match t {
            "image" => "图片",
            "video" => "视频",
            "audio" => "音频",
            _ => "未知",
        }
    }

    let active_count = folders.iter().filter(|f| f.active).count();
    let total = folders.len();

    let mut menu_builder = MenuBuilder::new(&app);

    // Folder submenus
    if folders.is_empty() {
        let empty = MenuItemBuilder::with_id("tray-empty", "暂无监听文件夹")
            .enabled(false)
            .build(&app)
            .map_err(|e| e.to_string())?;
        menu_builder = menu_builder.item(&empty);
    } else {
        for (i, f) in folders.iter().enumerate() {
            let dot = if f.active { "●" } else { "○" };
            let label = format!("{} {}", dot, f.name);

            let mut sub = SubmenuBuilder::with_id(&app, format!("tray-sub-{}", i), &label);

            let type_item = MenuItemBuilder::with_id(
                format!("tray-sub-{}-type", i),
                format!("类型: {}", media_label(&f.media_type)),
            )
            .enabled(false)
            .build(&app)
            .map_err(|e| e.to_string())?;
            sub = sub.item(&type_item);

            let auto_text = if f.auto_process { "开启" } else { "关闭" };
            let auto_item = MenuItemBuilder::with_id(
                format!("tray-sub-{}-auto", i),
                format!("自动处理: {}", auto_text),
            )
            .enabled(false)
            .build(&app)
            .map_err(|e| e.to_string())?;
            sub = sub.item(&auto_item);

            sub = sub.item(
                &PredefinedMenuItem::separator(&app).map_err(|e| e.to_string())?,
            );

            let toggle_text = if f.active { "停止监听" } else { "开始监听" };
            let toggle_item = MenuItemBuilder::with_id(
                format!("tray-toggle-{}", i),
                toggle_text,
            )
            .build(&app)
            .map_err(|e| e.to_string())?;
            sub = sub.item(&toggle_item);

            let submenu = sub.build().map_err(|e| e.to_string())?;
            menu_builder = menu_builder.item(&submenu);
        }
    }

    menu_builder = menu_builder.item(
        &PredefinedMenuItem::separator(&app).map_err(|e| e.to_string())?,
    );

    let show_item = MenuItemBuilder::with_id("tray-show", "显示窗口")
        .build(&app)
        .map_err(|e| e.to_string())?;
    menu_builder = menu_builder.item(&show_item);

    let quit_item = MenuItemBuilder::with_id("tray-quit", "退出")
        .build(&app)
        .map_err(|e| e.to_string())?;
    menu_builder = menu_builder.item(&quit_item);

    let menu = menu_builder.build().map_err(|e| e.to_string())?;

    let tooltip = if active_count > 0 {
        format!("媒体工坊 · 监听中 ({}/{})", active_count, total)
    } else {
        "媒体工坊".to_string()
    };

    let title = if active_count > 0 {
        format!("监听中 ({}/{})", active_count, total)
    } else if total > 0 {
        format!("已停止 ({})", total)
    } else {
        "无监听文件夹".to_string()
    };

    // Update shared folder paths for menu event handler
    {
        let mut paths = FOLDER_PATHS.lock().unwrap();
        *paths = folders.iter().map(|f| f.path.clone()).collect();
    }

    if TRAY_CREATED.load(Ordering::SeqCst) {
        if let Some(tray) = app.tray_by_id("main-tray") {
            tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
            tray.set_tooltip(Some(&tooltip)).map_err(|e| e.to_string())?;
            tray.set_title(Some(&title)).map_err(|e| e.to_string())?;
        }
    } else {
        TRAY_CREATED.store(true, Ordering::SeqCst);

        let icon = Image::from_bytes(include_bytes!("../../icons/32x32.png"))
            .map_err(|e| e.to_string())?;

        let tray = TrayIconBuilder::with_id("main-tray")
            .icon(icon)
            .tooltip(&tooltip)
            .title(&title)
            .menu(&menu)
            .on_menu_event(|app, event| {
                let id = event.id().0.as_str();
                if id.starts_with("tray-toggle-") {
                    if let Ok(idx) = id.strip_prefix("tray-toggle-").unwrap().parse::<usize>() {
                        let paths = FOLDER_PATHS.lock().unwrap();
                        if let Some(path) = paths.get(idx) {
                            let _ = app.emit("tray:toggle-folder", path.clone());
                        }
                    }
                } else {
                    match id {
                        "tray-show" => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                        "tray-quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                }
            })
            .build(&app)
            .map_err(|e| e.to_string())?;

        tray.set_title(Some(&title)).map_err(|e| e.to_string())?;
    }

    Ok(())
}

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

#[cfg(not(target_os = "macos"))]
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // メニューバー専用アプリとして Dock に出さない
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let quit = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;

            let tray = TrayIconBuilder::with_id("main")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    if event.id.as_ref() == "quit" {
                        app.exit(0);
                    }
                });

            // macOS はメニューバーに残り時間テキストのみ表示 (🍅 が icon 代わり)。
            // Windows/Linux は title 非対応なので icon + tooltip で代替
            #[cfg(target_os = "macos")]
            let tray = tray.title("🍅 --:--");
            #[cfg(not(target_os = "macos"))]
            let tray = tray
                .icon(app.default_window_icon().expect("bundled icon").clone())
                .tooltip("pomo-tree");

            tray.build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

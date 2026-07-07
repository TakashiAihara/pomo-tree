mod engine;
pub mod timer;
mod tray;

use std::{sync::Mutex, time::Instant};

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager,
};

use timer::{Timer, TimerConfig};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // メニューバー専用アプリとして Dock に出さない
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let timer = Timer::new(TimerConfig::default());
            let now = Instant::now();
            let initial_text =
                tray::countdown_text(timer.phase(), timer.status(), timer.remaining(now));

            let toggle = MenuItem::with_id(app, engine::MENU_TOGGLE, "開始", true, None::<&str>)?;
            let skip = MenuItem::with_id(app, engine::MENU_SKIP, "スキップ", true, None::<&str>)?;
            let reset = MenuItem::with_id(app, engine::MENU_RESET, "リセット", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, engine::MENU_QUIT, "終了", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(app, &[&toggle, &skip, &reset, &separator, &quit])?;

            let tray = TrayIconBuilder::with_id(tray::TRAY_ID)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| engine::handle_menu_event(app, event.id.as_ref()));

            // macOS はメニューバーに残り時間テキストのみ表示 (🍅 が icon 代わり)。
            // Windows/Linux は title 非対応なので状態色 icon + tooltip で代替
            #[cfg(target_os = "macos")]
            let tray = tray.title(&initial_text);
            #[cfg(not(target_os = "macos"))]
            let tray = tray
                .icon(tray::state_icon(timer.phase(), timer.status()))
                .tooltip(&initial_text);

            tray.build(app)?;

            app.manage(engine::AppState {
                timer: Mutex::new(timer),
                toggle_item: toggle,
            });
            engine::spawn_ticker(app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

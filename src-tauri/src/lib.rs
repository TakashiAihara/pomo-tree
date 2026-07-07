mod engine;
mod history;
mod notify;
mod settings;
pub mod timer;
mod tray;

use std::{sync::Mutex, time::Instant};

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

use settings::Settings;
use timer::Timer;

#[tauri::command]
fn get_settings(app: AppHandle) -> Settings {
    settings::load(&app)
}

#[tauri::command]
fn set_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    settings.validate()?;
    settings::save(&app, &settings)?;

    let state = app.state::<engine::AppState>();
    state
        .timer
        .lock()
        .unwrap()
        .update_config(settings.to_timer_config());
    engine::refresh_ui(&app);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![get_settings, set_settings])
        .setup(|app| {
            // メニューバー専用アプリとして Dock に出さない
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let timer = Timer::new(settings::load(app.handle()).to_timer_config());
            let now = Instant::now();
            let initial_text =
                tray::countdown_text(timer.phase(), timer.status(), timer.remaining(now));

            let toggle = MenuItem::with_id(app, engine::MENU_TOGGLE, "開始", true, None::<&str>)?;
            let skip = MenuItem::with_id(app, engine::MENU_SKIP, "スキップ", true, None::<&str>)?;
            let reset = MenuItem::with_id(app, engine::MENU_RESET, "リセット", true, None::<&str>)?;
            let open_settings =
                MenuItem::with_id(app, engine::MENU_SETTINGS, "設定…", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, engine::MENU_QUIT, "終了", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[
                    &toggle,
                    &skip,
                    &reset,
                    &PredefinedMenuItem::separator(app)?,
                    &open_settings,
                    &PredefinedMenuItem::separator(app)?,
                    &quit,
                ],
            )?;

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

            let history_path = app.path().app_data_dir()?.join(history::HISTORY_FILE);

            app.manage(engine::AppState {
                timer: Mutex::new(timer),
                toggle_item: toggle,
                history: history::HistoryWriter::new(history_path),
                session_started_at: Mutex::new(None),
            });
            engine::spawn_ticker(app.handle().clone());

            Ok(())
        })
        .on_window_event(|window, event| {
            // 設定ウィンドウの close はアプリ終了ではなく非表示に落とす
            // (終了はトレイメニューの「終了」から)
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

//! タイマーの駆動 (ticker スレッド) とトレイメニュー操作の配線。

use std::{
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

use tauri::{menu::MenuItem, AppHandle, Manager, Wry};

use crate::{
    timer::{Status, Timer},
    tray,
};

pub const MENU_TOGGLE: &str = "toggle";
pub const MENU_SKIP: &str = "skip";
pub const MENU_RESET: &str = "reset";
pub const MENU_QUIT: &str = "quit";

const TICK_INTERVAL: Duration = Duration::from_millis(500);

pub struct AppState {
    pub timer: Mutex<Timer>,
    /// 開始 / 一時停止 / 再開 を状態に応じて差し替えるトグル項目
    pub toggle_item: MenuItem<Wry>,
}

pub fn handle_menu_event(app: &AppHandle, id: &str) {
    if id == MENU_QUIT {
        app.exit(0);
        return;
    }

    let state = app.state::<AppState>();
    let now = Instant::now();
    let mut timer = state.timer.lock().unwrap();

    match id {
        MENU_TOGGLE => match timer.status() {
            Status::Idle => timer.start(now),
            Status::Running => timer.pause(now),
            Status::Paused => timer.resume(now),
        },
        MENU_SKIP => timer.skip(),
        MENU_RESET => timer.reset(),
        _ => {}
    }

    sync_ui(app, &timer, &state.toggle_item, now);
}

/// 500ms ごとに満了判定とトレイ表示更新を行うスレッドを起動する。
/// 残り時間は Instant 起点の算出なので tick の遅延・スリープで精度は落ちない。
pub fn spawn_ticker(app: AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(TICK_INTERVAL);

        let state = app.state::<AppState>();
        let now = Instant::now();
        let mut timer = state.timer.lock().unwrap();

        if let Some(_completed) = timer.poll(now) {
            // PR3: ここでセッション終了通知と履歴記録を行う
        }

        sync_ui(&app, &timer, &state.toggle_item, now);
    });
}

fn sync_ui(app: &AppHandle, timer: &Timer, toggle_item: &MenuItem<Wry>, now: Instant) {
    tray::update(app, timer, now);
    let _ = toggle_item.set_text(toggle_label(timer.status()));
}

fn toggle_label(status: Status) -> &'static str {
    match status {
        Status::Idle => "開始",
        Status::Running => "一時停止",
        Status::Paused => "再開",
    }
}

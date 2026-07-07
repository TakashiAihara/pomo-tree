//! タイマーの駆動 (ticker スレッド) とトレイメニュー操作の配線。

use std::{
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

use chrono::{DateTime, Local};
use tauri::{menu::MenuItem, AppHandle, Manager, Wry};

use crate::{
    history::HistoryWriter,
    notify,
    timer::{Status, Timer},
    tray,
};

pub const MENU_TOGGLE: &str = "toggle";
pub const MENU_SKIP: &str = "skip";
pub const MENU_RESET: &str = "reset";
pub const MENU_SETTINGS: &str = "settings";
pub const MENU_QUIT: &str = "quit";

pub const SETTINGS_WINDOW: &str = "main";

const TICK_INTERVAL: Duration = Duration::from_millis(500);

pub struct AppState {
    pub timer: Mutex<Timer>,
    /// 開始 / 一時停止 / 再開 を状態に応じて差し替えるトグル項目
    pub toggle_item: MenuItem<Wry>,
    pub history: HistoryWriter,
    /// 進行中セッションの開始時刻 (履歴記録用の wall-clock)。
    /// Running/Paused 中のみ Some。timer の後にロックすること (ロック順固定)
    pub session_started_at: Mutex<Option<DateTime<Local>>>,
}

pub fn handle_menu_event(app: &AppHandle, id: &str) {
    if id == MENU_QUIT {
        app.exit(0);
        return;
    }

    if id == MENU_SETTINGS {
        if let Some(window) = app.get_webview_window(SETTINGS_WINDOW) {
            let _ = window.show();
            let _ = window.set_focus();
        }
        return;
    }

    let state = app.state::<AppState>();
    let now = Instant::now();
    let mut timer = state.timer.lock().unwrap();

    match id {
        MENU_TOGGLE => match timer.status() {
            Status::Idle => {
                timer.start(now);
                *state.session_started_at.lock().unwrap() = Some(Local::now());
            }
            Status::Running => timer.pause(now),
            Status::Paused => timer.resume(now),
        },
        MENU_SKIP | MENU_RESET => {
            // 走り出していたセッションの破棄は completed=false で記録する
            let abandoned_phase = timer.phase();
            if let Some(started) = state.session_started_at.lock().unwrap().take() {
                state
                    .history
                    .append(abandoned_phase, started, Local::now(), false);
            }
            if id == MENU_SKIP {
                timer.skip();
            } else {
                timer.reset();
            }
        }
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

        if let Some(completed) = timer.poll(now) {
            let ended_at = Local::now();
            let mut session = state.session_started_at.lock().unwrap();
            if let Some(started) = session.take() {
                state
                    .history
                    .append(completed.finished, started, ended_at, true);
            }
            if completed.auto_started {
                *session = Some(ended_at);
            }
            drop(session);

            notify::session_finished(&app, &completed);
        }

        sync_ui(&app, &timer, &state.toggle_item, now);
    });
}

/// 設定変更コマンドなど、メニュー / ticker 以外の契機からの表示更新に使う
pub fn refresh_ui(app: &AppHandle) {
    let state = app.state::<AppState>();
    let timer = state.timer.lock().unwrap();
    sync_ui(app, &timer, &state.toggle_item, Instant::now());
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

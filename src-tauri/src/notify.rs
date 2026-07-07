//! セッション終了時の OS ネイティブ通知。
//!
//! サウンドは builder で指定できない (tauri-plugin-notification の sound は
//! mobile 専用、Windows はデフォルト音が自動再生、macOS は OS の通知設定に従う)。
//! dev ビルドの macOS では通知の出所が Terminal 扱いになる (notify-rust の仕様)。

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::timer::{Phase, PhaseCompleted};

pub fn session_finished(app: &AppHandle, event: &PhaseCompleted) {
    let (title, body) = match (event.finished, event.next) {
        (Phase::Work, Phase::LongBreak) => {
            ("ポモドーロ完了 🍅", "おつかれさま。長休憩にしましょう 🌿")
        }
        (Phase::Work, _) => ("ポモドーロ完了 🍅", "短休憩にしましょう ☕"),
        (Phase::ShortBreak | Phase::LongBreak, _) => {
            ("休憩おわり", "次のポモドーロを始めましょう 🍅")
        }
    };

    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        eprintln!("failed to show notification: {e}");
    }
}

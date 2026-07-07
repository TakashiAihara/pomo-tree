//! トレイ表示の更新。macOS (title テキスト) / Windows・Linux (icon + tooltip) の
//! プラットフォーム差分はこのモジュールに閉じ込める。

use std::time::{Duration, Instant};

use tauri::AppHandle;

use crate::timer::{Phase, Status, Timer};

pub const TRAY_ID: &str = "main";

/// メニューバー / tooltip に出すカウントダウン文字列。
/// MM:SS 固定幅にしてメニューバーの幅ブレを抑える。
pub fn countdown_text(phase: Phase, status: Status, remaining: Duration) -> String {
    let emoji = match (status, phase) {
        (Status::Paused, _) => "⏸",
        (_, Phase::Work) => "🍅",
        (_, Phase::ShortBreak) => "☕",
        (_, Phase::LongBreak) => "🌿",
    };
    let total = remaining.as_secs();

    format!("{} {:02}:{:02}", emoji, total / 60, total % 60)
}

pub fn update(app: &AppHandle, timer: &Timer, now: Instant) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let text = countdown_text(timer.phase(), timer.status(), timer.remaining(now));

    #[cfg(target_os = "macos")]
    let _ = tray.set_title(Some(&text));

    #[cfg(not(target_os = "macos"))]
    {
        use std::sync::Mutex;

        let _ = tray.set_tooltip(Some(&text));

        // icon の再生成は状態が変わったときだけにする
        static LAST_KEY: Mutex<Option<(Phase, Status)>> = Mutex::new(None);
        let key = (timer.phase(), timer.status());
        let mut last = LAST_KEY.lock().unwrap();
        if *last != Some(key) {
            let _ = tray.set_icon(Some(state_icon(key.0, key.1)));
            *last = Some(key);
        }
    }
}

/// Windows/Linux 用の状態色アイコン (作業=赤 / 休憩=緑 / 停止=グレー)。
/// アセットファイルを持たず 32x32 の塗り circle を実行時に生成する。
#[cfg(not(target_os = "macos"))]
pub fn state_icon(phase: Phase, status: Status) -> tauri::image::Image<'static> {
    let rgb = match (status, phase) {
        (Status::Running, Phase::Work) => (0xE5, 0x4D, 0x42),
        (Status::Running, Phase::ShortBreak | Phase::LongBreak) => (0x4C, 0xAF, 0x50),
        (Status::Idle | Status::Paused, _) => (0x9E, 0x9E, 0x9E),
    };

    const SIZE: usize = 32;
    let mut buf = vec![0u8; SIZE * SIZE * 4];
    let center = (SIZE as f32 - 1.0) / 2.0;
    let radius = SIZE as f32 / 2.0 - 1.0;

    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            if dx * dx + dy * dy <= radius * radius {
                let i = (y * SIZE + x) * 4;
                buf[i] = rgb.0;
                buf[i + 1] = rgb.1;
                buf[i + 2] = rgb.2;
                buf[i + 3] = 0xFF;
            }
        }
    }

    tauri::image::Image::new_owned(buf, SIZE as u32, SIZE as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn countdown_text_shows_phase_emoji_and_fixed_width_time() {
        let text = countdown_text(Phase::Work, Status::Running, Duration::from_secs(1471));
        assert_eq!(text, "🍅 24:31");

        let text = countdown_text(Phase::ShortBreak, Status::Idle, Duration::from_secs(300));
        assert_eq!(text, "☕ 05:00");

        let text = countdown_text(Phase::LongBreak, Status::Idle, Duration::from_secs(900));
        assert_eq!(text, "🌿 15:00");
    }

    #[test]
    fn countdown_text_shows_pause_marker_over_phase() {
        let text = countdown_text(Phase::Work, Status::Paused, Duration::from_secs(60));
        assert_eq!(text, "⏸ 01:00");
    }
}

//! ユーザー設定の定義と tauri-plugin-store への永続化。

use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::timer::TimerConfig;

pub const STORE_FILE: &str = "settings.json";
pub const STORE_KEY: &str = "settings";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub work_minutes: u32,
    pub short_break_minutes: u32,
    pub long_break_minutes: u32,
    pub pomodoros_until_long_break: u32,
    pub auto_start_next: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            pomodoros_until_long_break: 4,
            auto_start_next: false,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        for (label, minutes) in [
            ("作業時間", self.work_minutes),
            ("短休憩", self.short_break_minutes),
            ("長休憩", self.long_break_minutes),
        ] {
            if !(1..=180).contains(&minutes) {
                return Err(format!("{label}は 1〜180 分で指定してください"));
            }
        }

        if !(1..=12).contains(&self.pomodoros_until_long_break) {
            return Err("長休憩までのポモドーロ数は 1〜12 で指定してください".into());
        }

        Ok(())
    }

    pub fn to_timer_config(&self) -> TimerConfig {
        TimerConfig {
            work: Duration::from_secs(u64::from(self.work_minutes) * 60),
            short_break: Duration::from_secs(u64::from(self.short_break_minutes) * 60),
            long_break: Duration::from_secs(u64::from(self.long_break_minutes) * 60),
            pomodoros_until_long_break: self.pomodoros_until_long_break,
            auto_start_next: self.auto_start_next,
        }
    }
}

/// store から設定を読む。未保存・壊れた値は default に落とす
pub fn load(app: &AppHandle) -> Settings {
    let Ok(store) = app.store(STORE_FILE) else {
        return Settings::default();
    };

    store
        .get(STORE_KEY)
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default()
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    let value = serde_json::to_value(settings).map_err(|e| e.to_string())?;

    store.set(STORE_KEY, value);
    store.save().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_valid() {
        assert_eq!(Settings::default().validate(), Ok(()));
    }

    #[test]
    fn rejects_out_of_range_minutes() {
        for (work_minutes, expected_ok) in [(0, false), (181, false), (1, true), (180, true)] {
            let settings = Settings {
                work_minutes,
                ..Settings::default()
            };
            assert_eq!(settings.validate().is_ok(), expected_ok);
        }
    }

    #[test]
    fn rejects_out_of_range_pomodoro_count() {
        for count in [0, 13] {
            let settings = Settings {
                pomodoros_until_long_break: count,
                ..Settings::default()
            };
            assert!(settings.validate().is_err());
        }
    }

    #[test]
    fn converts_minutes_to_timer_config_durations() {
        let settings = Settings {
            work_minutes: 50,
            short_break_minutes: 10,
            long_break_minutes: 30,
            pomodoros_until_long_break: 2,
            auto_start_next: true,
        };
        let config = settings.to_timer_config();

        assert_eq!(config.work, Duration::from_secs(3000));
        assert_eq!(config.short_break, Duration::from_secs(600));
        assert_eq!(config.long_break, Duration::from_secs(1800));
        assert_eq!(config.pomodoros_until_long_break, 2);
        assert!(config.auto_start_next);
    }

    #[test]
    fn deserializes_camel_case_with_missing_fields_as_default() {
        let settings: Settings =
            serde_json::from_str(r#"{"workMinutes": 50}"#).expect("should deserialize");
        assert_eq!(settings.work_minutes, 50);
        assert_eq!(settings.short_break_minutes, 5);
    }
}

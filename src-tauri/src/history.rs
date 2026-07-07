//! セッション履歴の JSONL 追記。v0.1 では記録のみで表示 UI を持たない
//! (v0.2 のトマトの樹・統計の育成素材)。1 行 1 セッションで、後方互換を
//! 保ったままフィールドを足せる (v0.3 で taskId を追加予定)。

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use chrono::{DateTime, Local};
use serde::Serialize;

use crate::timer::Phase;

pub const HISTORY_FILE: &str = "sessions.jsonl";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionRecord {
    phase: &'static str,
    started_at: String,
    ended_at: String,
    completed: bool,
}

pub struct HistoryWriter {
    path: PathBuf,
}

impl HistoryWriter {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// 1 セッションを追記する。履歴の書き込み失敗でタイマー本体を
    /// 止めたくないので、エラーは stderr に出すだけで握りつぶす。
    pub fn append(
        &self,
        phase: Phase,
        started_at: DateTime<Local>,
        ended_at: DateTime<Local>,
        completed: bool,
    ) {
        let record = SessionRecord {
            phase: phase_label(phase),
            started_at: started_at.to_rfc3339(),
            ended_at: ended_at.to_rfc3339(),
            completed,
        };

        if let Err(e) = self.try_append(&record) {
            eprintln!("failed to append session history: {e}");
        }
    }

    fn try_append(&self, record: &SessionRecord) -> std::io::Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }

        let line = serde_json::to_string(record).map_err(std::io::Error::other)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{line}")
    }
}

fn phase_label(phase: Phase) -> &'static str {
    match phase {
        Phase::Work => "work",
        Phase::ShortBreak => "shortBreak",
        Phase::LongBreak => "longBreak",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn timestamp(hour: u32, min: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(2026, 7, 7, hour, min, 0).unwrap()
    }

    #[test]
    fn appends_one_json_line_per_session() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(HISTORY_FILE);
        let writer = HistoryWriter::new(path.clone());

        writer.append(Phase::Work, timestamp(10, 0), timestamp(10, 25), true);
        writer.append(
            Phase::ShortBreak,
            timestamp(10, 25),
            timestamp(10, 27),
            false,
        );

        let content = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);

        let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(first["phase"], "work");
        assert_eq!(first["completed"], true);
        assert!(first["startedAt"]
            .as_str()
            .unwrap()
            .starts_with("2026-07-07T10:00:00"));
        assert!(first["endedAt"]
            .as_str()
            .unwrap()
            .starts_with("2026-07-07T10:25:00"));

        let second: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(second["phase"], "shortBreak");
        assert_eq!(second["completed"], false);
    }

    #[test]
    fn creates_parent_directory_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("dir").join(HISTORY_FILE);
        let writer = HistoryWriter::new(path.clone());

        writer.append(Phase::Work, timestamp(9, 0), timestamp(9, 25), true);

        assert!(path.exists());
    }
}

//! ポモドーロタイマーの状態機械。OS / Tauri 非依存の純粋ロジック。
//!
//! 残り時間は「開始時点の Instant からの経過」で算出する (1 秒 interval の
//! 減算カウンタにしない)。スリープ復帰やイベントループ遅延があっても、
//! poll() 時点の now で正しい残り時間・満了判定に収束する。

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Idle,
    Running,
    Paused,
}

#[derive(Debug, Clone)]
pub struct TimerConfig {
    pub work: Duration,
    pub short_break: Duration,
    pub long_break: Duration,
    pub pomodoros_until_long_break: u32,
    pub auto_start_next: bool,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            work: Duration::from_secs(25 * 60),
            short_break: Duration::from_secs(5 * 60),
            long_break: Duration::from_secs(15 * 60),
            pomodoros_until_long_break: 4,
            auto_start_next: false,
        }
    }
}

/// poll() でフェーズが満了したときに返るイベント。PR3 で通知・履歴記録に使う。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseCompleted {
    pub finished: Phase,
    pub next: Phase,
    pub auto_started: bool,
}

#[derive(Debug)]
pub struct Timer {
    config: TimerConfig,
    phase: Phase,
    status: Status,
    /// Idle / Paused 時の残り時間。Running 中は running_since と併せて算出する
    remaining: Duration,
    running_since: Option<Instant>,
    completed_pomodoros: u32,
}

impl Timer {
    pub fn new(config: TimerConfig) -> Self {
        let remaining = config.work;
        Self {
            config,
            phase: Phase::Work,
            status: Status::Idle,
            remaining,
            running_since: None,
            completed_pomodoros: 0,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn completed_pomodoros(&self) -> u32 {
        self.completed_pomodoros
    }

    pub fn remaining(&self, now: Instant) -> Duration {
        match (self.status, self.running_since) {
            (Status::Running, Some(since)) => {
                self.remaining.saturating_sub(now.duration_since(since))
            }
            _ => self.remaining,
        }
    }

    pub fn start(&mut self, now: Instant) {
        if self.status == Status::Idle {
            self.running_since = Some(now);
            self.status = Status::Running;
        }
    }

    pub fn pause(&mut self, now: Instant) {
        if self.status == Status::Running {
            self.remaining = self.remaining(now);
            self.running_since = None;
            self.status = Status::Paused;
        }
    }

    pub fn resume(&mut self, now: Instant) {
        if self.status == Status::Paused {
            self.running_since = Some(now);
            self.status = Status::Running;
        }
    }

    /// 現フェーズを完了扱いにせず破棄して次フェーズの先頭 (Idle) に進む。
    /// 作業フェーズのスキップはポモドーロカウントに数えない (完了ではないため)。
    /// スキップ後の遷移先は「作業 → 短休憩 / 休憩 → 作業」に固定する
    /// (長休憩の判定は完了ベースのカウントにのみ紐付ける)。
    pub fn skip(&mut self) {
        let next = match self.phase {
            Phase::Work => Phase::ShortBreak,
            Phase::ShortBreak | Phase::LongBreak => Phase::Work,
        };
        self.enter_idle(next);
    }

    /// 現フェーズを破棄して作業フェーズの先頭 (Idle) に戻す。
    /// completed_pomodoros は維持する (リセットはやり直しであって帳消しではない)。
    pub fn reset(&mut self) {
        self.enter_idle(Phase::Work);
    }

    /// Running 中の満了判定。満了していればフェーズを進めてイベントを返す。
    /// スリープ復帰などで大幅に超過していても完了は 1 回だけ発火し、
    /// 超過分を次フェーズに繰り越さない (auto start 時は now を起点に開始する)。
    pub fn poll(&mut self, now: Instant) -> Option<PhaseCompleted> {
        if self.status != Status::Running || self.remaining(now) > Duration::ZERO {
            return None;
        }

        let finished = self.phase;
        if finished == Phase::Work {
            self.completed_pomodoros += 1;
        }

        let next = match finished {
            Phase::Work => {
                if self
                    .completed_pomodoros
                    .is_multiple_of(self.config.pomodoros_until_long_break)
                {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                }
            }
            Phase::ShortBreak | Phase::LongBreak => Phase::Work,
        };

        let auto_started = self.config.auto_start_next;
        self.phase = next;
        self.remaining = self.duration_of(next);
        if auto_started {
            self.running_since = Some(now);
            self.status = Status::Running;
        } else {
            self.running_since = None;
            self.status = Status::Idle;
        }

        Some(PhaseCompleted {
            finished,
            next,
            auto_started,
        })
    }

    /// 設定変更を反映する。Idle 中は現フェーズの残り時間を新設定で引き直し、
    /// Running/Paused 中のフェーズは触らず次フェーズから新しい長さを使う
    /// (進行中の残り時間が突然変わると混乱するため)。
    pub fn update_config(&mut self, config: TimerConfig) {
        self.config = config;
        if self.status == Status::Idle {
            self.remaining = self.duration_of(self.phase);
        }
    }

    fn enter_idle(&mut self, phase: Phase) {
        self.phase = phase;
        self.remaining = self.duration_of(phase);
        self.running_since = None;
        self.status = Status::Idle;
    }

    fn duration_of(&self, phase: Phase) -> Duration {
        match phase {
            Phase::Work => self.config.work,
            Phase::ShortBreak => self.config.short_break,
            Phase::LongBreak => self.config.long_break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    fn test_config() -> TimerConfig {
        TimerConfig {
            work: secs(1500),
            short_break: secs(300),
            long_break: secs(900),
            pomodoros_until_long_break: 4,
            auto_start_next: false,
        }
    }

    fn new_timer() -> (Timer, Instant) {
        (Timer::new(test_config()), Instant::now())
    }

    /// timer を running のまま満了時刻まで進めて poll する
    fn run_to_completion(timer: &mut Timer, start: Instant) -> (PhaseCompleted, Instant) {
        timer.start(start);
        let full = timer.remaining(start);
        let at_end = start + full;
        let event = timer.poll(at_end).expect("phase should complete");
        (event, at_end)
    }

    #[test]
    fn starts_idle_at_full_work_duration() {
        let (timer, now) = new_timer();
        assert_eq!(timer.phase(), Phase::Work);
        assert_eq!(timer.status(), Status::Idle);
        assert_eq!(timer.remaining(now), secs(1500));
        assert_eq!(timer.completed_pomodoros(), 0);
    }

    #[test]
    fn remaining_decreases_while_running() {
        let (mut timer, now) = new_timer();
        timer.start(now);
        assert_eq!(timer.status(), Status::Running);
        assert_eq!(timer.remaining(now + secs(10)), secs(1490));
    }

    #[test]
    fn start_is_noop_unless_idle() {
        let (mut timer, now) = new_timer();
        timer.start(now);
        timer.start(now + secs(100)); // running 中の start は無視
        assert_eq!(timer.remaining(now + secs(100)), secs(1400));

        timer.pause(now + secs(100));
        timer.start(now + secs(200)); // paused 中の start も無視 (resume を使う)
        assert_eq!(timer.status(), Status::Paused);
    }

    #[test]
    fn pause_freezes_and_resume_continues() {
        let (mut timer, now) = new_timer();
        timer.start(now);
        timer.pause(now + secs(100));
        assert_eq!(timer.status(), Status::Paused);
        assert_eq!(timer.remaining(now + secs(9999)), secs(1400));

        timer.resume(now + secs(200));
        assert_eq!(timer.status(), Status::Running);
        assert_eq!(timer.remaining(now + secs(210)), secs(1390));
    }

    #[test]
    fn poll_returns_none_before_expiry() {
        let (mut timer, now) = new_timer();
        timer.start(now);
        assert_eq!(timer.poll(now + secs(1499)), None);
    }

    #[test]
    fn poll_is_noop_when_not_running() {
        let (mut timer, now) = new_timer();
        assert_eq!(timer.poll(now + secs(9999)), None); // idle

        timer.start(now);
        timer.pause(now + secs(10));
        assert_eq!(timer.poll(now + secs(9999)), None); // paused
    }

    #[test]
    fn work_completion_moves_to_short_break_idle() {
        let (mut timer, now) = new_timer();
        let (event, at_end) = run_to_completion(&mut timer, now);

        assert_eq!(
            event,
            PhaseCompleted {
                finished: Phase::Work,
                next: Phase::ShortBreak,
                auto_started: false,
            }
        );
        assert_eq!(timer.completed_pomodoros(), 1);
        assert_eq!(timer.status(), Status::Idle);
        assert_eq!(timer.remaining(at_end), secs(300));
    }

    #[test]
    fn completion_fires_only_once() {
        let (mut timer, now) = new_timer();
        let (_, at_end) = run_to_completion(&mut timer, now);
        assert_eq!(timer.poll(at_end + secs(1)), None);
    }

    #[test]
    fn fourth_work_completion_leads_to_long_break() {
        let (mut timer, mut now) = new_timer();
        for expected in [
            (1, Phase::ShortBreak),
            (2, Phase::ShortBreak),
            (3, Phase::ShortBreak),
            (4, Phase::LongBreak),
        ] {
            let (event, at_end) = run_to_completion(&mut timer, now);
            assert_eq!((timer.completed_pomodoros(), event.next), expected);

            // 休憩も完了させて次の作業フェーズへ
            let (event, at_end) = run_to_completion(&mut timer, at_end);
            assert_eq!(event.next, Phase::Work);
            now = at_end;
        }
    }

    #[test]
    fn auto_start_next_begins_next_phase_running() {
        let mut config = test_config();
        config.auto_start_next = true;
        let mut timer = Timer::new(config);
        let now = Instant::now();

        let (event, at_end) = run_to_completion(&mut timer, now);
        assert!(event.auto_started);
        assert_eq!(timer.status(), Status::Running);
        assert_eq!(timer.remaining(at_end), secs(300));
    }

    #[test]
    fn sleep_overrun_completes_once_without_carryover() {
        // スリープ復帰想定: 満了から 2 時間超過した now で poll しても
        // 完了は 1 回だけ、次フェーズはフル尺で Idle (超過分を繰り越さない)
        let (mut timer, now) = new_timer();
        timer.start(now);
        let long_after = now + secs(1500 + 7200);

        let event = timer.poll(long_after).expect("should complete");
        assert_eq!(event.finished, Phase::Work);
        assert_eq!(timer.status(), Status::Idle);
        assert_eq!(timer.remaining(long_after), secs(300));
        assert_eq!(timer.poll(long_after + secs(1)), None);
    }

    #[test]
    fn sleep_overrun_with_auto_start_starts_next_at_poll_time() {
        let mut config = test_config();
        config.auto_start_next = true;
        let mut timer = Timer::new(config);
        let now = Instant::now();
        timer.start(now);

        let long_after = now + secs(1500 + 7200);
        timer.poll(long_after).expect("should complete");
        // 次フェーズの起点は満了時刻ではなく poll した now
        assert_eq!(timer.remaining(long_after + secs(10)), secs(290));
    }

    #[test]
    fn skip_work_discards_without_counting() {
        let (mut timer, now) = new_timer();
        timer.start(now);
        timer.skip();

        assert_eq!(timer.completed_pomodoros(), 0);
        assert_eq!(timer.phase(), Phase::ShortBreak);
        assert_eq!(timer.status(), Status::Idle);
        assert_eq!(timer.remaining(now), secs(300));
    }

    #[test]
    fn skip_break_returns_to_work() {
        let (mut timer, now) = new_timer();
        let (_, at_end) = run_to_completion(&mut timer, now);
        assert_eq!(timer.phase(), Phase::ShortBreak);

        timer.skip();
        assert_eq!(timer.phase(), Phase::Work);
        assert_eq!(timer.status(), Status::Idle);
        assert_eq!(timer.remaining(at_end), secs(1500));
    }

    #[test]
    fn update_config_refreshes_remaining_while_idle() {
        let (mut timer, now) = new_timer();

        let mut config = test_config();
        config.work = secs(3000);
        timer.update_config(config);

        assert_eq!(timer.remaining(now), secs(3000));
    }

    #[test]
    fn update_config_keeps_running_phase_untouched() {
        let (mut timer, now) = new_timer();
        timer.start(now);

        let mut config = test_config();
        config.work = secs(3000);
        config.short_break = secs(600);
        timer.update_config(config);

        // 進行中フェーズは旧設定のまま満了する
        assert_eq!(timer.remaining(now + secs(100)), secs(1400));
        let event = timer.poll(now + secs(1500)).expect("should complete");
        assert_eq!(event.next, Phase::ShortBreak);

        // 次フェーズからは新設定の長さになる
        assert_eq!(timer.remaining(now + secs(1500)), secs(600));
    }

    #[test]
    fn reset_returns_to_work_keeping_count() {
        let (mut timer, now) = new_timer();
        let (_, at_end) = run_to_completion(&mut timer, now);
        timer.start(at_end);
        timer.reset();

        assert_eq!(timer.completed_pomodoros(), 1);
        assert_eq!(timer.phase(), Phase::Work);
        assert_eq!(timer.status(), Status::Idle);
        assert_eq!(timer.remaining(at_end), secs(1500));
    }
}

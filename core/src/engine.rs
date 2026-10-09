//! The state machine that decides when to move or click.
//!
//! It never sleeps and never reads the clock itself: the caller passes a
//! monotonic `now` in milliseconds to every call. That is what lets the tests
//! run a simulated eight-hour day in a few milliseconds.

use crate::config::{AreaOrder, Mode, Settings, StopKind, WiggleStyle};
use crate::path::{glide, STEP_MS};
use crate::platform::{Platform, Point};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    #[default]
    Idle,
    Running,
    /// Paused by the user (button or tray).
    Paused,
    /// Auto-paused because the user is using the mouse or keyboard.
    UserActive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Origin {
    #[default]
    Manual,
    Schedule,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StopReason {
    Manual,
    ActionLimit,
    TimeLimit,
    ClockTime,
    ScheduleEnd,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Started,
    Stopped {
        reason: StopReason,
        message: Option<String>,
    },
    PhaseChanged(Phase),
    Action {
        count: u64,
    },
    /// The clock jumped: the computer slept or the process was frozen.
    SleepGap {
        ms: u64,
    },
}

/// A tick arriving this late means the machine was asleep.
pub const SLEEP_GAP_MS: u64 = 10_000;
/// Cursor drift (in pixels) we tolerate before deciding the user moved it.
const REST_TOLERANCE: f64 = 2.0;
/// Same, while we are moving the cursor ourselves (events land asynchronously).
const MOTION_TOLERANCE: f64 = 40.0;
/// How close to the target the cursor must be for a click to go ahead.
const ARRIVAL_TOLERANCE: f64 = 3.0;
/// Slack when comparing the OS idle timer with our own bookkeeping.
const IDLE_SLACK_MS: u64 = 300;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Then {
    Click(Point),
    CountWiggle,
    Nothing,
}

#[derive(Clone, Debug)]
struct Motion {
    points: Vec<Point>,
    idx: usize,
    next_ms: u64,
    then: Then,
}

#[derive(Clone, Debug)]
pub struct Engine {
    cfg: Settings,
    rng: Rng,
    phase: Phase,
    origin: Origin,
    last_tick_ms: u64,
    active_ms: u64,
    user_pause_ms: u64,
    sleep_ms: u64,
    actions: u64,
    next_action_ms: u64,
    deadline_ms: Option<u64>,
    deadline_reason: StopReason,
    paused_at_ms: u64,
    expected: Option<Point>,
    settle: bool,
    motion: Option<Motion>,
    last_synth_ms: u64,
    last_user_ms: u64,
    last_seen: Option<Point>,
    prev_cur: Option<Point>,
    pending_click: Option<Point>,
    last_step: f64,
    seq_idx: usize,
    home: Option<Point>,
}

impl Engine {
    pub fn new(seed: u64) -> Self {
        Engine {
            cfg: Settings::default(),
            rng: Rng::new(seed),
            phase: Phase::Idle,
            origin: Origin::Manual,
            last_tick_ms: 0,
            active_ms: 0,
            user_pause_ms: 0,
            sleep_ms: 0,
            actions: 0,
            next_action_ms: 0,
            deadline_ms: None,
            deadline_reason: StopReason::TimeLimit,
            paused_at_ms: 0,
            expected: None,
            settle: true,
            motion: None,
            last_synth_ms: 0,
            last_user_ms: 0,
            last_seen: None,
            prev_cur: None,
            pending_click: None,
            last_step: 0.0,
            seq_idx: 0,
            home: None,
        }
    }

    // ---- read-only view

    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn origin(&self) -> Origin {
        self.origin
    }
    pub fn mode(&self) -> Mode {
        self.cfg.mode
    }
    pub fn actions(&self) -> u64 {
        self.actions
    }
    pub fn active_ms(&self) -> u64 {
        self.active_ms
    }
    pub fn user_pause_ms(&self) -> u64 {
        self.user_pause_ms
    }
    pub fn sleep_ms(&self) -> u64 {
        self.sleep_ms
    }
    pub fn is_active(&self) -> bool {
        self.phase != Phase::Idle
    }
    /// True while a glide is in progress; the caller should tick faster.
    pub fn in_motion(&self) -> bool {
        self.motion.is_some() || self.pending_click.is_some()
    }
    pub fn next_action_in_ms(&self, now: u64) -> Option<u64> {
        match self.phase {
            Phase::Running => Some(self.next_action_ms.saturating_sub(now)),
            _ => None,
        }
    }
    /// Time left before the stop rule fires, if it is time-based.
    pub fn remaining_ms(&self, now: u64) -> Option<u64> {
        let d = self.deadline_ms?;
        let d = if self.phase == Phase::Paused && self.deadline_reason == StopReason::TimeLimit {
            d + now.saturating_sub(self.paused_at_ms)
        } else {
            d
        };
        Some(d.saturating_sub(now))
    }
    pub fn actions_left(&self) -> Option<u64> {
        match self.cfg.stop.kind {
            StopKind::AfterActions if self.origin == Origin::Manual => {
                Some((self.cfg.stop.actions as u64).saturating_sub(self.actions))
            }
            _ => None,
        }
    }

    // ---- commands

    /// Start a session. `sec_of_day` is the local time, needed for "stop at 5 pm".
    pub fn start(
        &mut self,
        mut cfg: Settings,
        origin: Origin,
        now: u64,
        sec_of_day: u32,
    ) -> Result<(), String> {
        cfg.sanitize();
        if cfg.mode == Mode::Click && cfg.areas.is_empty() {
            return Err("no-target".into());
        }
        self.deadline_ms = None;
        self.deadline_reason = StopReason::TimeLimit;
        // Stop rules belong to sessions the user starts by hand. A scheduled
        // session ends when its timetable segment ends.
        if origin == Origin::Manual {
            match cfg.stop.kind {
                StopKind::AfterMinutes => {
                    self.deadline_ms = Some(now + cfg.stop.minutes as u64 * 60_000);
                }
                StopKind::AtTime => {
                    let target = cfg.stop.at_minute as i64 * 60;
                    let mut delta = target - sec_of_day as i64;
                    if delta <= 0 {
                        delta += 86_400;
                    }
                    self.deadline_ms = Some(now + delta as u64 * 1000);
                    self.deadline_reason = StopReason::ClockTime;
                }
                _ => {}
            }
        }
        self.cfg = cfg;
        self.origin = origin;
        self.phase = Phase::Running;
        self.last_tick_ms = now;
        self.active_ms = 0;
        self.user_pause_ms = 0;
        self.sleep_ms = 0;
        self.actions = 0;
        self.motion = None;
        self.pending_click = None;
        self.expected = None;
        self.settle = true;
        self.last_synth_ms = now;
        self.last_user_ms = now;
        self.last_seen = None;
        self.prev_cur = None;
        self.pending_click = None;
        self.seq_idx = 0;
        self.home = None;
        // First action comes quickly so the user sees it working.
        self.next_action_ms = now + self.rng.range_f64(2000.0, 4000.0) as u64;
        Ok(())
    }

    /// Live-apply edited settings to a running session (intervals, toggles).
    pub fn update_settings(&mut self, mut cfg: Settings) {
        cfg.sanitize();
        if self.is_active() {
            // mode and stop rule are fixed for the life of a session
            cfg.mode = self.cfg.mode;
            cfg.stop = self.cfg.stop;
            if cfg.mode == Mode::Click && cfg.areas.is_empty() {
                cfg.areas = self.cfg.areas.clone();
            }
            if cfg.areas != self.cfg.areas {
                // never finish a glide towards a target that was just edited away
                self.motion = None;
                self.pending_click = None;
                self.home = None;
                self.settle = true;
            }
        }
        self.cfg = cfg;
    }

    pub fn stop(&mut self, reason: StopReason, message: Option<String>) -> Vec<Event> {
        if self.phase == Phase::Idle {
            return Vec::new();
        }
        self.phase = Phase::Idle;
        self.motion = None;
        self.pending_click = None;
        vec![Event::Stopped { reason, message }]
    }

    pub fn pause(&mut self, now: u64) -> Vec<Event> {
        match self.phase {
            Phase::Running | Phase::UserActive => {
                self.phase = Phase::Paused;
                self.paused_at_ms = now;
                self.motion = None;
                self.pending_click = None;
                vec![Event::PhaseChanged(Phase::Paused)]
            }
            _ => Vec::new(),
        }
    }

    pub fn resume(&mut self, now: u64) -> Vec<Event> {
        match self.phase {
            Phase::Paused => {
                let paused = now.saturating_sub(self.paused_at_ms);
                // "Stop after 60 minutes" means 60 minutes of running, so the
                // deadline moves along. "Stop at 17:00" stays at 17:00.
                if self.deadline_reason == StopReason::TimeLimit {
                    if let Some(d) = &mut self.deadline_ms {
                        *d += paused;
                    }
                }
                self.next_action_ms += paused;
                self.back_to_running(now);
                vec![Event::PhaseChanged(Phase::Running)]
            }
            // "Resume" while auto-paused: the user insists, carry on now.
            Phase::UserActive => {
                self.back_to_running(now);
                vec![Event::PhaseChanged(Phase::Running)]
            }
            _ => Vec::new(),
        }
    }

    fn back_to_running(&mut self, now: u64) {
        self.phase = Phase::Running;
        self.last_tick_ms = now;
        self.settle = true;
        self.last_synth_ms = now;
        if self.next_action_ms < now + 1000 {
            self.next_action_ms = now + self.rng.range_f64(1500.0, 3500.0) as u64;
        }
    }

    fn interval_ms(&mut self) -> u64 {
        let (lo, hi) = self.cfg.interval_range_s();
        (self.rng.range_f64(lo, hi) * 1000.0) as u64
    }

    // ---- the heartbeat

    pub fn tick(&mut self, now: u64, p: &mut dyn Platform) -> Vec<Event> {
        let mut ev = Vec::new();
        let gap = now.saturating_sub(self.last_tick_ms);
        self.last_tick_ms = now;
        if matches!(self.phase, Phase::Idle | Phase::Paused) {
            return ev;
        }
        match self.step(now, gap, p, &mut ev) {
            Ok(()) => ev,
            // The pointer is out of reach for now: a lock screen, a Windows
            // permission prompt, or an administrator window in front. That is
            // not a reason to end the session; wait as if the user were busy.
            Err(msg) if msg == "cursor-unavailable" || msg == "input-blocked" => {
                self.motion = None;
                self.pending_click = None;
                self.home = None;
                self.last_user_ms = now;
                self.last_seen = None;
                if self.phase == Phase::Running {
                    self.phase = Phase::UserActive;
                    ev.push(Event::PhaseChanged(Phase::UserActive));
                }
                ev
            }
            Err(msg) => {
                ev.extend(self.stop(StopReason::Error, Some(msg)));
                ev
            }
        }
    }

    fn step(
        &mut self,
        now: u64,
        gap: u64,
        p: &mut dyn Platform,
        ev: &mut Vec<Event>,
    ) -> Result<(), String> {
        if gap > SLEEP_GAP_MS {
            // We were asleep. Do not fire a burst of overdue actions and do
            // not mistake the wake-up mouse nudge for anything else.
            self.sleep_ms += gap;
            self.motion = None;
            self.pending_click = None;
            self.settle = true;
            self.last_synth_ms = now;
            self.last_user_ms = now;
            self.next_action_ms = now + self.rng.range_f64(2000.0, 5000.0) as u64;
            ev.push(Event::SleepGap { ms: gap });
        } else {
            match self.phase {
                Phase::Running => self.active_ms += gap,
                Phase::UserActive => self.user_pause_ms += gap,
                _ => {}
            }
        }

        if let Some(d) = self.deadline_ms {
            if now >= d {
                let reason = self.deadline_reason.clone();
                ev.extend(self.stop(reason, None));
                return Ok(());
            }
        }

        let cur = p.cursor()?;
        let prev = self.prev_cur.replace(cur);

        if self.phase == Phase::UserActive {
            if self.last_seen != Some(cur) {
                self.last_seen = Some(cur);
                self.last_user_ms = now;
            }
            if let Some(idle) = p.idle_ms() {
                let t = now.saturating_sub(idle);
                if t > self.last_user_ms {
                    self.last_user_ms = t;
                }
            }
            if now.saturating_sub(self.last_user_ms) >= self.cfg.resume_after_s as u64 * 1000 {
                self.back_to_running(now);
                self.settle = false;
                self.expected = Some(cur);
                ev.push(Event::PhaseChanged(Phase::Running));
            }
            return Ok(());
        }

        // ---- Running: did the user take over?
        if self.settle {
            self.expected = Some(cur);
            self.settle = false;
        } else if self.cfg.pause_on_user {
            let busy = self.motion.is_some() || self.pending_click.is_some();
            // while we are moving, allow for the OS applying our last steps late
            let tol = if busy {
                MOTION_TOLERANCE + 2.0 * self.last_step
            } else {
                REST_TOLERANCE
            };
            let moved = match self.expected {
                Some(e) if cur.dist(e) > tol => !(busy && against_wall(cur, e, prev)),
                _ => false,
            };
            let typed = match p.idle_ms() {
                Some(idle) => idle + IDLE_SLACK_MS < now.saturating_sub(self.last_synth_ms),
                None => false,
            };
            if moved || typed {
                self.phase = Phase::UserActive;
                self.motion = None;
                self.pending_click = None;
                self.last_user_ms = now;
                self.last_seen = Some(cur);
                ev.push(Event::PhaseChanged(Phase::UserActive));
                return Ok(());
            }
        } else {
            // Not watching the user: just follow the cursor.
            self.expected = Some(cur);
        }

        // ---- the cursor arrived last tick: click, if it really is on target
        if let Some(pt) = self.pending_click.take() {
            self.settle = true;
            if cur.dist(pt) <= ARRIVAL_TOLERANCE {
                p.click(pt, self.cfg.button)?;
                self.last_synth_ms = now;
                self.count_action(ev);
            }
            // (otherwise the target is unreachable, e.g. on a display that
            // was unplugged: never click somewhere the user did not choose)
            if let (true, Some(home)) = (self.cfg.return_cursor, self.home.take()) {
                if self.cfg.smooth_move {
                    let points = glide(cur, home, &mut self.rng);
                    // a short human pause before heading back
                    let wait = self.rng.range_f64(120.0, 320.0) as u64;
                    self.expected = Some(cur);
                    self.settle = false;
                    self.motion = Some(Motion {
                        points,
                        idx: 0,
                        next_ms: now + wait,
                        then: Then::Nothing,
                    });
                } else {
                    p.move_to(home)?;
                }
            }
            return Ok(());
        }

        // ---- carry on with a glide in progress
        if self.motion.is_some() {
            let mut finished = None;
            if let Some(m) = &mut self.motion {
                while m.idx < m.points.len() && now >= m.next_ms {
                    let pt = m.points[m.idx];
                    p.move_to(pt)?;
                    self.last_step = self.expected.map(|e| e.dist(pt)).unwrap_or(0.0);
                    self.expected = Some(pt);
                    self.last_synth_ms = now;
                    m.idx += 1;
                    m.next_ms += STEP_MS;
                }
                if m.idx >= m.points.len() {
                    finished = Some(m.then);
                }
            }
            if let Some(then) = finished {
                self.motion = None;
                self.pending_click = None;
                self.after_motion(then, now, p, ev)?;
            }
            return Ok(());
        }

        // ---- stop after N actions (checked once the cursor is back home)
        if self.origin == Origin::Manual
            && self.cfg.stop.kind == StopKind::AfterActions
            && self.actions >= self.cfg.stop.actions as u64
        {
            ev.extend(self.stop(StopReason::ActionLimit, None));
            return Ok(());
        }

        // ---- time for the next action?
        if now >= self.next_action_ms {
            self.next_action_ms = now + self.interval_ms();
            match self.cfg.mode {
                Mode::Wiggle => self.begin_wiggle(cur, now, p, ev)?,
                Mode::Click => self.begin_click(cur, now, p, ev)?,
            }
        }
        Ok(())
    }

    fn begin_wiggle(
        &mut self,
        cur: Point,
        now: u64,
        p: &mut dyn Platform,
        ev: &mut Vec<Event>,
    ) -> Result<(), String> {
        match self.cfg.wiggle_style {
            WiggleStyle::Subtle => {
                let mut side = Point::new(cur.x + 1, cur.y);
                if let Some(b) = p.bounds() {
                    if !b.contains(side) {
                        side = Point::new(cur.x - 1, cur.y);
                    }
                }
                p.move_to(side)?;
                p.move_to(cur)?;
                self.last_synth_ms = now;
                self.settle = true;
                self.count_action(ev);
            }
            WiggleStyle::Natural => {
                let reach = self.cfg.wiggle_distance as f64;
                let len = self.rng.range_f64(reach * 0.4, reach);
                let angle = self.rng.range_f64(0.0, std::f64::consts::TAU);
                let mut target = Point::new(
                    cur.x + (len * angle.cos()).round() as i32,
                    cur.y + (len * angle.sin()).round() as i32,
                );
                if let Some(b) = p.bounds() {
                    if !b.contains(target) {
                        // bounce off the edge instead of pushing against it
                        target = Point::new(2 * cur.x - target.x, 2 * cur.y - target.y);
                        target = b.clamp(target);
                    }
                }
                let mut points = glide(cur, target, &mut self.rng);
                points.extend(glide(target, cur, &mut self.rng));
                self.motion = Some(Motion {
                    points,
                    idx: 0,
                    next_ms: now,
                    then: Then::CountWiggle,
                });
            }
        }
        Ok(())
    }

    fn begin_click(
        &mut self,
        cur: Point,
        now: u64,
        p: &mut dyn Platform,
        ev: &mut Vec<Event>,
    ) -> Result<(), String> {
        if self.cfg.areas.is_empty() {
            return Err("no-target".into());
        }
        let n = self.cfg.areas.len();
        let i = match self.cfg.area_order {
            AreaOrder::Random => self.rng.range_i64(0, n as i64 - 1) as usize,
            AreaOrder::Sequence => {
                let i = self.seq_idx % n;
                self.seq_idx = (self.seq_idx + 1) % n;
                i
            }
        };
        let a = self.cfg.areas[i];
        let target = Point::new(
            a.x + self.rng.range_i64(0, (a.w - 1).max(0) as i64) as i32,
            a.y + self.rng.range_i64(0, (a.h - 1).max(0) as i64) as i32,
        );
        self.home = Some(cur);
        if self.cfg.smooth_move {
            let points = glide(cur, target, &mut self.rng);
            self.motion = Some(Motion {
                points,
                idx: 0,
                next_ms: now,
                then: Then::Click(target),
            });
            Ok(())
        } else {
            p.move_to(target)?;
            self.last_synth_ms = now;
            self.after_motion(Then::Click(target), now, p, ev)
        }
    }

    fn after_motion(
        &mut self,
        then: Then,
        now: u64,
        _p: &mut dyn Platform,
        ev: &mut Vec<Event>,
    ) -> Result<(), String> {
        self.settle = true;
        self.last_synth_ms = now;
        match then {
            Then::Click(pt) => {
                // Click on the next tick, once the OS has caught up and we
                // can check where the cursor actually is.
                self.pending_click = Some(pt);
                self.expected = Some(pt);
                self.settle = false;
            }
            Then::CountWiggle => self.count_action(ev),
            Then::Nothing => {}
        }
        Ok(())
    }

    fn count_action(&mut self, ev: &mut Vec<Event>) {
        self.actions += 1;
        ev.push(Event::Action {
            count: self.actions,
        });
    }
}

/// True if the cursor is off course only because a screen edge is in the way:
/// on each axis it either follows our commands or is pinned in place.
fn against_wall(cur: Point, expected: Point, prev: Option<Point>) -> bool {
    let Some(prev) = prev else { return false };
    let ok = |c: i32, e: i32, p: i32| (c - e).abs() <= 12 || c == p;
    ok(cur.x, expected.x, prev.x) && ok(cur.y, expected.y, prev.y)
}

//! Glue between the user's settings, the timetable and the engine.
//! The desktop app talks to this type only.

use crate::config::{Mode, Settings};
use crate::engine::{Engine, Event, Origin, Phase, StopReason};
use crate::history::SessionRecord;
use crate::platform::Platform;
use crate::schedule::{active_segment, next_segment, Segment};
use serde::Serialize;

/// "What time is it", supplied by the app on every call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clock {
    /// Monotonic milliseconds (never goes backwards, unaffected by DST).
    pub mono_ms: u64,
    /// Local calendar day, as days since 1970-01-01.
    pub day: i64,
    /// 0 = Monday.
    pub weekday: u8,
    /// Seconds since local midnight.
    pub sec_of_day: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ScheduleState {
    Off,
    /// Enabled, next segment has not started.
    Waiting,
    Running,
    /// "Skip today" was pressed.
    Skipped,
    /// The user stopped by hand during a segment; resumes at the next one.
    Held,
    /// Nothing more planned today.
    Done,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleStatus {
    pub state: ScheduleState,
    pub next_start_s: Option<u32>,
    pub next_end_s: Option<u32>,
    pub today: Vec<Segment>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastStop {
    pub reason: StopReason,
    pub message: Option<String>,
    pub actions: u64,
    pub active_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub phase: Phase,
    pub mode: Mode,
    pub origin: Origin,
    pub actions: u64,
    pub active_ms: u64,
    pub next_in_ms: Option<u64>,
    pub remaining_ms: Option<u64>,
    /// Local second of day at which the session will stop, if known.
    pub stops_at_s: Option<u32>,
    pub actions_left: Option<u64>,
    pub schedule: ScheduleStatus,
    pub last_stop: Option<LastStop>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outcome {
    /// Something the UI should redraw.
    pub changed: bool,
    /// Sessions that just ended, to append to the history.
    pub finished: Vec<SessionRecord>,
}

pub struct Supervisor {
    settings: Settings,
    engine: Engine,
    seed: u64,
    plan: Option<(i64, Vec<Segment>)>,
    /// (day, second of day) until which the timetable must not auto-start.
    hold: Option<(i64, u32)>,
    session_start: Option<(i64, u32)>,
    last_stop: Option<LastStop>,
}

impl Supervisor {
    pub fn new(mut settings: Settings, seed: u64) -> Self {
        settings.sanitize();
        Supervisor {
            settings,
            engine: Engine::new(seed ^ 0xA5A5_5A5A_1234_5678),
            seed,
            plan: None,
            hold: None,
            session_start: None,
            last_stop: None,
        }
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn set_settings(&mut self, mut s: Settings) {
        s.sanitize();
        if s.schedule != self.settings.schedule {
            self.plan = None;
            self.hold = None;
        }
        self.engine.update_settings(s.clone());
        self.settings = s;
    }

    /// The concrete plan for any day (used for the week view).
    pub fn plan_for(&self, day: i64, weekday: u8) -> Vec<Segment> {
        self.settings.schedule.resolve_day(day, weekday, self.seed)
    }

    fn today(&mut self, c: Clock) -> Vec<Segment> {
        match &self.plan {
            Some((d, p)) if *d == c.day => p.clone(),
            _ => {
                let p = self.plan_for(c.day, c.weekday);
                self.plan = Some((c.day, p.clone()));
                p
            }
        }
    }

    fn schedule_live(&self, c: Clock) -> bool {
        self.settings.schedule.enabled && self.settings.schedule.skip_day != Some(c.day)
    }

    fn held(&self, c: Clock) -> bool {
        matches!(self.hold, Some((d, until)) if d == c.day && c.sec_of_day < until)
    }

    // ---- commands from the UI, tray and hotkey

    pub fn start(&mut self, c: Clock) -> Result<Outcome, String> {
        if self.engine.is_active() {
            return Ok(Outcome::default());
        }
        self.begin(c, Origin::Manual, None)?;
        Ok(Outcome {
            changed: true,
            finished: Vec::new(),
        })
    }

    pub fn stop(&mut self, c: Clock) -> Outcome {
        let ev = self.engine.stop(StopReason::Manual, None);
        self.absorb(ev, c)
    }

    pub fn toggle(&mut self, c: Clock) -> Result<Outcome, String> {
        if self.engine.is_active() {
            Ok(self.stop(c))
        } else {
            self.start(c)
        }
    }

    pub fn pause(&mut self, c: Clock) -> Outcome {
        let ev = self.engine.pause(c.mono_ms);
        self.absorb(ev, c)
    }

    pub fn resume(&mut self, c: Clock) -> Outcome {
        let ev = self.engine.resume(c.mono_ms);
        self.absorb(ev, c)
    }

    fn begin(&mut self, c: Clock, origin: Origin, mode: Option<Mode>) -> Result<(), String> {
        let mut cfg = self.settings.clone();
        if let Some(m) = mode {
            cfg.mode = m;
        }
        self.engine.start(cfg, origin, c.mono_ms, c.sec_of_day)?;
        self.session_start = Some((c.day, c.sec_of_day));
        self.last_stop = None;
        Ok(())
    }

    // ---- heartbeat

    pub fn tick(&mut self, c: Clock, p: &mut dyn Platform) -> Outcome {
        let mut out = Outcome::default();

        // 1. timetable decisions
        let live = self.schedule_live(c);
        let plan = if live { self.today(c) } else { Vec::new() };
        let seg = active_segment(&plan, c.sec_of_day).copied();
        if let Some((d, until)) = self.hold {
            if d != c.day || c.sec_of_day >= until {
                self.hold = None;
                out.changed = true;
            }
        }
        match (seg, self.engine.is_active()) {
            (Some(s), false) if !self.held(c) => match self.begin(c, Origin::Schedule, s.mode) {
                Ok(()) => out.changed = true,
                Err(msg) => {
                    // Cannot start (for example Click mode with no target).
                    // Say so once and leave this segment alone.
                    self.hold = Some((c.day, s.end_s));
                    self.last_stop = Some(LastStop {
                        reason: StopReason::Error,
                        message: Some(msg),
                        actions: 0,
                        active_ms: 0,
                    });
                    out.changed = true;
                }
            },
            (None, true) if self.engine.origin() == Origin::Schedule => {
                let ev = self.engine.stop(StopReason::ScheduleEnd, None);
                out = merge(out, self.absorb(ev, c));
            }
            _ => {}
        }

        // 2. the engine itself
        let ev = self.engine.tick(c.mono_ms, p);
        merge(out, self.absorb(ev, c))
    }

    fn absorb(&mut self, events: Vec<Event>, c: Clock) -> Outcome {
        let mut out = Outcome::default();
        for e in events {
            out.changed = true;
            if let Event::Stopped { reason, message } = e {
                let (day, start_s) = self.session_start.take().unwrap_or((c.day, c.sec_of_day));
                out.finished.push(SessionRecord {
                    day,
                    start_s,
                    end_s: c.sec_of_day,
                    mode: self.engine.mode(),
                    actions: self.engine.actions(),
                    by_schedule: self.engine.origin() == Origin::Schedule,
                    reason: reason.clone(),
                    user_pause_s: (self.engine.user_pause_ms() / 1000) as u32,
                    sleep_s: (self.engine.sleep_ms() / 1000) as u32,
                });
                // Any stop other than "the timetable said so" means: leave me
                // alone until the current segment is over.
                if reason != StopReason::ScheduleEnd && self.schedule_live(c) {
                    let plan = self.today(c);
                    if let Some(s) = active_segment(&plan, c.sec_of_day) {
                        self.hold = Some((c.day, s.end_s));
                    }
                }
                self.last_stop = Some(LastStop {
                    reason,
                    message,
                    actions: self.engine.actions(),
                    active_ms: self.engine.active_ms(),
                });
            }
        }
        out
    }

    // ---- view for the UI

    pub fn status(&mut self, c: Clock) -> Status {
        let e = &self.engine;
        let phase = e.phase();
        let origin = e.origin();
        let mut remaining_ms = e.remaining_ms(c.mono_ms);
        let (actions, active_ms) = (e.actions(), e.active_ms());
        let next_in_ms = e.next_action_in_ms(c.mono_ms);
        let actions_left = if phase == Phase::Idle {
            None
        } else {
            e.actions_left()
        };
        let mode = if phase == Phase::Idle {
            self.settings.mode
        } else {
            e.mode()
        };

        let live = self.schedule_live(c);
        let today = if self.settings.schedule.enabled {
            self.today(c)
        } else {
            Vec::new()
        };
        let seg = if live {
            active_segment(&today, c.sec_of_day).copied()
        } else {
            None
        };
        let next = if live {
            next_segment(&today, c.sec_of_day).copied()
        } else {
            None
        };

        let mut stops_at_s = None;
        if phase == Phase::Idle {
            remaining_ms = None;
        } else if origin == Origin::Schedule {
            if let Some(s) = seg {
                stops_at_s = Some(s.end_s % 86_400);
                remaining_ms = Some((s.end_s.saturating_sub(c.sec_of_day)) as u64 * 1000);
            }
        } else if let Some(r) = remaining_ms {
            stops_at_s = Some(((c.sec_of_day as u64 + r / 1000) % 86_400) as u32);
        }

        let state = if !self.settings.schedule.enabled {
            ScheduleState::Off
        } else if !live {
            ScheduleState::Skipped
        } else if phase != Phase::Idle && origin == Origin::Schedule {
            ScheduleState::Running
        } else if seg.is_some() && self.held(c) {
            ScheduleState::Held
        } else if next.is_some() || seg.is_some() {
            ScheduleState::Waiting
        } else {
            ScheduleState::Done
        };

        Status {
            phase,
            mode,
            origin,
            actions,
            active_ms,
            next_in_ms,
            remaining_ms,
            stops_at_s,
            actions_left,
            schedule: ScheduleStatus {
                state,
                next_start_s: next.map(|s| s.start_s),
                next_end_s: seg.or(next).map(|s| s.end_s),
                today,
            },
            last_stop: self.last_stop.clone(),
        }
    }
}

fn merge(mut a: Outcome, b: Outcome) -> Outcome {
    a.changed |= b.changed;
    a.finished.extend(b.finished);
    a
}

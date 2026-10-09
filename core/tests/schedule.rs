mod common;
use common::*;
use wiggler_core::engine::{Origin, Phase, StopReason};
use wiggler_core::schedule::*;
use wiggler_core::supervisor::ScheduleState;
use wiggler_core::*;

fn nine_to_five() -> Schedule {
    Schedule {
        enabled: true,
        blocks: vec![Block::default()],
        skip_day: None,
    }
}

#[test]
fn preset_resolves_inside_its_windows_with_a_lunch_gap() {
    let s = nine_to_five();
    let mut starts = std::collections::HashSet::new();
    for d in 0..200i64 {
        let day = MONDAY + d;
        let wd = (d % 7) as u8;
        let plan = s.resolve_day(day, wd, 1234);
        if wd >= 5 {
            assert!(plan.is_empty(), "weekends are free");
            continue;
        }
        assert_eq!(plan.len(), 2, "morning and afternoon");
        let (am, pm) = (plan[0], plan[1]);
        assert!(
            (8 * 3600 + 50 * 60..=9 * 3600).contains(&am.start_s),
            "start {}",
            am.start_s
        );
        assert!((12 * 3600 + 55 * 60..=13 * 3600 + 5 * 60).contains(&am.end_s));
        assert!((13 * 3600 + 55 * 60..=14 * 3600 + 5 * 60).contains(&pm.start_s));
        assert!(
            (17 * 3600..=17 * 3600 + 10 * 60).contains(&pm.end_s),
            "end {}",
            pm.end_s
        );
        assert!(am.earliest_s <= am.start_s && pm.latest_s >= pm.end_s);
        assert_eq!(am.earliest_s, 8 * 3600 + 50 * 60);
        assert_eq!(pm.latest_s, 17 * 3600 + 10 * 60);
        starts.insert(am.start_s);
    }
    assert!(starts.len() > 100, "start time differs from day to day");
}

#[test]
fn a_days_plan_is_stable_but_differs_between_installs() {
    let s = nine_to_five();
    assert_eq!(s.resolve_day(MONDAY, 0, 7), s.resolve_day(MONDAY, 0, 7));
    assert_ne!(s.resolve_day(MONDAY, 0, 7), s.resolve_day(MONDAY, 0, 8));
    assert_ne!(s.resolve_day(MONDAY, 0, 7), s.resolve_day(MONDAY + 1, 1, 7));
}

#[test]
fn exact_times_and_overlapping_blocks() {
    let exact = Block {
        start_jitter: Jitter::default(),
        end_jitter: Jitter::default(),
        pause: None,
        ..Block::default()
    };
    let late = Block {
        start_min: 16 * 60,
        end_min: 20 * 60,
        ..exact.clone()
    };
    let s = Schedule {
        enabled: true,
        blocks: vec![exact, late],
        skip_day: None,
    };
    let plan = s.resolve_day(MONDAY, 0, 1);
    assert_eq!(plan.len(), 1, "overlap is merged");
    assert_eq!((plan[0].start_s, plan[0].end_s), (9 * 3600, 20 * 3600));
}

#[test]
fn nonsense_blocks_are_repaired() {
    let mut s = Schedule {
        enabled: true,
        blocks: vec![Block {
            start_min: 900,
            end_min: 600,
            start_jitter: Jitter {
                kind: JitterKind::Either,
                minutes: 9999,
            },
            pause: Some(Break {
                start_min: 100,
                end_min: 50,
                ..Break::default()
            }),
            ..Block::default()
        }],
        skip_day: None,
    };
    s.sanitize();
    let b = &s.blocks[0];
    assert!(b.end_min > b.start_min);
    assert!(b.start_jitter.minutes <= 60);
    assert!(b.pause.is_none());
    for d in 0..7 {
        for seg in s.resolve_day(MONDAY + d, d as u8, 5) {
            assert!(seg.start_s < seg.end_s && seg.end_s <= 86_400);
        }
    }
}

/// Simulate a whole day in one-second steps (20 ms steps while gliding).
struct Day {
    sup: Supervisor,
    f: Fake,
    day: i64,
    sod_ms: u64,
    log: Vec<wiggler_core::history::SessionRecord>,
}

impl Day {
    fn new(settings: Settings, day: i64) -> Self {
        Day {
            sup: Supervisor::new(settings, 42),
            f: Fake::new(),
            day,
            sod_ms: 0,
            log: vec![],
        }
    }
    fn clock(&self) -> Clock {
        clock(self.f.now, self.day, (self.sod_ms / 1000) as u32)
    }
    fn until(&mut self, sec_of_day: u32) {
        while self.sod_ms < sec_of_day as u64 * 1000 {
            let step = if self.sup.engine().in_motion() {
                TICK
            } else {
                1000 - self.sod_ms % 1000
            };
            self.sod_ms += step;
            self.f.now += step;
            let c = self.clock();
            let out = self.sup.tick(c, &mut self.f);
            self.log.extend(out.finished);
        }
    }
    fn status(&mut self) -> Status {
        let c = self.clock();
        self.sup.status(c)
    }
}

fn scheduled() -> Settings {
    Settings {
        schedule: nine_to_five(),
        ..Settings::default()
    }
}

const H: u32 = 3600;

#[test]
fn a_full_working_day_runs_itself() {
    let mut d = Day::new(scheduled(), MONDAY);
    let plan = d.sup.plan_for(MONDAY, 0);
    let (am, pm) = (plan[0], plan[1]);

    d.until(8 * H);
    let st = d.status();
    assert_eq!(st.phase, Phase::Idle);
    assert_eq!(st.schedule.state, ScheduleState::Waiting);
    assert_eq!(st.schedule.next_start_s, Some(am.start_s));

    d.until(10 * H);
    let st = d.status();
    assert_eq!(st.phase, Phase::Running);
    assert_eq!(st.origin, Origin::Schedule);
    assert_eq!(st.schedule.state, ScheduleState::Running);
    assert_eq!(st.stops_at_s, Some(am.end_s));

    d.until(13 * H + 30 * 60);
    assert_eq!(d.status().phase, Phase::Idle, "lunch");
    assert_eq!(d.status().schedule.state, ScheduleState::Waiting);

    d.until(15 * H);
    assert_eq!(d.status().phase, Phase::Running);

    d.until(18 * H);
    let st = d.status();
    assert_eq!(st.phase, Phase::Idle);
    assert_eq!(st.schedule.state, ScheduleState::Done);

    assert_eq!(d.log.len(), 2);
    assert_eq!((d.log[0].start_s, d.log[0].end_s), (am.start_s, am.end_s));
    assert_eq!((d.log[1].start_s, d.log[1].end_s), (pm.start_s, pm.end_s));
    for r in &d.log {
        assert!(r.by_schedule);
        assert_eq!(r.reason, StopReason::ScheduleEnd);
        assert!(
            r.actions > 200,
            "about one wiggle every 35 s: {}",
            r.actions
        );
    }
    assert!(d.f.clicks.is_empty());
}

#[test]
fn stopping_by_hand_holds_until_the_next_segment() {
    let mut d = Day::new(scheduled(), MONDAY);
    d.until(10 * H);
    assert_eq!(d.status().phase, Phase::Running);
    let c = d.clock();
    let out = d.sup.stop(c);
    assert_eq!(out.finished.len(), 1);
    assert_eq!(out.finished[0].reason, StopReason::Manual);
    d.until(12 * H);
    let st = d.status();
    assert_eq!(
        st.phase,
        Phase::Idle,
        "it does not restart behind your back"
    );
    assert_eq!(st.schedule.state, ScheduleState::Held);
    d.until(15 * H);
    assert_eq!(
        d.status().phase,
        Phase::Running,
        "the afternoon starts as planned"
    );
}

#[test]
fn skip_today_and_weekends_do_nothing() {
    let mut s = scheduled();
    s.schedule.skip_day = Some(MONDAY);
    let mut d = Day::new(s, MONDAY);
    d.until(18 * H);
    assert!(d.log.is_empty());
    assert_eq!(d.status().schedule.state, ScheduleState::Skipped);

    let mut d = Day::new(scheduled(), MONDAY + 5); // Saturday
    d.until(18 * H);
    assert!(d.log.is_empty());
    assert_eq!(d.status().schedule.state, ScheduleState::Done);
    assert!(d.f.moves.is_empty());
}

#[test]
fn a_manual_session_is_not_cut_short_by_the_timetable() {
    let mut d = Day::new(scheduled(), MONDAY);
    d.until(8 * H);
    let c = d.clock();
    d.sup.start(c).unwrap();
    d.until(13 * H + 30 * 60); // through lunch
    let st = d.status();
    assert_eq!(st.phase, Phase::Running);
    assert_eq!(st.origin, Origin::Manual);
    assert!(d.log.is_empty());
}

#[test]
fn scheduled_click_mode_without_a_target_reports_once_and_waits() {
    let mut s = scheduled();
    s.schedule.blocks[0].mode = Some(Mode::Click);
    let mut d = Day::new(s, MONDAY);
    d.until(11 * H);
    let st = d.status();
    assert_eq!(st.phase, Phase::Idle);
    assert_eq!(st.schedule.state, ScheduleState::Held);
    let last = st.last_stop.unwrap();
    assert_eq!(last.reason, StopReason::Error);
    assert_eq!(last.message.as_deref(), Some("no-target"));
}

#[test]
fn turning_the_timetable_off_stops_a_scheduled_session() {
    let mut d = Day::new(scheduled(), MONDAY);
    d.until(10 * H);
    assert_eq!(d.status().phase, Phase::Running);
    let mut s = d.sup.settings().clone();
    s.schedule.enabled = false;
    d.sup.set_settings(s);
    d.until(10 * H + 5);
    assert_eq!(d.status().phase, Phase::Idle);
    assert_eq!(d.log.last().unwrap().reason, StopReason::ScheduleEnd);
}

#[test]
fn status_reports_time_left_for_manual_stop_rules() {
    let mut s = Settings::default();
    s.stop = StopRule {
        kind: StopKind::AfterMinutes,
        minutes: 90,
        ..StopRule::default()
    };
    let mut d = Day::new(s, MONDAY);
    d.until(14 * H);
    let c = d.clock();
    d.sup.start(c).unwrap();
    d.until(14 * H + 600);
    let st = d.status();
    assert_eq!(st.remaining_ms, Some(80 * 60_000));
    assert_eq!(st.stops_at_s, Some(15 * H + 30 * 60));
    d.until(15 * H + 31 * 60);
    assert_eq!(d.status().phase, Phase::Idle);
    assert_eq!(d.log[0].reason, StopReason::TimeLimit);
    let json = serde_json::to_string(&d.status()).unwrap();
    assert!(
        json.contains("\"lastStop\"") && json.contains("\"timeLimit\""),
        "{json}"
    );
}

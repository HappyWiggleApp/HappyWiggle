mod common;
use common::*;
use wiggler_core::engine::{Engine, Event, Origin, Phase, StopReason};
use wiggler_core::platform::{Button, Point, Rect};
use wiggler_core::*;

fn run(e: &mut Engine, f: &mut Fake, ms: u64) -> Vec<Event> {
    let mut all = vec![];
    let end = f.now + ms;
    while f.now < end {
        f.now += TICK;
        all.extend(e.tick(f.now, f));
    }
    all
}

fn wiggle_cfg() -> Settings {
    Settings {
        mode: Mode::Wiggle,
        wiggle_min_s: 20.0,
        wiggle_max_s: 50.0,
        ..Settings::default()
    }
}

fn click_cfg() -> Settings {
    Settings {
        mode: Mode::Click,
        areas: vec![Rect {
            x: 1000,
            y: 200,
            w: 300,
            h: 100,
        }],
        click_min_s: 5.0,
        click_max_s: 10.0,
        ..Settings::default()
    }
}

#[test]
fn natural_wiggle_moves_never_clicks_and_comes_home() {
    let mut f = Fake::new();
    let mut e = Engine::new(1);
    e.start(wiggle_cfg(), Origin::Manual, 0, 0).unwrap();
    run(&mut e, &mut f, 10 * 60_000);
    // 10 minutes at one wiggle every 20-50 s
    assert!((11..=31).contains(&e.actions()), "wiggles: {}", e.actions());
    assert!(f.clicks.is_empty());
    assert!(f.moves.len() > 100, "a natural wiggle is many small steps");
    assert_eq!(
        f.cursor,
        Point::new(500, 500),
        "cursor ends where it started"
    );
    let far = f
        .moves
        .iter()
        .map(|p| p.dist(Point::new(500, 500)))
        .fold(0.0, f64::max);
    assert!(far <= 75.0 && far >= 20.0, "furthest reach {far}");
    assert_eq!(e.phase(), Phase::Running);
}

#[test]
fn natural_wiggle_steps_are_small() {
    let mut f = Fake::new();
    let mut e = Engine::new(2);
    e.start(wiggle_cfg(), Origin::Manual, 0, 0).unwrap();
    run(&mut e, &mut f, 5 * 60_000);
    let mut prev = Point::new(500, 500);
    for m in &f.moves {
        assert!(m.dist(prev) < 25.0, "jump of {} px", m.dist(prev));
        prev = *m;
    }
}

#[test]
fn subtle_wiggle_is_one_pixel() {
    let mut f = Fake::new();
    let mut e = Engine::new(3);
    let cfg = Settings {
        wiggle_style: WiggleStyle::Subtle,
        ..wiggle_cfg()
    };
    e.start(cfg, Origin::Manual, 0, 0).unwrap();
    run(&mut e, &mut f, 5 * 60_000);
    assert!(e.actions() >= 5);
    assert_eq!(f.moves.len() as u64, e.actions() * 2);
    for m in &f.moves {
        assert!(m.dist(Point::new(500, 500)) <= 1.0);
    }
    assert_eq!(f.cursor, Point::new(500, 500));
}

#[test]
fn clicks_land_inside_the_area_and_cursor_returns() {
    let mut f = Fake::new();
    let mut e = Engine::new(4);
    e.start(click_cfg(), Origin::Manual, 0, 0).unwrap();
    run(&mut e, &mut f, 5 * 60_000);
    while e.in_motion() {
        run(&mut e, &mut f, TICK); // let a glide in progress finish
    }
    assert!(
        (28..=62).contains(&f.clicks.len()),
        "clicks: {}",
        f.clicks.len()
    );
    let area = Rect {
        x: 1000,
        y: 200,
        w: 300,
        h: 100,
    };
    let mut distinct = std::collections::HashSet::new();
    for (p, b, _) in &f.clicks {
        assert!(area.contains(*p), "{p:?} outside area");
        assert_eq!(*b, Button::Left);
        distinct.insert((p.x, p.y));
    }
    assert!(distinct.len() > 20, "spots should vary");
    assert_eq!(f.cursor, Point::new(500, 500));
    assert_eq!(e.actions() as usize, f.clicks.len());
}

#[test]
fn click_intervals_respect_the_range() {
    let mut f = Fake::new();
    let mut e = Engine::new(5);
    e.start(click_cfg(), Origin::Manual, 0, 0).unwrap();
    run(&mut e, &mut f, 10 * 60_000);
    let times: Vec<u64> = f.clicks.iter().map(|c| c.2).collect();
    let gaps: Vec<u64> = times.windows(2).map(|w| w[1] - w[0]).collect();
    for g in &gaps {
        // range is 5-10 s; allow for glide time either side
        assert!((4_000..=11_500).contains(g), "gap {g} ms");
    }
    let (min, max) = (gaps.iter().min().unwrap(), gaps.iter().max().unwrap());
    assert!(max - min > 2_000, "intervals should not be constant");
}

#[test]
fn exact_spot_and_jump_mode() {
    let mut f = Fake::new();
    let mut e = Engine::new(6);
    let cfg = Settings {
        areas: vec![Rect {
            x: 42,
            y: 43,
            w: 1,
            h: 1,
        }],
        smooth_move: false,
        return_cursor: false,
        button: Button::Right,
        ..click_cfg()
    };
    e.start(cfg, Origin::Manual, 0, 0).unwrap();
    run(&mut e, &mut f, 60_000);
    assert!(!f.clicks.is_empty());
    for (p, b, _) in &f.clicks {
        assert_eq!(*p, Point::new(42, 43));
        assert_eq!(*b, Button::Right);
    }
    assert_eq!(f.cursor, Point::new(42, 43));
}

#[test]
fn sequence_order_visits_areas_in_turn() {
    let mut f = Fake::new();
    let mut e = Engine::new(7);
    let cfg = Settings {
        areas: vec![
            Rect {
                x: 0,
                y: 0,
                w: 10,
                h: 10,
            },
            Rect {
                x: 100,
                y: 0,
                w: 10,
                h: 10,
            },
            Rect {
                x: 200,
                y: 0,
                w: 10,
                h: 10,
            },
        ],
        area_order: AreaOrder::Sequence,
        ..click_cfg()
    };
    e.start(cfg, Origin::Manual, 0, 0).unwrap();
    run(&mut e, &mut f, 2 * 60_000);
    assert!(f.clicks.len() >= 6);
    for (i, (p, _, _)) in f.clicks.iter().enumerate() {
        assert_eq!(p.x / 100, (i % 3) as i32);
    }
}

#[test]
fn click_mode_needs_a_target() {
    let mut e = Engine::new(8);
    let cfg = Settings {
        mode: Mode::Click,
        ..Settings::default()
    };
    assert_eq!(e.start(cfg, Origin::Manual, 0, 0), Err("no-target".into()));
    assert_eq!(e.phase(), Phase::Idle);
}

#[test]
fn touching_the_mouse_pauses_then_resumes() {
    let mut f = Fake::new();
    let mut e = Engine::new(9);
    e.start(wiggle_cfg(), Origin::Manual, 0, 0).unwrap();
    run(&mut e, &mut f, 60_000);
    let before = e.actions();
    assert!(before >= 1);

    // the user moves around for 2 minutes
    let moves_before = f.moves.len();
    let mut saw_pause = false;
    for i in 0..(120_000 / TICK) {
        f.now += TICK;
        if i % 10 == 0 {
            f.user_move(Point::new(300 + (i % 200) as i32, 300));
        }
        for ev in e.tick(f.now, &mut f) {
            if ev == Event::PhaseChanged(Phase::UserActive) {
                saw_pause = true;
            }
        }
    }
    assert!(saw_pause);
    assert_eq!(e.phase(), Phase::UserActive);
    assert_eq!(
        e.actions(),
        before,
        "nothing happens while the user is active"
    );
    assert_eq!(
        f.moves.len(),
        moves_before,
        "we never fight the user for the cursor"
    );

    // the user lets go: after resume_after_s (5 s) we carry on
    run(&mut e, &mut f, 4_000);
    assert_eq!(e.phase(), Phase::UserActive);
    run(&mut e, &mut f, 1_500);
    assert_eq!(e.phase(), Phase::Running);
    run(&mut e, &mut f, 60_000);
    assert!(e.actions() > before);
    assert!(e.user_pause_ms() >= 120_000);
}

#[test]
fn grabbing_the_mouse_mid_glide_stops_the_glide() {
    let mut f = Fake::new();
    let mut e = Engine::new(10);
    let cfg = Settings {
        areas: vec![Rect {
            x: 1800,
            y: 900,
            w: 5,
            h: 5,
        }],
        ..click_cfg()
    };
    e.start(cfg, Origin::Manual, 0, 0).unwrap();
    // run until the glide towards the target has begun
    while f.moves.len() < 3 {
        f.now += TICK;
        e.tick(f.now, &mut f);
    }
    f.now += TICK;
    f.user_move(Point::new(100, 100));
    e.tick(f.now, &mut f);
    assert_eq!(e.phase(), Phase::UserActive);
    let n = f.moves.len();
    run(&mut e, &mut f, 1_000);
    assert_eq!(f.moves.len(), n);
    assert!(
        f.clicks.is_empty(),
        "the click is abandoned, not delivered somewhere random"
    );
}

#[test]
fn typing_pauses_too_when_the_os_reports_idle_time() {
    let mut f = Fake::new();
    f.report_idle = true;
    let mut e = Engine::new(11);
    e.start(wiggle_cfg(), Origin::Manual, 0, 0).unwrap();
    // our own moves reset the idle timer and must not count as the user
    run(&mut e, &mut f, 3 * 60_000);
    assert_eq!(e.phase(), Phase::Running);
    assert!(e.actions() >= 3);
    assert_eq!(e.user_pause_ms(), 0);

    // wait for a quiet moment between wiggles, then press a key
    while e.in_motion() || e.next_action_in_ms(f.now).unwrap_or(0) < 3_000 {
        f.now += TICK;
        e.tick(f.now, &mut f);
    }
    run(&mut e, &mut f, 1_000);
    f.user_key();
    run(&mut e, &mut f, 100);
    assert_eq!(e.phase(), Phase::UserActive);
    run(&mut e, &mut f, 6_000);
    assert_eq!(e.phase(), Phase::Running);
}

#[test]
fn pause_on_user_can_be_switched_off() {
    let mut f = Fake::new();
    let mut e = Engine::new(12);
    let cfg = Settings {
        pause_on_user: false,
        ..wiggle_cfg()
    };
    e.start(cfg, Origin::Manual, 0, 0).unwrap();
    for i in 0..(120_000 / TICK) {
        f.now += TICK;
        if i % 50 == 0 && !e.in_motion() {
            f.user_move(Point::new(300 + (i % 200) as i32, 300));
        }
        e.tick(f.now, &mut f);
    }
    assert_eq!(e.phase(), Phase::Running);
    assert!(e.actions() >= 2);
}

#[test]
fn stops_after_n_actions() {
    let mut f = Fake::new();
    let mut e = Engine::new(13);
    let cfg = Settings {
        stop: StopRule {
            kind: StopKind::AfterActions,
            actions: 7,
            ..StopRule::default()
        },
        ..click_cfg()
    };
    e.start(cfg, Origin::Manual, 0, 0).unwrap();
    let ev = run(&mut e, &mut f, 10 * 60_000);
    assert_eq!(f.clicks.len(), 7);
    assert_eq!(e.phase(), Phase::Idle);
    assert!(ev.contains(&Event::Stopped {
        reason: StopReason::ActionLimit,
        message: None
    }));
    assert_eq!(
        f.cursor,
        Point::new(500, 500),
        "cursor is returned before stopping"
    );
}

#[test]
fn stops_after_n_minutes_and_pausing_extends_it() {
    let mut f = Fake::new();
    let mut e = Engine::new(14);
    let cfg = Settings {
        stop: StopRule {
            kind: StopKind::AfterMinutes,
            minutes: 10,
            ..StopRule::default()
        },
        ..wiggle_cfg()
    };
    e.start(cfg, Origin::Manual, 0, 0).unwrap();
    run(&mut e, &mut f, 4 * 60_000);
    assert_eq!(e.remaining_ms(f.now), Some(6 * 60_000));
    e.pause(f.now);
    let frozen = f.moves.len();
    run(&mut e, &mut f, 30 * 60_000);
    assert_eq!(e.phase(), Phase::Paused);
    assert_eq!(f.moves.len(), frozen);
    assert_eq!(
        e.remaining_ms(f.now),
        Some(6 * 60_000),
        "time left is frozen while paused"
    );
    e.resume(f.now);
    run(&mut e, &mut f, 6 * 60_000 - 1_000);
    assert_eq!(e.phase(), Phase::Running);
    let ev = run(&mut e, &mut f, 1_100);
    assert_eq!(e.phase(), Phase::Idle);
    assert!(ev.contains(&Event::Stopped {
        reason: StopReason::TimeLimit,
        message: None
    }));
}

#[test]
fn stops_at_a_clock_time_even_if_paused_in_between() {
    let mut f = Fake::new();
    let mut e = Engine::new(15);
    let cfg = Settings {
        stop: StopRule {
            kind: StopKind::AtTime,
            at_minute: 17 * 60,
            ..StopRule::default()
        },
        ..wiggle_cfg()
    };
    // started at 16:30:00
    e.start(cfg.clone(), Origin::Manual, 0, 16 * 3600 + 30 * 60)
        .unwrap();
    assert_eq!(e.remaining_ms(0), Some(30 * 60_000));
    run(&mut e, &mut f, 10 * 60_000);
    e.pause(f.now);
    run(&mut e, &mut f, 10 * 60_000);
    e.resume(f.now);
    run(&mut e, &mut f, 9 * 60_000);
    assert_eq!(e.phase(), Phase::Running);
    let ev = run(&mut e, &mut f, 61_000);
    assert!(ev.contains(&Event::Stopped {
        reason: StopReason::ClockTime,
        message: None
    }));

    // started at 18:00 with "stop at 17:00" means tomorrow, not immediately
    let mut e = Engine::new(15);
    e.start(cfg, Origin::Manual, 0, 18 * 3600).unwrap();
    assert_eq!(e.remaining_ms(0), Some(23 * 3600 * 1000));
}

#[test]
fn waking_from_sleep_does_not_fire_a_burst() {
    let mut f = Fake::new();
    let mut e = Engine::new(16);
    e.start(click_cfg(), Origin::Manual, 0, 0).unwrap();
    run(&mut e, &mut f, 60_000);
    let n = f.clicks.len();
    // lid closed for 3 hours; on wake the user has nudged the mouse
    f.now += 3 * 3600 * 1000;
    f.cursor = Point::new(900, 900);
    let ev = e.tick(f.now, &mut f);
    assert!(ev.iter().any(|x| matches!(x, Event::SleepGap { .. })));
    assert_eq!(f.clicks.len(), n);
    assert_eq!(e.phase(), Phase::Running);
    run(&mut e, &mut f, 1_500);
    assert_eq!(f.clicks.len(), n, "no instant click on wake");
    run(&mut e, &mut f, 30_000);
    assert!(f.clicks.len() > n);
    assert!(e.sleep_ms() >= 3 * 3600 * 1000);
    assert!(
        e.active_ms() < 2 * 60_000,
        "sleep is not counted as running time"
    );
}

#[test]
fn a_platform_error_stops_cleanly() {
    let mut f = Fake::new();
    let mut e = Engine::new(17);
    e.start(wiggle_cfg(), Origin::Manual, 0, 0).unwrap();
    run(&mut e, &mut f, 5_000);
    f.fail = Some("permission");
    let ev = run(&mut e, &mut f, 100);
    assert_eq!(e.phase(), Phase::Idle);
    assert!(ev.contains(&Event::Stopped {
        reason: StopReason::Error,
        message: Some("permission".into())
    }));
}

#[test]
fn wiggles_stay_on_screen_in_a_corner() {
    let mut f = Fake::new();
    f.bounds = Some(Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1080,
    });
    f.walls = f.bounds;
    f.cursor = Point::new(0, 0);
    let mut e = Engine::new(18);
    e.start(wiggle_cfg(), Origin::Manual, 0, 0).unwrap();
    run(&mut e, &mut f, 20 * 60_000);
    assert!(e.actions() >= 20);
    assert_eq!(
        e.phase(),
        Phase::Running,
        "hitting the edge is not mistaken for the user"
    );
    assert_eq!(e.user_pause_ms(), 0);
    for m in &f.moves {
        assert!(m.x >= 0 && m.y >= 0);
    }
}

#[test]
fn glide_always_ends_on_target() {
    let mut rng = wiggler_core::rng::Rng::new(99);
    for i in 0..500 {
        let a = Point::new(
            rng.range_i64(-3000, 3000) as i32,
            rng.range_i64(-2000, 2000) as i32,
        );
        let b = Point::new(
            rng.range_i64(-3000, 3000) as i32,
            rng.range_i64(-2000, 2000) as i32,
        );
        let path = wiggler_core::path::glide(a, b, &mut rng);
        assert_eq!(*path.last().unwrap(), b, "case {i}");
        assert!(path.len() <= 46);
    }
}

#[test]
fn settings_survive_json_and_tolerate_missing_or_wild_values() {
    let s = Settings::default();
    let json = serde_json::to_string(&s).unwrap();
    assert!(
        json.contains("\"wiggleMinS\""),
        "camelCase for the UI: {json}"
    );
    assert_eq!(serde_json::from_str::<Settings>(&json).unwrap(), s);

    // an old or hand-edited file
    let mut old: Settings = serde_json::from_str(
        r#"{"mode":"click","clickMinS":0.001,"clickMaxS":-5,"futureField":1}"#,
    )
    .unwrap();
    old.sanitize();
    assert_eq!(old.mode, Mode::Click);
    assert!(old.click_min_s >= 1.0 && old.click_max_s >= old.click_min_s);
    assert_eq!(old.hotkey, "F8");
    assert!(old.keep_awake);
}

#[test]
fn screen_edges_are_not_mistaken_for_the_user_when_the_os_reports_no_bounds() {
    // macOS case: we do not know where the desktop ends.
    for (seed, corner) in [
        (19, Point::new(0, 0)),
        (20, Point::new(1919, 1079)),
        (21, Point::new(0, 600)),
    ] {
        let mut f = Fake::new();
        f.walls = Some(Rect {
            x: 0,
            y: 0,
            w: 1920,
            h: 1080,
        });
        f.cursor = corner;
        let mut e = Engine::new(seed);
        e.start(wiggle_cfg(), Origin::Manual, 0, 0).unwrap();
        run(&mut e, &mut f, 30 * 60_000);
        assert!(e.actions() >= 30, "wiggles: {}", e.actions());
        assert_eq!(e.phase(), Phase::Running);
        assert_eq!(e.user_pause_ms(), 0, "corner {corner:?}");
    }
}

#[test]
fn a_target_that_cannot_be_reached_is_never_clicked() {
    let mut f = Fake::new();
    f.walls = Some(Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1080,
    });
    let mut e = Engine::new(22);
    // the area was on a second monitor that has been unplugged
    let cfg = Settings {
        areas: vec![Rect {
            x: 2500,
            y: 300,
            w: 50,
            h: 50,
        }],
        ..click_cfg()
    };
    e.start(cfg, Origin::Manual, 0, 0).unwrap();
    run(&mut e, &mut f, 3 * 60_000);
    assert!(f.clicks.is_empty());
    assert_eq!(e.phase(), Phase::Running);
}

#[test]
fn a_lock_screen_or_permission_prompt_pauses_instead_of_ending_the_session() {
    let mut f = Fake::new();
    let mut e = Engine::new(23);
    e.start(wiggle_cfg(), Origin::Manual, 0, 0).unwrap();
    run(&mut e, &mut f, 30_000);
    let before = e.actions();
    // Windows shows a permission prompt: the pointer cannot be read for a minute
    f.fail = Some("cursor-unavailable");
    let ev = run(&mut e, &mut f, 60_000);
    assert_eq!(e.phase(), Phase::UserActive);
    assert!(!ev.iter().any(|x| matches!(x, Event::Stopped { .. })));
    assert_eq!(e.actions(), before);
    f.fail = None;
    run(&mut e, &mut f, 6_000);
    assert_eq!(e.phase(), Phase::Running);
    run(&mut e, &mut f, 60_000);
    assert!(e.actions() > before);
}

#[test]
fn removing_a_target_mid_glide_cancels_that_click() {
    let mut f = Fake::new();
    let mut e = Engine::new(24);
    let cfg = Settings {
        areas: vec![Rect {
            x: 1800,
            y: 900,
            w: 5,
            h: 5,
        }],
        ..click_cfg()
    };
    e.start(cfg.clone(), Origin::Manual, 0, 0).unwrap();
    while f.moves.len() < 3 {
        f.now += TICK;
        e.tick(f.now, &mut f);
    }
    let moved = Settings {
        areas: vec![Rect {
            x: 100,
            y: 100,
            w: 5,
            h: 5,
        }],
        ..cfg
    };
    e.update_settings(moved);
    run(&mut e, &mut f, 1_500);
    assert!(f.clicks.is_empty(), "the old target must not be clicked");
    run(&mut e, &mut f, 30_000);
    assert!(!f.clicks.is_empty());
    for (p, _, _) in &f.clicks {
        assert!(Rect {
            x: 100,
            y: 100,
            w: 5,
            h: 5
        }
        .contains(*p));
    }
}

#[test]
fn a_long_glide_across_a_big_desktop_is_not_mistaken_for_the_user() {
    // the OS applies each of our moves one step late
    struct Laggy {
        inner: Fake,
        queued: Option<Point>,
    }
    impl wiggler_core::Platform for Laggy {
        fn cursor(&mut self) -> Result<Point, String> {
            self.inner.cursor()
        }
        fn move_to(&mut self, p: Point) -> Result<(), String> {
            if let Some(q) = self.queued.replace(p) {
                self.inner.cursor = q;
            }
            self.inner.moves.push(p);
            Ok(())
        }
        fn click(&mut self, p: Point, b: Button) -> Result<(), String> {
            self.inner.click(p, b)
        }
    }
    let mut l = Laggy {
        inner: Fake::new(),
        queued: None,
    };
    l.inner.cursor = Point::new(10, 10);
    let mut e = Engine::new(25);
    let cfg = Settings {
        areas: vec![Rect {
            x: 3700,
            y: 2000,
            w: 40,
            h: 40,
        }],
        return_cursor: false,
        ..click_cfg()
    };
    e.start(cfg, Origin::Manual, 0, 0).unwrap();
    let mut now = 0;
    for _ in 0..(20_000 / TICK) {
        now += TICK;
        l.inner.now = now;
        // the delayed move lands between ticks
        if !e.in_motion() {
            if let Some(q) = l.queued.take() {
                l.inner.cursor = q;
            }
        }
        e.tick(now, &mut l);
    }
    assert_eq!(e.user_pause_ms(), 0);
}

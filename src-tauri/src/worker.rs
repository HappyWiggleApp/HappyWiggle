//! The heartbeat: one background thread that ticks the engine, keeps the
//! computer awake while a session runs, and tells the windows what changed.

use crate::input::OsMouse;
use crate::state::Shared;
use crate::tray;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use wiggler_core::Phase;

pub fn spawn(app: AppHandle, shared: Arc<Shared>) {
    std::thread::Builder::new()
        .name("wiggler".into())
        .spawn(move || run(app, shared))
        .expect("could not start the worker thread");
}

fn run(app: AppHandle, shared: Arc<Shared>) {
    let mut mouse = OsMouse;
    // Held while a session is live; dropping it lets the computer sleep again.
    let mut awake: Option<keepawake::KeepAwake> = None;
    let mut awake_failed = false;
    let mut last_emit = Instant::now();
    let mut last_phase = Phase::Idle;

    loop {
        let clock = shared.clock();
        let (outcome, status, fast, keep_awake, compact) = {
            let mut sup = shared.sup.lock().unwrap();
            let outcome = sup.tick(clock, &mut mouse);
            let status = sup.status(clock);
            (
                outcome,
                status,
                sup.engine().in_motion(),
                sup.settings().keep_awake,
                sup.settings().compact_on_start,
            )
        };
        shared.record(outcome.finished);

        let live = matches!(status.phase, Phase::Running | Phase::UserActive);
        if live && keep_awake {
            if awake.is_none() && !awake_failed {
                awake = keepawake::Builder::default()
                    .display(true)
                    .idle(true)
                    .reason("HappyWiggle session running")
                    .app_name("HappyWiggle")
                    .app_reverse_domain(crate::APP_ID)
                    .create()
                    .ok();
                // if the system refuses, do not ask again on every tick
                awake_failed = awake.is_none();
            }
        } else {
            awake = None;
            awake_failed = false;
        }

        let phase_changed = status.phase != last_phase;
        if phase_changed {
            tray::refresh(&app, &shared, status.phase);
            // A session the timetable started goes straight to the small bar
            // if that is how the user likes it.
            if last_phase == Phase::Idle {
                crate::cancel_picking(&app, &shared);
                if compact {
                    let _ = app.emit("session-started", ());
                }
            }
            last_phase = status.phase;
        }
        let period = if status.phase == Phase::Idle {
            1000
        } else {
            250
        };
        if outcome.changed || phase_changed || last_emit.elapsed() >= Duration::from_millis(period)
        {
            let _ = app.emit("status", &status);
            last_emit = Instant::now();
        }

        std::thread::sleep(Duration::from_millis(if fast {
            10
        } else if status.phase == Phase::Idle {
            250
        } else {
            100
        }));
    }
}

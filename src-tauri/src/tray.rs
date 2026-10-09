//! Tray / menu-bar icon: start, stop and pause without opening the window.

use crate::state::Shared;
use std::sync::Arc;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};
use wiggler_core::Phase;

const TRAY_ID: &str = "main";
const ICON_ON: &[u8] = include_bytes!("../icons/tray-on.png");
const ICON_OFF: &[u8] = include_bytes!("../icons/tray-off.png");

/// Menu texts. English by default; the window sends translations at startup.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(default)]
pub struct Labels {
    pub start: String,
    pub stop: String,
    pub pause: String,
    pub resume: String,
    pub show: String,
    pub quit: String,
    pub idle: String,
    pub running: String,
    pub paused: String,
    pub waiting: String,
}

impl Default for Labels {
    fn default() -> Self {
        Labels {
            start: "Start".into(),
            stop: "Stop".into(),
            pause: "Pause".into(),
            resume: "Resume".into(),
            show: "Open SoftMouseWiggler".into(),
            quit: "Quit".into(),
            idle: "SoftMouseWiggler: off".into(),
            running: "SoftMouseWiggler: running".into(),
            paused: "SoftMouseWiggler: paused".into(),
            waiting: "SoftMouseWiggler: waiting for you to let go of the mouse".into(),
        }
    }
}

/// Handles kept so the texts can change while the app runs.
pub struct Items {
    toggle: MenuItem<Wry>,
    pause: MenuItem<Wry>,
    show: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

pub fn build(app: &tauri::App, _shared: &Arc<Shared>) -> tauri::Result<()> {
    let l = Labels::default();
    let toggle = MenuItem::with_id(app, "toggle", &l.start, true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", &l.pause, false, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", &l.show, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", &l.quit, true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&toggle, &pause, &sep, &show, &sep2, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(ICON_OFF)?)
        .tooltip(&l.idle)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let shared = app.state::<Arc<Shared>>().inner().clone();
            let result = match event.id.as_ref() {
                "toggle" => crate::run_control(app, &shared, "toggle"),
                "pause" => {
                    let phase = shared.sup.lock().unwrap().engine().phase();
                    let action = if phase == Phase::Running {
                        "pause"
                    } else {
                        "resume"
                    };
                    crate::run_control(app, &shared, action)
                }
                "show" => {
                    crate::show_compact(app, false);
                    Ok(())
                }
                "quit" => {
                    app.exit(0);
                    Ok(())
                }
                _ => Ok(()),
            };
            if let Err(e) = result {
                // e.g. Click mode with no target yet: open the window and say why
                crate::show_compact(app, false);
                let _ = app.emit("problem", e);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::show_compact(tray.app_handle(), false);
            }
        })
        .build(app)?;

    app.manage(Items {
        toggle,
        pause,
        show,
        quit,
    });
    Ok(())
}

/// Bring icon, tooltip and menu texts in line with the engine's phase.
pub fn refresh(app: &AppHandle, shared: &Shared, phase: Phase) {
    let l = shared.tray_labels.lock().unwrap().clone();
    if let Some(items) = app.try_state::<Items>() {
        let active = phase != Phase::Idle;
        let _ = items
            .toggle
            .set_text(if active { &l.stop } else { &l.start });
        let _ = items.pause.set_text(if phase == Phase::Running || !active {
            &l.pause
        } else {
            &l.resume
        });
        let _ = items.pause.set_enabled(active);
        let _ = items.show.set_text(&l.show);
        let _ = items.quit.set_text(&l.quit);
    }
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let (tip, icon) = match phase {
            Phase::Idle => (&l.idle, ICON_OFF),
            Phase::Running => (&l.running, ICON_ON),
            Phase::Paused => (&l.paused, ICON_OFF),
            Phase::UserActive => (&l.waiting, ICON_ON),
        };
        let _ = tray.set_tooltip(Some(tip.as_str()));
        if let Ok(img) = Image::from_bytes(icon) {
            let _ = tray.set_icon(Some(img));
        }
    }
}

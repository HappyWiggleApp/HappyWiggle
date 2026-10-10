//! HappyWiggle desktop shell.
//!
//! All decisions (when to move, when to stop, the timetable) live in the
//! `wiggler-core` crate. This file only wires that engine to the real world:
//! windows, tray icon, global hotkey, settings on disk and a background thread
//! that calls `tick`.

mod input;
mod startup;
mod state;
mod tray;
mod worker;

use state::{emit_status, Links, Shared};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use wiggler_core::history::History;
use wiggler_core::platform::{Point, Rect};
use wiggler_core::schedule::Segment;
use wiggler_core::{Settings, Status, MAX_AREAS};

pub const APP_ID: &str = "app.happywiggle";

/// Apple wants tips to go through in-app purchase, so the Mac App Store build
/// shows no external donation link.
const DONATE: bool = cfg!(not(feature = "mas"));

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Info {
    version: String,
    os: &'static str,
    /// "switch" or "windowsSettings": how launch-at-login is offered.
    autostart: startup::Mode,
    donate: bool,
    permission: bool,
    links: Links,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    info: Info,
    settings: Settings,
    status: Status,
}

fn info(app: &AppHandle, shared: &Shared) -> Info {
    let mut links = shared.links.clone();
    if !DONATE {
        links.donate.clear();
    }
    Info {
        version: app.package_info().version.to_string(),
        os: std::env::consts::OS,
        autostart: startup::MODE,
        donate: DONATE && !links.donate.is_empty(),
        permission: input::has_permission(),
        links,
    }
}

// ---------------------------------------------------------------- commands

#[tauri::command]
fn get_state(app: AppHandle, shared: State<'_, Arc<Shared>>) -> Snapshot {
    let clock = shared.clock();
    let mut sup = shared.sup.lock().unwrap();
    Snapshot {
        info: info(&app, &shared),
        settings: sup.settings().clone(),
        status: sup.status(clock),
    }
}

#[tauri::command]
fn save_settings(
    app: AppHandle,
    shared: State<'_, Arc<Shared>>,
    settings: Settings,
) -> Result<Settings, String> {
    let (old, new) = {
        let mut sup = shared.sup.lock().unwrap();
        let old = sup.settings().clone();
        sup.set_settings(settings);
        (old, sup.settings().clone())
    };
    shared.save_settings(&new);
    let mut result = Ok(());
    if old.hotkey != new.hotkey {
        result = apply_hotkey(&app, &new.hotkey);
        if result.is_err() {
            // keep the previous, working hotkey
            let _ = apply_hotkey(&app, &old.hotkey);
            let mut sup = shared.sup.lock().unwrap();
            let mut s = sup.settings().clone();
            s.hotkey = old.hotkey.clone();
            sup.set_settings(s.clone());
            shared.save_settings(&s);
        }
    }
    if old.autostart != new.autostart && startup::set(&app, new.autostart).is_err() {
        // the system said no: show the switch as it really is
        let mut sup = shared.sup.lock().unwrap();
        let mut s = sup.settings().clone();
        s.autostart = old.autostart;
        sup.set_settings(s.clone());
        shared.save_settings(&s);
        if result.is_ok() {
            result = Err("autostart".into());
        }
    }
    let current = shared.sup.lock().unwrap().settings().clone();
    let _ = app.emit("settings", &current);
    emit_status(&app, &shared);
    result.map(|_| current)
}

/// start | stop | pause | resume | toggle
#[tauri::command]
fn control(app: AppHandle, shared: State<'_, Arc<Shared>>, action: String) -> Result<(), String> {
    run_control(&app, &shared, &action)
}

pub(crate) fn run_control(
    app: &AppHandle,
    shared: &Arc<Shared>,
    action: &str,
) -> Result<(), String> {
    let clock = shared.clock();
    let outcome = {
        let mut sup = shared.sup.lock().unwrap();
        let starting =
            matches!(action, "start") || (action == "toggle" && !sup.engine().is_active());
        if starting && !input::has_permission() {
            drop(sup);
            input::request_permission();
            return Err("permission".into());
        }
        if starting {
            drop(sup);
            cancel_picking(app, shared);
            sup = shared.sup.lock().unwrap();
        }
        match action {
            "start" => sup.start(clock)?,
            "stop" => sup.stop(clock),
            "pause" => sup.pause(clock),
            "resume" => sup.resume(clock),
            "toggle" => sup.toggle(clock)?,
            other => return Err(format!("unknown action: {other}")),
        }
    };
    shared.record(outcome.finished);
    emit_status(app, shared);
    Ok(())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DayPlan {
    day: i64,
    weekday: u8,
    today: bool,
    segments: Vec<Segment>,
}

/// The plan for the current week, Monday to Sunday.
#[tauri::command]
fn week(shared: State<'_, Arc<Shared>>) -> Vec<DayPlan> {
    let clock = shared.clock();
    let sup = shared.sup.lock().unwrap();
    let monday = clock.day - clock.weekday as i64;
    (0..7u8)
        .map(|wd| {
            let day = monday + wd as i64;
            DayPlan {
                day,
                weekday: wd,
                today: day == clock.day,
                segments: sup.plan_for(day, wd),
            }
        })
        .collect()
}

#[tauri::command]
fn history(shared: State<'_, Arc<Shared>>) -> History {
    shared.history.lock().unwrap().clone()
}

#[tauri::command]
fn clear_history(shared: State<'_, Arc<Shared>>) {
    let mut h = shared.history.lock().unwrap();
    h.sessions.clear();
    shared.save_history(&h);
}

/// macOS: is the app allowed to post mouse events? `request` shows the system prompt.
#[tauri::command]
fn permission(request: bool) -> bool {
    if request {
        input::request_permission()
    } else {
        input::has_permission()
    }
}

/// Where the mouse is right now, in the same coordinates the engine clicks in.
/// Used by "capture a spot with the pointer".
#[tauri::command]
fn capture_cursor() -> Result<Point, String> {
    input::cursor()
}

/// Move the cursor to the middle of an area so the user can see where it is.
#[tauri::command]
fn preview_area(shared: State<'_, Arc<Shared>>, index: usize) -> Result<(), String> {
    let sup = shared.sup.lock().unwrap();
    if sup.engine().is_active() {
        return Err("busy".into());
    }
    let a = *sup.settings().areas.get(index).ok_or("no-target")?;
    drop(sup);
    if !input::has_permission() {
        input::request_permission();
        return Err("permission".into());
    }
    use wiggler_core::Platform;
    input::OsMouse.move_to(Point::new(a.x + a.w / 2, a.y + a.h / 2))
}

/// Dim every screen so the user can draw a target area.
#[tauri::command]
async fn pick_area(
    app: AppHandle,
    shared: State<'_, Arc<Shared>>,
    index: Option<usize>,
) -> Result<(), String> {
    if shared.sup.lock().unwrap().engine().is_active() {
        return Err("busy".into());
    }
    close_overlays(&app);
    *shared.picking.lock().unwrap() = index;
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.hide();
    }
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    let mut opened = 0;
    for (i, m) in monitors.iter().enumerate() {
        let built = WebviewWindowBuilder::new(
            &app,
            format!("overlay-{i}"),
            WebviewUrl::App("overlay.html".into()),
        )
        .title("HappyWiggle")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .visible(false)
        .build();
        if let Ok(w) = built {
            if cfg!(target_os = "macos") {
                // macOS converts positions with the window's *current*
                // screen scale, so hand it points for the target screen.
                let sf = m.scale_factor();
                let _ = w.set_position(m.position().to_logical::<f64>(sf));
                let _ = w.set_size(m.size().to_logical::<f64>(sf));
            } else {
                let _ = w.set_position(*m.position());
                let _ = w.set_size(*m.size());
            }
            let _ = w.show();
            let _ = w.set_focus();
            opened += 1;
        }
    }
    if opened == 0 {
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.show();
        }
        return Err("overlay-failed".into());
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct CssRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// Called by an overlay window when the user has drawn a box (or pressed Esc).
#[tauri::command]
fn overlay_done(
    app: AppHandle,
    window: WebviewWindow,
    shared: State<'_, Arc<Shared>>,
    rect: Option<CssRect>,
) -> Result<(), String> {
    let index = shared.picking.lock().unwrap().take();
    let mut saved = None;
    if let Some(r) = rect {
        let pos = window.inner_position().map_err(|e| e.to_string())?;
        let scale = window.scale_factor().map_err(|e| e.to_string())?;
        // The page reports CSS pixels inside the window. The engine needs the
        // unit the OS input API uses: points on macOS, physical pixels elsewhere.
        let (ox, oy, k) = if cfg!(target_os = "macos") {
            (pos.x as f64 / scale, pos.y as f64 / scale, 1.0)
        } else {
            (pos.x as f64, pos.y as f64, scale)
        };
        let area = Rect {
            x: (ox + r.x * k).round() as i32,
            y: (oy + r.y * k).round() as i32,
            w: ((r.w * k).round() as i32).max(1),
            h: ((r.h * k).round() as i32).max(1),
        };
        let mut sup = shared.sup.lock().unwrap();
        let mut s = sup.settings().clone();
        match index {
            Some(i) if i < s.areas.len() => s.areas[i] = area,
            _ if s.areas.len() < MAX_AREAS => s.areas.push(area),
            _ => {}
        }
        sup.set_settings(s);
        saved = Some(sup.settings().clone());
    }
    if let Some(s) = &saved {
        shared.save_settings(s);
        let _ = app.emit("settings", s);
    }
    close_overlays(&app);
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
    Ok(())
}

/// Abandon area drawing: a session must never click on the drawing layer.
pub(crate) fn cancel_picking(app: &AppHandle, shared: &Shared) {
    *shared.picking.lock().unwrap() = None;
    close_overlays(app);
}

fn close_overlays(app: &AppHandle) {
    for (label, w) in app.webview_windows() {
        if label.starts_with("overlay-") {
            let _ = w.close();
        }
    }
}

/// Swap between the full window and the small always-on-top bar.
#[tauri::command]
fn set_compact(app: AppHandle, compact: bool, quiet: Option<bool>) {
    if !compact && quiet == Some(true) {
        // session over: put the bar away without popping the main window up
        if let Some(bar) = app.get_webview_window("bar") {
            let _ = bar.hide();
        }
        return;
    }
    show_compact(&app, compact);
}

pub(crate) fn show_compact(app: &AppHandle, compact: bool) {
    let (main, bar) = (
        app.get_webview_window("main"),
        app.get_webview_window("bar"),
    );
    if compact {
        if let Some(b) = bar {
            let _ = b.show();
        }
        if let Some(m) = main {
            let _ = m.hide();
        }
    } else {
        if let Some(b) = bar {
            let _ = b.hide();
        }
        show_main(app);
    }
}

pub(crate) fn show_main(app: &AppHandle) {
    if let Some(m) = app.get_webview_window("main") {
        let _ = m.show();
        let _ = m.unminimize();
        let _ = m.set_focus();
    }
}

#[tauri::command]
fn set_tray_labels(app: AppHandle, shared: State<'_, Arc<Shared>>, labels: tray::Labels) {
    *shared.tray_labels.lock().unwrap() = labels;
    let phase = shared.sup.lock().unwrap().engine().phase();
    tray::refresh(&app, &shared, phase);
}

/// Open one of the app's own links in the browser. The page cannot pass an
/// arbitrary URL: only the fixed set in `links.json` can be opened.
#[tauri::command]
fn open_link(app: AppHandle, shared: State<'_, Arc<Shared>>, which: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let l = &shared.links;
    let url = match which.as_str() {
        "source" => &l.source,
        "donate" if DONATE => &l.donate,
        "privacy" => &l.privacy,
        "issues" => &l.issues,
        _ => return Err("unknown link".into()),
    };
    if !url.starts_with("https://") {
        return Err("link not configured".into());
    }
    app.opener()
        .open_url(url.as_str(), None::<&str>)
        .map_err(|e| e.to_string())
}

/// Microsoft Store build: take the user to Windows' own startup switch.
#[tauri::command]
fn open_startup_settings(app: AppHandle) -> Result<(), String> {
    startup::open_system_settings(&app)
}

/// Called by a window once its interface has finished loading.
///
/// Normally does nothing. The automated start-up test sets the
/// HAPPYWIGGLE_SELFTEST variable to a folder; the app then leaves a small file
/// there per window, which proves on a real Windows, macOS or Linux machine
/// that the window came up with its content.
#[tauri::command]
fn ui_ready(window: WebviewWindow) {
    if let Some(dir) = std::env::var_os("HAPPYWIGGLE_SELFTEST") {
        let dir = std::path::PathBuf::from(dir);
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join(format!("{}.txt", window.label())), "ready");
    }
}

#[tauri::command]
fn quit(app: AppHandle) {
    app.exit(0);
}

// ---------------------------------------------------------------- plumbing

fn apply_hotkey(app: &AppHandle, key: &str) -> Result<(), String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    gs.on_shortcut(key, |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            let shared = app.state::<Arc<Shared>>().inner().clone();
            if let Err(e) = run_control(app, &shared, "toggle") {
                let _ = app.emit("problem", e);
                show_main(app);
            }
        }
    })
    .map_err(|_| "hotkey".to_string())
}

pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default();

    // A second launch just brings the existing window forward.
    #[cfg(not(feature = "mas"))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app);
        }));
    }
    #[cfg(not(any(feature = "mas", feature = "msstore")))]
    {
        builder = builder.plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--hidden"])
                .build(),
        );
    }

    builder
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_state,
            save_settings,
            control,
            week,
            history,
            clear_history,
            permission,
            capture_cursor,
            preview_area,
            pick_area,
            overlay_done,
            set_compact,
            set_tray_labels,
            open_link,
            open_startup_settings,
            ui_ready,
            quit
        ])
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            let shared = Arc::new(Shared::load(dir));
            app.manage(shared.clone());
            let handle = app.handle().clone();

            tray::build(app, &shared)?;

            // The user may have changed launch-at-login outside the app
            // (Task Manager, System Settings): show what is really set.
            if let Some(on) = startup::is_on(&handle) {
                let mut sup = shared.sup.lock().unwrap();
                if sup.settings().autostart != on {
                    let mut s = sup.settings().clone();
                    s.autostart = on;
                    sup.set_settings(s.clone());
                    shared.save_settings(&s);
                }
            }

            let settings = shared.sup.lock().unwrap().settings().clone();
            if apply_hotkey(&handle, &settings.hotkey).is_err() {
                // Another program owns the key. The app still works; the
                // Settings screen shows the hotkey so the user can change it.
                let _ = handle.emit("problem", "hotkey");
            }

            let hidden = settings.start_hidden || std::env::args().any(|a| a == "--hidden");
            if !hidden {
                show_main(&handle);
            }

            worker::spawn(handle, shared);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                match window.label() {
                    // Closing the window keeps the app in the tray, where the
                    // timetable and hotkey keep working. "Quit" really quits.
                    "main" => {
                        api.prevent_close();
                        let _ = window.hide();
                        let _ = window.app_handle().emit("hidden-to-tray", ());
                    }
                    "bar" => {
                        api.prevent_close();
                        show_compact(window.app_handle(), false);
                    }
                    _ => {}
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while starting HappyWiggle")
        .run(|_app, _event| {
            // macOS: clicking the Dock icon brings the window back.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = _event {
                show_compact(_app, false);
            }
        });
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use tauri_plugin_global_shortcut::Shortcut;

    /// The Settings screen builds hotkey strings from browser key codes.
    /// Every shape it can produce must be understood by the hotkey library.
    #[test]
    fn hotkey_strings_from_the_settings_screen_parse() {
        for s in [
            "F8",
            "F1",
            "F12",
            "Shift+F5",
            "Ctrl+Shift+KeyK",
            "Alt+Digit1",
            "Super+Shift+Period",
            "Ctrl+Alt+Space",
            "Ctrl+Comma",
            "Super+Slash",
            "Alt+Semicolon",
            "Ctrl+KeyZ",
            "Ctrl+Digit0",
        ] {
            assert!(
                Shortcut::from_str(s).is_ok(),
                "hotkey string not understood: {s}"
            );
        }
        assert!(Shortcut::from_str("NotAKey").is_err());
    }

    #[test]
    fn default_settings_hotkey_parses() {
        assert!(Shortcut::from_str(&wiggler_core::Settings::default().hotkey).is_ok());
    }

    #[test]
    fn bundled_links_file_is_valid() {
        let _: crate::state::Links =
            serde_json::from_str(include_str!("../links.json")).expect("links.json");
    }
}

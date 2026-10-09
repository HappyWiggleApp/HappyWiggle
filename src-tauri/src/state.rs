//! Shared application state and everything that touches the disk.

use crate::tray;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use wiggler_core::history::{History, SessionRecord};
use wiggler_core::{Clock, Settings, Supervisor};

/// Web addresses the app may open. Filled in `src-tauri/links.json`; a link
/// left empty simply hides its button.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Links {
    pub source: String,
    pub issues: String,
    pub privacy: String,
    pub donate: String,
}

/// A millisecond counter that keeps counting while the computer sleeps.
///
/// `Instant` alone pauses during sleep on some systems, which would hide the
/// sleep from the engine; the wall clock alone can jump backwards. We take
/// whichever moved further forward, within reason.
struct Ticker {
    total: Duration,
    instant: Instant,
    wall: SystemTime,
}

impl Ticker {
    fn new() -> Self {
        Ticker {
            total: Duration::from_secs(1),
            instant: Instant::now(),
            wall: SystemTime::now(),
        }
    }
    fn advance(&mut self) -> u64 {
        let (now_i, now_w) = (Instant::now(), SystemTime::now());
        let di = now_i.duration_since(self.instant);
        let dw = now_w.duration_since(self.wall).unwrap_or(di);
        // Durations are added whole; rounding to milliseconds happens once,
        // at the end, so the counter does not drift.
        self.total += if dw > di && dw < Duration::from_secs(7 * 24 * 3600) {
            dw
        } else {
            di
        };
        self.instant = now_i;
        self.wall = now_w;
        self.total.as_millis() as u64
    }
}

pub struct Shared {
    pub sup: Mutex<Supervisor>,
    pub history: Mutex<History>,
    pub tray_labels: Mutex<tray::Labels>,
    /// Which area slot the overlay is currently picking for.
    pub picking: Mutex<Option<usize>>,
    pub links: Links,
    ticker: Mutex<Ticker>,
    dir: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Install {
    seed: u64,
}

fn read_json<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Option<T> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// Write to a temporary file, then rename: a crash never leaves half a file.
fn write_json<T: serde::Serialize>(path: &PathBuf, value: &T) {
    let Ok(text) = serde_json::to_string_pretty(value) else {
        return;
    };
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, text).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

impl Shared {
    pub fn load(dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&dir);
        let settings: Settings = read_json(&dir.join("settings.json")).unwrap_or_default();
        let history: History = read_json(&dir.join("history.json")).unwrap_or_default();
        // A random number fixed at first launch. It makes this install's
        // "random" timetable different from everyone else's.
        let install_path = dir.join("install.json");
        let install = read_json::<Install>(&install_path).unwrap_or_else(|| {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(1);
            let fresh = Install {
                seed: wiggler_core::rng::hash64(&[nanos, std::process::id() as u64]),
            };
            write_json(&install_path, &fresh);
            fresh
        });
        let links: Links = serde_json::from_str(include_str!("../links.json")).unwrap_or_default();
        Shared {
            sup: Mutex::new(Supervisor::new(settings, install.seed)),
            history: Mutex::new(history),
            tray_labels: Mutex::new(tray::Labels::default()),
            picking: Mutex::new(None),
            links,
            ticker: Mutex::new(Ticker::new()),
            dir,
        }
    }

    pub fn clock(&self) -> Clock {
        use chrono::{Datelike, Timelike};
        let mono_ms = self.ticker.lock().unwrap().advance();
        let now = chrono::Local::now();
        Clock {
            mono_ms,
            day: now.date_naive().num_days_from_ce() as i64 - 719_163, // days since 1970-01-01
            weekday: now.weekday().num_days_from_monday() as u8,
            sec_of_day: now.time().num_seconds_from_midnight(),
        }
    }

    pub fn save_settings(&self, s: &Settings) {
        write_json(&self.dir.join("settings.json"), s);
    }

    pub fn save_history(&self, h: &History) {
        write_json(&self.dir.join("history.json"), h);
    }

    /// Append finished sessions to the history file.
    pub fn record(&self, finished: Vec<SessionRecord>) {
        if finished.is_empty() {
            return;
        }
        let mut h = self.history.lock().unwrap();
        for r in finished {
            h.push(r);
        }
        self.save_history(&h);
    }
}

pub fn emit_status(app: &AppHandle, shared: &Shared) {
    let clock = shared.clock();
    let status = shared.sup.lock().unwrap().status(clock);
    tray::refresh(app, shared, status.phase);
    let _ = app.emit("status", &status);
}

//! The real mouse. One small file per operating system, all exposing the same
//! four functions; `OsMouse` plugs them into the engine's `Platform` trait.

use wiggler_core::platform::{Button, Platform, Point, Rect};

#[cfg(windows)]
#[path = "windows.rs"]
mod os;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod os;

#[cfg(not(any(windows, target_os = "macos")))]
#[path = "other.rs"]
mod os;

pub use os::{cursor, has_permission, request_permission};

pub struct OsMouse;

impl Platform for OsMouse {
    fn cursor(&mut self) -> Result<Point, String> {
        os::cursor()
    }
    fn move_to(&mut self, p: Point) -> Result<(), String> {
        os::move_to(p)
    }
    fn click(&mut self, p: Point, button: Button) -> Result<(), String> {
        os::click(p, button)
    }
    fn idle_ms(&mut self) -> Option<u64> {
        os::idle_ms()
    }
    fn bounds(&mut self) -> Option<Rect> {
        os::bounds()
    }
}

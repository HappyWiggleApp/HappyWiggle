//! Linux (X11) support, mainly so the app can be developed and tested there.
//! Wayland does not let ordinary apps move the pointer, so this is best-effort.

use enigo::{Coordinate, Direction, Enigo, Mouse, Settings};
use std::cell::RefCell;
use wiggler_core::platform::{Button, Point, Rect};

thread_local! {
    static ENIGO: RefCell<Option<Enigo>> = const { RefCell::new(None) };
}

fn with<T>(f: impl FnOnce(&mut Enigo) -> Result<T, String>) -> Result<T, String> {
    ENIGO.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            *slot = Some(
                Enigo::new(&Settings::default()).map_err(|e| format!("input-unavailable: {e}"))?,
            );
        }
        f(slot.as_mut().unwrap())
    })
}

pub fn has_permission() -> bool {
    true
}

pub fn request_permission() -> bool {
    true
}

pub fn cursor() -> Result<Point, String> {
    with(|e| {
        e.location()
            .map(|(x, y)| Point::new(x, y))
            .map_err(|e| e.to_string())
    })
}

pub fn move_to(p: Point) -> Result<(), String> {
    with(|e| {
        e.move_mouse(p.x, p.y, Coordinate::Abs)
            .map_err(|e| e.to_string())
    })
}

pub fn click(p: Point, button: Button) -> Result<(), String> {
    let b = match button {
        Button::Left => enigo::Button::Left,
        Button::Right => enigo::Button::Right,
    };
    with(|e| {
        e.move_mouse(p.x, p.y, Coordinate::Abs)
            .map_err(|e| e.to_string())?;
        e.button(b, Direction::Click).map_err(|e| e.to_string())
    })
}

pub fn idle_ms() -> Option<u64> {
    None
}

pub fn bounds() -> Option<Rect> {
    with(|e| e.main_display().map_err(|e| e.to_string()))
        .ok()
        .map(|(w, h)| Rect { x: 0, y: 0, w, h })
}

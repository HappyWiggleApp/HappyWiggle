//! macOS: Quartz events posted with `CGEvent.post`.
//!
//! We deliberately avoid the Accessibility API (AXUIElement / AXIsProcessTrusted):
//! it is unavailable to sandboxed apps. Posting events has its own privacy
//! permission ("post event access"), which the user grants once under
//! System Settings > Privacy & Security > Accessibility, and which Apple
//! documents as compatible with the App Sandbox.
//!
//! Coordinates are points, origin at the top-left of the main display.

use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;
use wiggler_core::platform::{Button, Point, Rect};

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightPostEventAccess() -> bool;
    fn CGRequestPostEventAccess() -> bool;
    fn CGEventSourceSecondsSinceLastEventType(state_id: i32, event_type: u32) -> f64;
}

/// `kCGEventSourceStateHIDSystemState`
const HID_STATE: i32 = 1;
/// `kCGAnyInputEventType`
const ANY_INPUT: u32 = u32::MAX;

pub fn has_permission() -> bool {
    unsafe { CGPreflightPostEventAccess() }
}

/// Shows the system prompt the first time; afterwards just reports the state.
pub fn request_permission() -> bool {
    unsafe { CGRequestPostEventAccess() }
}

fn source() -> Result<CGEventSource, String> {
    CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| "event-source".to_string())
}

pub fn cursor() -> Result<Point, String> {
    let e = CGEvent::new(source()?).map_err(|_| "cursor-unavailable".to_string())?;
    let p = e.location();
    Ok(Point::new(p.x.round() as i32, p.y.round() as i32))
}

fn post(kind: CGEventType, p: Point, button: CGMouseButton) -> Result<(), String> {
    let at = CGPoint::new(p.x as f64, p.y as f64);
    let e = CGEvent::new_mouse_event(source()?, kind, at, button)
        .map_err(|_| "event-create".to_string())?;
    e.post(CGEventTapLocation::HID);
    Ok(())
}

pub fn move_to(p: Point) -> Result<(), String> {
    if !has_permission() {
        return Err("permission".into());
    }
    post(CGEventType::MouseMoved, p, CGMouseButton::Left)
}

pub fn click(p: Point, button: Button) -> Result<(), String> {
    if !has_permission() {
        return Err("permission".into());
    }
    let (down, up, b) = match button {
        Button::Left => (
            CGEventType::LeftMouseDown,
            CGEventType::LeftMouseUp,
            CGMouseButton::Left,
        ),
        Button::Right => (
            CGEventType::RightMouseDown,
            CGEventType::RightMouseUp,
            CGMouseButton::Right,
        ),
    };
    post(CGEventType::MouseMoved, p, CGMouseButton::Left)?;
    post(down, p, b)?;
    std::thread::sleep(std::time::Duration::from_millis(35));
    post(up, p, b)
}

pub fn idle_ms() -> Option<u64> {
    let s = unsafe { CGEventSourceSecondsSinceLastEventType(HID_STATE, ANY_INPUT) };
    if s.is_finite() && s >= 0.0 {
        Some((s * 1000.0) as u64)
    } else {
        None
    }
}

/// Not reported: with several displays the desktop is not one rectangle.
/// The engine copes by noticing when the cursor is pinned against an edge.
pub fn bounds() -> Option<Rect> {
    None
}

//! Windows: `SendInput`, the same API a physical mouse driver ends up in.
//! Coordinates are physical pixels on the virtual desktop (all monitors).

use std::mem::size_of;
use wiggler_core::platform::{Button, Point, Rect};
use windows_sys::Win32::Foundation::POINT;
use windows_sys::Win32::System::SystemInformation::GetTickCount;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetLastInputInfo, SendInput, INPUT, INPUT_0, INPUT_MOUSE, LASTINPUTINFO, MOUSEEVENTF_ABSOLUTE,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK, MOUSEINPUT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_SWAPBUTTON,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

pub fn has_permission() -> bool {
    true
}

pub fn request_permission() -> bool {
    true
}

pub fn cursor() -> Result<Point, String> {
    let mut p = POINT { x: 0, y: 0 };
    // Fails on the lock screen / secure desktop, which is exactly when we
    // should not be doing anything.
    if unsafe { GetCursorPos(&mut p) } == 0 {
        return Err("cursor-unavailable".into());
    }
    Ok(Point::new(p.x, p.y))
}

fn virtual_screen() -> Rect {
    unsafe {
        Rect {
            x: GetSystemMetrics(SM_XVIRTUALSCREEN),
            y: GetSystemMetrics(SM_YVIRTUALSCREEN),
            w: GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1),
            h: GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1),
        }
    }
}

pub fn bounds() -> Option<Rect> {
    Some(virtual_screen())
}

fn send(flags: u32, dx: i32, dy: i32) -> Result<(), String> {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let sent = unsafe { SendInput(1, &input, size_of::<INPUT>() as i32) };
    if sent == 1 {
        Ok(())
    } else {
        // Typically: an administrator window is in front (Windows blocks
        // input from normal apps into elevated ones).
        Err("input-blocked".into())
    }
}

/// Absolute moves use a 0..65535 scale across the virtual desktop. Rounding
/// up makes Windows land on exactly the pixel we asked for.
fn normalise(v: i32, origin: i32, size: i32) -> i32 {
    let size = size.max(1) as i64;
    let off = (v as i64 - origin as i64).clamp(0, size - 1);
    ((off * 65_536 + size - 1) / size) as i32
}

pub fn move_to(p: Point) -> Result<(), String> {
    let vs = virtual_screen();
    send(
        MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
        normalise(p.x, vs.x, vs.w),
        normalise(p.y, vs.y, vs.h),
    )
}

pub fn click(p: Point, button: Button) -> Result<(), String> {
    // Position and button travel in the same event, so the click cannot land
    // somewhere else if the pointer is nudged in between.
    let vs = virtual_screen();
    let (x, y) = (normalise(p.x, vs.x, vs.w), normalise(p.y, vs.y, vs.h));
    let at = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
    // Respect "swap primary and secondary buttons" (left-handed setups).
    let swapped = unsafe { GetSystemMetrics(SM_SWAPBUTTON) } != 0;
    let physical_left = matches!(button, Button::Left) != swapped;
    let (down, up) = if physical_left {
        (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP)
    } else {
        (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP)
    };
    send(at | down, x, y)?;
    std::thread::sleep(std::time::Duration::from_millis(35));
    send(at | up, x, y)
}

pub fn idle_ms() -> Option<u64> {
    let mut info = LASTINPUTINFO {
        cbSize: size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    if unsafe { GetLastInputInfo(&mut info) } == 0 {
        return None;
    }
    let now = unsafe { GetTickCount() };
    Some(now.wrapping_sub(info.dwTime) as u64)
}

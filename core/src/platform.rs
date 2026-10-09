//! The only door between the engine and the operating system.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
    pub fn dist(self, o: Point) -> f64 {
        let dx = (self.x - o.x) as f64;
        let dy = (self.y - o.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }
}

/// A rectangle in the coordinate space the OS input API uses
/// (physical pixels on Windows, points on macOS).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn clamp(&self, p: Point) -> Point {
        let max_x = self.x + (self.w - 1).max(0);
        let max_y = self.y + (self.h - 1).max(0);
        Point::new(p.x.clamp(self.x, max_x), p.y.clamp(self.y, max_y))
    }
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x
            && p.y >= self.y
            && p.x < self.x + self.w.max(1)
            && p.y < self.y + self.h.max(1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Button {
    #[default]
    Left,
    Right,
}

pub trait Platform {
    /// Current cursor position.
    fn cursor(&mut self) -> Result<Point, String>;
    /// Move the cursor, as real input (so the OS idle timer resets).
    fn move_to(&mut self, p: Point) -> Result<(), String>;
    /// Press and release a button at `p`.
    fn click(&mut self, p: Point, button: Button) -> Result<(), String>;
    /// Milliseconds since the last input of any kind (keyboard included), if
    /// the OS can tell us. Used to notice the user typing.
    fn idle_ms(&mut self) -> Option<u64> {
        None
    }
    /// Bounds of the whole desktop, if known. Used to keep wiggles on screen.
    fn bounds(&mut self) -> Option<Rect> {
        None
    }
}

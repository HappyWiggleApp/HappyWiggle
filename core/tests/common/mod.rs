#![allow(dead_code)]
use wiggler_core::platform::{Button, Platform, Point, Rect};
use wiggler_core::Clock;

/// A pretend computer: remembers every move and click the engine asks for.
pub struct Fake {
    pub now: u64,
    pub cursor: Point,
    pub moves: Vec<Point>,
    pub clicks: Vec<(Point, Button, u64)>,
    pub last_input_ms: u64,
    pub report_idle: bool,
    /// What the OS reports as the desktop size (not every OS does).
    pub bounds: Option<Rect>,
    /// The physical screen edges: the cursor cannot leave this.
    pub walls: Option<Rect>,
    /// When set, reading the cursor fails with this error.
    pub fail: Option<&'static str>,
}

impl Fake {
    pub fn new() -> Self {
        Fake {
            now: 0,
            cursor: Point::new(500, 500),
            moves: vec![],
            clicks: vec![],
            last_input_ms: 0,
            report_idle: false,
            bounds: None,
            walls: None,
            fail: None,
        }
    }
    /// The human moves the mouse.
    pub fn user_move(&mut self, p: Point) {
        self.cursor = p;
        self.last_input_ms = self.now;
    }
    /// The human presses a key.
    pub fn user_key(&mut self) {
        self.last_input_ms = self.now;
    }
}

impl Platform for Fake {
    fn cursor(&mut self) -> Result<Point, String> {
        if let Some(e) = self.fail {
            return Err(e.into());
        }
        Ok(self.cursor)
    }
    fn move_to(&mut self, p: Point) -> Result<(), String> {
        let p = match self.walls {
            Some(b) => b.clamp(p),
            None => p,
        };
        self.cursor = p;
        self.moves.push(p);
        self.last_input_ms = self.now;
        Ok(())
    }
    fn click(&mut self, p: Point, b: Button) -> Result<(), String> {
        self.cursor = p;
        self.clicks.push((p, b, self.now));
        self.last_input_ms = self.now;
        Ok(())
    }
    fn idle_ms(&mut self) -> Option<u64> {
        if self.report_idle {
            Some(self.now - self.last_input_ms)
        } else {
            None
        }
    }
    fn bounds(&mut self) -> Option<Rect> {
        self.bounds
    }
}

pub const TICK: u64 = 20;

/// Monday 2026-10-05 is day 20731 since 1970-01-01.
pub const MONDAY: i64 = 20_731;

pub fn clock(mono_ms: u64, day: i64, sec_of_day: u32) -> Clock {
    Clock {
        mono_ms,
        day,
        weekday: ((day - MONDAY).rem_euclid(7)) as u8,
        sec_of_day,
    }
}

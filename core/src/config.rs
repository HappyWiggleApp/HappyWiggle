//! Everything the user can configure. Saved as JSON; unknown or missing
//! fields fall back to defaults so old settings files keep loading.

use crate::platform::{Button, Rect};
use crate::schedule::Schedule;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    /// Click inside chosen areas at random intervals.
    Click,
    /// Move the mouse, never click.
    #[default]
    Wiggle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum WiggleStyle {
    /// A short, curved, human-looking drift that comes back to where it started.
    #[default]
    Natural,
    /// One pixel out and back: enough for the OS, invisible to you.
    Subtle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum AreaOrder {
    #[default]
    Random,
    Sequence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum StopKind {
    #[default]
    Never,
    AfterActions,
    AfterMinutes,
    AtTime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StopRule {
    pub kind: StopKind,
    /// Number of clicks / wiggles for `AfterActions`.
    pub actions: u32,
    /// Duration for `AfterMinutes`.
    pub minutes: u32,
    /// Minute of the day (0..1440) for `AtTime`.
    pub at_minute: u32,
}

impl Default for StopRule {
    fn default() -> Self {
        StopRule {
            kind: StopKind::Never,
            actions: 100,
            minutes: 60,
            at_minute: 17 * 60,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub mode: Mode,

    // ---- click mode
    /// Target areas, in OS input coordinates. A 1x1 area is an exact spot.
    pub areas: Vec<Rect>,
    pub area_order: AreaOrder,
    pub button: Button,
    pub click_min_s: f64,
    pub click_max_s: f64,
    /// Glide to the target instead of jumping.
    pub smooth_move: bool,
    /// Put the cursor back where it was after each click.
    pub return_cursor: bool,

    // ---- wiggle mode
    pub wiggle_style: WiggleStyle,
    pub wiggle_min_s: f64,
    pub wiggle_max_s: f64,
    /// Furthest a natural wiggle travels, in pixels.
    pub wiggle_distance: u32,

    // ---- both
    pub stop: StopRule,
    pub pause_on_user: bool,
    /// How long the mouse must be left alone before we carry on.
    pub resume_after_s: u32,
    pub keep_awake: bool,

    // ---- app
    pub hotkey: String,
    pub compact_on_start: bool,
    /// "system" | "light" | "dark"
    pub theme: String,
    /// "auto" or a language code.
    pub language: String,
    pub autostart: bool,
    pub start_hidden: bool,
    pub onboarded: bool,

    pub schedule: Schedule,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            mode: Mode::Wiggle,
            areas: Vec::new(),
            area_order: AreaOrder::Random,
            button: Button::Left,
            click_min_s: 20.0,
            click_max_s: 60.0,
            smooth_move: true,
            return_cursor: true,
            wiggle_style: WiggleStyle::Natural,
            wiggle_min_s: 20.0,
            wiggle_max_s: 50.0,
            wiggle_distance: 60,
            stop: StopRule::default(),
            pause_on_user: true,
            resume_after_s: 5,
            keep_awake: true,
            hotkey: "F8".into(),
            compact_on_start: false,
            theme: "system".into(),
            language: "auto".into(),
            autostart: false,
            start_hidden: false,
            onboarded: false,
            schedule: Schedule::default(),
        }
    }
}

/// Hard floor for intervals, so a typo cannot turn the app into a click storm.
pub const MIN_INTERVAL_S: f64 = 1.0;
pub const MAX_INTERVAL_S: f64 = 3600.0;
pub const MAX_AREAS: usize = 5;

impl Settings {
    /// Bring every value into a safe range. Called on load and on every save.
    pub fn sanitize(&mut self) {
        fn fix(min: &mut f64, max: &mut f64) {
            if !min.is_finite() {
                *min = MIN_INTERVAL_S;
            }
            if !max.is_finite() {
                *max = *min;
            }
            *min = min.clamp(MIN_INTERVAL_S, MAX_INTERVAL_S);
            *max = max.clamp(MIN_INTERVAL_S, MAX_INTERVAL_S);
            if *max < *min {
                std::mem::swap(min, max);
            }
        }
        fix(&mut self.click_min_s, &mut self.click_max_s);
        fix(&mut self.wiggle_min_s, &mut self.wiggle_max_s);
        self.wiggle_distance = self.wiggle_distance.clamp(4, 400);
        self.resume_after_s = self.resume_after_s.clamp(1, 600);
        self.stop.actions = self.stop.actions.clamp(1, 1_000_000);
        self.stop.minutes = self.stop.minutes.clamp(1, 24 * 60);
        self.stop.at_minute %= 1440;
        self.areas.truncate(MAX_AREAS);
        for a in &mut self.areas {
            a.w = a.w.max(1);
            a.h = a.h.max(1);
        }
        if self.hotkey.trim().is_empty() {
            self.hotkey = "F8".into();
        }
        self.schedule.sanitize();
    }

    pub fn interval_range_s(&self) -> (f64, f64) {
        match self.mode {
            Mode::Click => (self.click_min_s, self.click_max_s),
            Mode::Wiggle => (self.wiggle_min_s, self.wiggle_max_s),
        }
    }
}

//! Weekly timetable: "weekdays 9 to 5, lunch 1 to 2", with start and stop
//! times that drift a little every day so it is not a robot clocking in at
//! 9:00:00 sharp.

use crate::config::Mode;
use crate::rng::{hash64, Rng};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum JitterKind {
    /// Exactly at the chosen time.
    #[default]
    Exact,
    /// Somewhere in the minutes before.
    Before,
    /// Somewhere in the minutes after.
    After,
    /// Either side.
    Either,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Jitter {
    pub kind: JitterKind,
    pub minutes: u32,
}

impl Jitter {
    pub fn before(minutes: u32) -> Self {
        Jitter {
            kind: JitterKind::Before,
            minutes,
        }
    }
    pub fn after(minutes: u32) -> Self {
        Jitter {
            kind: JitterKind::After,
            minutes,
        }
    }
    /// Earliest and latest second of the day this boundary can land on.
    pub fn window_s(&self, minute: u32) -> (i64, i64) {
        let base = minute as i64 * 60;
        let j = self.minutes as i64 * 60;
        match self.kind {
            JitterKind::Exact => (base, base),
            JitterKind::Before => (base - j, base),
            JitterKind::After => (base, base + j),
            JitterKind::Either => (base - j, base + j),
        }
    }
    fn resolve(&self, minute: u32, rng: &mut Rng) -> i64 {
        let (lo, hi) = self.window_s(minute);
        rng.range_i64(lo, hi)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Break {
    pub start_min: u32,
    pub end_min: u32,
    pub start_jitter: Jitter,
    pub end_jitter: Jitter,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Block {
    /// Monday first.
    pub days: [bool; 7],
    pub start_min: u32,
    pub end_min: u32,
    pub start_jitter: Jitter,
    pub end_jitter: Jitter,
    #[serde(rename = "break")]
    pub pause: Option<Break>,
    /// Mode to run in; `None` means "whatever is selected on the Run screen".
    pub mode: Option<Mode>,
}

impl Default for Block {
    /// The "Weekdays, 9 to 5" preset.
    fn default() -> Self {
        Block {
            days: [true, true, true, true, true, false, false],
            start_min: 9 * 60,
            end_min: 17 * 60,
            start_jitter: Jitter::before(10),
            end_jitter: Jitter::after(10),
            pause: Some(Break {
                start_min: 13 * 60,
                end_min: 14 * 60,
                start_jitter: Jitter {
                    kind: JitterKind::Either,
                    minutes: 5,
                },
                end_jitter: Jitter {
                    kind: JitterKind::Either,
                    minutes: 5,
                },
            }),
            mode: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Schedule {
    pub enabled: bool,
    pub blocks: Vec<Block>,
    /// Local day number (days since 1970-01-01) to skip, set by "Skip today".
    pub skip_day: Option<i64>,
}

/// One stretch of the day during which the app should be running.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    /// Seconds since local midnight, already jittered.
    pub start_s: u32,
    pub end_s: u32,
    /// Outer limits the jitter could have reached; drawn as the pale outline.
    pub earliest_s: u32,
    pub latest_s: u32,
    pub mode: Option<Mode>,
}

const DAY_S: i64 = 86_400;
pub const MAX_BLOCKS: usize = 6;

fn clamp_day(s: i64) -> u32 {
    s.clamp(0, DAY_S) as u32
}

impl Schedule {
    pub fn sanitize(&mut self) {
        self.blocks.truncate(MAX_BLOCKS);
        for b in &mut self.blocks {
            b.start_min = b.start_min.min(1439);
            b.end_min = b.end_min.min(1440);
            if b.end_min <= b.start_min {
                b.end_min = (b.start_min + 1).min(1440);
                b.start_min = b.end_min - 1;
            }
            b.start_jitter.minutes = b.start_jitter.minutes.min(60);
            b.end_jitter.minutes = b.end_jitter.minutes.min(60);
            if let Some(p) = &mut b.pause {
                p.start_jitter.minutes = p.start_jitter.minutes.min(30);
                p.end_jitter.minutes = p.end_jitter.minutes.min(30);
                // a break has to sit strictly inside its block
                if p.end_min <= p.start_min || p.start_min <= b.start_min || p.end_min >= b.end_min
                {
                    b.pause = None;
                }
            }
        }
    }

    /// The concrete plan for one day. `weekday` is 0 = Monday.
    ///
    /// The same `(seed, day)` always gives the same plan, so restarting the
    /// app in the middle of the day does not reshuffle the times.
    pub fn resolve_day(&self, day: i64, weekday: u8, seed: u64) -> Vec<Segment> {
        let mut out = Vec::new();
        for (i, b) in self.blocks.iter().enumerate() {
            if !b.days[(weekday % 7) as usize] {
                continue;
            }
            let mut rng = Rng::new(hash64(&[seed, day as u64, i as u64]));
            let start = b.start_jitter.resolve(b.start_min, &mut rng);
            let end = b.end_jitter.resolve(b.end_min, &mut rng);
            let (earliest, _) = b.start_jitter.window_s(b.start_min);
            let (_, latest) = b.end_jitter.window_s(b.end_min);
            let mut pieces: Vec<(i64, i64, i64, i64)> = Vec::new();
            match &b.pause {
                Some(p) => {
                    let ps = p.start_jitter.resolve(p.start_min, &mut rng);
                    let pe = p.end_jitter.resolve(p.end_min, &mut rng).max(ps + 60);
                    let (_, ps_latest) = p.start_jitter.window_s(p.start_min);
                    let (pe_earliest, _) = p.end_jitter.window_s(p.end_min);
                    pieces.push((start, ps, earliest, ps_latest));
                    pieces.push((pe, end, pe_earliest, latest));
                }
                None => pieces.push((start, end, earliest, latest)),
            }
            for (s, e, lo, hi) in pieces {
                let (s, e) = (clamp_day(s), clamp_day(e));
                if e > s {
                    out.push(Segment {
                        start_s: s,
                        end_s: e,
                        earliest_s: clamp_day(lo).min(s),
                        latest_s: clamp_day(hi).max(e),
                        mode: b.mode,
                    });
                }
            }
        }
        out.sort_by_key(|s| s.start_s);
        // If two blocks overlap, merge them so we never start "twice".
        let mut merged: Vec<Segment> = Vec::new();
        for s in out {
            match merged.last_mut() {
                Some(last) if s.start_s <= last.end_s => {
                    last.end_s = last.end_s.max(s.end_s);
                    last.latest_s = last.latest_s.max(s.latest_s);
                }
                _ => merged.push(s),
            }
        }
        merged
    }
}

/// The segment that contains `sec_of_day`, if any.
pub fn active_segment(plan: &[Segment], sec_of_day: u32) -> Option<&Segment> {
    plan.iter()
        .find(|s| s.start_s <= sec_of_day && sec_of_day < s.end_s)
}

/// The next segment that has not started yet.
pub fn next_segment(plan: &[Segment], sec_of_day: u32) -> Option<&Segment> {
    plan.iter().find(|s| s.start_s > sec_of_day)
}

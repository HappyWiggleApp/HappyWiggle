//! What actually happened, so the Scheduler tab can show "planned vs. real".

use crate::config::Mode;
use crate::engine::StopReason;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecord {
    /// Local day number the session started on.
    pub day: i64,
    pub start_s: u32,
    pub end_s: u32,
    pub mode: Mode,
    pub actions: u64,
    pub by_schedule: bool,
    pub reason: StopReason,
    /// Seconds spent waiting because the user was using the mouse.
    pub user_pause_s: u32,
    /// Seconds lost to the computer sleeping or the app being frozen.
    pub sleep_s: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct History {
    pub sessions: Vec<SessionRecord>,
}

/// Keep roughly a month.
pub const KEEP_DAYS: i64 = 35;
const MAX_RECORDS: usize = 2000;

impl History {
    pub fn push(&mut self, rec: SessionRecord) {
        let cutoff = rec.day - KEEP_DAYS;
        self.sessions.push(rec);
        self.sessions.retain(|s| s.day >= cutoff);
        if self.sessions.len() > MAX_RECORDS {
            let drop = self.sessions.len() - MAX_RECORDS;
            self.sessions.drain(0..drop);
        }
    }
}

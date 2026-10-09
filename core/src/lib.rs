//! Platform-independent engine for HappyWiggle.
//!
//! Nothing in this crate touches the operating system. The app hands it a
//! [`platform::Platform`] (the real mouse, or a fake one in tests) and a
//! [`supervisor::Clock`], and calls `tick` a few times per second. That makes
//! every behaviour here testable without a screen.

pub mod config;
pub mod engine;
pub mod history;
pub mod path;
pub mod platform;
pub mod rng;
pub mod schedule;
pub mod supervisor;

pub use config::*;
pub use engine::{Engine, Event, Origin, Phase, StopReason};
pub use platform::{Platform, Point, Rect};
pub use supervisor::{Clock, Status, Supervisor};

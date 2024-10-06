//! # Metric Time
//!
//! A Rust library for working with metric (decimal) time format.
//!
//! This crate provides functionality for:
//!
//! - Converting between standard 12-hour time, 24-hour time and metric time
//! - Emitting the current time in standard 12-hour time, 24-hour or metric time
//! - Building and configuring clocks with custom settings
//! - Working with different time periods, ranges and rotations
//! - Error handling for time operations and conversions
//!
//! ## Usage
//!
//! The library exposes several key types:
//!
//! - `Clock` - Core clock functionality
//! - `Time` - Time representation and manipulation
//! - `TimeEmitter` - Time event emission
//! - `Builder` - Clock configuration
//! - `Period`, `TimeBounds`, `TimeComponents` - Time utilities
//!
//! The library handles all conversions and formatting internally while providing
//! a clean API for working with metric time measurements and displays.

// Public exports -------------------------------------------------------------------------------- /

pub use clock::Clock;
pub use clock_lib::{ClockError, ClockSettings};
pub use time::Time;
pub use time_emitter::{Context, MessageType, Settings, Subscription, TimeEmitter};
pub use time_lib::{
    Period, TimeBounds, TimeComponents, TimeConversionTrait, TimeKind, TimeRangeError,
    TimeRotationComponents,
};
pub use util::builder::Builder;

// Modules --------------------------------------------------------------------------------------- /

mod time_helpers {
    pub mod conversion_utils;
    pub mod time_conversions;
}
mod util {
    pub mod builder;
}
mod clock;
mod clock_lib;
mod constants;
mod time;
mod time_emitter;
mod time_lib;

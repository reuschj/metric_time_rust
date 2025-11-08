//! # 🕰️ Metric Time
//!
//! A Rust library for working with metric (decimal) time format.
//!
//! This crate provides functionality for:
//!
//! - 🔄 Converting between standard 12-hour time, 24-hour time and metric time
//! - ⏱️ Emitting the current time in standard 12-hour time, 24-hour or metric time
//! - ⚙️ Building and configuring clocks with custom settings
//! - 📊 Working with different time periods, ranges and rotations
//! - ❌ Error handling for time operations and conversions
//! - 🌐 WebAssembly (WASM) support for using the library in web applications
//!
//! ## 📚 Usage
//!
//! The library exposes several key types:
//!
//! - 🕒 `Clock` - The core clock functionality, responsible for keeping and managing time
//! - ⏰ `Time` - Represents a specific point in time, with methods for manipulation
//! - 🔄 `Converter` and `TimeConversions` - Utilities for converting `Time` into various formats (metric, 12-hour, 24-hour)
//! - 📡 `Emitter` - Handles time event emissions. Its implementation is conditional:
//!   - On native targets, it uses `ThreadEmitter` for multi-threaded environments
//!   - With the `web` feature, it uses `WebEmitter` for WebAssembly (WASM) applications
//! - 📋 `Period`, `TimeBounds`, `TimeComponents` - Various utilities for handling time-related data structures
//!
//! The library handles all conversions and formatting internally while providing
//! a clean API for working with metric time measurements and displays.

// 📤 Public exports -------------------------------------------------------------------------------- /

// Clock
pub use clock::clock::Clock;
pub use clock::lib::{ClockError, ClockSettings};

// Emitters
pub use emitters::lib::{EmitterContext, EmitterSettingsTrait};
#[cfg(not(feature = "web"))]
pub use emitters::lib::{ThreadStartable, ThreadStoppable};
#[cfg(feature = "web")]
pub use emitters::lib::{WebStartable, WebStoppable};
#[cfg(not(feature = "web"))]
pub use emitters::thread_emitter::{
    Settings as EmitterSettings, Subscription as EmitterSubscription, ThreadEmitter as Emitter,
};
#[cfg(feature = "web")]
pub use emitters::web_emitter::{Settings as EmitterSettings, WebEmitter as Emitter};

// Time
pub use time::lib::{
    Period, TimeBounds, TimeComponent, TimeComponentWithValue, TimeComponents, TimeConversionTrait,
    TimeKind, TimeRangeError, TimeRotationComponents,
};
pub use time::time::Time;
pub use time::time_conversions::{Converter, TimeConversions};

// 📦 Modules --------------------------------------------------------------------------------------- /

mod clock {
    pub mod clock;
    pub mod lib;
}
mod emitters {
    pub mod lib;
    #[cfg(not(feature = "web"))]
    pub mod thread_emitter;
    #[cfg(feature = "web")]
    pub mod web_emitter;
}
mod time {
    pub mod conversion_utils;
    pub mod lib;
    pub mod time;
    pub mod time_conversions;
}
mod constants;

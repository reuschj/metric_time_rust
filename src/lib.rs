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
//! - 🕒 `Clock` - Core clock functionality
//! - ⏰ `Time` - Time representation and manipulation
//! - 📡 `Emitter` - Time event emission (not for web applications)
//! - 📡 `WebEmitter` - Time event emission (for web applications)
//! - 📋 `Period`, `TimeBounds`, `TimeComponents` - Time utilities
//!
//! The library handles all conversions and formatting internally while providing
//! a clean API for working with metric time measurements and displays.

// 📤 Public exports -------------------------------------------------------------------------------- /

// Clock
pub use clock::clock::Clock;
pub use clock::lib::{ClockError, ClockSettings};

// Emitters
pub use emitters::lib::{EmitterContext, EmitterSettingsTrait, Startable, Stoppable, WebStartable};
#[cfg(not(feature = "web"))]
pub use emitters::std_emitter::{
    Emitter, Settings as EmitterSettings, Subscription as EmitterSubscription,
};
#[cfg(feature = "web")]
pub use emitters::web_emitter::{Settings as WebEmitterSettings, WebEmitter};

// Time
pub use time::lib::{
    Period, TimeBounds, TimeComponents, TimeConversionTrait, TimeKind, TimeRangeError,
    TimeRotationComponents,
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
    #[cfg(feature = "standard")]
    pub mod std_emitter;
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

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
//! - 📡 `TimeEmitter` - Time event emission
//! - 🛠️ `Builder` - Clock configuration
//! - 📋 `Period`, `TimeBounds`, `TimeComponents` - Time utilities
//!
//! The library handles all conversions and formatting internally while providing
//! a clean API for working with metric time measurements and displays.
//!
//! ## 🌐 WebAssembly Support
//!
//! When compiled with the `wasm32` target, this library exposes WASM bindings
//! for using metric time functionality in web applications.

// 📤 Public exports -------------------------------------------------------------------------------- /

pub use clock::Clock;
pub use clock_lib::{ClockError, ClockSettings};
pub use time::Time;
pub use time_emitter_lib::{Context, TimeEmitterTrait};
#[cfg(not(target_arch = "wasm32"))]
pub use time_emitter_standard::{Settings, Subscription, TimeEmitter};
#[cfg(target_arch = "wasm32")]
pub use time_emitter_wasm::{Settings, Subscription, TimeEmitter};
pub use time_helpers::time_conversions::{Converter, TimeConversions};
pub use time_lib::{
    Period, TimeBounds, TimeComponents, TimeConversionTrait, TimeKind, TimeRangeError,
    TimeRotationComponents,
};
pub use util::builder::Builder;

// 📦 Modules --------------------------------------------------------------------------------------- /

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
mod time_emitter_lib;
#[cfg(not(target_arch = "wasm32"))]
mod time_emitter_standard;
#[cfg(target_arch = "wasm32")]
mod time_emitter_wasm;
mod time_lib;

//! 📡 This module provides traits and common types for time emitters.
//!
//! Time emitters are responsible for periodically emitting time events at specified intervals.
//! This module defines the common behavior for both native and WebAssembly implementations.
//!
//! # 📋 Overview
//!
//! The time emitter system provides a way to generate regular time events that can be
//! used for various purposes such as scheduling, animation, or any periodic task.
//!
//! # 🧩 Key Components
//!
//! - [`EmitterSettingsTrait`] - ⚙️ A trait for configuring time emitters
//! - [`Emittable`] - 🧩 Core functionality that all time emitters must implement
//! - [`Context`] - 📦 Context information provided with each time event
//!
//! # 🌐 Cross-platform Support
//!
//! The traits in this module are designed to abstract over platform-specific implementation details,
//! allowing for both native (standard) and WebAssembly implementations to share a common interface.

use std::fmt::Debug;

use crate::{Time, TimeKind};

/// ⚙️ Time emitter settings trait ---------------------------------------------------------------- /

/// ⚙️ A trait for time emitter settings that abstracts over the duration type.
///
/// This trait provides a common interface for configuring time emitters across
/// different platforms. It allows for different duration types to be used,
/// depending on the platform (e.g., `std::time::Duration` for native and
/// `web_time::Duration` for WebAssembly).
pub trait EmitterSettingsTrait: Debug + Clone + Copy + Default {
    /// ⏱️ The duration type used by this settings implementation.
    type Duration: Debug + Copy + Clone;

    /// 🆕 Creates a new instance of the settings with default values.
    ///
    /// This is a convenience method that calls `Self::default()`.
    fn new() -> Self {
        Self::default()
    }

    // 📥 Getters --------- /

    /// 🔢 Get the maximum number of events to emit.
    ///
    /// Returns `None` if there is no maximum (emitter will run indefinitely).
    fn max_events(&self) -> Option<u64>;

    /// ⏱️ Get the interval between events.
    ///
    /// This defines how much time passes between each emitted event.
    fn interval(&self) -> Self::Duration;

    /// 🔢 Get the kind of time to emit.
    ///
    /// This determines the time format (e.g., Base10, Base24) for the emitted times.
    fn kind(&self) -> TimeKind;

    // 📝 Setters --------- /

    /// 🔢 Sets the maximum number of events.
    ///
    /// After this many events have been emitted, the emitter will stop automatically.
    fn set_max_events(self, max_events: u64) -> Self;

    /// 🧹 Clears the maximum number of events.
    ///
    /// This removes any limit on the number of events, causing the emitter to run indefinitely
    /// until manually stopped.
    fn clear_max_events(self) -> Self;

    /// ⏱️ Sets the interval between events.
    ///
    /// This defines the duration between successive time events.
    fn set_interval(self, interval: Self::Duration) -> Self;

    /// 🔢 Sets the time kind for emitted times.
    ///
    /// This determines the format of the time values that will be emitted.
    fn set_kind(self, kind: TimeKind) -> Self;
}

// 📦 Context ----------------------------------------------------------------------------------- /

/// 📦 Context information provided with each time event.
///
/// This structure contains metadata about the emitted time event,
/// including its sequential index and the settings used for the emitter.
#[derive(Debug, Clone, Copy)]
pub struct EmitterContext<S: EmitterSettingsTrait> {
    /// 🔢 The sequential index of the event (0-based).
    pub index: u64,
    /// ⚙️ The settings used for the emitter.
    pub settings: S,
}

// 🏁 Startable trait ------------------------------------------------------------------------- /

pub trait Startable: Debug + Clone {
    type Settings: EmitterSettingsTrait;
    type ErrorType: std::error::Error;

    fn start<F>(settings: Self::Settings, on_emit: F) -> Result<Self, Self::ErrorType>
    where
        Self: Sized,
        F: FnMut(Time, EmitterContext<Self::Settings>) -> () + Send + Sync + 'static;

    fn settings(&self) -> &Self::Settings;
}

pub trait WebStartable: Debug + Clone {
    type Settings: EmitterSettingsTrait;
    type ErrorType: std::error::Error;

    fn start<F>(settings: Self::Settings, on_emit: F) -> Result<Self, Self::ErrorType>
    where
        Self: Sized,
        F: FnMut(Time, EmitterContext<Self::Settings>) -> () + 'static;

    fn settings(&self) -> &Self::Settings;
}

/// 🛑 Stoppable trait --------------------------------------------------------------------------- /

pub trait Stoppable: Debug + Clone {
    type OnStopValue;
    type ErrorType: std::error::Error;

    fn stop(&mut self) -> Result<Self::OnStopValue, Self::ErrorType>;
    fn await_completion(&mut self) -> Result<Self::OnStopValue, Self::ErrorType>;
}

// 🧪 Tests -------------------------------------------------------------------------------- /

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // Native duration settings for testing
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TestSettings {
        max_events: Option<u64>,
        interval: Duration,
        kind: TimeKind,
    }

    impl EmitterSettingsTrait for TestSettings {
        type Duration = Duration;

        fn max_events(&self) -> Option<u64> {
            self.max_events
        }

        fn interval(&self) -> Self::Duration {
            self.interval
        }

        fn kind(&self) -> TimeKind {
            self.kind
        }

        fn set_max_events(mut self, max_events: u64) -> Self {
            self.max_events = Some(max_events);
            self
        }

        fn clear_max_events(mut self) -> Self {
            self.max_events = None;
            self
        }

        fn set_interval(mut self, interval: Self::Duration) -> Self {
            self.interval = interval;
            self
        }

        fn set_kind(mut self, kind: TimeKind) -> Self {
            self.kind = kind;
            self
        }
    }

    impl Default for TestSettings {
        fn default() -> Self {
            Self {
                max_events: None,
                interval: Duration::from_secs(1),
                kind: TimeKind::Base24,
            }
        }
    }

    #[test]
    fn test_settings_modifications() {
        let settings = TestSettings::new()
            .set_max_events(10)
            .set_interval(Duration::from_millis(500))
            .set_kind(TimeKind::Base10);

        assert_eq!(settings.max_events(), Some(10));
        assert_eq!(settings.interval(), Duration::from_millis(500));
        assert_eq!(settings.kind(), TimeKind::Base10);

        let cleared_settings = settings.clear_max_events();
        assert_eq!(cleared_settings.max_events(), None);
    }

    #[test]
    fn test_settings_defaults() {
        let settings = TestSettings::new();

        assert_eq!(settings.max_events(), None);
        assert_eq!(settings.interval(), Duration::from_secs(1));
        assert_eq!(settings.kind(), TimeKind::Base24);
    }
}

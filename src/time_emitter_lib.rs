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
//! - [`TimeEmitterSettingsTrait`] - ⚙️ A trait for configuring time emitters
//! - [`TimeEmitterTrait`] - 🧩 Core functionality that all time emitters must implement
//! - [`Context`] - 📦 Context information provided with each time event
//!
//! # 🌐 Cross-platform Support
//!
//! The traits in this module are designed to abstract over platform-specific implementation details,
//! allowing for both native (standard) and WebAssembly implementations to share a common interface.

use std::fmt::Debug;

use crate::time::Time;
use crate::time_lib::TimeKind;

/// ⚙️ Time emitter settings trait ---------------------------------------------------------------- /

/// ⚙️ A trait for time emitter settings that abstracts over the duration type.
///
/// This trait provides a common interface for configuring time emitters across
/// different platforms. It allows for different duration types to be used,
/// depending on the platform (e.g., `std::time::Duration` for native and
/// `web_time::Duration` for WebAssembly).
pub trait TimeEmitterSettingsTrait: Debug + Clone + Copy + Default {
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
pub struct Context<S: TimeEmitterSettingsTrait> {
    /// 🔢 The sequential index of the event (0-based).
    pub index: u64,
    /// ⚙️ The settings used for the emitter.
    pub settings: S,
}

// 📡 Time emitter trait ------------------------------------------------------------------------- /

/// 📡 A trait that defines the core functionality of a time emitter.
///
/// This trait provides a common interface for time emitters across different platforms.
/// Implementations of this trait are responsible for periodically emitting time events
/// based on the provided settings and invoking the callback function for each event.
pub trait TimeEmitterTrait: Debug + Clone + Send + Sync {
    /// ⚙️ The settings type used by this emitter.
    type Settings: TimeEmitterSettingsTrait;

    /// ❌ The error type returned by this emitter's operations.
    type Error: std::error::Error;

    /// 🚀 Starts a new time emitter with the given settings and callback.
    ///
    /// The callback will be called for each time event with the current time and context.
    ///
    /// # 📥 Parameters
    ///
    /// * `settings` - ⚙️ Configuration for the time emitter, including interval and max events
    /// * `on_emit` - 🔔 Callback function that will be invoked for each time event
    ///
    /// # 📤 Returns
    ///
    /// A new instance of the time emitter
    fn start<F>(settings: Self::Settings, on_emit: F) -> Self
    where
        F: 'static + Fn(Time, Context<Self::Settings>) -> () + Clone + Send + Sync;

    /// ⚙️ Gets a reference to the settings of the time emitter.
    fn settings(&self) -> &Self::Settings;

    /// 🔢 Gets the time kind of the emitter.
    ///
    /// This is a convenience method that retrieves the time kind from the emitter's settings.
    fn kind(&self) -> TimeKind {
        self.settings().kind()
    }

    /// 🛑 Stops the time emitter.
    ///
    /// This method halts the emission of time events. The exact behavior depends on the
    /// implementation (e.g., stopping a thread, clearing an interval).
    fn stop(&self) -> Result<(), Self::Error>;

    /// ⏳ Waits for the time emitter to complete.
    ///
    /// This is a blocking call in standard environments (using thread::join),
    /// and a no-op in WASM environments (where we don't have threads).
    ///
    /// In environments with async support, this method should be called
    /// after `stop()` to ensure proper cleanup of resources.
    ///
    /// # 📤 Returns
    ///
    /// A Result indicating success or failure of the waiting operation.
    /// The default implementation simply returns Ok(()).
    fn await_completion(&mut self) -> Result<(), Self::Error> {
        // Default implementation does nothing
        Ok(())
    }
}

// 📝 Type aliases ----------------------------------------------------------------------------- /

/// 🔔 A convenience type alias for callbacks used by time emitters.
///
/// This type represents a function that takes a Time and Context and produces no return value.
/// The function must be thread-safe (Send + Sync) to support both standard and WASM environments.
pub type TimeEmitterCallback<S> = dyn Fn(Time, Context<S>) -> () + Send + Sync;

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

    impl TimeEmitterSettingsTrait for TestSettings {
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

//! 🕰️ A clock implementation that emits time events at regular intervals.
//!
//! This module provides the [`Clock`] struct, which uses a time emitter to
//! periodically update and emit time events. It supports different time formats
//! and customizable intervals.
//!
//! # 📝 Examples
//!
//! ```rust
//! use metric_time::{Clock, TimeKind};
//! use std::time::Duration;
//!
//! // Create a clock with custom settings
//! let clock = Clock::new()
//!     .set_kind(TimeKind::Base10)
//!     .set_interval(Duration::from_millis(100));
//!
//! // Start the clock with a callback
//! clock.start(|time, _ctx| {
//!     println!("Current time: {}", time);
//! }).expect("Failed to start clock");
//!
//! // The clock will continue emitting time events until stopped
//! ```

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    ClockError, ClockSettings, EmitterContext, EmitterSettingsTrait, Stoppable, Time, TimeKind,
};

// Use the appropriate time emitter based on environment

#[cfg(not(feature = "web"))]
use super::super::emitters::lib::Startable;
#[cfg(not(feature = "web"))]
use super::super::emitters::std_emitter::{Emitter, Settings as EmitterSettings};

#[cfg(feature = "web")]
use super::super::emitters::lib::WebStartable;
#[cfg(feature = "web")]
use super::super::emitters::web_emitter::{Settings as EmitterSettings, WebEmitter as Emitter};

// 🕰️ Clock --------------------------------------------------------------------------- /

/// 🕰️ A clock that emits time events at regular intervals.
///
/// The `Clock` struct provides a high-level interface for working with time events.
/// It wraps a time emitter to periodically update and emit time events based on
/// the configured settings.
///
/// # 🔒 Thread Safety
///
/// The `Clock` is designed to be thread-safe, using `Arc<Mutex<_>>` to safely
/// share state between threads. This allows the clock to be used in concurrent
/// contexts, such as between a UI thread and a background worker.
///
/// # ✨ Features
///
/// - 🔢 Configurable time format (Base10, Base12, Base24)
/// - ⏱️ Adjustable interval between time updates
/// - 🧵 Thread-safe access to the current time
/// - 🔢 Event counting
#[derive(Debug, Clone)]
pub struct Clock {
    /// 📡 Reference to the time emitter
    emitter_ref: Arc<Mutex<Option<Emitter>>>,
    /// ⏰ The most recently emitted time
    time_ref: Arc<Mutex<Option<Time>>>,
    /// 🔢 Counter for tracking the number of time events
    counter_ref: Arc<Mutex<u128>>,
    /// ⚙️ Configuration settings for the clock
    settings: ClockSettings,
}

impl Clock {
    /// 🆕 Creates a new `Clock` with default settings.
    ///
    /// The default settings use a Base24 time format with a 1-second interval.
    ///
    /// # 🔄 Returns
    ///
    /// A new `Clock` instance with default settings.
    ///
    /// # 📝 Examples
    ///
    /// ```
    /// use metric_time::Clock;
    ///
    /// let clock = Clock::new();
    /// ```
    pub fn new() -> Self {
        let emitter_ref = Arc::new(Mutex::new(None));
        let time_ref = Arc::new(Mutex::new(None));
        let counter_ref = Arc::new(Mutex::new(0 as u128));
        let settings = ClockSettings::defaults();
        Self {
            emitter_ref,
            time_ref,
            counter_ref,
            settings,
        }
    }

    // ⚙️ Configuration ------------------------------------------------------------ /

    /// ⚙️ Configures the clock with the provided settings.
    ///
    /// This method allows setting all clock settings at once using a `ClockSettings` struct.
    ///
    /// # 📥 Parameters
    ///
    /// * `settings` - The configuration settings to use
    ///
    /// # 🔄 Returns
    ///
    /// The updated `Clock` instance with the new settings
    ///
    /// # 📝 Examples
    ///
    /// ```
    /// use metric_time::{Clock, ClockSettings, TimeKind};
    /// use std::time::Duration;
    ///
    /// let settings = ClockSettings {
    ///     kind: TimeKind::Base10,
    ///     interval: Duration::from_millis(500),
    /// };
    ///
    /// let clock = Clock::new().setup(settings);
    /// ```
    pub fn setup(mut self, settings: ClockSettings) -> Self {
        self.settings = settings;
        self
    }

    /// 🔢 Sets the time kind (format) for the clock.
    ///
    /// This method configures the format of time that will be emitted
    /// by the clock (e.g., Base10, Base12, Base24).
    ///
    /// # 📥 Parameters
    ///
    /// * `kind` - The time kind to use
    ///
    /// # 🔄 Returns
    ///
    /// The updated `Clock` instance with the new time kind
    ///
    /// # 📝 Examples
    ///
    /// ```
    /// use metric_time::{Clock, TimeKind};
    ///
    /// let clock = Clock::new().set_kind(TimeKind::Base10);
    /// ```
    pub fn set_kind(mut self, kind: TimeKind) -> Self {
        self.settings = self.settings.set_kind(kind);
        self
    }

    /// ⏱️ Sets the interval between time updates.
    ///
    /// This method configures how frequently the clock will emit time events.
    ///
    /// # 📥 Parameters
    ///
    /// * `interval` - The duration between time updates
    ///
    /// # 🔄 Returns
    ///
    /// The updated `Clock` instance with the new interval
    ///
    /// # 📝 Examples
    ///
    /// ```
    /// use metric_time::Clock;
    /// use std::time::Duration;
    ///
    /// let clock = Clock::new().set_interval(Duration::from_millis(100));
    /// ```
    pub fn set_interval(mut self, interval: Duration) -> Self {
        self.settings = self.settings.set_interval(interval);
        self
    }

    // 📊 State Access ------------------------------------------------------------ /

    /// ⏰ Gets the most recently emitted time.
    ///
    /// # 🔄 Returns
    ///
    /// The current time, or `None` if the clock hasn't emitted any time events yet
    /// or if the mutex couldn't be locked.
    ///
    /// # 📝 Examples
    ///
    /// ```
    /// use metric_time::Clock;
    ///
    /// let clock = Clock::new();
    /// let current_time = clock.time(); // Will be None initially
    /// ```
    pub fn time(&self) -> Option<Time> {
        match self.time_ref.lock() {
            Ok(time) => *time,
            Err(_) => None,
        }
    }

    /// 🔢 Gets the time kind (format) configured for this clock.
    ///
    /// # 🔄 Returns
    ///
    /// The time kind currently set for this clock
    ///
    /// # 📝 Examples
    ///
    /// ```
    /// use metric_time::{Clock, TimeKind};
    ///
    /// let clock = Clock::new();
    /// assert_eq!(clock.kind(), TimeKind::Base24); // Default
    /// ```
    pub fn kind(&self) -> TimeKind {
        self.settings.kind
    }

    /// ⏱️ Gets the interval between time updates.
    ///
    /// # 🔄 Returns
    ///
    /// The duration between time events
    ///
    /// # 📝 Examples
    ///
    /// ```
    /// use metric_time::Clock;
    /// use std::time::Duration;
    ///
    /// let clock = Clock::new();
    /// assert_eq!(clock.interval(), Duration::from_secs(1)); // Default
    /// ```
    pub fn interval(&self) -> Duration {
        self.settings.interval
    }

    /// 🔢 Gets the number of time events that have been emitted.
    ///
    /// # 🔄 Returns
    ///
    /// The count of time events that have occurred since the clock started,
    /// or 0 if the mutex couldn't be locked.
    ///
    /// # 📝 Examples
    ///
    /// ```
    /// use metric_time::Clock;
    ///
    /// let clock = Clock::new();
    /// assert_eq!(clock.count(), 0); // Initially zero
    /// ```
    pub fn count(&self) -> u128 {
        match self.counter_ref.lock() {
            Ok(counter) => *counter,
            Err(_) => 0 as u128,
        }
    }

    /// ▶️ Starts the clock, emitting time events at the configured interval.
    ///
    /// This method initializes a time emitter that will periodically emit time events.
    /// The provided callback function will be invoked for each time event.
    ///
    /// # 📥 Parameters
    ///
    /// * `on_emit` - A callback function that will be called for each time event.
    ///   The callback receives the current time and context information.
    ///
    /// # 🔄 Returns
    ///
    /// A `Result` indicating success or an error if the clock couldn't be started.
    ///
    /// # ❌ Errors
    ///
    /// Returns `ClockError::CouldNotSetTimeEmitter` if the clock's mutex couldn't be locked.
    ///
    /// # 📝 Examples
    ///
    /// ```
    /// use metric_time::Clock;
    ///
    /// let clock = Clock::new();
    /// let result = clock.start(|time, _| {
    ///     println!("Time: {}", time);
    /// });
    /// assert!(result.is_ok());
    /// ```
    ///
    /// # 🔒 Thread Safety
    ///
    /// The callback must be thread-safe (implement `Send` and `Sync`) and have a
    /// static lifetime, as it will be executed on a background thread.
    pub fn start<F>(&self, on_emit: F) -> Result<(), ClockError>
    where
        F: Fn(Time, EmitterContext<EmitterSettings>) -> () + Clone + Send + Sync + 'static,
    {
        let time_ref = Arc::clone(&self.time_ref);
        let counter_ref = Arc::clone(&self.counter_ref);

        match self.emitter_ref.lock() {
            Ok(mut emitter) => {
                // Create the handler for time events
                let on_emit_handler =
                    move |time: Time, context: EmitterContext<EmitterSettings>| {
                        on_emit(time.clone(), context);
                        match time_ref.lock() {
                            Ok(mut current_time) => {
                                *current_time = Some(time);
                                match counter_ref.lock() {
                                    Ok(mut counter) => {
                                        *counter += 1;
                                    }
                                    Err(_) => (),
                                }
                            }
                            Err(_) => (),
                        };
                    };

                // Create settings
                let settings = <EmitterSettings as EmitterSettingsTrait>::new()
                    .set_kind(self.settings.kind)
                    .set_interval(self.settings.interval);

                // Create the emitter
                #[cfg(not(feature = "web"))]
                let new_emitter_result = <Emitter as Startable>::start(settings, on_emit_handler);

                #[cfg(feature = "web")]
                let new_emitter_result =
                    <Emitter as WebStartable>::start(settings, on_emit_handler);

                match new_emitter_result {
                    Ok(new_emitter) => {
                        *emitter = Some(new_emitter);
                        Ok(())
                    }
                    Err(_) => Err(ClockError::CouldNotSetTimeEmitter),
                }
            }
            Err(_) => Err(ClockError::CouldNotSetTimeEmitter),
        }?;
        Ok(())
    }

    /// ⏹️ Stops the clock and returns the current time.
    ///
    /// This method stops the time emitter, preventing any further time events
    /// from being emitted. It also waits for the emitter to complete any
    /// pending operations.
    ///
    /// # 🔄 Returns
    ///
    /// A `Result` containing the current time if successful, or an error if
    /// the clock couldn't be stopped.
    ///
    /// # ❌ Errors
    ///
    /// Returns `ClockError::CouldNotUnsubscribe` if:
    /// - The clock's mutex couldn't be locked
    /// - No emitter was found (the clock wasn't started)
    /// - The emitter couldn't be stopped
    ///
    /// # 📝 Examples
    ///
    /// ```
    /// use metric_time::Clock;
    /// use std::time::Duration;
    /// use std::thread;
    ///
    /// let clock = Clock::new();
    /// clock.start(|_, _| {}).expect("Failed to start clock");
    ///
    /// // Let it run for a bit
    /// thread::sleep(Duration::from_millis(10));
    ///
    /// // Stop the clock and get the current time
    /// let current_time = clock.stop().expect("Failed to stop clock");
    /// ```
    pub fn stop(&self) -> Result<Time, ClockError> {
        match self.emitter_ref.lock() {
            Ok(mut time_emitter) => match &mut *time_emitter {
                Some(emitter) => {
                    // First stop the emitter
                    match <Emitter as Stoppable>::stop(emitter) {
                        Ok(_) => {
                            // Then wait for completion
                            let _ = <Emitter as Stoppable>::await_completion(emitter);
                            let time = self.time().unwrap_or(Time::now());
                            Ok(time)
                        }
                        Err(_) => Err(ClockError::CouldNotUnsubscribe),
                    }
                }
                None => Err(ClockError::CouldNotUnsubscribe),
            },
            Err(_) => Err(ClockError::CouldNotUnsubscribe),
        }
    }
}

// 🧪 Tests --------------------------------------------------------------------------- /
// 🧪 Unit tests for the Clock implementation

#[cfg(all(test, not(feature = "web")))]
mod tests {
    use std::sync::mpsc;

    use crate::Period;

    #[test]
    fn test_clock_new_with_defaults() {
        use super::*;
        let clock = Clock::new();
        assert_eq!(clock.kind(), TimeKind::Base24);
        assert_eq!(clock.interval(), Duration::from_secs(1));
    }

    #[test]
    fn test_clock_with_custom_setup() {
        use super::*;
        let settings = ClockSettings {
            kind: TimeKind::Base10,
            interval: Duration::from_millis(500),
        };
        let clock = Clock::new().setup(settings);
        assert_eq!(clock.kind(), TimeKind::Base10);
        assert_eq!(clock.interval(), Duration::from_millis(500));
    }

    #[test]
    fn test_clock_configuration_by_builder_pattern() {
        use super::*;
        let clock = Clock::new()
            .set_kind(TimeKind::Base12(Period::AM))
            .set_interval(Duration::from_millis(100));
        assert_eq!(clock.kind(), TimeKind::Base12(Period::AM));
        assert_eq!(clock.interval(), Duration::from_millis(100));
    }

    #[test]
    fn test_starting_and_stopping_clock() {
        use super::*;
        use std::thread;

        let start_time = Time::now();
        let clock = Clock::new().set_interval(Duration::from_millis(5));

        let (tx, rx) = mpsc::channel::<Time>();

        // Start the clock
        let result = clock.start(move |time, _ctx| {
            tx.send(time).unwrap();
        });
        assert!(result.is_ok());

        thread::sleep(Duration::from_millis(15));

        let result = clock.stop();
        assert!(result.is_ok());

        let rx_result = rx.recv();
        assert!(rx_result.is_ok());
        let current_time = rx_result.unwrap();

        let stop_time = result.unwrap();
        assert!(clock.count() >= 3);
        assert!(stop_time > start_time);
        assert_eq!(current_time.hours(), stop_time.hours());
        assert_eq!(current_time.minutes(), stop_time.minutes());
        assert_eq!(current_time.seconds(), stop_time.seconds());
    }

    #[test]
    fn test_clock_time_updates() {
        use super::*;
        use std::thread;

        let start_time = Time::now();
        let clock = Clock::new().set_interval(Duration::from_millis(5));

        let (tx, rx) = mpsc::channel::<Time>();

        // Start the clock
        clock
            .start(move |time, _ctx| {
                let _ = tx.send(time);
            })
            .expect("Failed to start clock");

        thread::sleep(Duration::from_millis(1));

        let first_time = clock.time();
        assert!(first_time.is_some());

        thread::sleep(Duration::from_millis(20));

        let last_time = clock.time();
        assert!(last_time.is_some());
        assert!(last_time.unwrap() > first_time.unwrap());

        let result = clock.stop();
        assert!(result.is_ok());

        let all_times: Vec<Time> = rx.try_iter().collect();
        assert!(!all_times.is_empty());
        let current_time = *all_times.last().unwrap();

        let stop_time = result.unwrap();
        assert!(stop_time > start_time);
        assert_eq!(current_time.hours(), stop_time.hours());
        assert_eq!(current_time.minutes(), stop_time.minutes());
        assert!((current_time.seconds() as i16 - stop_time.seconds() as i16).abs() <= 1);
    }
}

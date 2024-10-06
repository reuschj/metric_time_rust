//! A clock module for managing time events and measurements.
//!
//! This module provides functionality for creating and managing clocks that can emit time events
//! at specified intervals. It supports different time formats and customizable settings.
//!
//! # Example
//!
//! ```
//! use metric_time::clock::Clock;
//! use metric_time::clock_lib::ClockTrait;
//! use std::time::Duration;
//!
//! let clock = Clock::new()
//!     .set_interval(Duration::from_millis(100));
//!
//! let subscription = clock.start().unwrap();
//! // Clock is now ticking every 100ms
//!
//! let final_time = clock.stop().unwrap();
//! ```
//!
//! # Features
//!
//! - Configurable time intervals
//! - Multiple time formats (Base24, Base10, Base12)
//! - Event subscription system
//! - Thread-safe time tracking
//! - Customizable clock settings

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    clock_lib::{ClockError, ClockSettings},
    time::Time,
    time_emitter::{self, Subscription, TimeEmitter},
    time_lib::TimeKind,
};

// Clock --------------------------------------------------------------------------- /

/// A clock that emits time events at specified intervals.
///
/// The `Clock` struct provides functionality to track time and emit events at regular intervals.
/// It can be configured with different time formats and intervals, and provides thread-safe
/// access to time measurements.
///
/// # Features
///
/// - Configurable time intervals and formats
/// - Thread-safe time tracking
/// - Event subscription system
/// - Support for Base24, Base10, and Base12 time formats
///
/// # Examples
///
/// Basic usage:
/// ```
/// use metric_time::clock::Clock;
/// use metric_time::clock_lib::ClockTrait;
/// use std::time::Duration;
///
/// let clock = Clock::new()
///     .set_interval(Duration::from_millis(100));
///
/// let subscription = clock.start().unwrap();
/// // Clock is now ticking...
/// let final_time = clock.stop().unwrap();
/// ```
///
#[derive(Debug, Clone)]
pub struct Clock {
    emitter_ref: Arc<Mutex<Option<TimeEmitter>>>,
    time_ref: Arc<Mutex<Option<Time>>>,
    subscription_ref: Arc<Mutex<Option<Subscription>>>,
    counter_ref: Arc<Mutex<u128>>,
    settings: ClockSettings,
}

impl Clock {
    /// Creates a new Clock instance with default settings.
    ///
    /// Returns a Clock configured with default values for all settings including:
    /// - Default time emitter (None)
    /// - Default time reference (None)
    /// - Default subscription (None)
    /// - Counter starting at 0
    /// - Default clock settings
    pub fn new() -> Self {
        let emitter_ref = Arc::new(Mutex::new(None));
        let time_ref = Arc::new(Mutex::new(None));
        let subscription_ref = Arc::new(Mutex::new(None));
        let counter_ref = Arc::new(Mutex::new(0 as u128));
        let settings = ClockSettings::defaults();
        Self {
            emitter_ref,
            time_ref,
            subscription_ref,
            counter_ref,
            settings,
        }
    }

    // ------------------------------------------------------------ /

    /// Sets up the clock with custom settings.
    ///
    /// # Arguments
    /// * `settings` - The ClockSettings to configure this clock with
    ///
    /// # Returns
    /// Returns self for method chaining
    pub fn setup(mut self, settings: ClockSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Sets the time kind for the clock.
    ///
    /// # Arguments
    /// * `kind` - The TimeKind to use (Base24, Base10, etc)
    ///
    /// # Returns
    /// Returns self for method chaining
    pub fn set_kind(mut self, kind: TimeKind) -> Self {
        self.settings = self.settings.set_kind(kind);
        self
    }

    /// Sets the tick interval for the clock.
    ///
    /// # Arguments
    /// * `interval` - The Duration between ticks
    ///
    /// # Returns
    /// Returns self for method chaining
    pub fn set_interval(mut self, interval: Duration) -> Self {
        self.settings = self.settings.set_interval(interval);
        self
    }

    // ------------------------------------------------------------ /

    /// Gets the current time value.
    ///
    /// # Returns
    /// Returns Option<Time> containing the current time, or None if not available
    pub fn time(&self) -> Option<Time> {
        match self.time_ref.lock() {
            Ok(time) => *time,
            Err(_) => None,
        }
    }

    /// Gets the current time kind setting.
    ///
    /// # Returns
    /// Returns the TimeKind this clock is configured to use
    pub fn kind(&self) -> TimeKind {
        self.settings.kind
    }

    /// Gets the current interval setting.
    ///
    /// # Returns
    /// Returns the Duration between ticks
    pub fn interval(&self) -> Duration {
        self.settings.interval
    }

    /// Gets the current tick count.
    ///
    /// # Returns
    /// Returns the number of ticks that have occurred
    pub fn count(&self) -> u128 {
        match self.counter_ref.lock() {
            Ok(counter) => *counter,
            Err(_) => 0 as u128,
        }
    }

    /// Starts the clock ticking.
    ///
    /// # Arguments
    /// * `on_emit` - Callback function that is called on each tick with the current time and context
    ///
    /// # Returns
    /// Returns Result containing the Subscription if successful,
    /// or a ClockError if starting fails
    ///
    /// # Example
    /// ```
    /// use metric_time::clock::Clock;
    ///
    /// let clock = Clock::new();
    /// let subscription = clock.start(|time, ctx| {
    ///     println!("Current time: {}", time);
    /// }).unwrap();
    /// ```
    pub fn start<F>(&self, on_emit: F) -> Result<Subscription, ClockError>
    where
        F: Fn(Time, time_emitter::Context) -> () + Clone + Send + 'static,
    {
        let emitter = match self.emitter_ref.lock() {
            Ok(mut emitter) => {
                let emitter_settings = time_emitter::Settings::new()
                    .set_kind(self.settings.kind)
                    .set_interval(self.settings.interval);
                let new_emitter = TimeEmitter::new().setup(emitter_settings);
                *emitter = Some(new_emitter);
                Ok(new_emitter)
            }
            Err(_) => Err(ClockError::CouldNotSetTimeEmitter),
        }?;

        let time_ref = Arc::clone(&self.time_ref);
        let counter_ref = Arc::clone(&self.counter_ref);

        let (subscription, _) = emitter.emit(move |time, context| {
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
        });

        match self.subscription_ref.lock() {
            Ok(mut subscription_ref) => {
                *subscription_ref = Some(subscription.clone());
            }
            Err(_) => {
                subscription.unsubscribe().unwrap();
                return Err(ClockError::CouldNotSetTime);
            }
        }

        Ok(subscription)
    }

    /// Stops the clock from ticking.
    ///
    /// # Returns
    /// Returns Result containing the final Time if successful,
    /// or a ClockError if stopping fails
    pub fn stop(&self) -> Result<Time, ClockError> {
        match self.subscription_ref.lock() {
            Ok(subscription_ref) => match &*subscription_ref {
                Some(subscription) => match subscription.unsubscribe() {
                    Ok(_) => {
                        let time = self.time().unwrap_or(Time::now());
                        Ok(time)
                    }
                    Err(_) => Err(ClockError::CouldNotUnsubscribe),
                },
                None => Err(ClockError::CouldNotUnsubscribe),
            },
            Err(_) => Err(ClockError::CouldNotUnsubscribe),
        }
    }
}

/*
🧪 Tests --------------------------------------------------------------------------- /
*/

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use crate::time_lib::Period;

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

        thread::sleep(Duration::from_millis(20));

        let result = clock.stop();
        assert!(result.is_ok());

        let rx_result = rx.recv();
        assert!(rx_result.is_ok());
        let current_time = rx_result.unwrap();

        let stop_time = result.unwrap();
        assert_eq!(clock.count(), 4);
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
        let result = clock.start(move |time, _ctx| {
            tx.send(time);
        });
        assert!(result.is_ok());

        thread::sleep(Duration::from_millis(1));

        let first_time = clock.time();
        assert!(first_time.is_some());

        thread::sleep(Duration::from_millis(20));

        let last_time = clock.time();
        assert!(last_time.is_some());
        assert!(last_time.unwrap() > first_time.unwrap());

        let result = clock.stop();
        assert!(result.is_ok());

        let rx_result = rx.recv();
        assert!(rx_result.is_ok());
        let current_time = rx_result.unwrap();

        let stop_time = result.unwrap();
        assert!(stop_time > start_time);
        assert_eq!(current_time.hours(), stop_time.hours());
        assert_eq!(current_time.minutes(), stop_time.minutes());
        assert_eq!(current_time.seconds(), stop_time.seconds());
    }
}

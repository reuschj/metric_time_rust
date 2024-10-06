//! TimeEmitter
//!
//! A module that provides a configurable time emission system with the following features:
//!
//! - Customizable time intervals
//! - Maximum event limits
//! - Multiple time base support (Base10, Base24)
//! - Thread-safe event counting
//! - Subscription-based event management
//!
//! # Example
//!
//! ```rust
//! use metric_time::clock::{TimeEmitter, Settings, TimeKind};
//! use std::time::Duration;
//!
//! let settings = Settings::new()
//!     .set_interval(Duration::from_secs(1))
//!     .set_kind(TimeKind::Base10);
//!
//! let emitter = TimeEmitter::new().setup(settings);
//! let (subscription, handle) = emitter.emit(|time, ctx| {
//!     println!("Time: {}", time);
//! });
//! ```

use std::sync::mpsc::{SendError, Sender};
use std::sync::{mpsc::channel, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::time::Time;
use crate::time_lib::{TimeConversionTrait, TimeKind};

// Context ----------------------------------------------------------------------- /

/// Event context that provides timing information and settings
///
/// # Examples
///
/// ```
/// use metric_time::clock::{Context, Settings};
///
/// let context = Context {
///     index: 0,
///     settings: Settings::defaults()
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    pub index: u64,
    pub settings: Settings,
}

// Settings ----------------------------------------------------------------------- /

/// Settings for configuring time emission behavior
///
/// # Fields
///
/// * `max_events` - Optional maximum number of events to emit
/// * `interval` - Time duration between emissions
/// * `kind` - Time base system (Base10 or Base24)
///
/// # Examples
///
/// ```rust
/// use metric_time::clock::{Settings, TimeKind};
/// use std::time::Duration;
///
/// let settings = Settings::new()
///     .set_interval(Duration::from_secs(1))
///     .set_max_events(10)
///     .set_kind(TimeKind::Base10);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Settings {
    pub max_events: Option<u64>,
    pub interval: Duration,
    pub kind: TimeKind,
}

impl Settings {
    /// Creates a new Settings instance with default values
    ///
    /// # Examples
    ///
    /// ```
    /// use metric_time::clock::Settings;
    ///
    /// let settings = Settings::new();
    /// ```
    pub fn new() -> Self {
        Self::defaults()
    }

    /// Creates a new Settings instance with default values:
    /// - No maximum events limit
    /// - 1 second interval
    /// - Base24 time kind
    ///
    /// # Examples
    ///
    /// ```
    /// use metric_time::clock::Settings;
    ///
    /// let settings = Settings::defaults();
    /// ```
    pub fn defaults() -> Self {
        Settings {
            max_events: None,
            interval: Duration::from_secs(1),
            kind: TimeKind::Base24,
        }
    }

    /// Sets the maximum number of events to emit
    ///
    /// # Arguments
    ///
    /// * `max_events` - The maximum number of events
    ///
    /// # Examples
    ///
    /// ```
    /// use metric_time::clock::Settings;
    ///
    /// let settings = Settings::new().set_max_events(10);
    /// ```
    pub fn set_max_events(mut self, max_events: u64) -> Self {
        self.max_events = Some(max_events);
        self
    }

    /// Removes the maximum events limit
    ///
    /// # Examples
    ///
    /// ```
    /// use metric_time::clock::Settings;
    ///
    /// let settings = Settings::new()
    ///     .set_max_events(10)
    ///     .clear_max_events();
    /// ```
    pub fn clear_max_events(mut self) -> Self {
        self.max_events = None;
        self
    }

    /// Sets the interval between emissions
    ///
    /// # Arguments
    ///
    /// * `interval` - The time duration between emissions
    ///
    /// # Examples
    ///
    /// ```
    /// use metric_time::clock::Settings;
    /// use std::time::Duration;
    ///
    /// let settings = Settings::new()
    ///     .set_interval(Duration::from_secs(2));
    /// ```
    pub fn set_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Sets the time base system kind
    ///
    /// # Arguments
    ///
    /// * `kind` - The time base system (Base10 or Base24)
    ///
    /// # Examples
    ///
    /// ```
    /// use metric_time::clock::{Settings, TimeKind};
    ///
    /// let settings = Settings::new()
    ///     .set_kind(TimeKind::Base10);
    /// ```
    pub fn set_kind(mut self, kind: TimeKind) -> Self {
        self.kind = kind;
        self
    }
}

// MessageType ----------------------------------------------------------------------- /

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessageType {
    Start,
    Continue,
    Unsubscribe,
}

// Subscription ----------------------------------------------------------------------- /

/// A subscription handle for controlling time emission events
///
/// # Examples
///
/// ```rust
/// use metric_time::clock::{TimeEmitter, Settings};
///
/// let emitter = TimeEmitter::new();
/// let (subscription, handle) = emitter.emit(|time, ctx| {
///     println!("Time: {}", time);
/// });
///
/// // Later, unsubscribe to stop receiving events
/// subscription.unsubscribe().unwrap();
/// ```
///
/// The subscription can be used to:
/// - Unsubscribe from events
/// - Control the event flow
/// - Clean up resources when no longer needed
#[derive(Debug, Clone)]
pub struct Subscription {
    tx: Sender<MessageType>,
}

impl Subscription {
    /// Creates a new Subscription instance
    ///
    /// # Arguments
    ///
    /// * `sender` - A channel sender for subscription messages
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::mpsc::channel;
    /// use metric_time::clock::{MessageType, Subscription};
    ///
    /// let (tx, _rx) = channel();
    /// let subscription = Subscription::new(tx);
    /// ```
    pub fn new(sender: Sender<MessageType>) -> Self {
        Self { tx: sender }
    }

    /// Unsubscribes from receiving further events
    ///
    /// # Returns
    ///
    /// * `Result<(), SendError<MessageType>>` - Ok if unsubscribe message was sent successfully,
    ///    Err if the channel has been closed
    ///
    /// # Examples
    ///
    /// ```
    /// use metric_time::clock::{TimeEmitter, Settings};
    ///
    /// let emitter = TimeEmitter::new();
    /// let (subscription, _handle) = emitter.emit(|time, _| {
    ///     println!("Time: {}", time);
    /// });
    ///
    /// subscription.unsubscribe().unwrap();
    /// ```
    pub fn unsubscribe(&self) -> Result<(), SendError<MessageType>> {
        self.tx.send(MessageType::Unsubscribe)
    }
}

// TimeEmitter ----------------------------------------------------------------------- /

/// A time emission system that provides configurable time-based event emission
///
/// The TimeEmitter allows you to:
/// - Emit time events at configurable intervals
/// - Limit the total number of events
/// - Choose between different time base systems
/// - Subscribe to and unsubscribe from events
/// - Get contextual information with each event
///
/// # Examples
///
/// ```rust
/// use metric_time::clock::{TimeEmitter, Settings, TimeKind};
/// use std::time::Duration;
///
/// // Create an emitter with custom settings
/// let settings = Settings::new()
///     .set_interval(Duration::from_secs(1))
///     .set_kind(TimeKind::Base10);
///
/// let emitter = TimeEmitter::new().setup(settings);
///
/// // Subscribe to time events
/// let (subscription, handle) = emitter.emit(|time, ctx| {
///     println!("Time: {} (Event #{})", time, ctx.index);
/// });
///
/// // Unsubscribe when done
/// subscription.unsubscribe().unwrap();
/// handle.join().unwrap();
/// ```
///
/// # Thread Safety
///
/// The TimeEmitter is thread-safe and can be used across multiple threads.
/// Event counting and message passing are handled through thread-safe mechanisms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeEmitter {
    pub settings: Settings,
}

impl TimeEmitter {
    /// Creates a new TimeEmitter with default settings
    ///
    /// # Returns
    ///
    /// A new TimeEmitter instance initialized with default settings
    ///
    /// # Examples
    ///
    /// ```
    /// use metric_time::clock::TimeEmitter;
    ///
    /// let emitter = TimeEmitter::new();
    /// ```
    pub fn new() -> Self {
        Self {
            settings: Settings::defaults(),
        }
    }

    /// Configures the TimeEmitter with custom settings
    ///
    /// # Arguments
    ///
    /// * `settings` - Custom settings to configure the emitter
    ///
    /// # Returns
    ///
    /// The configured TimeEmitter instance
    ///
    /// # Examples
    ///
    /// ```
    /// use metric_time::clock::{TimeEmitter, Settings};
    /// use std::time::Duration;
    ///
    /// let settings = Settings::new()
    ///     .set_interval(Duration::from_secs(1));
    /// let emitter = TimeEmitter::new().setup(settings);
    /// ```
    pub fn setup(mut self, settings: Settings) -> Self {
        self.settings = settings;
        self
    }

    /// Starts emitting time events with the configured settings
    ///
    /// # Arguments
    ///
    /// * `on_emit` - Callback function that receives time events and context
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - Subscription: Used to control and stop the event emission
    /// - JoinHandle: Thread handle for the emission process
    ///
    /// # Examples
    ///
    /// ```
    /// use metric_time::clock::TimeEmitter;
    ///
    /// let emitter = TimeEmitter::new();
    /// let (subscription, handle) = emitter.emit(|time, ctx| {
    ///     println!("Time: {} (Event #{})", time, ctx.index);
    /// });
    /// ```
    pub fn emit<F>(&self, on_emit: F) -> (Subscription, JoinHandle<()>)
    where
        F: Fn(Time, Context) -> () + Clone + Send + 'static,
    {
        let settings = self.settings;

        let (tx, rx) = channel::<MessageType>();
        let thread_tx = tx.clone();

        let event_counter = Arc::new(Mutex::new(0 as u64));
        let event_counter_for_thread = Arc::clone(&event_counter);

        let handle = thread::spawn(move || loop {
            let mut current_count = event_counter_for_thread.lock().unwrap();
            let index = (*current_count).clone();

            match rx.recv() {
                Ok(message) => match message {
                    MessageType::Start => {}
                    MessageType::Continue => (),
                    MessageType::Unsubscribe => {
                        break;
                    }
                },
                _ => (),
            };

            if let Some(max_events) = settings.max_events {
                if *current_count >= max_events {
                    break;
                }
            }

            let time = Time::now().to(settings.kind);

            on_emit(time, Context { index, settings });
            thread::sleep(settings.interval);
            *current_count += 1;
            thread_tx.send(MessageType::Continue).unwrap_or_else(|err| {
                eprint!("Send Error: {}", err);
            });
        });

        tx.send(MessageType::Start).unwrap_or_else(|err| {
            eprint!("Send Error: {}", err);
        });

        (Subscription::new(tx), handle)
    }
}

/*
🧪 Tests --------------------------------------------------------------------------- /
*/

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_settings_defaults() {
        let settings = Settings::defaults();
        assert_eq!(settings.max_events, None);
        assert_eq!(settings.interval, Duration::from_secs(1));
    }

    #[test]
    fn test_settings_modifications() {
        let settings = Settings::new()
            .set_max_events(10)
            .set_interval(Duration::from_millis(500))
            .set_kind(TimeKind::Base10);

        assert_eq!(settings.max_events, Some(10));
        assert_eq!(settings.interval, Duration::from_millis(500));
        assert_eq!(settings.kind, TimeKind::Base10);

        let cleared_settings = settings.clear_max_events();
        assert_eq!(cleared_settings.max_events, None);
    }

    #[test]
    fn test_time_emitter_creation() {
        let emitter = TimeEmitter::new();
        assert_eq!(emitter.settings.max_events, None);
        assert_eq!(emitter.settings.interval, Duration::from_secs(1));
    }

    #[test]
    fn test_time_emitter_custom_settings() {
        let custom_settings = Settings::new()
            .set_max_events(5)
            .set_interval(Duration::from_millis(100))
            .set_kind(TimeKind::Base10);

        let emitter = TimeEmitter::new().setup(custom_settings);
        assert_eq!(emitter.settings.max_events, Some(5));
        assert_eq!(emitter.settings.interval, Duration::from_millis(100));
        assert_eq!(emitter.settings.kind, TimeKind::Base10);
    }

    #[test]
    fn test_emitter_max_events() {
        let current_time = Arc::new(Mutex::new(None as Option<Time>));
        let event_count = Arc::new(Mutex::new(0 as u64));
        let max_events: u64 = 10;

        let settings = Settings::new()
            .set_max_events(max_events)
            .set_interval(Duration::from_millis(10))
            .set_kind(TimeKind::Base10);

        let emitter = TimeEmitter::new().setup(settings);

        let callback_current_time = Arc::clone(&current_time);
        let callback_event_count = Arc::clone(&event_count);
        let (subscription, handle) = emitter.emit(move |time, _| {
            let mut current_count = callback_event_count.lock().unwrap();
            *current_count += 1;
            let mut current_time = callback_current_time.lock().unwrap();
            *current_time = Some(time);
        });

        thread::sleep(Duration::from_millis(20));
        subscription.unsubscribe().unwrap_or_else(|err| {
            println!("{}", err);
        });
        handle.join().unwrap();

        let final_time = current_time.lock().unwrap();
        let final_count = event_count.lock().unwrap();

        assert!((*final_time).is_some());
        assert_eq!((*final_time).unwrap().kind(), settings.kind);
        assert!(*final_count <= max_events);
    }

    #[test]
    fn test_emitter_unsubscribe() {
        let emitter =
            TimeEmitter::new().setup(Settings::new().set_interval(Duration::from_millis(50)));

        let initial_event_count: u64 = 0;
        let event_count = Arc::new(Mutex::new(initial_event_count));
        let callback_event_count = Arc::clone(&event_count);
        let (subscription, handle) = emitter.emit(move |_, _| {
            let mut current_count = callback_event_count.lock().unwrap();
            *current_count += 1;
        });

        // Let it run for a short time
        thread::sleep(Duration::from_millis(20));

        // Unsubscribe
        subscription.unsubscribe().unwrap_or_else(|err| {
            println!("{}", err);
        });
        handle.join().unwrap();

        // Store the count
        let count_at_unsubscribe = event_count.lock().unwrap();

        // Wait a bit more
        thread::sleep(Duration::from_millis(50));

        // Should have only had time for 1 event in this timeframe
        assert_eq!(*count_at_unsubscribe, initial_event_count + 1);
    }

    #[test]
    fn test_context_values() {
        let last_index = Arc::new(Mutex::new(None as Option<u64>));
        let settings = Settings::new()
            .set_max_events(3)
            .set_interval(Duration::from_millis(10));

        let emitter = TimeEmitter::new().setup(settings);

        let (subscription, handle) = emitter.emit(move |_, ctx| {
            let mut last_index = last_index.lock().unwrap();
            if let Some(last) = *last_index {
                assert_eq!(ctx.index, last + 1);
            }
            *last_index = Some(ctx.index);
            assert_eq!(ctx.settings.max_events, Some(3));
            assert_eq!(ctx.settings.interval, Duration::from_millis(10));
        });

        thread::sleep(Duration::from_millis(50));
        subscription.unsubscribe().unwrap_or_else(|err| {
            println!("{}", err);
        });
        handle.join().unwrap();
    }

    #[tokio::test]
    async fn test_async_operation() {
        let event_count = Arc::new(Mutex::new(0));
        let event_count_clone = Arc::clone(&event_count);

        let emitter = TimeEmitter::new().setup(
            Settings::new()
                .set_max_events(3)
                .set_interval(Duration::from_millis(10)),
        );

        let (subscription, handle) = emitter.emit(move |_, _| {
            let mut count = event_count_clone.lock().unwrap();
            *count += 1;
        });

        tokio::time::sleep(Duration::from_millis(50)).await;
        subscription.unsubscribe().unwrap_or_else(|err| {
            println!("{}", err);
        });
        handle.join().unwrap();

        assert_eq!(*event_count.lock().unwrap(), 3);
    }
}

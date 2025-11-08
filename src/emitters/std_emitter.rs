//! 📡 Standard environment implementation of the emitter.
//!
//! This module provides a emitter implementation for standard (non-WebAssembly)
//! environments. It uses native threads and synchronization primitives to
//! implement periodic time event emission.
//!
//! # 📋 Overview
//!
//! The [`Emitter`] in this module uses a background thread to emit time events at
//! regular intervals. It supports configurable settings like interval duration,
//! maximum event count, and time format.
//!
//! # 📚 Usage Example
//!
//! ```rust,no_run
//! use metric_time::{Emitter, EmitterSettings, Startable};
//! use std::time::Duration;
//!
//! // Create emitter with default settings (1 second interval)
//! let interval = Emitter::start(EmitterSettings::default(), |time, ctx| {
//!     println!("Time: {}, Event #{}", time, ctx.index);
//! });
//!
//! // The interval will continue running until dropped or explicitly stopped
//! ```

use std::any::Any;
use std::fmt::Debug;
use std::sync::mpsc::{self, SendError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::lib::{EmitterContext, EmitterSettingsTrait, Startable, Stoppable};
use crate::{Time, TimeConversionTrait, TimeKind};

// 📡 Emitter ----------------------------------------------------------------------- /

/// 📡 A emitter implementation for standard environments.
///
/// This struct provides a way to emit time events at regular intervals
/// using a background thread. It implements the [`TimeEmitterTrait`] trait
/// to provide a consistent interface across platforms.
///
/// # 🔒 Thread Safety
///
/// The `Emitter` uses thread-safe constructs (Arc, Mutex) to allow
/// safe sharing between threads. The background thread will continue running
/// until the interval is stopped or dropped.
pub struct Emitter {
    /// The configuration settings for this emitter
    settings: Settings,
    /// Subscription for communicating with the background thread
    subscription: Subscription,
    /// Optional handle to the background thread
    handle: Option<JoinHandle<()>>,
    /// The callback function that will be invoked for each time event
    callback: Arc<Mutex<Box<dyn FnMut(Time, EmitterContext<Settings>) -> () + Send + Sync>>>,
}

impl Drop for Emitter {
    fn drop(&mut self) {
        self.stop().unwrap_or_else(|err| {
            eprintln!("Failed to finish interval: {}", err);
        });
    }
}

impl Clone for Emitter {
    fn clone(&self) -> Self {
        Self {
            settings: self.settings.clone(),
            subscription: self.subscription.clone(),
            handle: None,
            callback: Arc::clone(&self.callback),
        }
    }
}

unsafe impl Send for Emitter {}

unsafe impl Sync for Emitter {}

impl Debug for Emitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interval")
            .field("settings", &self.settings)
            .field("subscription", &self.subscription)
            .finish()
    }
}

impl Startable for Emitter {
    type Settings = Settings;
    type ErrorType = Error;

    /// 🚀 Starts a new emitter with the given settings and callback.
    ///
    /// This method creates a new background thread that will emit time events
    /// at the interval specified in the settings. The callback function will
    /// be invoked for each time event with the current time and context.
    ///
    /// # 📥 Parameters
    ///
    /// * `settings` - ⚙️ Configuration for the emitter
    /// * `on_emit` - 🔔 Callback function that will be invoked for each time event
    ///
    /// # 📤 Returns
    ///
    /// A new instance of the emitter
    ///
    /// # 🔒 Thread Safety
    /// # Thread Behavior
    ///
    /// This method spawns a background thread that will continue running
    /// until the interval is stopped or dropped.
    fn start<F>(settings: Self::Settings, on_emit: F) -> Result<Self, Error>
    where
        F: FnMut(Time, EmitterContext<Self::Settings>) -> () + Send + Sync + 'static,
    {
        let event_count = Arc::new(Mutex::new(0));

        let (tx, rx) = mpsc::channel();

        let callback: Arc<
            Mutex<Box<dyn FnMut(Time, EmitterContext<Self::Settings>) + Send + Sync + 'static>>,
        > = Arc::new(Mutex::new(Box::new(on_emit)));

        let thread_callback = Arc::clone(&callback);
        let thread_tx = tx.clone();
        let thread_event_count = Arc::clone(&event_count);

        // Send an initial Start message to begin processing
        tx.send(MessageType::Start).unwrap_or_else(|err| {
            eprintln!("Error sending start message: {}", err);
        });

        let handle = thread::spawn(move || {
            loop {
                let current_count = {
                    let count = thread_event_count.lock().unwrap();
                    *count
                };

                match rx.recv() {
                    Ok(message) => match message {
                        MessageType::Start => {}
                        MessageType::Continue => (),
                        MessageType::Unsubscribe => {
                            break;
                        }
                    },
                    Err(_) => {
                        break;
                    }
                };

                // Process time events after receiving any message (Start or Continue)
                if let Some(max_events) = settings.max_events() {
                    if current_count >= max_events {
                        break;
                    }
                }

                let time = Time::now().to(settings.kind());

                // Call the callback with the current count as index
                let mut cb = thread_callback.lock().unwrap();
                (*cb)(
                    time,
                    EmitterContext {
                        index: current_count,
                        settings,
                    },
                );

                // Sleep for the specified interval
                thread::sleep(settings.interval());

                // Update the counter after the callback has completed
                {
                    let mut count = thread_event_count.lock().unwrap();
                    *count += 1;
                }

                thread_tx.send(MessageType::Continue).unwrap_or_else(|err| {
                    eprintln!("Send Error: {}", err);
                });
            }
        });

        Ok(Self {
            settings,
            subscription: Subscription::new(tx),
            handle: Some(handle),
            callback,
        })
    }

    /// ⚙️ Gets a reference to the settings of the emitter.
    ///
    /// This method provides read access to the settings used to configure this interval.
    ///
    /// # 📤 Returns
    ///
    /// A reference to the settings
    fn settings(&self) -> &Self::Settings {
        &self.settings
    }
}

impl Stoppable for Emitter {
    type OnStopValue = ();
    type ErrorType = Error;

    /// 🛑 Stops the emitter.
    ///
    /// This method sends a stop signal to the background thread, which will cause
    /// it to terminate. It does not wait for the thread to join, so the thread may
    /// continue running for a short time after this method returns.
    ///
    /// # 📤 Returns
    ///
    /// A Result indicating success or failure of the stop operation.
    /// If the background thread has already been stopped or joined, an error is returned.
    fn stop(&mut self) -> Result<Self::OnStopValue, Self::ErrorType> {
        self.subscription
            .unsubscribe()
            .map_err(|err| Error::StopError(err))
    }

    /// Waits for the emitter's background thread to complete.
    ///
    /// This is a blocking call that waits for the background thread to terminate.
    /// It should be called after `stop()` to ensure proper cleanup of resources.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the join operation
    ///
    /// # Notes
    ///
    /// - This method takes `&mut self` because it consumes the thread handle
    /// - If the handle has already been taken (e.g., by a previous call), this returns Ok(())
    fn await_completion(&mut self) -> Result<Self::OnStopValue, Self::ErrorType> {
        if let Some(handle) = self.handle.take() {
            handle.join().map_err(|err| Error::JoinError(err))
        } else {
            Ok(())
        }
    }
}

impl Emitter {
    /// Legacy method for joining the thread. Prefer using `await_completion()` instead.
    /// This method is kept for backward compatibility.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the join operation
    pub fn join(&mut self) -> Result<(), Error> {
        self.await_completion()
    }
}

// MessageType ----------------------------------------------------------------------- /

/// 📨 Messages that can be sent to the background thread.
///
/// These messages control the behavior of the emitter's background thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessageType {
    /// 🚀 Start the emitter
    Start,
    /// ⏭️ Continue processing (sent after each event)
    Continue,
    /// 🛑 Stop the emitter
    Unsubscribe,
}

// Subscription ----------------------------------------------------------------------- /

/// 📬 A subscription for communicating with the emitter's background thread.
///
/// This struct provides a way to send messages to the background thread,
/// particularly to stop it when the interval is no longer needed.
#[derive(Debug, Clone)]
pub struct Subscription {
    /// 📤 Channel sender for communicating with the background thread
    tx: Sender<MessageType>,
}

impl Subscription {
    /// 🆕 Creates a new subscription with the given sender.
    ///
    /// # Parameters
    ///
    /// * `sender` - 📤 The sender end of a channel connected to the background thread
    pub fn new(sender: Sender<MessageType>) -> Self {
        Self { tx: sender }
    }

    /// 🛑 Sends an unsubscribe message to the background thread.
    ///
    /// This instructs the background thread to terminate.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the message send
    pub fn unsubscribe(&self) -> Result<(), SendError<MessageType>> {
        self.tx.send(MessageType::Unsubscribe)
    }
}

// Define Settings for native emitter
// ⚙️ Settings ----------------------------------------------------------------------- /

/// ⚙️ Settings for configuring a standard environment emitter.
///
/// This struct implements the [`TimeEmitterSettingsTrait`] trait to provide
/// configuration options for the emitter, such as the interval between
/// events, the maximum number of events, and the time kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Settings {
    /// 🔢 Maximum number of events to emit (None means unlimited)
    max_events: Option<u64>,
    /// ⏱️ Interval between emitted events
    interval: Duration,
    /// 🔢 Kind of time to emit (e.g., Base10, Base24)
    kind: TimeKind,
}

impl EmitterSettingsTrait for Settings {
    type Duration = Duration;

    fn new() -> Self {
        Self::default()
    }

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

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_events: None,
            interval: Duration::from_secs(1),
            kind: TimeKind::Base24,
        }
    }
}

// Error ----------------------------------------------------------------------- /

/// ❌ An error that can occur when using the Emitter.
///
/// This enum represents the different types of errors that can occur
/// when working with a emitter in a standard environment.
#[derive(Debug)]
pub enum Error {
    /// 🧵 Error that occurred while joining the background thread
    JoinError(Box<dyn Any + Send + 'static>),
    /// 🛑 Error that occurred while trying to stop the interval
    StopError(SendError<MessageType>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::JoinError(_err) => write!(f, "Error while joining emitter handle"),
            Error::StopError(err) => write!(f, "Error stopping emitter: {}", err),
        }
    }
}

impl std::error::Error for Error {}

// 🧪 Tests ----------------------------------------------------------------------- /

#[cfg(test)]
// 🧪 Tests section for standard emitter
mod tests {
    use std::time::Duration;

    use super::*;

    /// ⚙️ Test for settings modifications
    #[test]
    fn test_settings_modifications() {
        let settings = Settings::new()
            .set_max_events(10)
            .set_interval(Duration::from_millis(500))
            .set_kind(TimeKind::Base10);

        assert_eq!(settings.max_events(), Some(10));
        assert_eq!(settings.interval(), Duration::from_millis(500));
        assert_eq!(settings.kind(), TimeKind::Base10);

        let cleared_settings = settings.clear_max_events();
        assert_eq!(cleared_settings.max_events(), None);
    }

    /// 🏗️ Test for creating a emitter with default settings
    #[test]
    fn test_emitter_creation() {
        let emitter = Emitter::start(Settings::default(), |_, _| {}).unwrap();
        assert_eq!(emitter.settings().max_events(), None);
        assert_eq!(emitter.settings().interval(), Duration::from_secs(1));
    }

    /// 🛠️ Test for creating a emitter with custom settings
    #[test]
    fn test_emitter_custom_settings() {
        let settings = Settings::new()
            .set_max_events(5)
            .set_interval(Duration::from_millis(100))
            .set_kind(TimeKind::Base10);

        let emitter = Emitter::start(settings, |_, _| {}).unwrap();
        assert_eq!(emitter.settings().max_events(), Some(5));
        assert_eq!(emitter.settings().interval(), Duration::from_millis(100));
        assert_eq!(emitter.settings().kind(), TimeKind::Base10);
    }

    /// 🔢 Test for emitter with maximum event count
    #[test]
    fn test_emitter_max_events() {
        let current_time = Arc::new(Mutex::new(None as Option<Time>));
        let event_count = Arc::new(Mutex::new(0 as u64));
        let max_events: u64 = 10;

        let settings = Settings::new()
            .set_max_events(max_events)
            .set_interval(Duration::from_millis(10))
            .set_kind(TimeKind::Base10);

        let callback_current_time = Arc::clone(&current_time);
        let callback_event_count = Arc::clone(&event_count);

        let mut emitter = Emitter::start(settings, move |time, _| {
            let mut current_count = callback_event_count.lock().unwrap();
            *current_count += 1;
            let mut current_time = callback_current_time.lock().unwrap();
            *current_time = Some(time);
        })
        .unwrap();

        thread::sleep(Duration::from_millis(20));
        emitter.stop().unwrap_or_else(|err| {
            println!("{}", err);
        });
        emitter.await_completion().unwrap_or_else(|err| {
            println!("{}", err);
        });

        let final_time = current_time.lock().unwrap();
        let final_count = event_count.lock().unwrap();

        assert!((*final_time).is_some());
        assert_eq!((*final_time).unwrap().kind(), settings.kind());
        assert!(*final_count <= max_events);
    }

    /// 🛑 Test for unsubscribing from the emitter
    #[test]
    fn test_emitter_unsubscribe() {
        let initial_event_count: u64 = 0;
        let event_count = Arc::new(Mutex::new(initial_event_count));
        let callback_event_count = Arc::clone(&event_count);

        let mut emitter = Emitter::start(
            Settings::new().set_interval(Duration::from_millis(50)),
            move |_, _| {
                let mut current_count = callback_event_count.lock().unwrap();
                *current_count += 1;
            },
        )
        .unwrap();

        // Let it run for a short time
        thread::sleep(Duration::from_millis(20));

        // Stop it
        emitter.stop().unwrap_or_else(|err| {
            println!("{}", err);
        });
        emitter.await_completion().unwrap_or_else(|err| {
            println!("{}", err);
        });

        // Store the count
        let count_at_stop = event_count.lock().unwrap();

        // Wait a bit more
        thread::sleep(Duration::from_millis(50));

        // Should have only had time for 1 event in this timeframe
        assert_eq!(*count_at_stop, initial_event_count + 1);
    }

    /// 📋 Test for context values in callback
    #[test]
    fn test_context_values() {
        let expected_index = Arc::new(Mutex::new(0));
        let settings = Settings::new()
            .set_max_events(3)
            .set_interval(Duration::from_millis(10));

        let mut emitter = Emitter::start(settings, move |_, ctx| {
            let mut expected_index = expected_index.lock().unwrap();
            assert_eq!(ctx.index, *expected_index);
            assert_eq!(ctx.settings.max_events(), Some(3));
            assert_eq!(ctx.settings.interval(), Duration::from_millis(10));
            *expected_index += 1;
        })
        .unwrap();

        thread::sleep(Duration::from_millis(50));

        emitter.stop().unwrap_or_else(|err| {
            eprintln!("{}", err);
        });
        emitter.await_completion().unwrap_or_else(|err| {
            eprintln!("{}", err);
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// ⚡ Test for async operation with tokio
    #[tokio::test]
    async fn test_async_operation() {
        let event_count = Arc::new(Mutex::new(0));
        let event_count_clone = Arc::clone(&event_count);

        let mut emitter = Emitter::start(
            Settings::new()
                .set_max_events(3)
                .set_interval(Duration::from_millis(10)),
            move |_, _| {
                let mut count = event_count_clone.lock().unwrap();
                *count += 1;
            },
        )
        .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        emitter.stop().unwrap_or_else(|err| {
            println!("{}", err);
        });

        emitter.await_completion().unwrap_or_else(|err| {
            println!("{}", err);
        });

        assert_eq!(*event_count.lock().unwrap(), 3);
    }
}

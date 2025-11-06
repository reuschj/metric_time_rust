//! 🌐 WebAssembly implementation of the time emitter.
//!
//! This module provides a time emitter implementation specifically for WebAssembly
//! environments. It uses browser APIs via the gloo and web_sys crates to implement
//! periodic time event emission.
//!
//! # 📋 Overview
//!
//! The [`WebEmitter`] in this module uses JavaScript's `setInterval` (via gloo)
//! to emit time events at regular intervals. It supports the same configuration options
//! as the standard implementation but is optimized for the browser environment.
//!
//! # 📚 Usage Example
//!
//! ```rust,no_run
//! use metric_time::{Settings, WebEmitter, Emittable};
//! use web_time::Duration;
//!
//! // Create time emitter with default settings (1 second interval)
//! let emitter = WebEmitter::start(Settings::default(), |time, ctx| {
//!     // Handle time event in the browser
//!     web_sys::console::log_1(&format!("Time: {}, Event #{}", time, ctx.index).into());
//! });
//!
//! // The emitter will continue running until dropped or explicitly stopped
//! ```

use std::fmt::Debug;
use std::sync::{Arc, Mutex};
#[cfg(feature = "web")]
use wasm_bindgen::{prelude::*, JsValue};
#[cfg(feature = "web")]
use web_sys::js_sys::Function;
#[cfg(feature = "web")]
use web_time::Duration;

use crate::{Emittable, EmitterContext, EmitterSettingsTrait, Time, TimeConversionTrait, TimeKind};

#[cfg(feature = "web")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "setInterval", catch)]
    fn set_interval(handler: &Function, timeout: i32) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = "clearInterval")]
    fn clear_interval(handle: JsValue) -> JsValue;
}

// 📡 Interval for web environments ----------------------------------------------------------------------- /

#[cfg(feature = "web")]
/// 📡 A time interval implementation for web environments.
///
/// This struct provides a way to emit time events at regular intervals
/// using JavaScript's `setInterval` API. It implements the [`TimeEmitterTrait`] trait
/// to provide a consistent interface across platforms.
///
/// # 🌐 Browser Integration
///
/// The `TimeEmitter` uses browser APIs through the `web_sys` and `gloo` crates
/// to schedule periodic callbacks. The interval ID is stored to allow for
/// proper cleanup when the emitter is stopped or dropped.
pub struct WebEmitter {
    /// ⚙️ The configuration settings for this time emitter
    settings: Settings,
    //
    closure: Arc<Mutex<Closure<dyn FnMut()>>>,
    handle: Arc<Mutex<Result<JsValue, Error>>>,
}

#[cfg(feature = "web")]
impl Debug for WebEmitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimeEmitter")
            .field("settings", &self.settings)
            .finish()
    }
}

#[cfg(feature = "web")]
impl Clone for WebEmitter {
    fn clone(&self) -> Self {
        Self {
            settings: self.settings.clone(),
            closure: Arc::clone(&self.closure),
            handle: Arc::clone(&self.handle),
        }
    }
}

#[cfg(feature = "web")]
impl Drop for WebEmitter {
    fn drop(&mut self) {
        match self.stop() {
            Ok(_) => (),
            Err(err) => eprintln!("Error stopping interval on drop: {}", err),
        }
    }
}

#[cfg(feature = "web")]
impl Emittable for WebEmitter {
    type Settings = Settings;
    type OnStopValue = JsValue;
    type ErrorType = Error;

    /// 🚀 Starts a new time emitter with the given settings and callback.
    ///
    /// This method creates a JavaScript interval that will emit time events
    /// at the interval specified in the settings. The callback function will
    /// be invoked for each time event with the current time and context.
    ///
    /// # 📥 Parameters
    ///
    /// * `settings` - ⚙️ Configuration for the time emitter
    /// * `on_emit` - 🔔 Callback function that will be invoked for each time event
    ///
    /// # 📤 Returns
    ///
    /// A new instance of the time emitter
    ///
    /// # 🌐 WASM Behavior
    ///
    /// This method uses JavaScript's `setInterval` to schedule periodic events.
    /// The interval ID is stored for later cleanup.
    fn start<F>(settings: Self::Settings, mut on_emit: F) -> Self
    where
        F: 'static + FnMut(Time, EmitterContext<Self::Settings>) -> (),
    {
        let event_count = Arc::new(Mutex::new(0));
        let thread_event_count = Arc::clone(&event_count);

        let interval_ms = settings.interval().as_millis() as i32;

        let closure = Closure::wrap(Box::new(move || {
            let mut current_count = thread_event_count.lock().unwrap();

            if let Some(max_events) = settings.max_events() {
                if *current_count >= max_events {
                    // The interval will continue to fire, but the callback will do nothing.
                    // The consumer of TimeEmitter should call .stop() or drop the emitter
                    // to clean up the interval.
                    return;
                }
            }

            let index = *current_count;
            let time = Time::now().to(settings.kind());

            on_emit(time, EmitterContext { index, settings });

            *current_count += 1;
        }) as Box<dyn FnMut()>);

        match { set_interval(closure.as_ref().unchecked_ref::<Function>(), interval_ms) } {
            Ok(handle) => Self {
                settings,
                closure: Arc::new(Mutex::new(closure)),
                handle: Arc::new(Mutex::new(Ok(handle))),
            },
            Err(err) => {
                eprintln!("{:?}", err);
                Self {
                    settings,
                    closure: Arc::new(Mutex::new(closure)),
                    handle: Arc::new(Mutex::new(Err(Error::StartError(err)))),
                }
            }
        }
    }

    /// ⚙️ Gets a reference to the settings of the time emitter.
    ///
    /// This method provides read access to the settings used to configure this emitter.
    ///
    /// # 📤 Returns
    ///
    /// A reference to the settings
    fn settings(&self) -> &Self::Settings {
        &self.settings
    }

    /// 🛑 Stops the time emitter.
    ///
    /// This method clears the JavaScript interval, preventing any further time events
    /// from being emitted. It uses the browser's `clearInterval` API.
    ///
    /// # 📤 Returns
    ///
    /// A Result indicating success or failure of the stop operation.
    /// If the browser window cannot be accessed, an error is returned.
    fn stop(&mut self) -> Result<JsValue, Error> {
        if let Ok(mut guard) = self.handle.lock() {
            // Take the value, leaving a placeholder.
            match std::mem::replace(&mut *guard, Err(Error::StopError)) {
                Ok(handle) => Ok(clear_interval(handle)),
                Err(start_err) => {
                    // It failed to start, so nothing to stop.
                    // Put the original error back.
                    *guard = Err(start_err);
                    Err(Error::StopError)
                }
            }
        } else {
            eprintln!("Failed to acquire lock on interval");
            Err(Error::StopError)
        }
    }

    /// ⏳ Implementation of await_completion for WASM.
    ///
    /// In WASM environment, there are no threads to join, so this is a no-op.
    /// This method is provided for API compatibility with the standard implementation.
    ///
    /// # 📤 Returns
    ///
    /// Always returns `Ok(())` since there's nothing to wait for in the WASM environment.
    ///
    /// # 📝 Notes
    ///
    /// This is primarily for cross-platform compatibility with the standard implementation,
    /// which has a meaningful implementation of this method to join the background thread.
    fn await_completion(&mut self) -> Result<JsValue, Error> {
        // No-op in WASM environment
        Ok(JsValue::undefined())
    }
}

// ⚙️ Define Settings for WASM time emitter

/// ⚙️ Settings for configuring a WebAssembly environment time emitter.
///
/// This struct implements the [`TimeEmitterSettingsTrait`] trait to provide
/// configuration options for the time emitter, such as the interval between
/// events, the maximum number of events, and the time kind.
///
/// Note that this uses `web_time::Duration` rather than `std::time::Duration`
/// since the latter is not fully supported in WebAssembly environments.
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

// ❌ Error ----------------------------------------------------------------------- /

/// ❌ An error that can occur when using the TimeEmitter in a WebAssembly environment.
///
/// This enum represents the different types of errors that can occur
/// when working with a time emitter in a WebAssembly environment.
#[derive(Debug)]
pub enum Error {
    StartError(JsValue),
    /// 🛑 Error that occurred while trying to stop the emitter
    /// (typically when the browser window cannot be accessed)
    StopError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::StartError(err) => write!(f, "Error starting time emitter: {:?}", err),
            Error::StopError => write!(f, "Error stopping time emitter."),
        }
    }
}

impl std::error::Error for Error {}

// 🧪 Tests --------------------------------------------------------------------------- /

#[cfg(test)]
mod tests {
    use super::*;
    use gloo_timers::future::TimeoutFuture;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;
    use web_time::Duration;

    // Configure tests to run asynchronously
    wasm_bindgen_test_configure!(run_in_browser);

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

    #[wasm_bindgen_test]
    fn test_time_emitter_creation() {
        let time_emitter = WebEmitter::start(Settings::default(), |_, _| {});
        assert_eq!(time_emitter.settings().max_events(), None);
        assert_eq!(time_emitter.settings().interval(), Duration::from_secs(1));
    }

    #[wasm_bindgen_test]
    fn test_time_emitter_custom_settings() {
        let settings = Settings::new()
            .set_max_events(5)
            .set_interval(Duration::from_millis(100))
            .set_kind(TimeKind::Base10);

        let time_emitter = WebEmitter::start(settings, |_, _| {});
        assert_eq!(time_emitter.settings().max_events(), Some(5));
        assert_eq!(
            time_emitter.settings().interval(),
            Duration::from_millis(100)
        );
        assert_eq!(time_emitter.settings().kind(), TimeKind::Base10);
    }

    #[wasm_bindgen_test]
    async fn test_emitter_max_events() {
        let current_time = Arc::new(Mutex::new(None as Option<Time>));
        let event_count = Arc::new(Mutex::new(0 as u64));
        let max_events: u64 = 3; // Reduced to very small number

        let settings = Settings::new()
            .set_max_events(max_events)
            .set_interval(Duration::from_millis(5)) // Reduced interval for faster execution
            .set_kind(TimeKind::Base10);

        let callback_current_time = Arc::clone(&current_time);
        let callback_event_count = Arc::clone(&event_count);

        let mut time_emitter = WebEmitter::start(settings, move |time, _| {
            let mut current_count = callback_event_count.lock().unwrap();
            *current_count += 1;
            let mut current_time = callback_current_time.lock().unwrap();
            *current_time = Some(time);
        });

        // Let it run just long enough for a few events
        TimeoutFuture::new(20).await;

        // Stop the emitter after waiting
        time_emitter
            .stop()
            .expect("Should be able to stop the emitter");

        time_emitter
            .await_completion()
            .expect("Should be able to await completion");

        // Check results - Get value then release lock immediately
        let final_time_value = {
            let guard = current_time.lock().unwrap();
            guard.clone()
        };

        let final_count = {
            let guard = event_count.lock().unwrap();
            *guard
        };

        assert!(final_time_value.is_some(), "Time should have been emitted");
        if let Some(time) = final_time_value {
            assert_eq!(
                time.kind(),
                settings.kind,
                "Time should have the correct kind"
            );
        }
        assert!(
            final_count <= max_events,
            "Event count should not exceed max_events"
        );
    }

    #[wasm_bindgen_test]
    async fn test_emitter_unsubscribe() {
        let initial_event_count: u64 = 0;
        let event_count = Arc::new(Mutex::new(initial_event_count));
        let callback_event_count = Arc::clone(&event_count);

        let mut time_emitter = WebEmitter::start(
            Settings::new().set_interval(Duration::from_millis(10)), // Use very short interval
            move |_, _| {
                let mut current_count = callback_event_count.lock().unwrap();
                *current_count += 1;
            },
        );

        // Let it run for a short time - just enough for one event
        TimeoutFuture::new(15).await;

        // Stop it
        time_emitter
            .stop()
            .expect("Should be able to stop the emitter");

        // Get count and release lock immediately
        let count_at_stop = {
            let guard = event_count.lock().unwrap();
            *guard
        };

        // Verify count (should be at least 1, but not more than 2 given timing)
        assert!(
            count_at_stop >= initial_event_count + 1,
            "Should have at least one event"
        );
        assert!(
            count_at_stop <= initial_event_count + 2,
            "Should not have too many events"
        );
    }

    #[wasm_bindgen_test]
    async fn test_context_values() {
        let expected_index = Arc::new(Mutex::new(0));
        let settings = Settings::new()
            .set_max_events(3)
            .set_interval(Duration::from_millis(10));

        let mut time_emitter = WebEmitter::start(settings, move |_, ctx| {
            let mut expected_index = expected_index.lock().unwrap();
            assert_eq!(ctx.index, *expected_index);
            assert_eq!(ctx.settings.max_events(), Some(3));
            assert_eq!(ctx.settings.interval(), Duration::from_millis(10));
            *expected_index += 1;
        });

        TimeoutFuture::new(50).await;

        time_emitter
            .stop()
            .expect("Should be able to stop the emitter");
    }

    #[wasm_bindgen_test]
    async fn test_async_operation() {
        let event_count = Arc::new(Mutex::new(0));
        let event_count_clone = Arc::clone(&event_count);

        let mut time_emitter = WebEmitter::start(
            Settings::new()
                .set_max_events(3)
                .set_interval(Duration::from_millis(10)),
            move |_, _| {
                let mut count = event_count_clone.lock().unwrap();
                *count += 1;
            },
        );

        TimeoutFuture::new(50).await;

        time_emitter
            .stop()
            .expect("Should be able to stop the emitter");

        assert_eq!(*event_count.lock().unwrap(), 3);
    }
}

//! This module provides the core clock functionality.
//!
//! # Components
//!
//! - `ClockSettings`: Configuration struct for clock behavior
//! - `ClockError`: Error types for clock operations
//! - `ClockTrait`: Core trait defining clock behavior
//!
//! # Examples
//!
//! ```
//! use crate::clock::{ClockSettings, ClockTrait};
//! use std::time::Duration;
//!
//! let settings = ClockSettings::new()
//!     .set_interval(Duration::from_secs(1));
//!
//! // Implement ClockTrait for your custom clock
//! // Then use settings to configure it
//! ```

use std::{error::Error, fmt::Display, time::Duration};

use crate::time_lib::TimeKind;

// ClockSetup ----------------------------------------------------------------------- /

/// The `ClockSettings` type provides configuration options for clock behavior.
///
/// # Settings
///
/// - `kind`: The time base system to use (e.g. Base24)
/// - `interval`: The update interval for the clock
///
/// # Example
///
/// ```
/// use crate::clock::ClockSettings;
/// use std::time::Duration;
/// use crate::time_lib::TimeKind;
///
/// let settings = ClockSettings::new()
///     .set_kind(TimeKind::Base24)
///     .set_interval(Duration::from_secs(1));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ClockSettings {
    pub kind: TimeKind,
    pub interval: Duration,
}

impl ClockSettings {
    /// Returns default settings for the clock
    ///
    /// # Returns
    ///
    /// A `ClockSettings` instance with:
    /// - `kind`: TimeKind::Base24
    /// - `interval`: 1 second
    pub fn defaults() -> Self {
        Self {
            kind: TimeKind::Base24,
            interval: Duration::from_secs(1),
        }
    }

    /// Creates a new `ClockSettings` instance with default values
    ///
    /// # Returns
    ///
    /// A `ClockSettings` instance with default settings
    pub fn new() -> Self {
        Self::defaults()
    }

    /// Sets the time base system kind
    ///
    /// # Arguments
    ///
    /// * `kind` - The TimeKind to use (e.g. Base24)
    ///
    /// # Returns
    ///
    /// Self with updated kind
    pub fn set_kind(mut self, kind: TimeKind) -> Self {
        self.kind = kind;
        self
    }

    /// Sets the clock update interval
    ///
    /// # Arguments
    ///
    /// * `interval` - The Duration between clock updates
    ///
    /// # Returns
    ///
    /// Self with updated interval
    pub fn set_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }
}

// ClockError ----------------------------------------------------------------------- /

#[derive(Debug)]
pub enum ClockError {
    NoTimeSet,
    CouldNotSetTime,
    CouldNotSetTimeEmitter,
    CouldNotUnsubscribe,
}

impl Error for ClockError {}

impl Display for ClockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClockError::NoTimeSet => write!(f, "No time set"),
            ClockError::CouldNotSetTime => write!(f, "Could not set time"),
            ClockError::CouldNotSetTimeEmitter => write!(f, "Could not set time emitter"),
            ClockError::CouldNotUnsubscribe => write!(f, "Could not unsubscribe"),
        }
    }
}

/*
🧪 Tests --------------------------------------------------------------------------- /
*/

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = ClockSettings::defaults();
        assert_eq!(settings.kind, TimeKind::Base24);
        assert_eq!(settings.interval, Duration::from_secs(1));
    }

    #[test]
    fn test_new_settings() {
        let settings = ClockSettings::new();
        assert_eq!(settings.kind, TimeKind::Base24);
        assert_eq!(settings.interval, Duration::from_secs(1));
    }

    #[test]
    fn test_set_kind() {
        let settings = ClockSettings::new().set_kind(TimeKind::Base24);
        assert_eq!(settings.kind, TimeKind::Base24);
    }

    #[test]
    fn test_set_interval() {
        let interval = Duration::from_millis(500);
        let settings = ClockSettings::new().set_interval(interval);
        assert_eq!(settings.interval, interval);
    }

    #[test]
    fn test_chain_settings() {
        let interval = Duration::from_millis(100);
        let settings = ClockSettings::new()
            .set_kind(TimeKind::Base24)
            .set_interval(interval);

        assert_eq!(settings.kind, TimeKind::Base24);
        assert_eq!(settings.interval, interval);
    }
}

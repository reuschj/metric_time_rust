//! Time Management Module
//!
//! This module provides functionality for working with different time formats and conversions
//! between them. It supports three main time formats:
//!
//! - Base10 (Metric time)
//! - Base12 (12-hour format with AM/PM)
//! - Base24 (24-hour format)
//!
//! # Examples
//!
//! ```
//! use crate::time::{Time, TimeComponents, TimeKind};
//!
//! // Create a new time in 24-hour format
//! let time = Time::new(
//!     TimeComponents {
//!         hours: 9,
//!         minutes: 23,
//!         seconds: 56,
//!         nanoseconds: 9_234_234
//!     },
//!     TimeKind::Base24
//! ).unwrap();
//!
//! // Convert to metric time
//! let metric_time = time.to(TimeKind::Base10);
//! ```
//!
//! # Features
//!
//! - Create time instances in different formats
//! - Convert between time formats
//! - Get current time
//! - Calculate clock hand rotations
//! - Compare time values
//! - Format time as strings
//!
//! The module provides robust error handling for invalid time values and
//! implements standard traits like Display, Clone, and comparison operators.

use chrono::{Local, NaiveTime, Timelike};
use std::fmt::{Display, Error, Formatter};

use crate::time_helpers;
use crate::time_lib::{
    Period, TimeBounds, TimeComponents, TimeConversionTrait, TimeKind, TimeRangeError,
    TimeRotationComponents,
};

// Time --------------------------------------------------------------------------- /

/// A type representing time in various formats (Base10, Base12, and Base24).
///
/// # Examples
///
/// ```
/// use crate::time::{Time, TimeComponents, TimeKind};
///
/// // Create a new time in 24-hour format
/// let time = Time::new(
///     TimeComponents {
///         hours: 14,
///         minutes: 30,
///         seconds: 0,
///         nanoseconds: 0
///     },
///     TimeKind::Base24
/// ).unwrap();
///
/// // Convert to 12-hour format
/// let time_12 = time.to(TimeKind::Base12(Period::PM));
/// assert_eq!(time_12.hours(), 2);
///
/// // Convert to metric time (Base10)
/// let metric = time.to(TimeKind::Base10);
/// ```
///
/// # Features
///
/// - Create time values in any supported format
/// - Convert between formats while preserving the exact time
/// - Access individual components (hours, minutes, seconds, nanoseconds)
/// - Calculate clock hand rotations
/// - Compare and order time values
/// - Format time as strings
///
/// # Properties
///
/// - `components`: The time components (hours, minutes, seconds, nanoseconds)
/// - `kind`: The time format (Base10, Base12, or Base24)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    components: TimeComponents,
    kind: TimeKind,
}

impl Time {
    /// Creates a new Time instance with the specified components and time format.
    ///
    /// # Arguments
    /// * `components` - The time components (hours, minutes, seconds, nanoseconds)
    /// * `kind` - The time format (Base10, Base12, or Base24)
    ///
    /// # Returns
    /// * `Result<Self, TimeRangeError>` - Ok with the new Time instance if valid, Err if components are out of bounds
    pub fn new(components: TimeComponents, kind: TimeKind) -> Result<Self, TimeRangeError> {
        match TimeBounds::new(kind).check(components) {
            Ok(components) => Ok(Self { components, kind }),
            Err(err) => Err(err),
        }
    }

    /// Creates a new Time instance in Base10 (metric) format.
    ///
    /// # Arguments
    /// * `components` - The time components
    ///
    /// # Returns
    /// * `Result<Self, TimeRangeError>` - Ok with metric time if valid, Err if out of bounds
    pub fn base10(components: TimeComponents) -> Result<Self, TimeRangeError> {
        Self::new(components, TimeKind::Base10)
    }

    /// Creates a new Time instance in Base12 (12-hour AM/PM) format.
    ///
    /// # Arguments
    /// * `components` - The time components
    /// * `period` - AM or PM indicator
    ///
    /// # Returns
    /// * `Result<Self, TimeRangeError>` - Ok with 12-hour time if valid, Err if out of bounds
    pub fn base12(components: TimeComponents, period: Period) -> Result<Self, TimeRangeError> {
        Self::new(components, TimeKind::Base12(period))
    }

    /// Creates a new Time instance in Base24 (24-hour) format.
    ///
    /// # Arguments
    /// * `components` - The time components
    ///
    /// # Returns
    /// * `Result<Self, TimeRangeError>` - Ok with 24-hour time if valid, Err if out of bounds
    pub fn base24(components: TimeComponents) -> Result<Self, TimeRangeError> {
        Self::new(components, TimeKind::Base24)
    }

    /// Creates a Time instance representing the current local time.
    ///
    /// # Returns
    /// * `Self` - Time instance in 24-hour format for current time
    pub fn now() -> Self {
        Self::from(Local::now().time())
    }

    // -------------------------------------------- /

    /// Returns the time components of this Time instance.
    ///
    /// # Returns
    /// * `TimeComponents` - Copy of the internal time components
    pub fn components(&self) -> TimeComponents {
        self.components
    }

    /// Returns the time format of this Time instance.
    ///
    /// # Returns
    /// * `TimeKind` - The format (Base10, Base12, or Base24)
    pub fn kind(&self) -> TimeKind {
        self.kind
    }

    // -------------------------------------------- /

    /// Returns the hours component of the time.
    ///
    /// # Returns
    /// * `u8` - Hours value
    pub fn hours(&self) -> u8 {
        self.components.hours
    }

    /// Returns the minutes component of the time.
    ///
    /// # Returns
    /// * `u8` - Minutes value
    pub fn minutes(&self) -> u8 {
        self.components.minutes
    }

    /// Returns the seconds component of the time.
    ///
    /// # Returns
    /// * `u8` - Seconds value
    pub fn seconds(&self) -> u8 {
        self.components.seconds
    }

    /// Returns the nanoseconds component of the time.
    ///
    /// # Returns
    /// * `u32` - Nanoseconds value
    pub fn nanoseconds(&self) -> u32 {
        self.components.nanoseconds
    }

    // -------------------------------------------- /

    /// Calculates the rotation angles for clock hands based on the current time.
    ///
    /// # Returns
    /// * `TimeRotationComponents` - Rotation angles for hours, minutes, seconds, and nanoseconds
    pub fn rotations(&self) -> TimeRotationComponents {
        TimeRotationComponents::new(self.components, self.kind)
    }
}

impl From<NaiveTime> for Time {
    /// Converts a chrono::NaiveTime into a Time instance.
    ///
    /// Creates a new Time in 24-hour format from the provided NaiveTime value.
    ///
    /// # Arguments
    /// * `value` - The NaiveTime to convert from
    ///
    /// # Returns
    /// * `Time` - A new Time instance in Base24 format
    ///
    /// # Examples
    /// ```
    /// use chrono::NaiveTime;
    /// use crate::time::Time;
    ///
    /// let naive_time = NaiveTime::from_hms(14, 30, 0);
    /// let time = Time::from(naive_time);
    /// assert_eq!(time.hours(), 14);
    /// assert_eq!(time.minutes(), 30);
    /// ```
    fn from(value: NaiveTime) -> Self {
        let components = TimeComponents {
            hours: value.hour() as u8,
            minutes: value.minute() as u8,
            seconds: value.second() as u8,
            nanoseconds: value.nanosecond(),
        };
        Self::new(components, TimeKind::Base24).unwrap()
    }
}

impl TimeConversionTrait for Time {
    /// Converts this Time to another time format.
    ///
    /// This method allows conversion between different time formats while preserving
    /// the exact time value. It handles all combinations of conversions between:
    ///
    /// - Base10 (Metric time)
    /// - Base12 (12-hour with AM/PM)
    /// - Base24 (24-hour)
    ///
    /// # Arguments
    /// * `kind` - The target time format to convert to
    ///
    /// # Returns
    /// * `Self` - A new Time instance in the requested format
    ///
    /// # Examples
    /// ```
    /// use crate::time::{Time, TimeComponents, TimeKind};
    ///
    /// let time_24 = Time::base24(TimeComponents::new(14, 30, 0, 0)).unwrap();
    ///
    /// // Convert to 12-hour format
    /// let time_12 = time_24.to(TimeKind::Base12(Period::PM));
    /// assert_eq!(time_12.hours(), 2);
    ///
    /// // Convert to metric time
    /// let time_10 = time_24.to(TimeKind::Base10);
    /// ```
    fn to(&self, kind: TimeKind) -> Self {
        match self.kind {
            TimeKind::Base10 => match kind {
                TimeKind::Base10 => self.clone(),
                TimeKind::Base12(_) => {
                    let components_24 =
                        time_helpers::conversion_utils::base10_to_base24(&self.components);
                    let (components, period) =
                        time_helpers::conversion_utils::base24_to_base12(&components_24);
                    Time::base12(components, period).unwrap()
                }
                TimeKind::Base24 => {
                    let components =
                        time_helpers::conversion_utils::base10_to_base24(&self.components);
                    Time::base24(components).unwrap()
                }
            },
            TimeKind::Base12(period) => match kind {
                TimeKind::Base10 => {
                    let components_24 =
                        time_helpers::conversion_utils::base12_to_base24(&self.components, &period);
                    let components =
                        time_helpers::conversion_utils::base24_to_base10(&components_24);
                    Time::base10(components).unwrap()
                }
                TimeKind::Base12(_) => self.clone(),
                TimeKind::Base24 => {
                    let components =
                        time_helpers::conversion_utils::base12_to_base24(&self.components, &period);
                    Time::base24(components).unwrap()
                }
            },
            TimeKind::Base24 => match kind {
                TimeKind::Base10 => {
                    let components =
                        time_helpers::conversion_utils::base24_to_base10(&self.components);
                    Time::base10(components).unwrap()
                }
                TimeKind::Base12(_) => {
                    let (components, period) =
                        time_helpers::conversion_utils::base24_to_base12(&self.components);
                    Time::base12(components, period).unwrap()
                }
                TimeKind::Base24 => self.clone(),
            },
        }
    }
}

impl Display for Time {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        match self.kind {
            TimeKind::Base10 => write!(formatter, "Metric: {}", self.components),
            TimeKind::Base12(period) => write!(formatter, "{} {}", self.components, period),
            TimeKind::Base24 => write!(formatter, "{}", self.components),
        }
    }
}

/*
🧪 Tests --------------------------------------------------------------------------- /
*/

#[cfg(test)]
mod tests {
    use chrono::NaiveTime;

    use crate::{
        time::Time,
        time_lib::{TimeComponents, TimeConversionTrait, TimeKind},
    };

    #[test]
    fn it_creates_a_new_time_and_converts_to_metric() {
        let time_components = TimeComponents {
            hours: 9,
            minutes: 23,
            seconds: 56,
            nanoseconds: 9_234_234,
        };
        let expected_time_components = TimeComponents {
            hours: 3,
            minutes: 91,
            seconds: 62,
            nanoseconds: 47_724_807,
        };
        let base_time_result = Time::new(time_components, TimeKind::Base24);
        assert!(base_time_result.is_ok());
        let base_time = base_time_result.unwrap();
        assert_eq!(base_time.components(), time_components);
        assert_eq!(base_time.kind(), TimeKind::Base24);
        assert_eq!(base_time.hours(), time_components.hours);
        assert_eq!(base_time.minutes(), time_components.minutes);
        assert_eq!(base_time.seconds(), time_components.seconds);
        assert_eq!(base_time.nanoseconds(), time_components.nanoseconds);
        assert_eq!(format!("{}", base_time), "9:23:56.9234234");
        let metric_time = base_time.to(TimeKind::Base10);
        assert_eq!(metric_time.components(), expected_time_components);
        assert_eq!(metric_time.kind(), TimeKind::Base10);
        assert_eq!(metric_time.hours(), expected_time_components.hours);
        assert_eq!(metric_time.minutes(), expected_time_components.minutes);
        assert_eq!(metric_time.seconds(), expected_time_components.seconds);
        assert_eq!(
            metric_time.nanoseconds(),
            expected_time_components.nanoseconds
        );
        assert_eq!(format!("{}", metric_time), "Metric: 3:91:62.47724807");
    }

    #[test]
    fn it_creates_a_new_metric_time_for_now() {
        let base = TimeComponents {
            hours: 9,
            minutes: 23,
            seconds: 56,
            nanoseconds: 9_234_234,
        };
        let naive_time = NaiveTime::from_hms_nano_opt(
            base.hours as u32,
            base.minutes as u32,
            base.seconds as u32,
            base.nanoseconds,
        )
        .unwrap();
        let expected_time_components = TimeComponents {
            hours: 3,
            minutes: 91,
            seconds: 62,
            nanoseconds: 47_724_807,
        };
        let metric_time = Time::from(naive_time).to(TimeKind::Base10);
        assert_eq!(metric_time.components(), expected_time_components);
        assert_eq!(metric_time.kind(), TimeKind::Base10);
        assert_eq!(metric_time.hours(), expected_time_components.hours);
        assert_eq!(metric_time.minutes(), expected_time_components.minutes);
        assert_eq!(metric_time.seconds(), expected_time_components.seconds);
        assert_eq!(
            metric_time.nanoseconds(),
            expected_time_components.nanoseconds
        );
        assert_eq!(format!("{}", metric_time), "Metric: 3:91:62.47724807");
    }

    #[test]
    fn it_allows_comparison_and_equality_checks() {
        let time_01 = Time::new(
            TimeComponents {
                hours: 9,
                minutes: 23,
                seconds: 56,
                nanoseconds: 9_234_234,
            },
            TimeKind::Base24,
        )
        .unwrap();
        let time_02 = Time::new(
            TimeComponents {
                hours: 9,
                minutes: 23,
                seconds: 56,
                nanoseconds: 9_234_234,
            },
            TimeKind::Base24,
        )
        .unwrap();
        let time_03 = Time::new(
            TimeComponents {
                hours: 9,
                minutes: 45,
                seconds: 21,
                nanoseconds: 2_232_342,
            },
            TimeKind::Base24,
        )
        .unwrap();
        assert!(time_01 == time_02);
        assert!(time_01 <= time_02);
        assert!(time_01 >= time_02);
        assert!(time_01 < time_03);
        assert!(time_03 > time_02);
    }

    #[test]
    fn it_calculates_clock_hand_rotations() {
        let metric_time = Time::new(
            TimeComponents::new(2, 45, 23, 234_000_000),
            TimeKind::Base10,
        )
        .unwrap();
        let rotations = metric_time.rotations();
        fn round_float(num: f64) -> f64 {
            (num * 100.00).round() / 100.00
        }
        assert_eq!(round_float(rotations.hours()), 73.63);
        assert_eq!(round_float(rotations.minutes()), 162.84);
        assert_eq!(round_float(rotations.seconds()), 83.64);
        assert_eq!(round_float(rotations.nanoseconds()), 84.24);
    }
}

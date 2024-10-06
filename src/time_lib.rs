//! Time management utilities and components for time conversion and display.
//!
//! This module provides various time-related types and utilities:
//!
//! - `Period`: Represents AM/PM periods for 12-hour time
//! - `TimeKind`: Different time base formats (10, 12, or 24 hour)
//! - `TimeComponents`: Core time unit storage (hours, minutes, seconds, nanoseconds)
//! - `TimeBounds`: Valid ranges for time components in different formats
//! - `TimeRotationComponents`: Calculates rotational angles for clock hands
//!
//! # Examples
//!
//! ```
//! use crate::time_lib::{TimeComponents, TimeKind, Period};
//!
//! // Create time components
//! let time = TimeComponents::new(11, 30, 0, 0);
//!
//! // Check if valid for 12-hour format
//! let bounds = TimeBounds::new(TimeKind::Base12(Period::AM));
//! assert!(bounds.check(time).is_ok());
//!
//! // Convert to rotation angles
//! let rotations = TimeRotationComponents::new(time, TimeKind::Base12(Period::AM));
//! ```

use std::fmt::{Display, Error, Formatter};
use std::ops::Range;

use crate::constants::{FULL_CIRCLE_DEGREES, NS_PER_SEC};
use crate::time_helpers::time_conversions::TimeConversions;

// Period --------------------------------------------------------------------------- /

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Period {
    AM,
    PM,
}

impl Display for Period {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        let description = match self {
            Period::AM => "AM",
            Period::PM => "PM",
        };
        write!(formatter, "{}", description)
    }
}

// Time Kind --------------------------------------------------------------------------- /

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeKind {
    Base10,
    Base12(Period),
    Base24,
}

impl TimeKind {}

impl Display for TimeKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        let description = match self {
            TimeKind::Base10 => String::from("Metric"),
            TimeKind::Base12(period) => format!("Standard {}", period),
            TimeKind::Base24 => String::from("24 hour"),
        };
        write!(formatter, "{}", description)
    }
}

impl From<&TimeKind> for TimeConversions {
    fn from(value: &TimeKind) -> Self {
        match value {
            TimeKind::Base10 => TimeConversions::metric(),
            _ => TimeConversions::standard(),
        }
    }
}

// TimeComponents --------------------------------------------------------------------------- /

/// Represents time components for hours, minutes, seconds, and nanoseconds.
///
/// This struct provides a way to store and manipulate time values across different time bases (10, 12, 24 hour).
///
/// # Examples
///
/// ```
/// use time_lib::TimeComponents;
///
/// let time = TimeComponents::new(11, 30, 0, 0);
/// assert_eq!(time.hours, 11);
/// assert_eq!(time.minutes, 30);
/// assert_eq!(time.seconds, 0);
/// assert_eq!(time.nanoseconds, 0);
/// ```
///
/// Time components can be used with different time bases:
///
/// - Base 10 (Metric): Hours 0-9, Minutes/Seconds 0-99
/// - Base 12 (AM/PM): Hours 1-12, Minutes/Seconds 0-59
/// - Base 24: Hours 0-23, Minutes/Seconds 0-59
///
/// Use [`TimeBounds`] to validate components for a specific time base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeComponents {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub nanoseconds: u32,
}

impl TimeComponents {
    /// Creates a new `TimeComponents` instance with the specified time values.
    ///
    /// # Arguments
    ///
    /// * `hours` - The hours value (range depends on time base)
    /// * `minutes` - The minutes value (range depends on time base)
    /// * `seconds` - The seconds value (range depends on time base)
    /// * `nanoseconds` - The nanoseconds value (0-999,999,999)
    ///
    /// # Examples
    ///
    /// ```
    /// use time_lib::TimeComponents;
    ///
    /// let time = TimeComponents::new(11, 30, 45, 0);
    /// ```
    pub fn new(hours: u8, minutes: u8, seconds: u8, nanoseconds: u32) -> Self {
        Self {
            hours,
            minutes,
            seconds,
            nanoseconds,
        }
    }
}

impl Display for TimeComponents {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            formatter,
            "{}:{:0>2}:{:0>2}.{}",
            self.hours.to_string(),
            self.minutes.to_string(),
            self.seconds.to_string(),
            self.nanoseconds.to_string()
        )
    }
}

// TimeRanges --------------------------------------------------------------------------- /

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRangeError {
    HoursLow,
    HoursHigh,
    MintuesLow,
    MintuesHigh,
    SecondsLow,
    SecondsHigh,
    NanosecondsLow,
    NanosecondsHigh,
}

impl std::error::Error for TimeRangeError {}

impl Display for TimeRangeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        let description = match self {
            TimeRangeError::HoursLow => "Hours under bounds",
            TimeRangeError::HoursHigh => "Hours over bounds",
            TimeRangeError::MintuesLow => "Mintues under bounds",
            TimeRangeError::MintuesHigh => "Mintues over bounds",
            TimeRangeError::SecondsLow => "Seconds under bounds",
            TimeRangeError::SecondsHigh => "Seconds over bounds",
            TimeRangeError::NanosecondsLow => "Nanoseconds under bounds",
            TimeRangeError::NanosecondsHigh => "Nanoseconds over bounds",
        };
        write!(formatter, "{}", description)
    }
}

// TimeBounds --------------------------------------------------------------------------- /

/// Defines valid ranges for time components in different time formats.
///
/// This struct provides bounds checking for time values in various bases:
///
/// - Base 10 (Metric): Hours 0-9, Minutes/Seconds 0-99
/// - Base 12 (AM/PM): Hours 1-12, Minutes/Seconds 0-59
/// - Base 24: Hours 0-23, Minutes/Seconds 0-59
///
/// # Examples
///
/// ```
/// use time_lib::{TimeBounds, TimeComponents, TimeKind};
///
/// let bounds = TimeBounds::new(TimeKind::Base24);
/// let time = TimeComponents::new(13, 30, 0, 0);
///
/// // Check if time is valid for 24-hour format
/// assert!(bounds.check(time).is_ok());
/// ```
///
/// The struct enforces valid ranges through the `check()` method which returns
/// a Result indicating if the time components are valid for the specified format.
#[derive(Debug, Clone)]
pub struct TimeBounds {
    pub hours: Range<u8>,
    pub minutes: Range<u8>,
    pub seconds: Range<u8>,
    pub nanoseconds: Range<u32>,
}

impl TimeBounds {
    /// Creates new time bounds for a specific time format.
    ///
    /// This establishes the valid ranges for hours, minutes, seconds and nanoseconds
    /// based on the time format specified:
    ///
    /// - Base 10 (Metric): Hours 0-9, Minutes/Seconds 0-99
    /// - Base 12 (AM/PM): Hours 1-12, Minutes/Seconds 0-59
    /// - Base 24: Hours 0-23, Minutes/Seconds 0-59
    ///
    /// # Arguments
    ///
    /// * `kind` - The time format to create bounds for (Base10, Base12, or Base24)
    ///
    /// # Examples
    ///
    /// ```
    /// use time_lib::{TimeBounds, TimeKind, Period};
    ///
    /// // Create bounds for 12-hour format
    /// let bounds = TimeBounds::new(TimeKind::Base12(Period::AM));
    ///
    /// // Create bounds for 24-hour format
    /// let bounds = TimeBounds::new(TimeKind::Base24);
    ///
    /// // Create bounds for metric time
    /// let bounds = TimeBounds::new(TimeKind::Base10);
    /// ```
    pub fn new(kind: TimeKind) -> Self {
        match kind {
            TimeKind::Base10 => Self {
                hours: 0..10,
                minutes: 0..100,
                seconds: 0..100,
                nanoseconds: 0..NS_PER_SEC,
            },
            TimeKind::Base12(_) => Self {
                hours: 1..13,
                minutes: 0..60,
                seconds: 0..60,
                nanoseconds: 0..NS_PER_SEC,
            },
            TimeKind::Base24 => Self {
                hours: 0..24,
                minutes: 0..60,
                seconds: 0..60,
                nanoseconds: 0..NS_PER_SEC,
            },
        }
    }

    /// Validates time components against the bounds for this time format.
    ///
    /// Checks if the provided time components fall within the valid ranges for hours,
    /// minutes, seconds and nanoseconds defined by these bounds.
    ///
    /// # Arguments
    ///
    /// * `components` - The time components to validate
    ///
    /// # Returns
    ///
    /// * `Ok(TimeComponents)` - If all components are within bounds
    /// * `Err(TimeRangeError)` - If any component is outside its valid range
    ///
    /// # Examples
    ///
    /// ```
    /// use time_lib::{TimeBounds, TimeComponents, TimeKind};
    ///
    /// let bounds = TimeBounds::new(TimeKind::Base24);
    /// let time = TimeComponents::new(13, 30, 0, 0);
    ///
    /// assert!(bounds.check(time).is_ok());
    /// ```
    pub fn check(&self, components: TimeComponents) -> Result<TimeComponents, TimeRangeError> {
        if !self.hours.contains(&components.hours) {
            if components.hours < self.hours.start {
                return Err(TimeRangeError::HoursLow);
            } else {
                return Err(TimeRangeError::HoursHigh);
            }
        }
        if !self.minutes.contains(&components.minutes) {
            if components.minutes < self.minutes.start {
                return Err(TimeRangeError::MintuesLow);
            } else {
                return Err(TimeRangeError::MintuesHigh);
            }
        }
        if !self.seconds.contains(&components.seconds) {
            if components.seconds < self.seconds.start {
                return Err(TimeRangeError::SecondsLow);
            } else {
                return Err(TimeRangeError::SecondsHigh);
            }
        }
        if !self.nanoseconds.contains(&components.nanoseconds) {
            if components.nanoseconds < self.nanoseconds.start {
                return Err(TimeRangeError::NanosecondsLow);
            } else {
                return Err(TimeRangeError::NanosecondsHigh);
            }
        }
        Ok(components)
    }
}

// TimeRotationComponents --------------------------------------------------------------------------- /

/// Represents rotational angles for clock hands.
///
/// This struct calculates the rotational angles (in degrees) for clock hands based on time components
/// and the specified time format (10, 12, or 24 hour).
///
/// The angles are calculated as:
/// - Hours hand: 0-360 degrees for a full rotation
/// - Minutes hand: 0-360 degrees for a full rotation
/// - Seconds hand: 0-360 degrees for a full rotation
/// - Nanoseconds hand: 0-360 degrees for a full rotation
///
/// The rotations take into account fractional components, so the hands move smoothly
/// rather than jumping between positions.
///
/// # Examples
///
/// ```
/// use time_lib::{TimeComponents, TimeKind, TimeRotationComponents};
///
/// let time = TimeComponents::new(3, 30, 0, 0);
/// let rotations = TimeRotationComponents::new(time, TimeKind::Base24);
///
/// // Hour hand will be at ~105 degrees (3/24 * 360 + small offset for minutes)
/// assert!(rotations.hours() > 105.0 && rotations.hours() < 106.0);
///
/// // Minute hand will be at 180 degrees (30/60 * 360)
/// assert_eq!(rotations.minutes(), 180.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TimeRotationComponents {
    hours: f64,
    minutes: f64,
    seconds: f64,
    nanoseconds: f64,
}

impl TimeRotationComponents {
    /// Creates a new `TimeRotationComponents` from time components and format.
    ///
    /// Calculates the rotational angles for clock hands based on the provided time
    /// components and time format. The angles are normalized to degrees (0-360)
    /// and take into account fractional components for smooth movement.
    ///
    /// # Arguments
    ///
    /// * `components` - The time components to convert to rotations
    /// * `kind` - The time format (Base10, Base12, or Base24)
    ///
    /// # Examples
    ///
    /// ```
    /// use time_lib::{TimeComponents, TimeKind, TimeRotationComponents};
    ///
    /// let time = TimeComponents::new(3, 30, 0, 0);
    /// let rotations = TimeRotationComponents::new(time, TimeKind::Base24);
    /// ```
    pub fn new(components: TimeComponents, kind: TimeKind) -> Self {
        let TimeComponents {
            hours,
            minutes,
            seconds,
            nanoseconds,
        } = components;
        let bounds = TimeBounds::new(kind);
        let nanoseconds_per = nanoseconds as f64 / bounds.nanoseconds.len() as f64;
        let seconds_per =
            (seconds as f64 / bounds.seconds.len() as f64) + (nanoseconds_per / 100.0);
        let minutes_per = (minutes as f64 / bounds.minutes.len() as f64) + (seconds_per / 100.0);
        let hours_per = (hours as f64 / bounds.hours.len() as f64) + (minutes_per / 100.0);
        Self {
            hours: FULL_CIRCLE_DEGREES * hours_per,
            minutes: FULL_CIRCLE_DEGREES * minutes_per,
            seconds: FULL_CIRCLE_DEGREES * seconds_per,
            nanoseconds: FULL_CIRCLE_DEGREES * nanoseconds_per,
        }
    }

    // -------------------------------------------- /

    /// Returns the rotation angle in degrees for the hours hand.
    ///
    /// The angle ranges from 0 to 360 degrees and includes fractional
    /// components from minutes for smooth movement.
    pub fn hours(&self) -> f64 {
        self.hours
    }

    /// Returns the rotation angle in degrees for the minutes hand.
    ///
    /// The angle ranges from 0 to 360 degrees and includes fractional
    /// components from seconds for smooth movement.
    pub fn minutes(&self) -> f64 {
        self.minutes
    }

    /// Returns the rotation angle in degrees for the seconds hand.
    ///
    /// The angle ranges from 0 to 360 degrees and includes fractional
    /// components from nanoseconds for smooth movement.
    pub fn seconds(&self) -> f64 {
        self.seconds
    }

    /// Returns the rotation angle in degrees for the nanoseconds hand.
    ///
    /// The angle ranges from 0 to 360 degrees based on the nanosecond
    /// component of the time.
    pub fn nanoseconds(&self) -> f64 {
        self.nanoseconds
    }
}

// TimeRanges --------------------------------------------------------------------------- /

pub trait TimeConversionTrait {
    fn to(&self, kind: TimeKind) -> Self;
}

/*
🧪 Tests --------------------------------------------------------------------------- /
*/

#[cfg(test)]
mod tests {
    use crate::{
        constants::NS_PER_SEC,
        time_lib::{TimeBounds, TimeComponents, TimeKind, TimeRangeError, TimeRotationComponents},
    };

    use super::Period;

    fn check_bounds(
        kind: TimeKind,
        components: TimeComponents,
        expectation: Option<TimeRangeError>,
    ) {
        let bounds = TimeBounds::new(kind);
        let result = bounds.check(components);
        if let Some(expected_err) = expectation {
            assert_eq!(result.is_err(), true);
            assert_eq!(result.unwrap_err(), expected_err);
        } else {
            assert_eq!(result.is_ok(), true);
            assert_eq!(result.unwrap(), components);
        }
    }

    #[test]
    fn it_can_validate_correct_metric_bounds() {
        check_bounds(TimeKind::Base10, TimeComponents::new(0, 0, 0, 0), None);
        check_bounds(
            TimeKind::Base10,
            TimeComponents::new(9, 99, 99, NS_PER_SEC - 1),
            None,
        );
        check_bounds(
            TimeKind::Base10,
            TimeComponents::new(10, 0, 0, 0),
            Some(TimeRangeError::HoursHigh),
        );
        check_bounds(
            TimeKind::Base10,
            TimeComponents::new(0, 100, 0, 0),
            Some(TimeRangeError::MintuesHigh),
        );
        check_bounds(
            TimeKind::Base10,
            TimeComponents::new(0, 0, 100, 0),
            Some(TimeRangeError::SecondsHigh),
        );
        check_bounds(
            TimeKind::Base10,
            TimeComponents::new(0, 0, 0, NS_PER_SEC),
            Some(TimeRangeError::NanosecondsHigh),
        );
    }

    #[test]
    fn it_can_validate_correct_standard_bounds_12_hour() {
        let kind = TimeKind::Base12(Period::AM);
        check_bounds(kind, TimeComponents::new(1, 0, 0, 0), None);
        check_bounds(kind, TimeComponents::new(11, 59, 59, NS_PER_SEC - 1), None);
        check_bounds(
            kind,
            TimeComponents::new(13, 0, 0, 0),
            Some(TimeRangeError::HoursHigh),
        );
        check_bounds(
            kind,
            TimeComponents::new(0, 0, 0, 0),
            Some(TimeRangeError::HoursLow),
        );
        check_bounds(
            kind,
            TimeComponents::new(1, 60, 0, 0),
            Some(TimeRangeError::MintuesHigh),
        );
        check_bounds(
            kind,
            TimeComponents::new(1, 0, 60, 0),
            Some(TimeRangeError::SecondsHigh),
        );
        check_bounds(
            kind,
            TimeComponents::new(1, 0, 0, NS_PER_SEC),
            Some(TimeRangeError::NanosecondsHigh),
        );
    }

    #[test]
    fn it_can_validate_correct_standard_bounds_24_hour() {
        let kind = TimeKind::Base24;
        check_bounds(kind, TimeComponents::new(0, 0, 0, 0), None);
        check_bounds(kind, TimeComponents::new(23, 59, 59, NS_PER_SEC - 1), None);
        check_bounds(
            kind,
            TimeComponents::new(24, 0, 0, 0),
            Some(TimeRangeError::HoursHigh),
        );
        check_bounds(
            kind,
            TimeComponents::new(0, 60, 0, 0),
            Some(TimeRangeError::MintuesHigh),
        );
        check_bounds(
            kind,
            TimeComponents::new(0, 0, 60, 0),
            Some(TimeRangeError::SecondsHigh),
        );
        check_bounds(
            kind,
            TimeComponents::new(0, 0, 0, NS_PER_SEC),
            Some(TimeRangeError::NanosecondsHigh),
        );
    }

    #[test]
    fn it_calculates_clock_hand_rotations_in_metric() {
        let components = TimeComponents::new(2, 45, 23, 234_000_000);
        fn round_float(num: f64) -> f64 {
            (num * 100.00).round() / 100.00
        }
        let rotations = TimeRotationComponents::new(components, TimeKind::Base10);
        assert_eq!(round_float(rotations.hours()), 73.63);
        assert_eq!(round_float(rotations.minutes()), 162.84);
        assert_eq!(round_float(rotations.seconds()), 83.64);
        assert_eq!(round_float(rotations.nanoseconds()), 84.24);
    }
}

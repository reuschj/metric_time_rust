//! This module provides the tools for converting between metric and standard time conversions.
//!
//! # Main Components
//!
//! - `Converter`: Handles conversion of time units between metric and standard systems
//! - `TimeConversions`: Provides time unit definitions for both metric and standard time
//!
//! # Examples
//!
//! ```
//! use metric_time::time_helpers::time_conversions::{Converter, TimeConversions};
//!
//! // Convert standard nanoseconds to metric
//! let converter = Converter::metric();
//! let std_ns = 1000;
//! let metric_ns = converter.to_dest_from(std_ns);
//!
//! // Get standard time definitions
//! let std_time = TimeConversions::standard();
//! assert_eq!(std_time.hours_per_day(), 24);
//! assert_eq!(std_time.min_per_hour(), 60);
//!
//! // Get metric time definitions
//! let metric_time = TimeConversions::metric();
//! assert_eq!(metric_time.hours_per_day(), 10);
//! assert_eq!(metric_time.min_per_hour(), 100);
//! ```

use std::fmt::{Display, Error, Formatter};

use num::{Num, ToPrimitive};

use crate::constants::{METRIC_CONVERSION_RATE, NS_PER_MS, NS_PER_SEC};

/// Represents a metric/standard time system converter.
/// A [`Converter`] can perform conversions between standard and metric time units.
///
/// # Examples
///
/// ```
/// use metric_time::time_helpers::time_conversions::Converter;
///
/// // Create a metric time converter
/// let converter = Converter::metric();
///
/// // Convert 1000 standard nanoseconds to metric
/// let std_ns = 1000;
/// let metric_ns = converter.to_dest_from(std_ns);
/// ```
///
/// You can also create a custom converter with a specific conversion rate:
///
/// ```
/// use metric_time::time_helpers::time_conversions::Converter;
///
/// let custom_rate = 0.5;
/// let converter = Converter::new(custom_rate);
/// ```

/// Provides time unit definitions for both metric and standard time systems.
/// The [`TimeConversions`] struct defines ratios between different time units.
///
/// # Examples
///
/// ```
/// use metric_time::time_helpers::time_conversions::TimeConversions;
///
/// // Get standard time definitions
/// let std_time = TimeConversions::standard();
/// assert_eq!(std_time.hours_per_day(), 24);
/// assert_eq!(std_time.min_per_hour(), 60);
///
/// // Get metric time definitions
/// let metric_time = TimeConversions::metric();
/// assert_eq!(metric_time.hours_per_day(), 10);
/// assert_eq!(metric_time.min_per_hour(), 100);
/// ```

#[derive(Debug, Clone, Copy)]
pub struct Converter {
    rate: f64,
}

impl Converter {
    pub fn new(rate: f64) -> Self {
        Self { rate }
    }

    pub fn metric() -> Self {
        Self::new(METRIC_CONVERSION_RATE)
    }

    pub fn rate(&self) -> f64 {
        self.rate
    }

    pub fn to_dest_from<T: Num + ToPrimitive>(self, origin: T) -> f64 {
        let input = origin.to_f64().unwrap();
        let input_has_frac = input.trunc() != input;
        let output = input / self.rate;
        if input_has_frac {
            output
        } else {
            output.floor()
        }
    }

    pub fn to_origin_from<T: Num + ToPrimitive>(self, dest: T) -> f64 {
        let input = dest.to_f64().unwrap();
        let input_has_frac = input.trunc() != input;
        let output = input * self.rate;
        if input_has_frac {
            output
        } else {
            output.ceil()
        }
    }
}

/// Represents time unit conversions for a specific time system.
/// A [`TimeConversions`] defines ratios between different time units like hours per day,
/// minutes per hour, and seconds per minute.
///
/// # Examples
///
/// ```
/// use metric_time::time_helpers::time_conversions::TimeConversions;
///
/// // Get standard time definitions
/// let std_time = TimeConversions::standard();
/// assert_eq!(std_time.hours_per_day(), 24);
/// assert_eq!(std_time.min_per_hour(), 60);
/// assert_eq!(std_time.secs_per_min(), 60);
///
/// // Get metric time definitions
/// let metric_time = TimeConversions::metric();
/// assert_eq!(metric_time.hours_per_day(), 10);
/// assert_eq!(metric_time.min_per_hour(), 100);
/// assert_eq!(metric_time.secs_per_min(), 100);
/// ```
///
/// You can also create custom time unit definitions:
///
/// ```
/// use metric_time::time_helpers::time_conversions::TimeConversions;
///
/// let custom_time = TimeConversions::new(12, 100, 100);
/// assert_eq!(custom_time.hours_per_day(), 12);
/// assert_eq!(custom_time.min_per_hour(), 100);
/// assert_eq!(custom_time.secs_per_min(), 100);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct TimeConversions {
    hr_per_day: u8,
    min_per_hr: u8,
    sec_per_min: u8,
}

impl TimeConversions {
    pub fn new(hours_per_day: u8, minutes_per_hour: u8, seconds_per_min: u8) -> Self {
        TimeConversions {
            hr_per_day: hours_per_day,
            min_per_hr: minutes_per_hour,
            sec_per_min: seconds_per_min,
        }
    }

    pub fn standard() -> Self {
        TimeConversions::new(24, 60, 60)
    }

    pub fn metric() -> Self {
        TimeConversions::new(10, 100, 100)
    }

    pub fn hours_per_day(&self) -> u8 {
        self.hr_per_day
    }

    pub fn min_per_hour(&self) -> u8 {
        self.min_per_hr
    }

    pub fn secs_per_min(&self) -> u8 {
        self.sec_per_min
    }

    pub fn mins_per_day(&self) -> u16 {
        self.hr_per_day as u16 * self.min_per_hr as u16
    }

    pub fn secs_per_hour(&self) -> u32 {
        self.min_per_hr as u32 * self.sec_per_min as u32
    }

    pub fn secs_per_day(&self) -> u32 {
        self.sec_per_min as u32 * self.mins_per_day() as u32
    }

    pub fn ns_per_day(&self) -> u64 {
        self.ns_per_hour() * self.hr_per_day as u64
    }

    pub fn ns_per_hour(&self) -> u64 {
        self.ns_per_min() * self.min_per_hr as u64
    }

    pub fn ns_per_min(&self) -> u64 {
        self.sec_per_min as u64 * NS_PER_SEC as u64
    }

    pub fn ns_per_sec(&self) -> u32 {
        NS_PER_SEC
    }

    pub fn ns_per_ms(&self) -> u32 {
        NS_PER_MS
    }
}

impl Display for TimeConversions {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        let TimeConversions {
            hr_per_day,
            min_per_hr,
            sec_per_min,
        } = self;
        write!(
            formatter,
            "{{ hr/day: {}, min/hr: {}, sec/min: {} }}",
            hr_per_day, min_per_hr, sec_per_min
        )
    }
}

/*
🧪 Tests --------------------------------------------------------------------------- /
*/

#[cfg(test)]
mod tests {
    use crate::{
        constants::METRIC_CONVERSION_RATE,
        time_helpers::time_conversions::{Converter, TimeConversions},
    };

    #[test]
    fn can_get_rate_from_converter() {
        let custom_rate = 0.5;
        let custom_conv = Converter::new(custom_rate);
        assert_eq!(custom_conv.rate(), custom_rate);
        let metric_conv = Converter::metric();
        assert_eq!(metric_conv.rate(), METRIC_CONVERSION_RATE);
    }

    #[test]
    fn can_convert_standard_ns_to_metric() {
        let metric_conv = Converter::metric();
        let std_ns: u32 = 1_000;
        let metric_ns = metric_conv.to_dest_from(std_ns) as u32;
        assert_eq!(metric_ns, 1_157);
    }

    #[test]
    fn can_convert_metric_ns_back_to_standard() {
        let metric_conv = Converter::metric();
        let metric_ns: u32 = 1_157;
        let std_ns = metric_conv.to_origin_from(metric_ns) as u32;
        assert_eq!(std_ns, 1_000);
    }

    #[test]
    fn can_convert_from_a_float_origin_to_float_destination() {
        let metric_conv = Converter::metric();
        let origin = 564.3;
        let dest = metric_conv.to_dest_from(origin);
        assert_eq!(dest, 653.125);
    }

    #[test]
    fn can_convert_from_a_float_destination_back_to_float_origin() {
        let metric_conv = Converter::metric();
        let dest = 653.125;
        let origin = metric_conv.to_origin_from(dest);
        assert_eq!(origin, 564.3);
    }

    #[test]
    fn converts_to_metric_and_back_to_standard_scenario_01() {
        let metric_conv = Converter::metric();
        let initial_std: u32 = 651;
        let metric_dest = metric_conv.to_dest_from(initial_std) as u32;
        assert_eq!(metric_dest, 753);
        let final_std = metric_conv.to_origin_from(metric_dest) as u32;
        assert_eq!(final_std, initial_std);
    }

    #[test]
    fn converts_to_metric_and_back_to_standard_scenario_02() {
        let metric_conv = Converter::metric();
        let initial_std: u64 = 123_456_789;
        let metric_dest = metric_conv.to_dest_from(initial_std) as u64;
        assert_eq!(metric_dest, 142_889_802);
        let final_std = metric_conv.to_origin_from(metric_dest) as u64;
        assert_eq!(final_std, initial_std);
    }

    #[test]
    fn converts_to_standard_and_back_to_metric_scenario_01() {
        let metric_conv = Converter::metric();
        let metric_dest: u32 = 651;
        let std_origin = metric_conv.to_origin_from(metric_dest) as u32;
        assert_eq!(std_origin, 563);
        let final_metric = metric_conv.to_dest_from(std_origin) as u32;
        assert_eq!(final_metric, metric_dest);
    }

    #[test]
    fn converts_to_standard_and_back_to_metric_scenario_02() {
        let metric_conv = Converter::metric();
        let metric_dest: u64 = 123_456_789;
        let std_origin = metric_conv.to_origin_from(metric_dest) as u64;
        assert_eq!(std_origin, 106_666_666);
        let final_metric = metric_conv.to_dest_from(std_origin) as u64;
        assert_eq!(final_metric, metric_dest);
    }

    #[test]
    fn standard_conversions() {
        let c = TimeConversions::standard();
        assert_eq!(c.hours_per_day(), 24);
        assert_eq!(c.min_per_hour(), 60);
        assert_eq!(c.secs_per_min(), 60);
        assert_eq!(c.mins_per_day(), 1_440);
        assert_eq!(c.secs_per_hour(), 3_600);
        assert_eq!(c.secs_per_day(), 86_400);
        assert_eq!(c.ns_per_ms(), 1_000_000);
        assert_eq!(c.ns_per_sec(), 1_000_000_000);
        assert_eq!(c.ns_per_min(), 60_000_000_000);
        assert_eq!(c.ns_per_hour(), 3_600_000_000_000);
        assert_eq!(c.ns_per_day(), 86_400_000_000_000);
    }

    #[test]
    fn metric_conversions() {
        let c = TimeConversions::metric();
        assert_eq!(c.hours_per_day(), 10);
        assert_eq!(c.min_per_hour(), 100);
        assert_eq!(c.secs_per_min(), 100);
        assert_eq!(c.mins_per_day(), 1_000);
        assert_eq!(c.secs_per_hour(), 10_000);
        assert_eq!(c.secs_per_day(), 100_000);
        assert_eq!(c.ns_per_ms(), 1_000_000);
        assert_eq!(c.ns_per_sec(), 1_000_000_000);
        assert_eq!(c.ns_per_min(), 100_000_000_000);
        assert_eq!(c.ns_per_hour(), 10_000_000_000_000);
        assert_eq!(c.ns_per_day(), 100_000_000_000_000);
    }
}

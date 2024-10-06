//! 🔄 Module containing utility functions for converting between different time representations.
//!
//! This module provides functions for converting between:
//!
//! - 🕛 Base-12 (AM/PM)
//! - 🕒 Base-24 (24-hour)
//! - 🔟 Base-10 (metric) time formats
//!
//! It also includes utilities for calculating nanoseconds since midnight for different time kinds.

use crate::time_lib::{Period, TimeComponents, TimeKind};

use super::time_conversions::{Converter, TimeConversions};

// 🧪 Tests ------------------------------------------------------- /

/// 🕛➡️🕒 Converts 12-hour format (AM/PM) to 24-hour format
///
/// # 📥 Arguments
/// * `components` - The time components to convert
/// * `period` - AM or PM indicator
///
/// # 🔄 Returns
/// Time components in 24-hour format
///
/// # 📝 Examples
/// ```ignore
/// // Internal function example:
/// use crate::time_lib::{TimeComponents, Period};
///
/// let components = TimeComponents::new(6, 10, 12, 12345);
/// let period = Period::AM;
/// let converted = base12_to_base24(&components, &period);
/// assert_eq!(converted.hours, 6);
/// assert_eq!(converted.minutes, 10);
/// assert_eq!(converted.seconds, 12);
/// assert_eq!(converted.nanoseconds, 12345);
/// ```
/// 🕛➡️🕒 Converts 12-hour format (AM/PM) to 24-hour format
pub fn base12_to_base24(components: &TimeComponents, period: &Period) -> TimeComponents {
    TimeComponents {
        hours: match period {
            Period::AM => {
                if components.hours == 12 {
                    0
                } else {
                    components.hours
                }
            }
            Period::PM => {
                if components.hours == 12 {
                    components.hours
                } else {
                    components.hours + 12
                }
            }
        },
        minutes: components.minutes,
        seconds: components.seconds,
        nanoseconds: components.nanoseconds,
    }
}

/// 🕒➡️🕛 Converts 24-hour format to 12-hour format (AM/PM)
/// and time kind (base-12, base-24, or base-10)
///
/// The total nanoseconds is calculated by converting the time components to base-24 format
/// if needed, then multiplying out each component by the appropriate nanosecond conversion
/// factor for that time kind.
///
/// # Arguments
/// * `components` - The time components to calculate from
/// * `kind` - The time kind (Base12, Base24, or Base10) determining conversion factors
///
/// # Returns
/// Total nanoseconds since midnight as u64
///
/// # Examples
/// ```ignore
/// // Internal function example:
/// use crate::time_lib::{TimeComponents, Period};
///
/// let components = TimeComponents::new(6, 10, 12, 12345);
/// let (converted, period) = base24_to_base12(&components);
/// assert_eq!(converted.hours, 6);
/// assert_eq!(converted.minutes, 10);
/// assert_eq!(converted.seconds, 12);
/// assert_eq!(converted.nanoseconds, 12345);
/// assert_eq!(period, Period::AM);
/// ```
///
/// ```ignore
/// // Internal function example:
/// use crate::time_lib::{TimeComponents, Period};
///
/// let components = TimeComponents::new(14, 30, 45, 123456);
/// let (converted, period) = base24_to_base12(&components);
/// assert_eq!(converted.hours, 2);
/// assert_eq!(converted.minutes, 30);
/// assert_eq!(converted.seconds, 45);
/// assert_eq!(converted.nanoseconds, 123456);
/// assert_eq!(period, Period::PM);
/// ```
/// 🕒➡️🕛 Converts 24-hour format to 12-hour format (AM/PM)
pub fn base24_to_base12(components: &TimeComponents) -> (TimeComponents, Period) {
    let period = if components.hours < 12 {
        Period::AM
    } else {
        Period::PM
    };
    let components = TimeComponents {
        hours: match &period {
            Period::AM => {
                if components.hours == 0 {
                    12
                } else {
                    components.hours
                }
            }
            Period::PM => {
                if components.hours == 12 {
                    components.hours
                } else {
                    components.hours - 12
                }
            }
        },
        minutes: components.minutes,
        seconds: components.seconds,
        nanoseconds: components.nanoseconds,
    };
    (components, period)
}

/// Calculate the total nanoseconds since midnight for a given set of time components
/// and time kind (base-12, base-24, or base-10)
///
/// The total nanoseconds is calculated by converting the time components to base-24 format
/// if needed, then multiplying out each component by the appropriate nanosecond conversion
/// factor for that time kind.
///
/// # Arguments
/// * `components` - The time components to calculate from
/// * `kind` - The time kind (Base12, Base24, or Base10) determining conversion factors
///
/// # Returns
/// Total nanoseconds since midnight as u64
///
/// # Examples
///
/// ```ignore
/// // Internal function example:
/// use crate::time_lib::{TimeComponents, TimeKind};
///
/// let components = TimeComponents::new(6, 10, 12, 12345);
/// let ns = calc_ns_since_midnight(&components, &TimeKind::Base24);
/// assert_eq!(ns, 22_212_000_012_345);
/// ```
///
/// This function is used internally by the conversion functions in this module:
/// - [`base10_to_base24`]
/// - [`base24_to_base10`]
/// ⏱️ Calculate the total nanoseconds since midnight for a given set of time components
fn calc_ns_since_midnight(components: &TimeComponents, kind: &TimeKind) -> u64 {
    let tc = TimeConversions::from(kind);
    let TimeComponents {
        hours,
        minutes,
        seconds,
        nanoseconds,
    } = match kind {
        TimeKind::Base12(period) => base12_to_base24(components, period),
        _ => components.clone(),
    };
    let hours = hours as u64 * tc.ns_per_hour() as u64;
    let minutes = minutes as u64 * tc.ns_per_min() as u64;
    let seconds = seconds as u64 * tc.ns_per_sec() as u64;
    hours + minutes + seconds + nanoseconds as u64
}

/// Calculate the total nanoseconds since midnight in metric time format
///
/// This helper function takes metric time components and calculates the total
/// nanoseconds elapsed since midnight. Used internally by the base10/base24
/// conversion functions.
///
/// # Arguments
/// * `components` - The time components in metric format
/// * `kind` - Must be TimeKind::Base10
///
/// # Returns
/// Total nanoseconds since midnight as u64
///
/// # Examples
/// ```ignore
/// // Internal function example:
/// use crate::time_lib::TimeComponents;
///
/// let components = TimeComponents::new(1, 92, 31, 624_371_283);
/// let converted = base10_to_base24(&components);
/// assert_eq!(converted.hours, 4);
/// assert_eq!(converted.minutes, 36);
/// assert_eq!(converted.seconds, 56);
/// assert!(converted.nanoseconds > 123_000_000);
/// ```
/// 🔟➡️🕒 Convert metric time (base-10) to standard time (base-24)
pub fn base10_to_base24(metric_components: &TimeComponents) -> TimeComponents {
    let metric_conv = Converter::metric();
    let std_conversions = TimeConversions::standard();
    let total_ns_metric = calc_ns_since_midnight(metric_components, &TimeKind::Base10);
    let total_ns_std = metric_conv.to_origin_from(total_ns_metric) as u64;
    let hours = total_ns_std / std_conversions.ns_per_hour();
    let min_remainder = total_ns_std % std_conversions.ns_per_hour();
    let minutes = min_remainder / std_conversions.ns_per_min();
    let sec_remainder = min_remainder % std_conversions.ns_per_min();
    let seconds = sec_remainder / std_conversions.ns_per_sec() as u64;
    let nanoseconds = sec_remainder % std_conversions.ns_per_sec() as u64;
    TimeComponents {
        hours: hours as u8,
        minutes: minutes as u8,
        seconds: seconds as u8,
        nanoseconds: nanoseconds as u32,
    }
}

/// Convert time from base-24 (24-hour) format to base-10 (metric) format
///
/// Takes time components in standard 24-hour format and converts them to
/// metric time format, where each hour is 100 minutes, each minute is 100
/// seconds, and each second is 100 centiseconds.
///
/// Metric time divides the day into 10 metric hours, each metric hour into
/// 100 metric minutes, and each metric minute into 100 metric seconds.
///
/// # Arguments
/// * `components` - The time components in 24-hour format
///
/// # Returns
/// New TimeComponents struct in base-10 (metric) format
///
/// # Examples
/// ```ignore
/// // Internal function example:
/// use crate::time_lib::TimeComponents;
///
/// let std_time = TimeComponents::new(16, 10, 23, 12345);
/// let metric = base24_to_base10(&std_time);
/// assert_eq!(metric.hours, 6);
/// assert_eq!(metric.minutes, 73);
/// assert_eq!(metric.seconds, 87);
/// assert!(metric.nanoseconds > 700_000_000);
/// ```
///
/// ```ignore
/// // Internal function example:
/// use crate::time_lib::TimeComponents;
///
/// let std_time = TimeComponents::new(4, 36, 56, 123_456_789);
/// let metric = base24_to_base10(&std_time);
/// assert_eq!(metric.hours, 1);
/// assert_eq!(metric.minutes, 92);
/// assert_eq!(metric.seconds, 31);
/// assert!(metric.nanoseconds > 600_000_000);
/// ```
/// 🕒➡️🔟 Convert standard time (base-24) to metric time (base-10)
pub fn base24_to_base10(standard_components: &TimeComponents) -> TimeComponents {
    let metric_conv = Converter::metric();
    let metric_conversions = TimeConversions::metric();
    let total_ns_std = calc_ns_since_midnight(standard_components, &TimeKind::Base24);
    let total_ns_metric = metric_conv.to_dest_from(total_ns_std) as u64;
    let hours = total_ns_metric / metric_conversions.ns_per_hour();
    let min_remainder = if hours > 0 {
        total_ns_metric % (hours * metric_conversions.ns_per_hour())
    } else {
        total_ns_metric
    };
    let minutes = min_remainder / metric_conversions.ns_per_min();
    let sec_remainder = if minutes > 0 {
        min_remainder % (minutes * metric_conversions.ns_per_min())
    } else {
        min_remainder
    };
    let seconds = sec_remainder / metric_conversions.ns_per_sec() as u64;
    let nanoseconds = if seconds > 0 {
        sec_remainder % (seconds * metric_conversions.ns_per_sec() as u64)
    } else {
        sec_remainder
    };
    TimeComponents {
        hours: hours as u8,
        minutes: minutes as u8,
        seconds: seconds as u8,
        nanoseconds: nanoseconds as u32,
    }
}

/*
🧪 Tests --------------------------------------------------------------------------- /
*/

#[cfg(test)]
mod tests {
    // 🧪 Standard time conversion tests
    mod std_conversions {
        use crate::{
            time_helpers::conversion_utils::{base12_to_base24, base24_to_base12},
            time_lib::{Period, TimeComponents},
        };

        fn test_base12_to_base24_conversion(
            input: TimeComponents,
            period: Period,
            expected: TimeComponents,
        ) {
            let result = base12_to_base24(&input, &period);
            assert_eq!(result, expected);
        }

        fn test_base24_to_base12_conversion(
            input: TimeComponents,
            expected: TimeComponents,
            expected_period: Period,
        ) {
            let (result, period) = base24_to_base12(&input);
            assert_eq!(result, expected);
            assert_eq!(period, expected_period);
        }

        #[test]
        fn it_converts_base12_to_base24_scenario_just_after_midnight() {
            test_base12_to_base24_conversion(
                TimeComponents::new(12, 36, 23, 12345),
                Period::AM,
                TimeComponents::new(0, 36, 23, 12345),
            );
        }

        #[test]
        fn it_converts_base12_to_base24_scenario_am_time() {
            test_base12_to_base24_conversion(
                TimeComponents::new(9, 36, 23, 12345),
                Period::AM,
                TimeComponents::new(9, 36, 23, 12345),
            );
        }

        #[test]
        fn it_converts_base12_to_base24_scenario_just_after_noon() {
            test_base12_to_base24_conversion(
                TimeComponents::new(12, 10, 23, 12345),
                Period::PM,
                TimeComponents::new(12, 10, 23, 12345),
            );
        }

        #[test]
        fn it_converts_base12_to_base24_scenario_pm_time() {
            test_base12_to_base24_conversion(
                TimeComponents::new(4, 10, 23, 12345),
                Period::PM,
                TimeComponents::new(16, 10, 23, 12345),
            );
        }

        #[test]
        fn it_converts_base24_to_base12_scenario_just_after_midnight() {
            test_base24_to_base12_conversion(
                TimeComponents::new(0, 36, 23, 12345),
                TimeComponents::new(12, 36, 23, 12345),
                Period::AM,
            );
        }

        #[test]
        fn it_converts_base24_to_base12_scenario_am_time() {
            test_base24_to_base12_conversion(
                TimeComponents::new(9, 36, 23, 12345),
                TimeComponents::new(9, 36, 23, 12345),
                Period::AM,
            );
        }

        #[test]
        fn it_converts_base24_to_base12_scenario_just_after_noon() {
            test_base24_to_base12_conversion(
                TimeComponents::new(12, 10, 23, 12345),
                TimeComponents::new(12, 10, 23, 12345),
                Period::PM,
            );
        }

        #[test]
        fn it_converts_base24_to_base12_scenario_pm_time() {
            test_base24_to_base12_conversion(
                TimeComponents::new(16, 10, 23, 12345),
                TimeComponents::new(4, 10, 23, 12345),
                Period::PM,
            );
        }
    }

    // 🧪 Nanosecond calculation tests
    mod calc_ns_since_midnight_util {
        use crate::{
            time_helpers::conversion_utils::calc_ns_since_midnight,
            time_lib::{Period, TimeComponents, TimeKind},
        };

        fn test_calc_ns_since_midnight(
            components: TimeComponents,
            kind: TimeKind,
            expectation: u64,
        ) {
            let result = calc_ns_since_midnight(&components, &kind);
            assert_eq!(expectation, result);
        }

        #[test]
        fn it_calcs_total_ns_scenario_base24_morning() {
            test_calc_ns_since_midnight(
                TimeComponents::new(6, 10, 12, 12_345),
                TimeKind::Base24,
                22_212_000_012_345,
            );
        }

        #[test]
        fn it_calcs_total_ns_scenario_base24_evening() {
            test_calc_ns_since_midnight(
                TimeComponents::new(23, 59, 58, 700_000_345),
                TimeKind::Base24,
                86_398_700_000_345,
            );
        }

        #[test]
        fn it_calcs_total_ns_scenario_base12_am() {
            test_calc_ns_since_midnight(
                TimeComponents::new(6, 10, 12, 12_345),
                TimeKind::Base12(Period::AM),
                22_212_000_012_345,
            );
        }

        #[test]
        fn it_calcs_total_ns_scenario_base12_pm() {
            test_calc_ns_since_midnight(
                TimeComponents::new(11, 59, 58, 700_000_345),
                TimeKind::Base12(Period::PM),
                86_398_700_000_345,
            );
        }

        #[test]
        fn it_calcs_total_ns_scenario_base10() {
            test_calc_ns_since_midnight(
                TimeComponents::new(8, 62, 92, 700_000_345),
                TimeKind::Base10,
                86_292_700_000_345,
            );
        }
    }

    // 🧪 Metric time conversion tests
    mod metric_conversions {
        use crate::{
            time_helpers::conversion_utils::{base10_to_base24, base24_to_base10},
            time_lib::TimeComponents,
        };

        fn test_base24_to_base10_conversion(input: TimeComponents, expected: TimeComponents) {
            let result = base24_to_base10(&input);
            assert_eq!(result, expected);
        }

        fn test_base10_to_base24_conversion(input: TimeComponents, expected: TimeComponents) {
            let result = base10_to_base24(&input);
            assert_eq!(result, expected);
        }

        #[test]
        fn it_converts_base24_to_base10_scenario_am_time() {
            test_base24_to_base10_conversion(
                TimeComponents::new(4, 36, 56, 123_456_789),
                TimeComponents::new(1, 92, 31, 624_371_283),
            );
        }

        #[test]
        fn it_converts_base24_to_base10_scenario_am_after_midnight() {
            test_base24_to_base10_conversion(
                TimeComponents::new(1, 10, 30, 123_456_789),
                TimeComponents::new(0, 48, 95, 976_223_135),
            );
        }

        #[test]
        fn it_converts_base24_to_base10_scenario_pm_time() {
            test_base24_to_base10_conversion(
                TimeComponents::new(16, 10, 23, 12_345),
                TimeComponents::new(6, 73, 87, 731_495_769),
            );
        }

        #[test]
        fn it_converts_base10_to_base24_scenario_am_time() {
            test_base10_to_base24_conversion(
                TimeComponents::new(1, 92, 31, 624_371_283),
                TimeComponents::new(4, 36, 56, 123_456_789),
            );
        }

        #[test]
        fn it_converts_base10_to_base24_scenario_pm_time() {
            test_base10_to_base24_conversion(
                TimeComponents::new(6, 73, 87, 731_495_769),
                TimeComponents::new(16, 10, 23, 12_345),
            );
        }
    }
}

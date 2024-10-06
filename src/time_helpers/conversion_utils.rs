//! Module containing utility functions for converting between different time representations.
//!
//! This module provides functions for converting between base-12 (AM/PM), base-24 (24-hour),
//! and base-10 (metric) time formats. It also includes utilities for calculating nanoseconds
//! since midnight for different time kinds.

use crate::Builder;

use crate::time_lib::{Period, TimeComponents, TimeKind};

use super::time_conversions::{Converter, TimeConversions};

// Utils ------------------------------------------------------- /

/// Calculate the total nanoseconds since midnight for a given set of time components
/// and time kind (base-12, base-24, or base-10)
///
/// # Arguments
/// * `components` - The time components to calculate from
/// * `kind` - The time kind (Base12, Base24, or Base10)
///
/// # Returns
/// Total nanoseconds since midnight as u64
///
/// # Examples
/// ```
/// let components = TimeComponents::new(6, 10, 12, 12345);
/// let ns = calc_ns_since_midnight(&components, &TimeKind::Base24);
/// assert_eq!(ns, 22_212_000_012_345);
/// ```
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
/// ```
/// let components = TimeComponents::new(6, 10, 12, 12345);
/// let ns = calc_ns_since_midnight(&components, &TimeKind::Base24);
/// assert_eq!(ns, 22_212_000_012_345);
/// ```
///
/// ```
/// let components = TimeComponents::new(11, 59, 58, 700_000_345);
/// let period = Period::PM;
/// let ns = calc_ns_since_midnight(&components, &TimeKind::Base12(period));
/// assert_eq!(ns, 86_398_700_000_345);
/// ```
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
    let hours_builder = Builder::new(hours as u64).mulitply_by(tc.ns_per_hour() as u64);
    let minutes_builder = Builder::new(minutes as u64).mulitply_by(tc.ns_per_min() as u64);
    let seconds_builder = Builder::new(seconds as u64).mulitply_by(tc.ns_per_sec() as u64);
    Builder::new(u64::MIN)
        .add_from_builder(hours_builder)
        .add_from_builder(minutes_builder)
        .add_from_builder(seconds_builder)
        .add(nanoseconds as u64)
        .build()
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
/// ```
/// let components = TimeComponents::new(1, 92, 31, 624_371_283);
/// let ns = calc_ns_since_midnight(&components, &TimeKind::Base10);
/// assert_eq!(ns, 16_616_123_456_789);
/// ```
pub fn base10_to_base24(components: &TimeComponents) -> TimeComponents {
    let metric_conv = Converter::metric();
    let std_conversions = TimeConversions::standard();
    let total_ns_metric = calc_ns_since_midnight(components, &TimeKind::Base10);
    let total_ns_std = metric_conv.to_origin_from(total_ns_metric) as u64;

    let hours = Builder::new(total_ns_std)
        .divide_by(std_conversions.ns_per_hour())
        .build();
    let min_remainder = Builder::new(total_ns_std)
        .modulo(if hours > 0 {
            hours * std_conversions.ns_per_hour()
        } else {
            1
        })
        .build();
    let minutes = Builder::new(min_remainder)
        .divide_by(std_conversions.ns_per_min())
        .build();
    let sec_remainder = Builder::new(min_remainder)
        .modulo(if minutes > 0 {
            minutes * std_conversions.ns_per_min() as u64
        } else {
            1
        })
        .build();
    let seconds = Builder::new(sec_remainder)
        .divide_by(std_conversions.ns_per_sec() as u64)
        .build();
    let nanoseconds = Builder::new(sec_remainder)
        .modulo(if seconds > 0 {
            seconds * std_conversions.ns_per_sec() as u64
        } else {
            1
        })
        .build();
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
/// ```
/// let std_time = TimeComponents::new(16, 10, 23, 12345);
/// let metric = base24_to_base10(&std_time);
/// assert_eq!(metric, TimeComponents::new(6, 73, 87, 731_495_769));
/// ```
///
/// ```
/// let std_time = TimeComponents::new(4, 36, 56, 123_456_789);
/// let metric = base24_to_base10(&std_time);
/// assert_eq!(metric, TimeComponents::new(1, 92, 31, 624_371_283));
/// ```
pub fn base24_to_base10(components: &TimeComponents) -> TimeComponents {
    let metric_conv = Converter::metric();
    let metric_conversions = TimeConversions::metric();
    let total_ns_std = calc_ns_since_midnight(components, &TimeKind::Base24);
    let total_ns_metric = metric_conv.to_dest_from(total_ns_std) as u64;
    let hours = Builder::new(total_ns_metric)
        .divide_by(metric_conversions.ns_per_hour())
        .build();
    let min_remainder = Builder::new(total_ns_metric)
        .modulo(if hours > 0 {
            hours * metric_conversions.ns_per_hour()
        } else {
            1
        })
        .build();
    let minutes = Builder::new(min_remainder)
        .divide_by(metric_conversions.ns_per_min())
        .build();
    let sec_remainder = Builder::new(min_remainder)
        .modulo(if minutes > 0 {
            minutes * metric_conversions.ns_per_min() as u64
        } else {
            1
        })
        .build();
    let seconds = Builder::new(sec_remainder)
        .divide_by(metric_conversions.ns_per_sec() as u64)
        .build();
    let nanoseconds = Builder::new(sec_remainder)
        .modulo(if seconds > 0 {
            seconds * metric_conversions.ns_per_sec() as u64
        } else {
            1
        })
        .build();
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

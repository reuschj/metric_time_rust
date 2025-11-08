//! 🔄 Module containing utility functions for converting between different time representations.
//!
//! This module provides functions for converting between:
//!
//! - 🕛 Base-12 (AM/PM)
//! - 🕒 Base-24 (24-hour)
//! - 🔟 Base-10 (metric) time formats
//!
//! It also includes utilities for calculating nanoseconds since midnight for different time kinds.

use crate::{Period, TimeComponents, TimeKind};

use super::time_conversions::{Converter, TimeConversions};

/// 🕛➡️🕒 Converts 12-hour format (AM/PM) to 24-hour format.
///
/// # Arguments
/// * `components` - The time components to convert.
/// * `period` - The `Period` (AM or PM).
///
/// # Returns
/// Time components in 24-hour format.
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

/// 🕒➡️🕛 Converts 24-hour format to 12-hour format (AM/PM).
///
/// # Arguments
/// * `components` - The time components in 24-hour format.
///
/// # Returns
/// A tuple containing the converted `TimeComponents` and the `Period` (AM or PM).
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

/// 🔟➡️🕒 Converts metric time (base-10) to standard time (base-24).
///
/// # Arguments
/// * `metric_components` - The time components in metric format.
///
/// # Returns
/// `TimeComponents` in 24-hour format.
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

/// 🕒➡️🔟 Converts standard time (base-24) to metric time (base-10).
///
/// # Arguments
/// * `standard_components` - The time components in 24-hour format.
///
/// # Returns
/// `TimeComponents` in metric (base-10) format.
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
            time::conversion_utils::{base12_to_base24, base24_to_base12},
            Period, TimeComponents,
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
            time::conversion_utils::calc_ns_since_midnight, Period, TimeComponents, TimeKind,
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
            time::conversion_utils::{base10_to_base24, base24_to_base10},
            TimeComponents,
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

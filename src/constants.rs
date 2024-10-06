use std::ops::Range;

pub static METRIC_CONVERSION_RATE: f64 = 0.864;

pub static FULL_CIRCLE_DEGREES: f64 = 360.00;

pub static NS_PER_MS: u32 = 1_000_000;
pub static NS_PER_SEC: u32 = 1_000_000_000;

pub static RANGE_HOUR_BASE_10: Range<u8> = 0..10;
pub static RANGE_HOUR_BASE_12: Range<u8> = 1..13;
pub static RANGE_HOUR_BASE_24: Range<u8> = 0..24;

pub static RANGE_MIN_SEC_BASE_10: Range<u8> = 0..100;
pub static RANGE_MIN_SEC_BASE_12_24: Range<u8> = 0..60;

pub static RANGE_NS: Range<u32> = 0..1_000_000_000;

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod error;
mod internal;

/// Momentum, strength, and bounded oscillator measures.
pub mod momentum;
/// Price structure, swing, pivot, and stop-and-reverse operators.
pub mod structure;
/// Generic sequence transforms, regression, and discrete setup counts.
pub mod transform;
/// Trend-following averages, oscillators, and directional measures.
pub mod trend;
/// Price-range, channel, and historical volatility measures.
pub mod volatility;
/// Volume, money-flow, and accumulation measures.
pub mod volume;

pub use error::IndicatorError;

/// Stable names of the 60 batch operators included in version 0.1.
pub const OPERATOR_NAMES: [&str; 60] = [
    "sma",
    "ema",
    "dema",
    "wma",
    "hma",
    "zlema",
    "alma",
    "evwma",
    "vwma",
    "macd",
    "adx",
    "gmma",
    "tdi",
    "trix",
    "dpo",
    "vhf",
    "kst",
    "po",
    "rsi",
    "cci",
    "cmo",
    "tsi",
    "smi",
    "wpr",
    "ultimate_oscillator",
    "roc",
    "momentum",
    "cti",
    "rvi",
    "dvi",
    "stoch",
    "kdj",
    "atr",
    "tr",
    "bollinger",
    "keltner",
    "donchian",
    "pbands",
    "volatility",
    "obv",
    "cmf",
    "vwap",
    "mfi",
    "emv",
    "clv",
    "chaikin_ad",
    "chaikin_volatility",
    "williams_ad",
    "zigzag",
    "pivots",
    "sar",
    "snr",
    "growth",
    "adj_ratios",
    "roll_sfm",
    "aroon",
    "td_setup",
    "td_countdown",
    "na_check",
    "lags",
];

/// Internal numerical primitives exposed only to support conformance testing.
#[doc(hidden)]
pub mod testing {
    pub use crate::internal::rolling::{rolling_max, rolling_mean, rolling_min, rolling_std};
    pub use crate::internal::smoothing::ema;
}

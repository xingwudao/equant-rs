#![forbid(unsafe_code)]
#![doc = "Pure Rust batch quantitative finance operators over borrowed slices."]

mod error;
mod internal;

/// Trend-following averages, oscillators, and directional measures.
pub mod trend;

pub use error::IndicatorError;

/// Internal numerical primitives exposed only to support conformance testing.
#[doc(hidden)]
pub mod testing {
    pub use crate::internal::rolling::{rolling_max, rolling_mean, rolling_min, rolling_std};
    pub use crate::internal::smoothing::ema;
}

use crate::internal::{smoothing, validation};
use crate::IndicatorError;

/// Average Directional Index and its positive and negative components.
#[derive(Clone, Debug, PartialEq)]
pub struct AdxOutput {
    /// Average Directional Index in the range 0 to 100.
    pub adx: Vec<f64>,
    /// Positive Directional Indicator.
    pub plus_di: Vec<f64>,
    /// Negative Directional Indicator.
    pub minus_di: Vec<f64>,
}

/// Calculate Wilder's Average Directional Index.
pub fn adx(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> Result<AdxOutput, IndicatorError> {
    validation::period(period)?;
    validation::same_length(high.len(), &[("low", low.len()), ("close", close.len())])?;
    let length = high.len();
    let mut true_range = vec![f64::NAN; length];
    let mut plus_movement = vec![f64::NAN; length];
    let mut minus_movement = vec![f64::NAN; length];

    for index in 1..length {
        if [
            high[index],
            low[index],
            close[index - 1],
            high[index - 1],
            low[index - 1],
        ]
        .iter()
        .all(|value| value.is_finite())
        {
            true_range[index] = (high[index] - low[index])
                .max((high[index] - close[index - 1]).abs())
                .max((low[index] - close[index - 1]).abs());
            let up = high[index] - high[index - 1];
            let down = low[index - 1] - low[index];
            plus_movement[index] = if up > down && up > 0.0 { up } else { 0.0 };
            minus_movement[index] = if down > up && down > 0.0 { down } else { 0.0 };
        }
    }

    let atr = smoothing::ema(&true_range, period, true)?;
    let plus = smoothing::ema(&plus_movement, period, true)?;
    let minus = smoothing::ema(&minus_movement, period, true)?;
    let mut plus_di = vec![f64::NAN; length];
    let mut minus_di = vec![f64::NAN; length];
    let mut dx = vec![f64::NAN; length];
    for index in 0..length {
        if atr[index].is_finite() && atr[index] != 0.0 {
            plus_di[index] = 100.0 * plus[index] / atr[index];
            minus_di[index] = 100.0 * minus[index] / atr[index];
            let total = plus_di[index] + minus_di[index];
            dx[index] = if total == 0.0 {
                0.0
            } else {
                100.0 * (plus_di[index] - minus_di[index]).abs() / total
            };
        }
    }
    Ok(AdxOutput {
        adx: smoothing::ema(&dx, period, true)?,
        plus_di,
        minus_di,
    })
}

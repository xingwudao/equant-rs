use crate::internal::{rolling, validation};
use crate::IndicatorError;

fn require_high_low(high: &[f64], low: &[f64]) -> Result<(), IndicatorError> {
    validation::same_length(high.len(), &[("low", low.len())])
}

/// Identify confirmed and active swing extremes.
///
/// This operator is repainting: a later bar can move the active extreme or
/// confirm an earlier location. `change` is a percentage when `percent` is
/// true and an absolute price distance otherwise.
pub fn zigzag(
    high: &[f64],
    low: &[f64],
    change: f64,
    percent: bool,
) -> Result<Vec<f64>, IndicatorError> {
    require_high_low(high, low)?;
    validation::positive(change, "change")?;
    let mut output = vec![f64::NAN; high.len()];
    let Some(start) = (0..high.len()).find(|&i| high[i].is_finite() && low[i].is_finite()) else {
        return Ok(output);
    };
    let mut direction = 0i8;
    let mut high_index = start;
    let mut low_index = start;
    let mut high_value = high[start];
    let mut low_value = low[start];

    for index in start + 1..high.len() {
        if !high[index].is_finite() || !low[index].is_finite() {
            continue;
        }
        let rises_enough = |new: f64, old: f64| {
            if percent {
                new >= old * (1.0 + change / 100.0)
            } else {
                new - old >= change
            }
        };
        let falls_enough = |new: f64, old: f64| {
            if percent {
                new <= old * (1.0 - change / 100.0)
            } else {
                old - new >= change
            }
        };
        match direction {
            0 => {
                if high[index] > high_value {
                    high_value = high[index];
                    high_index = index;
                }
                if low[index] < low_value {
                    low_value = low[index];
                    low_index = index;
                }
                if rises_enough(high_value, low_value) {
                    output[low_index] = low_value;
                    direction = 1;
                } else if falls_enough(low_value, high_value) {
                    output[high_index] = high_value;
                    direction = -1;
                }
            }
            1 => {
                if high[index] >= high_value {
                    high_value = high[index];
                    high_index = index;
                }
                if falls_enough(low[index], high_value) {
                    output[high_index] = high_value;
                    low_value = low[index];
                    low_index = index;
                    direction = -1;
                }
            }
            _ => {
                if low[index] <= low_value {
                    low_value = low[index];
                    low_index = index;
                }
                if rises_enough(high[index], low_value) {
                    output[low_index] = low_value;
                    high_value = high[index];
                    high_index = index;
                    direction = 1;
                }
            }
        }
    }
    if direction >= 0 {
        output[high_index] = high_value;
    } else {
        output[low_index] = low_value;
    }
    Ok(output)
}

/// Floor Pivot Point levels calculated from the previous bar.
#[derive(Clone, Debug, PartialEq)]
pub struct PivotOutput {
    /// Central pivot point.
    pub pivot: Vec<f64>,
    /// First resistance.
    pub resistance_1: Vec<f64>,
    /// Second resistance.
    pub resistance_2: Vec<f64>,
    /// First support.
    pub support_1: Vec<f64>,
    /// Second support.
    pub support_2: Vec<f64>,
}

/// Calculate classic floor pivots from prior-period OHLC data.
pub fn pivots(high: &[f64], low: &[f64], close: &[f64]) -> Result<PivotOutput, IndicatorError> {
    require_high_low(high, low)?;
    validation::same_length(high.len(), &[("close", close.len())])?;
    let mut output = PivotOutput {
        pivot: vec![f64::NAN; high.len()],
        resistance_1: vec![f64::NAN; high.len()],
        resistance_2: vec![f64::NAN; high.len()],
        support_1: vec![f64::NAN; high.len()],
        support_2: vec![f64::NAN; high.len()],
    };
    for index in 1..high.len() {
        let prior = index - 1;
        if high[prior].is_finite() && low[prior].is_finite() && close[prior].is_finite() {
            let pivot = (high[prior] + low[prior] + close[prior]) / 3.0;
            output.pivot[index] = pivot;
            output.resistance_1[index] = 2.0 * pivot - low[prior];
            output.support_1[index] = 2.0 * pivot - high[prior];
            output.resistance_2[index] = pivot + high[prior] - low[prior];
            output.support_2[index] = pivot - high[prior] + low[prior];
        }
    }
    Ok(output)
}

/// Wilder Parabolic Stop and Reverse.
pub fn sar(
    high: &[f64],
    low: &[f64],
    acceleration: f64,
    maximum: f64,
) -> Result<Vec<f64>, IndicatorError> {
    require_high_low(high, low)?;
    validation::positive(acceleration, "acceleration")?;
    validation::positive(maximum, "maximum")?;
    if acceleration > maximum {
        return Err(IndicatorError::InvalidParameter("acceleration"));
    }
    let mut output = vec![f64::NAN; high.len()];
    if high.len() < 2 {
        return Ok(output);
    }
    let mut cursor = 0;
    while cursor + 1 < high.len() {
        let Some(relative_start) = (cursor..high.len() - 1).position(|i| {
            [high[i], low[i], high[i + 1], low[i + 1]]
                .iter()
                .all(|value| value.is_finite())
        }) else {
            break;
        };
        let start = cursor + relative_start;
        let mut long = (high[start + 1] + low[start + 1]) >= (high[start] + low[start]);
        let mut point = if long { low[start] } else { high[start] };
        let mut extreme = if long {
            high[start + 1]
        } else {
            low[start + 1]
        };
        let mut factor = acceleration;
        let mut index = start + 1;
        while index < high.len() && high[index].is_finite() && low[index].is_finite() {
            point += factor * (extreme - point);
            if long {
                point = point.min(low[index - 1]);
                if index > start + 1 {
                    point = point.min(low[index - 2]);
                }
                if low[index] < point {
                    long = false;
                    point = extreme;
                    extreme = low[index];
                    factor = acceleration;
                } else if high[index] > extreme {
                    extreme = high[index];
                    factor = (factor + acceleration).min(maximum);
                }
            } else {
                point = point.max(high[index - 1]);
                if index > start + 1 {
                    point = point.max(high[index - 2]);
                }
                if high[index] > point {
                    long = true;
                    point = extreme;
                    extreme = high[index];
                    factor = acceleration;
                } else if low[index] < extreme {
                    extreme = low[index];
                    factor = (factor + acceleration).min(maximum);
                }
            }
            output[index] = point;
            index += 1;
        }
        cursor = index.saturating_add(1);
    }
    Ok(output)
}

/// Rolling support, resistance, and midpoint levels.
#[derive(Clone, Debug, PartialEq)]
pub struct SupportResistanceOutput {
    /// Highest high in the lookback window.
    pub resistance: Vec<f64>,
    /// Lowest low in the lookback window.
    pub support: Vec<f64>,
    /// Midpoint between support and resistance.
    pub middle: Vec<f64>,
}

/// Calculate causal trailing support and resistance levels.
pub fn snr(
    high: &[f64],
    low: &[f64],
    period: usize,
) -> Result<SupportResistanceOutput, IndicatorError> {
    require_high_low(high, low)?;
    let resistance = rolling::rolling_max(high, period)?;
    let support = rolling::rolling_min(low, period)?;
    let middle = resistance
        .iter()
        .zip(&support)
        .map(|(&resistance, &support)| {
            if resistance.is_finite() && support.is_finite() {
                (resistance + support) / 2.0
            } else {
                f64::NAN
            }
        })
        .collect();
    Ok(SupportResistanceOutput {
        resistance,
        support,
        middle,
    })
}

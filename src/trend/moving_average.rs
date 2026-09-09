use crate::internal::{rolling, smoothing, validation};
use crate::IndicatorError;

/// Simple moving average over a trailing finite window.
pub fn sma(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    rolling::rolling_mean(input, period)
}

/// Exponential moving average seeded by the first finite simple average.
pub fn ema(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    smoothing::ema(input, period, false)
}

/// Double exponential moving average, `2 * EMA(x) - EMA(EMA(x))`.
pub fn dema(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    let first = ema(input, period)?;
    let second = ema(&first, period)?;
    Ok(zip_map(&first, &second, |a, b| 2.0 * a - b))
}

/// Linearly weighted moving average with weights `1..=period`.
pub fn wma(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    rolling::rolling_wma(input, period)
}

/// Hull moving average using weighted averages at `n/2`, `n`, and `sqrt(n)`.
pub fn hma(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    let half = wma(input, (period / 2).max(1))?;
    let full = wma(input, period)?;
    let difference = zip_map(&half, &full, |a, b| 2.0 * a - b);
    wma(
        &difference,
        (period as f64).sqrt().round().max(1.0) as usize,
    )
}

/// Zero-lag EMA using the de-lagged input `x[t] + (x[t] - x[t-lag])`.
pub fn zlema(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    let lag = (period - 1) / 2;
    if lag == 0 {
        return ema(input, period);
    }
    let mut adjusted = vec![f64::NAN; input.len()];
    for index in lag..input.len() {
        let current = input[index];
        let past = input[index - lag];
        if current.is_finite() && past.is_finite() {
            adjusted[index] = 2.0 * current - past;
        }
    }
    ema(&adjusted, period)
}

/// Arnaud Legoux moving average with Gaussian weights.
pub fn alma(
    input: &[f64],
    period: usize,
    offset: f64,
    sigma: f64,
) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    if !offset.is_finite() || !(0.0..=1.0).contains(&offset) {
        return Err(IndicatorError::InvalidParameter("offset"));
    }
    validation::positive(sigma, "sigma")?;
    let mut output = vec![f64::NAN; input.len()];
    if period > input.len() {
        return Ok(output);
    }
    let center = offset * (period - 1) as f64;
    let width = period as f64 / sigma;
    let weights: Vec<f64> = (0..period)
        .map(|index| (-((index as f64 - center).powi(2)) / (2.0 * width * width)).exp())
        .collect();
    let normalizer: f64 = weights.iter().sum();
    for index in period - 1..input.len() {
        let window = &input[index + 1 - period..=index];
        if window.iter().all(|value| value.is_finite()) {
            output[index] = window
                .iter()
                .zip(&weights)
                .map(|(value, weight)| value * weight)
                .sum::<f64>()
                / normalizer;
        }
    }
    Ok(output)
}

/// Elastic volume-weighted moving average.
pub fn evwma(price: &[f64], volume: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    validation::same_length(price.len(), &[("volume", volume.len())])?;
    let volume_sum = rolling::rolling_sum(volume, period)?;
    let mut output = vec![f64::NAN; price.len()];
    let mut consecutive = 0usize;
    for index in 0..price.len() {
        if !price[index].is_finite() || !volume[index].is_finite() {
            consecutive = 0;
            continue;
        }
        consecutive += 1;
        if consecutive < period || !volume_sum[index].is_finite() {
            continue;
        }
        if index == 0 || !output[index - 1].is_finite() {
            output[index] = price[index];
        } else if volume_sum[index] != 0.0 {
            output[index] = ((volume_sum[index] - volume[index]) * output[index - 1]
                + volume[index] * price[index])
                / volume_sum[index];
        }
    }
    Ok(output)
}

/// Trailing volume-weighted moving average.
pub fn vwma(price: &[f64], volume: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::same_length(price.len(), &[("volume", volume.len())])?;
    let products: Vec<f64> = price
        .iter()
        .zip(volume)
        .map(|(price, volume)| {
            if price.is_finite() && volume.is_finite() {
                price * volume
            } else {
                f64::NAN
            }
        })
        .collect();
    let numerator = rolling::rolling_sum(&products, period)?;
    let denominator = rolling::rolling_sum(volume, period)?;
    Ok(zip_map(&numerator, &denominator, |sum, volume| {
        if volume == 0.0 {
            f64::NAN
        } else {
            sum / volume
        }
    }))
}

/// Guppy Multiple Moving Average using the standard 6 short and 6 long periods.
#[derive(Clone, Debug, PartialEq)]
pub struct GmmaOutput {
    /// EMAs for periods 3, 5, 8, 10, 12, and 15.
    pub short: [Vec<f64>; 6],
    /// EMAs for periods 30, 35, 40, 45, 50, and 60.
    pub long: [Vec<f64>; 6],
}

/// Calculate the standard Guppy Multiple Moving Average ribbons.
pub fn gmma(input: &[f64]) -> Result<GmmaOutput, IndicatorError> {
    let short = [3, 5, 8, 10, 12, 15].map(|period| ema(input, period));
    let long = [30, 35, 40, 45, 50, 60].map(|period| ema(input, period));
    Ok(GmmaOutput {
        short: collect_array(short)?,
        long: collect_array(long)?,
    })
}

fn collect_array<const N: usize>(
    values: [Result<Vec<f64>, IndicatorError>; N],
) -> Result<[Vec<f64>; N], IndicatorError> {
    let collected: Result<Vec<_>, _> = values.into_iter().collect();
    collected?
        .try_into()
        .map_err(|_| IndicatorError::InvalidParameter("array length"))
}

pub(crate) fn zip_map(
    left: &[f64],
    right: &[f64],
    calculate: impl Fn(f64, f64) -> f64,
) -> Vec<f64> {
    left.iter()
        .zip(right)
        .map(|(&left, &right)| {
            if left.is_finite() && right.is_finite() {
                calculate(left, right)
            } else {
                f64::NAN
            }
        })
        .collect()
}

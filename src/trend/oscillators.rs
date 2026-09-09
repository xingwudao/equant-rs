use crate::internal::{rolling, validation};
use crate::trend::moving_average::{ema, zip_map};
use crate::IndicatorError;

/// Moving Average Convergence Divergence output.
#[derive(Clone, Debug, PartialEq)]
pub struct MacdOutput {
    /// Fast EMA minus slow EMA.
    pub line: Vec<f64>,
    /// EMA of the MACD line.
    pub signal: Vec<f64>,
    /// MACD line minus signal line.
    pub histogram: Vec<f64>,
}

/// Calculate MACD using SMA-seeded exponential averages.
pub fn macd(
    input: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Result<MacdOutput, IndicatorError> {
    validation::period(fast_period)?;
    validation::period(slow_period)?;
    validation::period(signal_period)?;
    if fast_period >= slow_period {
        return Err(IndicatorError::InvalidParameter("fast_period"));
    }
    let fast = ema(input, fast_period)?;
    let slow = ema(input, slow_period)?;
    let line = zip_map(&fast, &slow, |fast, slow| fast - slow);
    let signal = ema(&line, signal_period)?;
    let histogram = zip_map(&line, &signal, |line, signal| line - signal);
    Ok(MacdOutput {
        line,
        signal,
        histogram,
    })
}

/// Trend Detection Index output.
#[derive(Clone, Debug, PartialEq)]
pub struct TdiOutput {
    /// Trend Detection Index.
    pub trend: Vec<f64>,
    /// Direction Indicator, a rolling sum of momentum.
    pub direction: Vec<f64>,
}

/// Calculate the Trend Detection Index from price momentum.
pub fn tdi(
    input: &[f64],
    period: usize,
    long_period: usize,
) -> Result<TdiOutput, IndicatorError> {
    validation::period(period)?;
    validation::period(long_period)?;
    if long_period < period {
        return Err(IndicatorError::InvalidParameter("long_period"));
    }
    let mut momentum = vec![f64::NAN; input.len()];
    for index in period..input.len() {
        if input[index].is_finite() && input[index - period].is_finite() {
            momentum[index] = input[index] - input[index - period];
        }
    }
    let direction = rolling::rolling_sum(&momentum, period)?;
    let absolute: Vec<f64> = momentum.iter().map(|value| value.abs()).collect();
    let short_abs = rolling::rolling_sum(&absolute, period)?;
    let long_abs = rolling::rolling_sum(&absolute, long_period)?;
    let trend = (0..input.len())
        .map(|index| {
            if direction[index].is_finite()
                && short_abs[index].is_finite()
                && long_abs[index].is_finite()
            {
                direction[index].abs() - (long_abs[index] - short_abs[index])
            } else {
                f64::NAN
            }
        })
        .collect();
    Ok(TdiOutput { trend, direction })
}

/// Triple exponential oscillator output.
#[derive(Clone, Debug, PartialEq)]
pub struct TrixOutput {
    /// One-period percentage rate of change of the triple EMA.
    pub line: Vec<f64>,
    /// EMA signal line.
    pub signal: Vec<f64>,
}

/// Calculate TRIX and its signal line.
pub fn trix(
    input: &[f64],
    period: usize,
    signal_period: usize,
) -> Result<TrixOutput, IndicatorError> {
    let first = ema(input, period)?;
    let second = ema(&first, period)?;
    let third = ema(&second, period)?;
    let mut line = vec![f64::NAN; input.len()];
    for index in 1..input.len() {
        if third[index].is_finite() && third[index - 1].is_finite() && third[index - 1] != 0.0 {
            line[index] = 100.0 * (third[index] / third[index - 1] - 1.0);
        }
    }
    let signal = ema(&line, signal_period)?;
    Ok(TrixOutput { line, signal })
}

/// Causal Detrended Price Oscillator using a lagged price and current SMA.
pub fn dpo(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    let average = rolling::rolling_mean(input, period)?;
    let shift = period / 2 + 1;
    let mut output = vec![f64::NAN; input.len()];
    for index in shift..input.len() {
        if input[index - shift].is_finite() && average[index].is_finite() {
            output[index] = input[index - shift] - average[index];
        }
    }
    Ok(output)
}

/// Vertical Horizontal Filter, a dimensionless trend-strength ratio.
pub fn vhf(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    let high = rolling::rolling_max(input, period)?;
    let low = rolling::rolling_min(input, period)?;
    let mut movement = vec![f64::NAN; input.len()];
    for index in 1..input.len() {
        if input[index].is_finite() && input[index - 1].is_finite() {
            movement[index] = (input[index] - input[index - 1]).abs();
        }
    }
    let path = rolling::rolling_sum(&movement, period.saturating_sub(1).max(1))?;
    Ok((0..input.len())
        .map(|index| {
            if high[index].is_finite() && low[index].is_finite() && path[index].is_finite() {
                if path[index] == 0.0 {
                    0.0
                } else {
                    (high[index] - low[index]) / path[index]
                }
            } else {
                f64::NAN
            }
        })
        .collect())
}

/// Standard Know Sure Thing configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KstConfig {
    /// Rate-of-change periods.
    pub roc_periods: [usize; 4],
    /// Moving-average periods applied to each ROC.
    pub sma_periods: [usize; 4],
    /// Integer component weights.
    pub weights: [u32; 4],
    /// Signal moving-average period.
    pub signal_period: usize,
}

impl Default for KstConfig {
    fn default() -> Self {
        Self {
            roc_periods: [10, 15, 20, 30],
            sma_periods: [10, 10, 10, 15],
            weights: [1, 2, 3, 4],
            signal_period: 9,
        }
    }
}

/// Know Sure Thing output.
#[derive(Clone, Debug, PartialEq)]
pub struct KstOutput {
    /// Weighted sum of smoothed rate-of-change components.
    pub line: Vec<f64>,
    /// Simple moving average signal line.
    pub signal: Vec<f64>,
}

/// Calculate the Know Sure Thing oscillator.
pub fn kst(input: &[f64], config: KstConfig) -> Result<KstOutput, IndicatorError> {
    for period in config.roc_periods.into_iter().chain(config.sma_periods) {
        validation::period(period)?;
    }
    validation::period(config.signal_period)?;
    let mut components = Vec::with_capacity(4);
    for index in 0..4 {
        let roc = rate_of_change(input, config.roc_periods[index]);
        components.push(rolling::rolling_mean(&roc, config.sma_periods[index])?);
    }
    let mut line = vec![f64::NAN; input.len()];
    for index in 0..input.len() {
        if components.iter().all(|component| component[index].is_finite()) {
            line[index] = components
                .iter()
                .zip(config.weights)
                .map(|(component, weight)| component[index] * weight as f64)
                .sum();
        }
    }
    let signal = rolling::rolling_mean(&line, config.signal_period)?;
    Ok(KstOutput { line, signal })
}

/// Percentage Price Oscillator, `100 * (EMA_fast - EMA_slow) / EMA_slow`.
pub fn po(
    input: &[f64],
    fast_period: usize,
    slow_period: usize,
) -> Result<Vec<f64>, IndicatorError> {
    if fast_period >= slow_period {
        return Err(IndicatorError::InvalidParameter("fast_period"));
    }
    let fast = ema(input, fast_period)?;
    let slow = ema(input, slow_period)?;
    Ok(zip_map(&fast, &slow, |fast, slow| {
        if slow == 0.0 {
            f64::NAN
        } else {
            100.0 * (fast - slow) / slow
        }
    }))
}

fn rate_of_change(input: &[f64], period: usize) -> Vec<f64> {
    let mut output = vec![f64::NAN; input.len()];
    for index in period..input.len() {
        let previous = input[index - period];
        if input[index].is_finite() && previous.is_finite() && previous != 0.0 {
            output[index] = 100.0 * (input[index] / previous - 1.0);
        }
    }
    output
}

use crate::internal::{rolling, smoothing, validation};
use crate::IndicatorError;

fn require_ohlc(high: &[f64], low: &[f64], close: &[f64]) -> Result<(), IndicatorError> {
    validation::same_length(high.len(), &[("low", low.len()), ("close", close.len())])
}

fn change(input: &[f64]) -> Vec<f64> {
    let mut output = vec![f64::NAN; input.len()];
    for index in 1..input.len() {
        if input[index].is_finite() && input[index - 1].is_finite() {
            output[index] = input[index] - input[index - 1];
        }
    }
    output
}

/// Wilder Relative Strength Index in the range 0 to 100.
pub fn rsi(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    let changes = change(input);
    let gains: Vec<f64> = changes
        .iter()
        .map(|value| {
            if value.is_finite() {
                value.max(0.0)
            } else {
                f64::NAN
            }
        })
        .collect();
    let losses: Vec<f64> = changes
        .iter()
        .map(|value| {
            if value.is_finite() {
                (-value).max(0.0)
            } else {
                f64::NAN
            }
        })
        .collect();
    let average_gain = smoothing::ema(&gains, period, true)?;
    let average_loss = smoothing::ema(&losses, period, true)?;
    Ok(average_gain
        .iter()
        .zip(average_loss)
        .map(|(&gain, loss)| {
            if !gain.is_finite() || !loss.is_finite() {
                f64::NAN
            } else if gain == 0.0 && loss == 0.0 {
                50.0
            } else if loss == 0.0 {
                100.0
            } else {
                100.0 - 100.0 / (1.0 + gain / loss)
            }
        })
        .collect())
}

/// Commodity Channel Index using typical price and a 0.015 scale constant.
pub fn cci(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> Result<Vec<f64>, IndicatorError> {
    require_ohlc(high, low, close)?;
    validation::period(period)?;
    let typical: Vec<f64> = high
        .iter()
        .zip(low)
        .zip(close)
        .map(|((&high, &low), &close)| {
            if high.is_finite() && low.is_finite() && close.is_finite() {
                (high + low + close) / 3.0
            } else {
                f64::NAN
            }
        })
        .collect();
    let average = rolling::rolling_mean(&typical, period)?;
    let mut output = vec![f64::NAN; high.len()];
    for index in period - 1..high.len() {
        let window = &typical[index + 1 - period..=index];
        if average[index].is_finite() && window.iter().all(|value| value.is_finite()) {
            let deviation = window
                .iter()
                .map(|value| (value - average[index]).abs())
                .sum::<f64>()
                / period as f64;
            output[index] = if deviation == 0.0 {
                0.0
            } else {
                (typical[index] - average[index]) / (0.015 * deviation)
            };
        }
    }
    Ok(output)
}

/// Chande Momentum Oscillator in the range -100 to 100.
pub fn cmo(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    let changes = change(input);
    let gains: Vec<f64> = changes
        .iter()
        .map(|value| {
            if value.is_finite() {
                value.max(0.0)
            } else {
                f64::NAN
            }
        })
        .collect();
    let losses: Vec<f64> = changes
        .iter()
        .map(|value| {
            if value.is_finite() {
                (-value).max(0.0)
            } else {
                f64::NAN
            }
        })
        .collect();
    let up = rolling::rolling_sum(&gains, period)?;
    let down = rolling::rolling_sum(&losses, period)?;
    Ok(up
        .iter()
        .zip(down)
        .map(|(&up, down)| {
            if up.is_finite() && down.is_finite() {
                let total = up + down;
                if total == 0.0 {
                    0.0
                } else {
                    100.0 * (up - down) / total
                }
            } else {
                f64::NAN
            }
        })
        .collect())
}

/// True Strength Index from double-smoothed price changes.
pub fn tsi(
    input: &[f64],
    slow_period: usize,
    fast_period: usize,
) -> Result<Vec<f64>, IndicatorError> {
    let momentum = change(input);
    let absolute: Vec<f64> = momentum.iter().map(|value| value.abs()).collect();
    let numerator = smoothing::ema(
        &smoothing::ema(&momentum, slow_period, false)?,
        fast_period,
        false,
    )?;
    let denominator = smoothing::ema(
        &smoothing::ema(&absolute, slow_period, false)?,
        fast_period,
        false,
    )?;
    Ok(numerator
        .iter()
        .zip(denominator)
        .map(|(&numerator, denominator)| {
            if numerator.is_finite() && denominator.is_finite() {
                if denominator == 0.0 {
                    0.0
                } else {
                    100.0 * numerator / denominator
                }
            } else {
                f64::NAN
            }
        })
        .collect())
}

/// Stochastic Momentum Index output.
#[derive(Clone, Debug, PartialEq)]
pub struct SmiOutput {
    /// Double-smoothed stochastic momentum line.
    pub line: Vec<f64>,
    /// EMA signal line.
    pub signal: Vec<f64>,
}

/// Calculate the Stochastic Momentum Index.
pub fn smi(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    range_period: usize,
    smooth_period: usize,
    signal_period: usize,
) -> Result<SmiOutput, IndicatorError> {
    require_ohlc(high, low, close)?;
    let highest = rolling::rolling_max(high, range_period)?;
    let lowest = rolling::rolling_min(low, range_period)?;
    let distance: Vec<f64> = (0..close.len())
        .map(|index| {
            if close[index].is_finite() && highest[index].is_finite() && lowest[index].is_finite() {
                close[index] - (highest[index] + lowest[index]) / 2.0
            } else {
                f64::NAN
            }
        })
        .collect();
    let range: Vec<f64> = highest
        .iter()
        .zip(lowest)
        .map(|(&high, low)| {
            if high.is_finite() && low.is_finite() {
                high - low
            } else {
                f64::NAN
            }
        })
        .collect();
    let numerator = smoothing::ema(
        &smoothing::ema(&distance, smooth_period, false)?,
        smooth_period,
        false,
    )?;
    let denominator = smoothing::ema(
        &smoothing::ema(&range, smooth_period, false)?,
        smooth_period,
        false,
    )?;
    let line: Vec<f64> = numerator
        .iter()
        .zip(denominator)
        .map(|(&numerator, denominator)| {
            if numerator.is_finite() && denominator.is_finite() {
                if denominator == 0.0 {
                    0.0
                } else {
                    200.0 * numerator / denominator
                }
            } else {
                f64::NAN
            }
        })
        .collect();
    let signal = smoothing::ema(&line, signal_period, false)?;
    Ok(SmiOutput { line, signal })
}

/// Williams Percent Range in the range -100 to 0.
pub fn wpr(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> Result<Vec<f64>, IndicatorError> {
    require_ohlc(high, low, close)?;
    let highest = rolling::rolling_max(high, period)?;
    let lowest = rolling::rolling_min(low, period)?;
    Ok((0..close.len())
        .map(|index| {
            let range = highest[index] - lowest[index];
            if highest[index].is_finite() && close[index].is_finite() {
                if range == 0.0 {
                    0.0
                } else {
                    -100.0 * (highest[index] - close[index]) / range
                }
            } else {
                f64::NAN
            }
        })
        .collect())
}

/// Larry Williams' Ultimate Oscillator using weights 4, 2, and 1.
pub fn ultimate_oscillator(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    short_period: usize,
    medium_period: usize,
    long_period: usize,
) -> Result<Vec<f64>, IndicatorError> {
    require_ohlc(high, low, close)?;
    if !(short_period < medium_period && medium_period < long_period) {
        return Err(IndicatorError::InvalidParameter("oscillator periods"));
    }
    let mut buying_pressure = vec![f64::NAN; close.len()];
    let mut true_range = vec![f64::NAN; close.len()];
    for index in 1..close.len() {
        if [high[index], low[index], close[index], close[index - 1]]
            .iter()
            .all(|value| value.is_finite())
        {
            let true_low = low[index].min(close[index - 1]);
            let true_high = high[index].max(close[index - 1]);
            buying_pressure[index] = close[index] - true_low;
            true_range[index] = true_high - true_low;
        }
    }
    let average = |period| -> Result<Vec<f64>, IndicatorError> {
        let pressure = rolling::rolling_sum(&buying_pressure, period)?;
        let range = rolling::rolling_sum(&true_range, period)?;
        Ok(pressure
            .iter()
            .zip(range)
            .map(|(&pressure, range)| {
                if pressure.is_finite() && range.is_finite() && range != 0.0 {
                    pressure / range
                } else {
                    f64::NAN
                }
            })
            .collect())
    };
    let short = average(short_period)?;
    let medium = average(medium_period)?;
    let long = average(long_period)?;
    Ok((0..close.len())
        .map(|index| {
            if short[index].is_finite() && medium[index].is_finite() && long[index].is_finite() {
                100.0 * (4.0 * short[index] + 2.0 * medium[index] + long[index]) / 7.0
            } else {
                f64::NAN
            }
        })
        .collect())
}

/// Percentage Rate of Change over `period` observations.
pub fn roc(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    let mut output = vec![f64::NAN; input.len()];
    for index in period..input.len() {
        let prior = input[index - period];
        if input[index].is_finite() && prior.is_finite() && prior != 0.0 {
            output[index] = 100.0 * (input[index] / prior - 1.0);
        }
    }
    Ok(output)
}

/// Arithmetic price momentum, `x[t] - x[t-period]`.
pub fn momentum(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    let mut output = vec![f64::NAN; input.len()];
    for index in period..input.len() {
        if input[index].is_finite() && input[index - period].is_finite() {
            output[index] = input[index] - input[index - period];
        }
    }
    Ok(output)
}

/// Correlation Trend Indicator: price correlation with `1..=period`.
pub fn cti(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    if period < 2 {
        return Err(IndicatorError::InvalidParameter("cti period"));
    }
    let mut output = vec![f64::NAN; input.len()];
    let x_mean = (period as f64 + 1.0) / 2.0;
    let x_variance: f64 = (1..=period).map(|x| (x as f64 - x_mean).powi(2)).sum();
    for index in period - 1..input.len() {
        let window = &input[index + 1 - period..=index];
        if window.iter().all(|value| value.is_finite()) {
            let mean = window.iter().sum::<f64>() / period as f64;
            let covariance: f64 = window
                .iter()
                .enumerate()
                .map(|(offset, value)| (offset as f64 + 1.0 - x_mean) * (value - mean))
                .sum();
            let y_variance: f64 = window.iter().map(|value| (value - mean).powi(2)).sum();
            output[index] = if y_variance == 0.0 {
                0.0
            } else {
                covariance / (x_variance * y_variance).sqrt()
            };
        }
    }
    Ok(output)
}

/// Relative Vigor Index output.
#[derive(Clone, Debug, PartialEq)]
pub struct RviOutput {
    /// Relative Vigor Index line.
    pub line: Vec<f64>,
    /// Four-period symmetrically weighted signal line.
    pub signal: Vec<f64>,
}

/// Calculate the Relative Vigor Index.
pub fn rvi(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> Result<RviOutput, IndicatorError> {
    validation::same_length(
        open.len(),
        &[
            ("high", high.len()),
            ("low", low.len()),
            ("close", close.len()),
        ],
    )?;
    let mut numerator = vec![f64::NAN; open.len()];
    let mut denominator = vec![f64::NAN; open.len()];
    for index in 3..open.len() {
        let rows = index - 3..=index;
        if rows.clone().all(|i| {
            [open[i], high[i], low[i], close[i]]
                .iter()
                .all(|v| v.is_finite())
        }) {
            numerator[index] = ((close[index] - open[index])
                + 2.0 * (close[index - 1] - open[index - 1])
                + 2.0 * (close[index - 2] - open[index - 2])
                + (close[index - 3] - open[index - 3]))
                / 6.0;
            denominator[index] = ((high[index] - low[index])
                + 2.0 * (high[index - 1] - low[index - 1])
                + 2.0 * (high[index - 2] - low[index - 2])
                + (high[index - 3] - low[index - 3]))
                / 6.0;
        }
    }
    let numerator = rolling::rolling_sum(&numerator, period)?;
    let denominator = rolling::rolling_sum(&denominator, period)?;
    let line: Vec<f64> = numerator
        .iter()
        .zip(denominator)
        .map(|(&numerator, denominator)| {
            if numerator.is_finite() && denominator.is_finite() && denominator != 0.0 {
                numerator / denominator
            } else {
                f64::NAN
            }
        })
        .collect();
    let mut signal = vec![f64::NAN; open.len()];
    for index in 3..open.len() {
        if line[index - 3..=index]
            .iter()
            .all(|value| value.is_finite())
        {
            signal[index] =
                (line[index] + 2.0 * line[index - 1] + 2.0 * line[index - 2] + line[index - 3])
                    / 6.0;
        }
    }
    Ok(RviOutput { line, signal })
}

/// Dynamic Volatility Index as short volatility divided by long volatility.
pub fn dvi(
    input: &[f64],
    long_period: usize,
    short_period: usize,
) -> Result<Vec<f64>, IndicatorError> {
    if short_period >= long_period {
        return Err(IndicatorError::InvalidParameter("short_period"));
    }
    let returns = change(input);
    let short = rolling::rolling_std(&returns, short_period, true)?;
    let long = rolling::rolling_std(&returns, long_period, true)?;
    Ok(short
        .iter()
        .zip(long)
        .map(|(&short, long)| {
            if short.is_finite() && long.is_finite() {
                if long == 0.0 {
                    0.0
                } else {
                    100.0 * short / long
                }
            } else {
                f64::NAN
            }
        })
        .collect())
}

/// Fast and slow Stochastic Oscillator output.
#[derive(Clone, Debug, PartialEq)]
pub struct StochasticOutput {
    /// Fast percent K.
    pub fast_k: Vec<f64>,
    /// Simple moving average of fast K.
    pub fast_d: Vec<f64>,
    /// Simple moving average of fast D.
    pub slow_d: Vec<f64>,
}

/// Calculate fast K, fast D, and slow D stochastic lines.
pub fn stoch(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    k_period: usize,
    d_period: usize,
    slow_period: usize,
) -> Result<StochasticOutput, IndicatorError> {
    require_ohlc(high, low, close)?;
    let highest = rolling::rolling_max(high, k_period)?;
    let lowest = rolling::rolling_min(low, k_period)?;
    let fast_k: Vec<f64> = (0..close.len())
        .map(|index| {
            let range = highest[index] - lowest[index];
            if close[index].is_finite() && highest[index].is_finite() {
                if range == 0.0 {
                    50.0
                } else {
                    100.0 * (close[index] - lowest[index]) / range
                }
            } else {
                f64::NAN
            }
        })
        .collect();
    let fast_d = rolling::rolling_mean(&fast_k, d_period)?;
    let slow_d = rolling::rolling_mean(&fast_d, slow_period)?;
    Ok(StochasticOutput {
        fast_k,
        fast_d,
        slow_d,
    })
}

/// KDJ oscillator output.
#[derive(Clone, Debug, PartialEq)]
pub struct KdjOutput {
    /// Smoothed RSV K line.
    pub k: Vec<f64>,
    /// Smoothed K D line.
    pub d: Vec<f64>,
    /// Divergence line, `3K - 2D`.
    pub j: Vec<f64>,
}

/// Calculate the Chinese-market KDJ variant.
pub fn kdj(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
    k_smooth: usize,
    d_smooth: usize,
) -> Result<KdjOutput, IndicatorError> {
    validation::period(k_smooth)?;
    validation::period(d_smooth)?;
    let stochastic = stoch(high, low, close, period, 1, 1)?;
    let mut k = vec![f64::NAN; close.len()];
    let mut d = vec![f64::NAN; close.len()];
    let mut previous_k = 50.0;
    let mut previous_d = 50.0;
    for (index, &rsv) in stochastic.fast_k.iter().enumerate() {
        if rsv.is_finite() {
            previous_k = ((k_smooth - 1) as f64 * previous_k + rsv) / k_smooth as f64;
            previous_d = ((d_smooth - 1) as f64 * previous_d + previous_k) / d_smooth as f64;
            k[index] = previous_k;
            d[index] = previous_d;
        } else {
            previous_k = 50.0;
            previous_d = 50.0;
        }
    }
    let j = k
        .iter()
        .zip(&d)
        .map(|(&k, &d)| {
            if k.is_finite() && d.is_finite() {
                3.0 * k - 2.0 * d
            } else {
                f64::NAN
            }
        })
        .collect();
    Ok(KdjOutput { k, d, j })
}

use crate::internal::{rolling, smoothing, validation};
use crate::IndicatorError;

fn require_ohlcv(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<(), IndicatorError> {
    validation::same_length(
        high.len(),
        &[
            ("low", low.len()),
            ("close", close.len()),
            ("volume", volume.len()),
        ],
    )
}

/// On-Balance Volume, starting each finite segment at zero.
pub fn obv(close: &[f64], volume: &[f64]) -> Result<Vec<f64>, IndicatorError> {
    validation::same_length(close.len(), &[("volume", volume.len())])?;
    let mut output = vec![f64::NAN; close.len()];
    let mut previous = None;
    let mut cumulative = 0.0;
    for index in 0..close.len() {
        if !close[index].is_finite() || !volume[index].is_finite() {
            previous = None;
            cumulative = 0.0;
            continue;
        }
        if let Some(prior) = previous {
            if close[index] > prior {
                cumulative += volume[index];
            } else if close[index] < prior {
                cumulative -= volume[index];
            }
        }
        output[index] = cumulative;
        previous = Some(close[index]);
    }
    Ok(output)
}

/// Close Location Value in the range -1 to 1.
pub fn clv(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, IndicatorError> {
    validation::same_length(high.len(), &[("low", low.len()), ("close", close.len())])?;
    Ok((0..high.len())
        .map(|index| {
            if high[index].is_finite() && low[index].is_finite() && close[index].is_finite() {
                let range = high[index] - low[index];
                if range == 0.0 {
                    0.0
                } else {
                    (2.0 * close[index] - high[index] - low[index]) / range
                }
            } else {
                f64::NAN
            }
        })
        .collect())
}

/// Chaikin Money Flow over a trailing window.
pub fn cmf(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
    period: usize,
) -> Result<Vec<f64>, IndicatorError> {
    require_ohlcv(high, low, close, volume)?;
    let location = clv(high, low, close)?;
    let money_volume: Vec<f64> = location
        .iter()
        .zip(volume)
        .map(|(&location, &volume)| {
            if location.is_finite() && volume.is_finite() {
                location * volume
            } else {
                f64::NAN
            }
        })
        .collect();
    let numerator = rolling::rolling_sum(&money_volume, period)?;
    let denominator = rolling::rolling_sum(volume, period)?;
    Ok(numerator
        .iter()
        .zip(denominator)
        .map(|(&numerator, denominator)| {
            if numerator.is_finite() && denominator.is_finite() {
                if denominator == 0.0 {
                    0.0
                } else {
                    numerator / denominator
                }
            } else {
                f64::NAN
            }
        })
        .collect())
}

/// Rolling volume-weighted average of typical price.
pub fn vwap(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
    period: usize,
) -> Result<Vec<f64>, IndicatorError> {
    require_ohlcv(high, low, close, volume)?;
    let weighted: Vec<f64> = (0..high.len())
        .map(|index| {
            if [high[index], low[index], close[index], volume[index]]
                .iter()
                .all(|v| v.is_finite())
            {
                (high[index] + low[index] + close[index]) / 3.0 * volume[index]
            } else {
                f64::NAN
            }
        })
        .collect();
    let numerator = rolling::rolling_sum(&weighted, period)?;
    let denominator = rolling::rolling_sum(volume, period)?;
    Ok(numerator
        .iter()
        .zip(denominator)
        .map(|(&numerator, denominator)| {
            if numerator.is_finite() && denominator.is_finite() && denominator != 0.0 {
                numerator / denominator
            } else {
                f64::NAN
            }
        })
        .collect())
}

/// Money Flow Index in the range 0 to 100.
pub fn mfi(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
    period: usize,
) -> Result<Vec<f64>, IndicatorError> {
    require_ohlcv(high, low, close, volume)?;
    let typical: Vec<f64> = (0..high.len())
        .map(|index| {
            if high[index].is_finite() && low[index].is_finite() && close[index].is_finite() {
                (high[index] + low[index] + close[index]) / 3.0
            } else {
                f64::NAN
            }
        })
        .collect();
    let mut positive = vec![f64::NAN; high.len()];
    let mut negative = vec![f64::NAN; high.len()];
    for index in 1..high.len() {
        if typical[index].is_finite() && typical[index - 1].is_finite() && volume[index].is_finite()
        {
            let flow = typical[index] * volume[index];
            positive[index] = if typical[index] > typical[index - 1] {
                flow
            } else {
                0.0
            };
            negative[index] = if typical[index] < typical[index - 1] {
                flow
            } else {
                0.0
            };
        }
    }
    let positive = rolling::rolling_sum(&positive, period)?;
    let negative = rolling::rolling_sum(&negative, period)?;
    Ok(positive
        .iter()
        .zip(negative)
        .map(|(&positive, negative)| {
            if positive.is_finite() && negative.is_finite() {
                if positive == 0.0 && negative == 0.0 {
                    50.0
                } else if negative == 0.0 {
                    100.0
                } else {
                    100.0 - 100.0 / (1.0 + positive / negative)
                }
            } else {
                f64::NAN
            }
        })
        .collect())
}

/// Ease of Movement smoothed by a simple moving average.
pub fn emv(
    high: &[f64],
    low: &[f64],
    volume: &[f64],
    period: usize,
) -> Result<Vec<f64>, IndicatorError> {
    validation::same_length(high.len(), &[("low", low.len()), ("volume", volume.len())])?;
    let mut raw = vec![f64::NAN; high.len()];
    for index in 1..high.len() {
        if [
            high[index],
            low[index],
            high[index - 1],
            low[index - 1],
            volume[index],
        ]
        .iter()
        .all(|value| value.is_finite())
        {
            let distance = (high[index] + low[index] - high[index - 1] - low[index - 1]) / 2.0;
            let range = high[index] - low[index];
            raw[index] = if volume[index] == 0.0 {
                0.0
            } else {
                distance * range / volume[index]
            };
        }
    }
    rolling::rolling_mean(&raw, period)
}

/// Chaikin Accumulation/Distribution line, resetting after missing data.
pub fn chaikin_ad(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<Vec<f64>, IndicatorError> {
    require_ohlcv(high, low, close, volume)?;
    let location = clv(high, low, close)?;
    let mut output = vec![f64::NAN; high.len()];
    let mut cumulative = 0.0;
    for index in 0..high.len() {
        if location[index].is_finite() && volume[index].is_finite() {
            cumulative += location[index] * volume[index];
            output[index] = cumulative;
        } else {
            cumulative = 0.0;
        }
    }
    Ok(output)
}

/// Percentage rate of change of an EMA of the daily high-low range.
pub fn chaikin_volatility(
    high: &[f64],
    low: &[f64],
    ema_period: usize,
    roc_period: usize,
) -> Result<Vec<f64>, IndicatorError> {
    validation::same_length(high.len(), &[("low", low.len())])?;
    validation::period(roc_period)?;
    let range: Vec<f64> = high
        .iter()
        .zip(low)
        .map(|(&high, &low)| {
            if high.is_finite() && low.is_finite() {
                high - low
            } else {
                f64::NAN
            }
        })
        .collect();
    let smoothed = smoothing::ema(&range, ema_period, false)?;
    let mut output = vec![f64::NAN; high.len()];
    for index in roc_period..high.len() {
        let prior = smoothed[index - roc_period];
        if smoothed[index].is_finite() && prior.is_finite() && prior != 0.0 {
            output[index] = 100.0 * (smoothed[index] / prior - 1.0);
        }
    }
    Ok(output)
}

/// Williams Accumulation/Distribution line.
pub fn williams_ad(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, IndicatorError> {
    validation::same_length(high.len(), &[("low", low.len()), ("close", close.len())])?;
    let mut output = vec![f64::NAN; close.len()];
    let mut cumulative = 0.0;
    if !close.is_empty() && close[0].is_finite() {
        output[0] = 0.0;
    }
    for index in 1..close.len() {
        if [high[index], low[index], close[index], close[index - 1]]
            .iter()
            .all(|v| v.is_finite())
        {
            let movement = if close[index] > close[index - 1] {
                close[index] - low[index].min(close[index - 1])
            } else if close[index] < close[index - 1] {
                close[index] - high[index].max(close[index - 1])
            } else {
                0.0
            };
            cumulative += movement;
            output[index] = cumulative;
        } else {
            cumulative = 0.0;
        }
    }
    Ok(output)
}

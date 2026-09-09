use crate::internal::{rolling, smoothing, validation};
use crate::IndicatorError;

fn require_ohlc(high: &[f64], low: &[f64], close: &[f64]) -> Result<(), IndicatorError> {
    validation::same_length(high.len(), &[("low", low.len()), ("close", close.len())])
}

/// True Range, including gaps from the previous close.
pub fn tr(high: &[f64], low: &[f64], close: &[f64]) -> Result<Vec<f64>, IndicatorError> {
    require_ohlc(high, low, close)?;
    let mut output = vec![f64::NAN; high.len()];
    for index in 1..high.len() {
        if high[index].is_finite()
            && low[index].is_finite()
            && close[index - 1].is_finite()
        {
            output[index] = (high[index] - low[index])
                .max((high[index] - close[index - 1]).abs())
                .max((low[index] - close[index - 1]).abs());
        }
    }
    Ok(output)
}

/// Average True Range using Wilder smoothing.
pub fn atr(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> Result<Vec<f64>, IndicatorError> {
    smoothing::ema(&tr(high, low, close)?, period, true)
}

/// Bollinger Bands and normalized position measures.
#[derive(Clone, Debug, PartialEq)]
pub struct BollingerOutput {
    /// Simple moving average.
    pub middle: Vec<f64>,
    /// Middle plus the configured population-standard-deviation multiple.
    pub upper: Vec<f64>,
    /// Middle minus the configured population-standard-deviation multiple.
    pub lower: Vec<f64>,
    /// Position within the bands, where 0 is lower and 1 is upper.
    pub percent_b: Vec<f64>,
    /// Band width divided by the absolute middle value.
    pub width: Vec<f64>,
}

/// Calculate Bollinger Bands using population standard deviation.
pub fn bollinger(
    input: &[f64],
    period: usize,
    deviations: f64,
) -> Result<BollingerOutput, IndicatorError> {
    validation::positive(deviations, "deviations")?;
    let middle = rolling::rolling_mean(input, period)?;
    let deviation = rolling::rolling_std(input, period, false)?;
    let mut upper = vec![f64::NAN; input.len()];
    let mut lower = vec![f64::NAN; input.len()];
    let mut percent_b = vec![f64::NAN; input.len()];
    let mut width = vec![f64::NAN; input.len()];
    for index in 0..input.len() {
        if middle[index].is_finite() && deviation[index].is_finite() {
            upper[index] = middle[index] + deviations * deviation[index];
            lower[index] = middle[index] - deviations * deviation[index];
            let span = upper[index] - lower[index];
            percent_b[index] = if span == 0.0 { 0.5 } else { (input[index] - lower[index]) / span };
            width[index] = if middle[index] == 0.0 { f64::NAN } else { span / middle[index].abs() };
        }
    }
    Ok(BollingerOutput { middle, upper, lower, percent_b, width })
}

/// Keltner Channel output.
#[derive(Clone, Debug, PartialEq)]
pub struct KeltnerOutput {
    /// EMA center line.
    pub middle: Vec<f64>,
    /// Center plus an ATR multiple.
    pub upper: Vec<f64>,
    /// Center minus an ATR multiple.
    pub lower: Vec<f64>,
}

/// Calculate EMA-centered Keltner Channels with Wilder ATR.
pub fn keltner(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    ema_period: usize,
    atr_period: usize,
    multiplier: f64,
) -> Result<KeltnerOutput, IndicatorError> {
    validation::positive(multiplier, "multiplier")?;
    let middle = smoothing::ema(close, ema_period, false)?;
    let range = atr(high, low, close, atr_period)?;
    let mut upper = vec![f64::NAN; close.len()];
    let mut lower = vec![f64::NAN; close.len()];
    for index in 0..close.len() {
        if middle[index].is_finite() && range[index].is_finite() {
            upper[index] = middle[index] + multiplier * range[index];
            lower[index] = middle[index] - multiplier * range[index];
        }
    }
    Ok(KeltnerOutput { middle, upper, lower })
}

/// Donchian Channel output.
#[derive(Clone, Debug, PartialEq)]
pub struct DonchianOutput {
    /// Highest high in the trailing window.
    pub upper: Vec<f64>,
    /// Lowest low in the trailing window.
    pub lower: Vec<f64>,
    /// Midpoint of upper and lower.
    pub middle: Vec<f64>,
}

/// Calculate the trailing Donchian Channel.
pub fn donchian(
    high: &[f64],
    low: &[f64],
    period: usize,
) -> Result<DonchianOutput, IndicatorError> {
    validation::same_length(high.len(), &[("low", low.len())])?;
    let upper = rolling::rolling_max(high, period)?;
    let lower = rolling::rolling_min(low, period)?;
    let middle = upper
        .iter()
        .zip(&lower)
        .map(|(&upper, &lower)| {
            if upper.is_finite() && lower.is_finite() { (upper + lower) / 2.0 } else { f64::NAN }
        })
        .collect();
    Ok(DonchianOutput { upper, lower, middle })
}

/// Percentage Bands relative to the absolute moving average.
#[derive(Clone, Debug, PartialEq)]
pub struct PriceBandsOutput {
    /// Positive band distance as a percentage of the center.
    pub upper: Vec<f64>,
    /// Negative band distance as a percentage of the center.
    pub lower: Vec<f64>,
}

/// Calculate standard-deviation percentage bands.
pub fn pbands(
    input: &[f64],
    period: usize,
    deviations: f64,
) -> Result<PriceBandsOutput, IndicatorError> {
    validation::positive(deviations, "deviations")?;
    let middle = rolling::rolling_mean(input, period)?;
    let deviation = rolling::rolling_std(input, period, false)?;
    let upper: Vec<f64> = middle
        .iter()
        .zip(deviation)
        .map(|(&middle, deviation)| {
            if middle.is_finite() && deviation.is_finite() && middle != 0.0 {
                100.0 * deviations * deviation / middle.abs()
            } else {
                f64::NAN
            }
        })
        .collect();
    let lower = upper.iter().map(|value| -*value).collect();
    Ok(PriceBandsOutput { upper, lower })
}

/// Supported historical volatility estimators.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolatilityEstimator {
    /// Standard deviation of close-to-close log returns.
    CloseToClose,
    /// Parkinson high-low estimator.
    Parkinson,
    /// Garman-Klass OHLC estimator.
    GarmanKlass,
    /// Rogers-Satchell OHLC estimator.
    RogersSatchell,
    /// Yang-Zhang overnight and intraday estimator.
    YangZhang,
}

/// Calculate annualized historical volatility.
pub fn volatility(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
    periods_per_year: f64,
    estimator: VolatilityEstimator,
) -> Result<Vec<f64>, IndicatorError> {
    validation::same_length(open.len(), &[("high", high.len()), ("low", low.len()), ("close", close.len())])?;
    validation::period(period)?;
    validation::positive(periods_per_year, "periods_per_year")?;
    let mut per_bar = vec![f64::NAN; close.len()];
    for index in 1..close.len() {
        let values = [open[index], high[index], low[index], close[index], close[index - 1]];
        if !values.iter().all(|value| value.is_finite() && *value > 0.0) {
            continue;
        }
        let high_low = (high[index] / low[index]).ln();
        let close_open = (close[index] / open[index]).ln();
        per_bar[index] = match estimator {
            VolatilityEstimator::CloseToClose => (close[index] / close[index - 1]).ln(),
            VolatilityEstimator::Parkinson => high_low.powi(2) / (4.0 * 2.0_f64.ln()),
            VolatilityEstimator::GarmanKlass => {
                0.5 * high_low.powi(2) - (2.0 * 2.0_f64.ln() - 1.0) * close_open.powi(2)
            }
            VolatilityEstimator::RogersSatchell => {
                (high[index] / close[index]).ln() * (high[index] / open[index]).ln()
                    + (low[index] / close[index]).ln() * (low[index] / open[index]).ln()
            }
            VolatilityEstimator::YangZhang => {
                let overnight = (open[index] / close[index - 1]).ln();
                let rogers_satchell = (high[index] / close[index]).ln()
                    * (high[index] / open[index]).ln()
                    + (low[index] / close[index]).ln() * (low[index] / open[index]).ln();
                overnight.powi(2) + 0.34 * close_open.powi(2) + 0.66 * rogers_satchell
            }
        };
    }
    let mut output = match estimator {
        VolatilityEstimator::CloseToClose => rolling::rolling_std(&per_bar, period, true)?,
        _ => rolling::rolling_mean(&per_bar, period)?,
    };
    for value in &mut output {
        if value.is_finite() {
            *value = if matches!(estimator, VolatilityEstimator::CloseToClose) {
                *value * periods_per_year.sqrt()
            } else {
                value.max(0.0).sqrt() * periods_per_year.sqrt()
            };
        }
    }
    Ok(output)
}

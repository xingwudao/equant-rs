use crate::internal::{statistics, validation};
use crate::IndicatorError;

/// Fractional growth, `(x[t] - x[t-period]) / abs(x[t-period])`.
pub fn growth(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    let mut output = vec![f64::NAN; input.len()];
    for index in period..input.len() {
        let prior = input[index - period];
        if input[index].is_finite() && prior.is_finite() && prior != 0.0 {
            output[index] = (input[index] - prior) / prior.abs();
        }
    }
    Ok(output)
}

/// Adjustment ratio, `adjusted / close`.
pub fn adj_ratios(close: &[f64], adjusted: &[f64]) -> Result<Vec<f64>, IndicatorError> {
    validation::same_length(close.len(), &[("adjusted", adjusted.len())])?;
    Ok(close
        .iter()
        .zip(adjusted)
        .map(|(&close, &adjusted)| {
            if close.is_finite() && adjusted.is_finite() && close != 0.0 {
                adjusted / close
            } else {
                f64::NAN
            }
        })
        .collect())
}

/// Rolling single-factor linear regression against `1..=period`.
#[derive(Clone, Debug, PartialEq)]
pub struct RegressionOutput {
    /// Fitted intercept.
    pub intercept: Vec<f64>,
    /// Fitted time slope.
    pub slope: Vec<f64>,
    /// Coefficient of determination in the range 0 to 1.
    pub r_squared: Vec<f64>,
}

/// Calculate a rolling single-factor time regression.
pub fn roll_sfm(input: &[f64], period: usize) -> Result<RegressionOutput, IndicatorError> {
    let output = statistics::rolling_regression(input, period)?;
    Ok(RegressionOutput {
        intercept: output.intercept,
        slope: output.slope,
        r_squared: output.r_squared,
    })
}

/// Aroon up, down, and oscillator output.
#[derive(Clone, Debug, PartialEq)]
pub struct AroonOutput {
    /// Recency of the highest high, scaled 0 to 100.
    pub up: Vec<f64>,
    /// Recency of the lowest low, scaled 0 to 100.
    pub down: Vec<f64>,
    /// Up minus down.
    pub oscillator: Vec<f64>,
}

/// Calculate Aroon using the most recent extreme when a window contains ties.
pub fn aroon(
    high: &[f64],
    low: &[f64],
    period: usize,
) -> Result<AroonOutput, IndicatorError> {
    validation::same_length(high.len(), &[("low", low.len())])?;
    if period < 2 {
        return Err(IndicatorError::InvalidParameter("aroon period"));
    }
    let mut up = vec![f64::NAN; high.len()];
    let mut down = vec![f64::NAN; high.len()];
    for index in period - 1..high.len() {
        let start = index + 1 - period;
        let high_window = &high[start..=index];
        let low_window = &low[start..=index];
        if high_window.iter().all(|value| value.is_finite())
            && low_window.iter().all(|value| value.is_finite())
        {
            let high_offset = high_window
                .iter()
                .enumerate()
                .max_by(|left, right| left.1.total_cmp(right.1).then(left.0.cmp(&right.0)))
                .map(|(offset, _)| offset)
                .expect("non-empty validated window");
            let low_offset = low_window
                .iter()
                .enumerate()
                .min_by(|left, right| left.1.total_cmp(right.1).then(right.0.cmp(&left.0)))
                .map(|(offset, _)| offset)
                .expect("non-empty validated window");
            let scale = 100.0 / (period - 1) as f64;
            up[index] = high_offset as f64 * scale;
            down[index] = low_offset as f64 * scale;
        }
    }
    let oscillator = up
        .iter()
        .zip(&down)
        .map(|(&up, &down)| if up.is_finite() && down.is_finite() { up - down } else { f64::NAN })
        .collect();
    Ok(AroonOutput { up, down, oscillator })
}

/// Tom DeMark Setup counts.
///
/// Positive values count buy setup bars where close is below the close four
/// bars earlier. Negative values count the analogous sell setup.
pub fn td_setup(close: &[f64]) -> Result<Vec<i32>, IndicatorError> {
    let mut output = vec![0; close.len()];
    let mut direction = 0i8;
    let mut count = 0i32;
    for index in 4..close.len() {
        if !close[index].is_finite() || !close[index - 4].is_finite() {
            direction = 0;
            count = 0;
            continue;
        }
        let next_direction = if close[index] < close[index - 4] {
            1
        } else if close[index] > close[index - 4] {
            -1
        } else {
            0
        };
        if next_direction == 0 {
            direction = 0;
            count = 0;
        } else if next_direction == direction {
            count += next_direction as i32;
        } else {
            direction = next_direction;
            count = next_direction as i32;
        }
        output[index] = count;
    }
    Ok(output)
}

/// Tom DeMark Countdown counts following a completed nine-bar setup.
///
/// Buy countdown bars require close at or below the low two bars earlier;
/// sell countdown bars require close at or above the high two bars earlier.
/// Counts need not occur on consecutive bars and stop at thirteen.
pub fn td_countdown(
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<i32>, IndicatorError> {
    validation::same_length(high.len(), &[("low", low.len()), ("close", close.len())])?;
    let setup = td_setup(close)?;
    let mut output = vec![0; close.len()];
    let mut direction = 0i8;
    let mut count = 0i32;
    for index in 0..close.len() {
        if setup[index] >= 9 {
            direction = 1;
            count = 0;
        } else if setup[index] <= -9 {
            direction = -1;
            count = 0;
        }
        if direction == 0 || index < 2 || !close[index].is_finite() {
            continue;
        }
        let qualifies = if direction > 0 {
            low[index - 2].is_finite() && close[index] <= low[index - 2]
        } else {
            high[index - 2].is_finite() && close[index] >= high[index - 2]
        };
        if qualifies && count.abs() < 13 {
            count += direction as i32;
            output[index] = count;
            if count.abs() == 13 {
                direction = 0;
            }
        }
    }
    Ok(output)
}

/// Return a mask that is true for NaN and infinite observations.
pub fn na_check(input: &[f64]) -> Result<Vec<bool>, IndicatorError> {
    Ok(input.iter().map(|value| !value.is_finite()).collect())
}

/// Shift values back by `period`, padding the leading positions with NaN.
pub fn lags(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    let mut output = vec![f64::NAN; input.len()];
    if period < input.len() {
        output[period..].copy_from_slice(&input[..input.len() - period]);
    }
    Ok(output)
}

use std::collections::VecDeque;

use crate::internal::validation;
use crate::IndicatorError;

pub fn rolling_sum(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    let mut output = vec![f64::NAN; input.len()];
    let mut sum = 0.0;
    let mut finite = 0usize;

    for (index, &value) in input.iter().enumerate() {
        if value.is_finite() {
            sum += value;
            finite += 1;
        }
        if index >= period {
            let old = input[index - period];
            if old.is_finite() {
                sum -= old;
                finite -= 1;
            }
        }
        if index + 1 >= period && finite == period {
            output[index] = sum;
        }
    }
    Ok(output)
}

/// Trailing arithmetic mean requiring a fully finite window.
pub fn rolling_mean(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    let mut output = rolling_sum(input, period)?;
    for value in &mut output {
        if value.is_finite() {
            *value /= period as f64;
        }
    }
    Ok(output)
}

fn rolling_extreme(
    input: &[f64],
    period: usize,
    keep: impl Fn(f64, f64) -> bool,
) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    let mut output = vec![f64::NAN; input.len()];
    let mut deque: VecDeque<usize> = VecDeque::new();
    let mut finite = 0usize;

    for (index, &value) in input.iter().enumerate() {
        if value.is_finite() {
            finite += 1;
            while let Some(&last) = deque.back() {
                if keep(input[last], value) {
                    break;
                }
                deque.pop_back();
            }
            deque.push_back(index);
        }
        if index >= period {
            let expired = index - period;
            if input[expired].is_finite() {
                finite -= 1;
            }
            if deque.front() == Some(&expired) {
                deque.pop_front();
            }
        }
        if index + 1 >= period && finite == period {
            output[index] = input[*deque.front().expect("finite window has an extreme")];
        }
    }
    Ok(output)
}

/// Trailing minimum requiring a fully finite window.
pub fn rolling_min(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    rolling_extreme(input, period, |old, new| old <= new)
}

/// Trailing maximum requiring a fully finite window.
pub fn rolling_max(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    rolling_extreme(input, period, |old, new| old >= new)
}

/// Numerically stable trailing standard deviation over fully finite windows.
pub fn rolling_std(input: &[f64], period: usize, sample: bool) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    if sample && period < 2 {
        return Err(IndicatorError::InvalidParameter("sample period"));
    }
    let mut output = vec![f64::NAN; input.len()];
    let mut mean = 0.0;
    let mut m2 = 0.0;
    let mut finite = 0usize;
    let denominator = if sample { period - 1 } else { period } as f64;

    for (index, &value) in input.iter().enumerate() {
        if index >= period {
            let old = input[index - period];
            if old.is_finite() {
                if finite == 1 {
                    mean = 0.0;
                    m2 = 0.0;
                    finite = 0;
                } else {
                    let new_count = finite - 1;
                    let new_mean = mean - (old - mean) / new_count as f64;
                    m2 -= (old - mean) * (old - new_mean);
                    mean = new_mean;
                    finite = new_count;
                }
            }
        }
        if value.is_finite() {
            finite += 1;
            let delta = value - mean;
            mean += delta / finite as f64;
            m2 += delta * (value - mean);
        }
        if index + 1 >= period && finite == period {
            let variance = (m2 / denominator).max(0.0);
            output[index] = variance.sqrt();
        }
    }
    Ok(output)
}

pub(crate) fn rolling_wma(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    let mut output = vec![f64::NAN; input.len()];
    if period > input.len() {
        return Ok(output);
    }
    let period_f64 = period as f64;
    let weight_sum = period_f64 * (period_f64 + 1.0) / 2.0;
    let mut sum = 0.0;
    let mut weighted = 0.0;
    let mut consecutive = 0usize;
    for (index, &value) in input.iter().enumerate() {
        if !value.is_finite() {
            sum = 0.0;
            weighted = 0.0;
            consecutive = 0;
            continue;
        }
        if consecutive < period {
            consecutive += 1;
            sum += value;
            weighted += consecutive as f64 * value;
            if consecutive == period {
                output[index] = weighted / weight_sum;
            }
        } else {
            let old = input[index - period];
            weighted = weighted - sum + period_f64 * value;
            sum += value - old;
            output[index] = weighted / weight_sum;
        }
    }
    Ok(output)
}

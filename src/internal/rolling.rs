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

pub fn rolling_min(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    rolling_extreme(input, period, |old, new| old <= new)
}

pub fn rolling_max(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    rolling_extreme(input, period, |old, new| old >= new)
}

pub fn rolling_std(
    input: &[f64],
    period: usize,
    sample: bool,
) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    if sample && period < 2 {
        return Err(IndicatorError::InvalidParameter("sample period"));
    }
    let mut output = vec![f64::NAN; input.len()];
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let mut finite = 0usize;
    let denominator = if sample { period - 1 } else { period } as f64;

    for (index, &value) in input.iter().enumerate() {
        if value.is_finite() {
            sum += value;
            sum_sq += value * value;
            finite += 1;
        }
        if index >= period {
            let old = input[index - period];
            if old.is_finite() {
                sum -= old;
                sum_sq -= old * old;
                finite -= 1;
            }
        }
        if index + 1 >= period && finite == period {
            let variance = ((sum_sq - sum * sum / period as f64) / denominator).max(0.0);
            output[index] = variance.sqrt();
        }
    }
    Ok(output)
}

pub(crate) fn rolling_wma(input: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    let mut output = vec![f64::NAN; input.len()];
    let weight_sum = (period * (period + 1) / 2) as f64;
    for index in period.saturating_sub(1)..input.len() {
        let window = &input[index + 1 - period..=index];
        if window.iter().all(|value| value.is_finite()) {
            output[index] = window
                .iter()
                .enumerate()
                .map(|(offset, value)| (offset + 1) as f64 * value)
                .sum::<f64>()
                / weight_sum;
        }
    }
    Ok(output)
}


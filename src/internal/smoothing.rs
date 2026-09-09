use crate::internal::validation;
use crate::IndicatorError;

pub fn ema(input: &[f64], period: usize, wilder: bool) -> Result<Vec<f64>, IndicatorError> {
    validation::period(period)?;
    let mut output = vec![f64::NAN; input.len()];
    let alpha = if wilder {
        1.0 / period as f64
    } else {
        2.0 / (period as f64 + 1.0)
    };
    let mut count = 0usize;
    let mut seed = 0.0;
    let mut previous = f64::NAN;

    for (index, &value) in input.iter().enumerate() {
        if !value.is_finite() {
            count = 0;
            seed = 0.0;
            previous = f64::NAN;
            continue;
        }
        if count < period {
            seed += value;
            count += 1;
            if count == period {
                previous = seed / period as f64;
                output[index] = previous;
            }
        } else {
            previous += alpha * (value - previous);
            output[index] = previous;
        }
    }
    Ok(output)
}

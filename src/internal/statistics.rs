use crate::internal::validation;
use crate::IndicatorError;

pub(crate) struct Regression {
    pub(crate) intercept: Vec<f64>,
    pub(crate) slope: Vec<f64>,
    pub(crate) r_squared: Vec<f64>,
}

pub(crate) fn rolling_regression(
    input: &[f64],
    period: usize,
) -> Result<Regression, IndicatorError> {
    validation::period(period)?;
    let mut output = Regression {
        intercept: vec![f64::NAN; input.len()],
        slope: vec![f64::NAN; input.len()],
        r_squared: vec![f64::NAN; input.len()],
    };
    if period < 2 {
        return Err(IndicatorError::InvalidParameter("regression period"));
    }
    if period > input.len() {
        return Ok(output);
    }
    let period_f64 = period as f64;
    let x_mean = (period_f64 + 1.0) / 2.0;
    let sxx = period_f64 * (period_f64 * period_f64 - 1.0) / 12.0;
    let mut origin = 0.0;
    let mut sum = 0.0;
    let mut sum_squared = 0.0;
    let mut weighted = 0.0;
    let mut consecutive = 0usize;

    for (index, &value) in input.iter().enumerate() {
        if !value.is_finite() {
            consecutive = 0;
            sum = 0.0;
            sum_squared = 0.0;
            weighted = 0.0;
            continue;
        }
        if consecutive == 0 {
            origin = value;
        }
        let shifted = value - origin;
        if consecutive < period {
            consecutive += 1;
            sum += shifted;
            sum_squared += shifted * shifted;
            weighted += consecutive as f64 * shifted;
            if consecutive < period {
                continue;
            }
        } else {
            let old = input[index - period] - origin;
            weighted = weighted - sum + period_f64 * shifted;
            sum += shifted - old;
            sum_squared += shifted * shifted - old * old;
        }
        let y_mean = origin + sum / period_f64;
        let sxy = weighted - x_mean * sum;
        let slope = sxy / sxx;
        let intercept = y_mean - slope * x_mean;
        let ss_total = (sum_squared - sum * sum / period_f64).max(0.0);
        output.intercept[index] = intercept;
        output.slope[index] = slope;
        output.r_squared[index] = if ss_total == 0.0 {
            1.0
        } else {
            (sxy * sxy / (sxx * ss_total)).clamp(0.0, 1.0)
        };
    }
    Ok(output)
}

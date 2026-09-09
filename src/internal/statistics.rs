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
    let x_mean = (period as f64 + 1.0) / 2.0;
    let sxx: f64 = (1..=period).map(|x| (x as f64 - x_mean).powi(2)).sum();

    for index in period - 1..input.len() {
        let window = &input[index + 1 - period..=index];
        if !window.iter().all(|value| value.is_finite()) {
            continue;
        }
        let y_mean = window.iter().sum::<f64>() / period as f64;
        let sxy: f64 = window
            .iter()
            .enumerate()
            .map(|(x, y)| (x as f64 + 1.0 - x_mean) * (y - y_mean))
            .sum();
        let slope = sxy / sxx;
        let intercept = y_mean - slope * x_mean;
        let ss_total: f64 = window.iter().map(|y| (y - y_mean).powi(2)).sum();
        let ss_residual: f64 = window
            .iter()
            .enumerate()
            .map(|(x, y)| y - (intercept + slope * (x as f64 + 1.0)))
            .map(|error| error * error)
            .sum();
        output.intercept[index] = intercept;
        output.slope[index] = slope;
        output.r_squared[index] = if ss_total == 0.0 {
            1.0
        } else {
            (1.0 - ss_residual / ss_total).clamp(0.0, 1.0)
        };
    }
    Ok(output)
}

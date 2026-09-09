use crate::IndicatorError;

pub(crate) fn period(value: usize) -> Result<(), IndicatorError> {
    if value == 0 {
        Err(IndicatorError::InvalidPeriod)
    } else {
        Ok(())
    }
}

pub(crate) fn same_length(
    expected: usize,
    values: &[(&'static str, usize)],
) -> Result<(), IndicatorError> {
    for &(name, actual) in values {
        if actual != expected {
            return Err(IndicatorError::LengthMismatch {
                input: name,
                expected,
                actual,
            });
        }
    }
    Ok(())
}

pub(crate) fn positive(value: f64, name: &'static str) -> Result<(), IndicatorError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(IndicatorError::InvalidParameter(name))
    }
}

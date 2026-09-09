use core::fmt;

/// Errors returned when an operator configuration or its inputs are invalid.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum IndicatorError {
    /// A period was zero.
    InvalidPeriod,
    /// A named parameter fell outside its documented domain.
    InvalidParameter(&'static str),
    /// Related input slices have different lengths.
    LengthMismatch {
        /// Name of the slice whose length differs from the primary input.
        input: &'static str,
        /// Required length.
        expected: usize,
        /// Supplied length.
        actual: usize,
    },
}

impl fmt::Display for IndicatorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPeriod => formatter.write_str("period must be greater than zero"),
            Self::InvalidParameter(name) => write!(formatter, "invalid parameter: {name}"),
            Self::LengthMismatch {
                input,
                expected,
                actual,
            } => write!(
                formatter,
                "input '{input}' has length {actual}, expected {expected}"
            ),
        }
    }
}

impl std::error::Error for IndicatorError {}


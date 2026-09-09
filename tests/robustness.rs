use equant::{momentum, structure, transform, trend, volatility, volume, IndicatorError};
use proptest::prelude::*;

#[test]
fn empty_inputs_are_valid_and_empty() {
    assert!(trend::sma(&[], 5).unwrap().is_empty());
    assert!(momentum::rsi(&[], 14).unwrap().is_empty());
    assert!(volatility::tr(&[], &[], &[]).unwrap().is_empty());
    assert!(volume::obv(&[], &[]).unwrap().is_empty());
    assert!(structure::zigzag(&[], &[], 5.0, true).unwrap().is_empty());
}

#[test]
fn length_mismatch_reports_the_named_input() {
    assert_eq!(
        volatility::tr(&[1.0, 2.0], &[1.0], &[1.0, 2.0]),
        Err(IndicatorError::LengthMismatch {
            input: "low",
            expected: 2,
            actual: 1,
        })
    );
}

#[test]
fn non_finite_input_never_panics() {
    let values = [1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 5.0];
    assert!(trend::sma(&values, 2).is_ok());
    assert!(trend::ema(&values, 2).is_ok());
    assert!(momentum::rsi(&values, 2).is_ok());
}

#[test]
fn oversized_periods_return_aligned_missing_outputs_without_large_allocations() {
    let input = [1.0, 2.0, 3.0];
    assert!(trend::wma(&input, usize::MAX)
        .unwrap()
        .iter()
        .all(|value| value.is_nan()));
    assert!(trend::alma(&input, usize::MAX, 0.85, 6.0)
        .unwrap()
        .iter()
        .all(|value| value.is_nan()));
    assert!(transform::roll_sfm(&input, usize::MAX)
        .unwrap()
        .slope
        .iter()
        .all(|value| value.is_nan()));
    assert!(momentum::cti(&input, usize::MAX)
        .unwrap()
        .iter()
        .all(|value| value.is_nan()));
}

#[test]
fn zero_period_errors_are_consistent_across_multi_period_operators() {
    let values = [1.0, 2.0];
    assert_eq!(
        trend::macd(&values, 0, 0, 0),
        Err(IndicatorError::InvalidPeriod)
    );
    assert_eq!(trend::po(&values, 0, 0), Err(IndicatorError::InvalidPeriod));
    assert_eq!(
        momentum::ultimate_oscillator(&values, &values, &values, 0, 0, 0),
        Err(IndicatorError::InvalidPeriod)
    );
    assert_eq!(
        momentum::dvi(&values, 0, 0),
        Err(IndicatorError::InvalidPeriod)
    );
}

proptest! {
    #[test]
    fn arbitrary_finite_series_do_not_panic(
        input in prop::collection::vec(-1.0e12f64..1.0e12, 0..250),
        period in 1usize..64,
    ) {
        let _ = trend::sma(&input, period);
        let _ = trend::ema(&input, period);
        let _ = trend::wma(&input, period);
        let _ = momentum::rsi(&input, period);
        let _ = momentum::cmo(&input, period);
        let _ = momentum::cti(&input, period);
    }
}

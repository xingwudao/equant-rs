use equant::{momentum, structure, trend, volatility, volume, IndicatorError};
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

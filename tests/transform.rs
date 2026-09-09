use equant::transform::{
    adj_ratios, aroon, growth, lags, na_check, roll_sfm, td_countdown, td_setup,
};
use equant::IndicatorError;

#[test]
fn missing_mask_and_lag_have_domain_types() {
    let values = [1.0, f64::NAN, 3.0];
    assert_eq!(na_check(&values).unwrap(), vec![false, true, false]);
    let lag = lags(&values, 1).unwrap();
    assert!(lag[0].is_nan());
    assert_eq!(lag[1], 1.0);
}

#[test]
fn all_transform_operators_return_equal_length_outputs() {
    let close: Vec<f64> = (0..100)
        .map(|index| 100.0 + index as f64 * 0.2 + (index as f64 / 4.0).sin())
        .collect();
    let adjusted: Vec<f64> = close.iter().map(|value| value * 0.5).collect();
    let high: Vec<f64> = close.iter().map(|value| value + 1.0).collect();
    let low: Vec<f64> = close.iter().map(|value| value - 1.0).collect();
    let n = close.len();

    assert_eq!(growth(&close, 5).unwrap().len(), n);
    assert_eq!(adj_ratios(&close, &adjusted).unwrap().len(), n);
    assert_eq!(lags(&close, 3).unwrap().len(), n);
    assert_eq!(na_check(&close).unwrap().len(), n);
    assert_eq!(td_setup(&close).unwrap().len(), n);
    assert_eq!(td_countdown(&high, &low, &close).unwrap().len(), n);

    let regression = roll_sfm(&close, 20).unwrap();
    assert_eq!(regression.slope.len(), n);
    assert_eq!(regression.r_squared.len(), n);

    let aroon = aroon(&high, &low, 20).unwrap();
    assert_eq!(aroon.up.len(), n);
    assert_eq!(aroon.oscillator.len(), n);
}

#[test]
fn linear_series_has_unit_regression_slope_and_fit() {
    let values: Vec<f64> = (1..=20).map(|value| value as f64).collect();
    let output = roll_sfm(&values, 10).unwrap();
    assert!((output.slope[19] - 1.0).abs() < 1e-12);
    assert!((output.r_squared[19] - 1.0).abs() < 1e-12);
}

#[test]
fn td_countdown_reaches_thirteen_in_a_qualifying_decline() {
    let close: Vec<f64> = (0..30).map(|index| 100.0 - index as f64).collect();
    let high: Vec<f64> = close.iter().map(|value| value + 0.25).collect();
    let low: Vec<f64> = close.iter().map(|value| value - 0.25).collect();
    let out = td_countdown(&high, &low, &close).unwrap();
    assert!(out.contains(&13));
}

#[test]
fn aroon_zero_period_uses_the_common_period_error() {
    assert_eq!(aroon(&[1.0], &[1.0], 0), Err(IndicatorError::InvalidPeriod));
}

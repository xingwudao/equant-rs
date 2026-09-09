use equant::testing::{ema, rolling_max, rolling_mean, rolling_min, rolling_std};
use equant::IndicatorError;

#[test]
fn rolling_mean_rewarms_after_nan() {
    let input = [1.0, 2.0, 3.0, f64::NAN, 5.0, 6.0, 7.0];
    let out = rolling_mean(&input, 3).unwrap();

    assert_eq!(out[2], 2.0);
    assert!(out[3..6].iter().all(|value| value.is_nan()));
    assert_eq!(out[6], 6.0);
}

#[test]
fn rolling_extrema_track_the_current_window() {
    let input = [3.0, 1.0, 4.0, 2.0, 5.0];
    let min = rolling_min(&input, 3).unwrap();
    let max = rolling_max(&input, 3).unwrap();

    assert_eq!(&min[2..], &[1.0, 1.0, 2.0]);
    assert_eq!(&max[2..], &[4.0, 4.0, 5.0]);
}

#[test]
fn rolling_population_std_is_zero_for_constant_data() {
    let out = rolling_std(&[2.0; 6], 3, false).unwrap();
    assert!(out[2..].iter().all(|value| *value == 0.0));
}

#[test]
fn ema_uses_sma_seed_and_resets_after_missing_data() {
    let out = ema(&[1.0, 2.0, 3.0, 4.0, f64::NAN, 6.0, 7.0, 8.0], 3, false).unwrap();
    assert_eq!(out[2], 2.0);
    assert_eq!(out[3], 3.0);
    assert!(out[4..7].iter().all(|value| value.is_nan()));
    assert_eq!(out[7], 7.0);
}

#[test]
fn zero_period_is_an_actionable_error() {
    assert_eq!(rolling_mean(&[1.0], 0), Err(IndicatorError::InvalidPeriod));
}

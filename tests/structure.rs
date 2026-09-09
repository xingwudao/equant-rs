use equant::structure::{pivots, sar, snr, zigzag};

#[test]
fn pivots_use_only_the_previous_bar() {
    let out = pivots(&[12.0, 20.0], &[8.0, 10.0], &[10.0, 15.0]).unwrap();
    assert!(out.pivot[0].is_nan());
    assert_eq!(out.pivot[1], 10.0);
    assert_eq!(out.resistance_1[1], 12.0);
    assert_eq!(out.support_1[1], 8.0);
}

#[test]
fn all_structure_operators_return_equal_length_outputs() {
    let close: Vec<f64> = (0..80)
        .map(|index| 100.0 + (index as f64 / 4.0).sin() * 10.0)
        .collect();
    let high: Vec<f64> = close.iter().map(|value| value + 1.0).collect();
    let low: Vec<f64> = close.iter().map(|value| value - 1.0).collect();
    let n = close.len();

    assert_eq!(zigzag(&high, &low, 5.0, true).unwrap().len(), n);
    assert_eq!(sar(&high, &low, 0.02, 0.2).unwrap().len(), n);
    let pivots = pivots(&high, &low, &close).unwrap();
    assert_eq!(pivots.resistance_2.len(), n);
    let levels = snr(&high, &low, 20).unwrap();
    assert_eq!(levels.resistance.len(), n);
    assert_eq!(levels.support.len(), n);
    assert_eq!(levels.middle.len(), n);
}

#[test]
fn rolling_support_never_exceeds_resistance() {
    let high = [5.0, 6.0, 4.0, 8.0];
    let low = [3.0, 2.0, 1.0, 6.0];
    let out = snr(&high, &low, 3).unwrap();
    for index in 0..high.len() {
        if out.resistance[index].is_finite() {
            assert!(out.support[index] <= out.middle[index]);
            assert!(out.middle[index] <= out.resistance[index]);
        }
    }
}

#[test]
fn sar_reinitializes_after_missing_rows() {
    let high = [2.0, 3.0, 4.0, f64::NAN, 6.0, 7.0];
    let low = [1.0, 2.0, 3.0, f64::NAN, 5.0, 6.0];
    let out = sar(&high, &low, 0.02, 0.2).unwrap();
    assert!(out[3].is_nan());
    assert!(out[4].is_nan());
    assert!(out[5].is_finite());
}

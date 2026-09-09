use equant::volume::{
    chaikin_ad, chaikin_volatility, clv, cmf, emv, mfi, obv, vwap, williams_ad,
};

fn ohlcv(length: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let close: Vec<f64> = (0..length)
        .map(|index| 100.0 + index as f64 * 0.2 + (index as f64 / 3.0).sin())
        .collect();
    let high: Vec<f64> = close.iter().map(|value| value + 1.0).collect();
    let low: Vec<f64> = close.iter().map(|value| value - 1.0).collect();
    let volume: Vec<f64> = (0..length).map(|index| 1_000.0 + 10.0 * index as f64).collect();
    (high, low, close, volume)
}

#[test]
fn flat_close_keeps_obv_at_zero() {
    let out = obv(&[10.0; 8], &[100.0; 8]).unwrap();
    assert_eq!(out, vec![0.0; 8]);
}

#[test]
fn all_volume_operators_return_equal_length_outputs() {
    let (high, low, close, volume) = ohlcv(80);
    let n = close.len();
    for output in [
        obv(&close, &volume).unwrap(),
        cmf(&high, &low, &close, &volume, 20).unwrap(),
        vwap(&high, &low, &close, &volume, 20).unwrap(),
        mfi(&high, &low, &close, &volume, 14).unwrap(),
        emv(&high, &low, &volume, 14).unwrap(),
        clv(&high, &low, &close).unwrap(),
        chaikin_ad(&high, &low, &close, &volume).unwrap(),
        chaikin_volatility(&high, &low, 10, 10).unwrap(),
        williams_ad(&high, &low, &close).unwrap(),
    ] {
        assert_eq!(output.len(), n);
    }
}

#[test]
fn clv_is_bounded() {
    let (high, low, close, _) = ohlcv(30);
    let out = clv(&high, &low, &close).unwrap();
    assert!(out
        .iter()
        .filter(|value| value.is_finite())
        .all(|value| (-1.0..=1.0).contains(value)));
}

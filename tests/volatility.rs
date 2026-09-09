use equant::volatility::{
    atr, bollinger, donchian, keltner, pbands, tr, volatility, VolatilityEstimator,
};

fn ohlc(length: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let close: Vec<f64> = (0..length)
        .map(|index| 100.0 + index as f64 * 0.2 + (index as f64 / 3.0).sin())
        .collect();
    let open: Vec<f64> = close.iter().map(|value| value - 0.1).collect();
    let high: Vec<f64> = close.iter().map(|value| value + 1.0).collect();
    let low: Vec<f64> = close.iter().map(|value| value - 1.0).collect();
    (open, high, low, close)
}

#[test]
fn true_range_accounts_for_the_previous_close() {
    let out = tr(&[10.0, 15.0], &[8.0, 13.0], &[9.0, 14.0]).unwrap();
    assert!(out[0].is_nan());
    assert_eq!(out[1], 6.0);
}

#[test]
fn all_volatility_operators_return_equal_length_outputs() {
    let (open, high, low, close) = ohlc(100);
    let n = close.len();

    assert_eq!(tr(&high, &low, &close).unwrap().len(), n);
    assert_eq!(atr(&high, &low, &close, 14).unwrap().len(), n);

    let bollinger = bollinger(&close, 20, 2.0).unwrap();
    assert_eq!(bollinger.percent_b.len(), n);
    let keltner = keltner(&high, &low, &close, 20, 14, 2.0).unwrap();
    assert_eq!(keltner.upper.len(), n);
    let donchian = donchian(&high, &low, 20).unwrap();
    assert_eq!(donchian.middle.len(), n);
    let percentage = pbands(&close, 20, 2.0).unwrap();
    assert_eq!(percentage.upper.len(), n);
    let historical = volatility(
        &open,
        &high,
        &low,
        &close,
        20,
        252.0,
        VolatilityEstimator::CloseToClose,
    )
    .unwrap();
    assert_eq!(historical.len(), n);
}

#[test]
fn channel_outputs_are_ordered() {
    let (_, high, low, close) = ohlc(80);
    let bands = bollinger(&close, 20, 2.0).unwrap();
    for index in 0..close.len() {
        if bands.upper[index].is_finite() {
            assert!(bands.upper[index] >= bands.middle[index]);
            assert!(bands.middle[index] >= bands.lower[index]);
        }
    }
    let channel = donchian(&high, &low, 20).unwrap();
    for index in 0..close.len() {
        if channel.upper[index].is_finite() {
            assert!(channel.upper[index] >= channel.middle[index]);
            assert!(channel.middle[index] >= channel.lower[index]);
        }
    }
}

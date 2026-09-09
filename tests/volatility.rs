use equant::volatility::{
    atr, bollinger, donchian, keltner, pbands, tr, volatility, VolatilityEstimator,
};
use equant::IndicatorError;

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

#[test]
fn yang_zhang_removes_constant_overnight_drift() {
    let close: Vec<f64> = (0..12).map(|index| 100.0 * 1.01_f64.powi(index)).collect();
    let out = volatility(
        &close,
        &close,
        &close,
        &close,
        5,
        252.0,
        VolatilityEstimator::YangZhang,
    )
    .unwrap();
    assert!(out.last().unwrap().abs() < 1e-12);
}

#[test]
fn estimators_only_require_their_formula_inputs() {
    let close = [100.0, 101.0, 102.0, 103.0];
    let missing = [f64::NAN; 4];
    let out = volatility(
        &missing,
        &missing,
        &missing,
        &close,
        2,
        252.0,
        VolatilityEstimator::CloseToClose,
    )
    .unwrap();
    assert!(out[2].is_finite());

    assert_eq!(
        volatility(
            &close,
            &close,
            &close,
            &close,
            1,
            252.0,
            VolatilityEstimator::YangZhang,
        ),
        Err(IndicatorError::InvalidParameter("yang_zhang period"))
    );
}

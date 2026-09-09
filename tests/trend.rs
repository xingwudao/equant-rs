use approx::assert_relative_eq;
use equant::trend::{
    adx, alma, dema, dpo, ema, evwma, gmma, hma, kst, macd, po, sma, tdi, trix, vhf, vwma, wma,
    zlema, KstConfig,
};

fn prices(length: usize) -> Vec<f64> {
    (0..length)
        .map(|index| 100.0 + index as f64 * 0.4 + (index as f64 / 3.0).sin())
        .collect()
}

#[test]
fn sma_and_ema_use_documented_seeds() {
    let input = [1.0, 2.0, 3.0, 4.0];
    let simple = sma(&input, 3).unwrap();
    let exponential = ema(&input, 3).unwrap();

    assert!(simple[0].is_nan() && simple[1].is_nan());
    assert_eq!(simple[2], 2.0);
    assert_eq!(simple[3], 3.0);
    assert_eq!(exponential[2], 2.0);
    assert_eq!(exponential[3], 3.0);
}

#[test]
fn all_trend_operators_return_equal_length_outputs() {
    let close = prices(160);
    let high: Vec<f64> = close.iter().map(|value| value + 1.0).collect();
    let low: Vec<f64> = close.iter().map(|value| value - 1.0).collect();
    let volume: Vec<f64> = (0..close.len())
        .map(|index| 1_000.0 + index as f64)
        .collect();
    let n = close.len();

    for output in [
        sma(&close, 10).unwrap(),
        ema(&close, 10).unwrap(),
        dema(&close, 10).unwrap(),
        wma(&close, 10).unwrap(),
        hma(&close, 10).unwrap(),
        zlema(&close, 10).unwrap(),
        alma(&close, 10, 0.85, 6.0).unwrap(),
        evwma(&close, &volume, 10).unwrap(),
        vwma(&close, &volume, 10).unwrap(),
        dpo(&close, 10).unwrap(),
        vhf(&close, 10).unwrap(),
        po(&close, 12, 26).unwrap(),
    ] {
        assert_eq!(output.len(), n);
    }

    let macd = macd(&close, 12, 26, 9).unwrap();
    assert_eq!(macd.line.len(), n);
    assert_eq!(macd.signal.len(), n);
    assert_eq!(macd.histogram.len(), n);

    let adx = adx(&high, &low, &close, 14).unwrap();
    assert_eq!(adx.adx.len(), n);
    assert_eq!(adx.plus_di.len(), n);
    assert_eq!(adx.minus_di.len(), n);

    let gmma = gmma(&close).unwrap();
    assert!(gmma.short.iter().all(|series| series.len() == n));
    assert!(gmma.long.iter().all(|series| series.len() == n));

    let tdi = tdi(&close, 13, 34).unwrap();
    assert_eq!(tdi.trend.len(), n);
    assert_eq!(tdi.direction.len(), n);

    let trix = trix(&close, 15, 9).unwrap();
    assert_eq!(trix.line.len(), n);
    assert_eq!(trix.signal.len(), n);

    let kst = kst(&close, KstConfig::default()).unwrap();
    assert_eq!(kst.line.len(), n);
    assert_eq!(kst.signal.len(), n);
}

#[test]
fn macd_histogram_is_line_minus_signal() {
    let close = prices(100);
    let out = macd(&close, 12, 26, 9).unwrap();

    for ((line, signal), histogram) in out.line.iter().zip(&out.signal).zip(&out.histogram) {
        if line.is_finite() && signal.is_finite() {
            assert_relative_eq!(*histogram, line - signal, epsilon = 1e-12);
        }
    }
}

#[test]
fn mismatched_volume_is_rejected() {
    let error = vwma(&[1.0, 2.0], &[10.0], 2).unwrap_err();
    assert!(error.to_string().contains("volume"));
}

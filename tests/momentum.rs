use equant::momentum::{
    cci, cmo, cti, dvi, kdj, momentum, roc, rsi, rvi, smi, stoch, tsi,
    ultimate_oscillator, wpr,
};

fn ohlc(length: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let close: Vec<f64> = (0..length)
        .map(|index| 100.0 + index as f64 * 0.1 + (index as f64 / 4.0).sin())
        .collect();
    let open: Vec<f64> = close.iter().map(|value| value - 0.2).collect();
    let high: Vec<f64> = close.iter().map(|value| value + 1.0).collect();
    let low: Vec<f64> = close.iter().map(|value| value - 1.0).collect();
    (open, high, low, close)
}

#[test]
fn rising_prices_reach_an_rsi_of_one_hundred() {
    let close: Vec<f64> = (1..=30).map(|value| value as f64).collect();
    let out = rsi(&close, 14).unwrap();
    assert_eq!(*out.last().unwrap(), 100.0);
}

#[test]
fn all_momentum_operators_return_equal_length_outputs() {
    let (open, high, low, close) = ohlc(320);
    let n = close.len();

    for output in [
        rsi(&close, 14).unwrap(),
        cci(&high, &low, &close, 20).unwrap(),
        cmo(&close, 14).unwrap(),
        tsi(&close, 25, 13).unwrap(),
        wpr(&high, &low, &close, 14).unwrap(),
        ultimate_oscillator(&high, &low, &close, 7, 14, 28).unwrap(),
        roc(&close, 10).unwrap(),
        momentum(&close, 10).unwrap(),
        cti(&close, 10).unwrap(),
        dvi(&close, 100, 5).unwrap(),
    ] {
        assert_eq!(output.len(), n);
    }

    let smi = smi(&high, &low, &close, 13, 5, 8).unwrap();
    assert_eq!(smi.line.len(), n);
    assert_eq!(smi.signal.len(), n);

    let rvi = rvi(&open, &high, &low, &close, 10).unwrap();
    assert_eq!(rvi.line.len(), n);
    assert_eq!(rvi.signal.len(), n);

    let stoch = stoch(&high, &low, &close, 14, 3, 3).unwrap();
    assert_eq!(stoch.fast_k.len(), n);
    assert_eq!(stoch.slow_d.len(), n);

    let kdj = kdj(&high, &low, &close, 9, 3, 3).unwrap();
    assert_eq!(kdj.k.len(), n);
    assert_eq!(kdj.j.len(), n);
}

#[test]
fn bounded_oscillators_stay_in_their_ranges() {
    let (_, high, low, close) = ohlc(100);
    let rsi = rsi(&close, 14).unwrap();
    let wpr = wpr(&high, &low, &close, 14).unwrap();

    assert!(rsi
        .iter()
        .filter(|value| value.is_finite())
        .all(|value| (0.0..=100.0).contains(value)));
    assert!(wpr
        .iter()
        .filter(|value| value.is_finite())
        .all(|value| (-100.0..=0.0).contains(value)));
}

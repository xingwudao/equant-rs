use equant::{momentum, trend, volatility, volume};
use proptest::prelude::*;

fn equal_or_nan(left: f64, right: f64) -> bool {
    (left.is_nan() && right.is_nan()) || (left - right).abs() <= 1e-10
}

fn prefix_matches(left: &[f64], right: &[f64]) -> bool {
    left.iter().zip(right).all(|(&left, &right)| equal_or_nan(left, right))
}

proptest! {
    #[test]
    fn representative_outputs_always_match_input_length(
        input in prop::collection::vec(-10_000.0f64..10_000.0, 0..300),
        period in 1usize..40,
    ) {
        prop_assert_eq!(trend::sma(&input, period).unwrap().len(), input.len());
        prop_assert_eq!(trend::ema(&input, period).unwrap().len(), input.len());
        prop_assert_eq!(momentum::rsi(&input, period).unwrap().len(), input.len());
        prop_assert_eq!(momentum::roc(&input, period).unwrap().len(), input.len());
    }

    #[test]
    fn causal_indicators_are_prefix_invariant(
        prefix in prop::collection::vec(1.0f64..1_000.0, 40..120),
        suffix in prop::collection::vec(1.0f64..1_000.0, 1..40),
        period in 2usize..20,
    ) {
        let mut extended = prefix.clone();
        extended.extend_from_slice(&suffix);
        prop_assert!(prefix_matches(
            &trend::sma(&prefix, period).unwrap(),
            &trend::sma(&extended, period).unwrap()[..prefix.len()],
        ));
        prop_assert!(prefix_matches(
            &trend::ema(&prefix, period).unwrap(),
            &trend::ema(&extended, period).unwrap()[..prefix.len()],
        ));
        prop_assert!(prefix_matches(
            &momentum::rsi(&prefix, period).unwrap(),
            &momentum::rsi(&extended, period).unwrap()[..prefix.len()],
        ));
    }
}

#[test]
fn scale_free_indicators_are_scale_invariant() {
    let close: Vec<f64> = (0..100).map(|i| 50.0 + i as f64 + (i as f64).sin()).collect();
    let high: Vec<f64> = close.iter().map(|value| value + 2.0).collect();
    let low: Vec<f64> = close.iter().map(|value| value - 2.0).collect();
    let volume: Vec<f64> = (0..100).map(|i| 1_000.0 + i as f64).collect();
    let scaled = |values: &[f64]| values.iter().map(|value| value * 7.0).collect::<Vec<_>>();

    let rsi = momentum::rsi(&close, 14).unwrap();
    let scaled_rsi = momentum::rsi(&scaled(&close), 14).unwrap();
    assert!(prefix_matches(&rsi, &scaled_rsi));

    let cmf = volume::cmf(&high, &low, &close, &volume, 20).unwrap();
    let scaled_cmf = volume::cmf(&scaled(&high), &scaled(&low), &scaled(&close), &volume, 20).unwrap();
    assert!(prefix_matches(&cmf, &scaled_cmf));
}

#[test]
fn volatility_and_oscillator_ranges_hold() {
    let close: Vec<f64> = (0..100).map(|i| 100.0 + (i as f64 / 5.0).sin()).collect();
    let high: Vec<f64> = close.iter().map(|value| value + 1.0).collect();
    let low: Vec<f64> = close.iter().map(|value| value - 1.0).collect();
    let atr = volatility::atr(&high, &low, &close, 14).unwrap();
    assert!(atr.iter().filter(|v| v.is_finite()).all(|v| *v >= 0.0));
    let rsi = momentum::rsi(&close, 14).unwrap();
    assert!(rsi.iter().filter(|v| v.is_finite()).all(|v| (0.0..=100.0).contains(v)));
}


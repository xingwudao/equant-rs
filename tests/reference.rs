use approx::assert_relative_eq;
use equant::trend;
use serde_json::Value;

fn assert_reference(actual: &[f64], expected: &[Value]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        match expected.as_f64() {
            Some(expected) => assert_relative_eq!(*actual, expected, epsilon = 1e-12),
            None => assert!(actual.is_nan()),
        }
    }
}

#[test]
fn moving_averages_match_hand_calculated_reference_fixture() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/reference.json")).unwrap();
    let input: Vec<f64> = fixture["input"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_f64().unwrap())
        .collect();
    let period = fixture["period"].as_u64().unwrap() as usize;

    assert_reference(
        &trend::sma(&input, period).unwrap(),
        fixture["sma"].as_array().unwrap(),
    );
    assert_reference(
        &trend::wma(&input, period).unwrap(),
        fixture["wma"].as_array().unwrap(),
    );
}

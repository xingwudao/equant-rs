use std::collections::HashSet;

use equant::OPERATOR_NAMES;

#[test]
fn catalog_contains_exactly_sixty_unique_operator_names() {
    assert_eq!(OPERATOR_NAMES.len(), 60);
    assert_eq!(OPERATOR_NAMES.iter().copied().collect::<HashSet<_>>().len(), 60);
    assert!(OPERATOR_NAMES.contains(&"sma"));
    assert!(OPERATOR_NAMES.contains(&"zigzag"));
    assert!(OPERATOR_NAMES.contains(&"lags"));
}

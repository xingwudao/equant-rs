use std::fs;

use equant::OPERATOR_NAMES;

#[test]
fn project_documentation_covers_every_operator() {
    let root = env!("CARGO_MANIFEST_DIR");
    let readme = fs::read_to_string(format!("{root}/README.md")).unwrap();
    let chinese = fs::read_to_string(format!("{root}/README.zh-CN.md")).unwrap();
    let catalog = fs::read_to_string(format!("{root}/OPERATORS.md")).unwrap();

    assert!(readme.contains("AI-generated Rust"));
    assert!(chinese.contains("AI"));
    for name in OPERATOR_NAMES {
        assert!(catalog.contains(&format!("`{name}`")), "missing {name}");
    }
}

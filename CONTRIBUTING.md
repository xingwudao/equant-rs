# Contributing

Changes to quantitative formulas require evidence, not only code.

Before submitting a change:

- describe the formula and convention in rustdoc and `OPERATORS.md`;
- add a small hand-verifiable example;
- add boundary and non-finite-input tests;
- add a prefix-invariance test for causal behavior;
- identify an independent reference and explain any intentional difference;
- run formatting, tests, documentation tests, and Clippy.

```bash
cargo fmt --check
cargo check --all-targets
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets -- -D warnings
```

Performance changes must include a Criterion comparison and must not trade
away documented numerical semantics.


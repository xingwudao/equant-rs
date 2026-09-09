# equant

`equant` is an AI-first, pure Rust quantitative finance library for batch
technical analysis indicators and market-data operators. It gives
AI-generated Rust programs a small, predictable API over borrowed `f64`
slices, with no runtime dependencies.

The first release includes 60 operators for trend, momentum, volatility,
volume, market structure, and sequence transforms. It is designed for
historical research, feature generation, algorithmic trading experiments,
and batch market data pipelines.

This crate is not a backtesting engine, broker client, DataFrame library, or
TA-Lib binding. It does not require Python, C, OpenXQuant, Pandas, or NumPy.

Chinese documentation: [README.zh-CN.md](README.zh-CN.md)

## Why equant

- Pure Rust with no runtime dependencies or unsafe code.
- Equal-length outputs aligned with input slices.
- Explicit warmup and non-finite-value behavior.
- Named result types for multi-output indicators.
- Actionable errors instead of panics.
- Formula and causality documentation for AI coding agents.
- Property, robustness, and prefix-invariance tests.
- Reproducible Criterion benchmarks without unsupported speed claims.

## Install

```toml
[dependencies]
equant = "0.1"
```

The minimum supported Rust version is 1.81.

## Quick Start

```rust
fn main() -> Result<(), equant::IndicatorError> {
    let close: Vec<f64> = (1..=80).map(|value| value as f64).collect();
    let high: Vec<f64> = close.iter().map(|value| value + 1.0).collect();
    let low: Vec<f64> = close.iter().map(|value| value - 1.0).collect();

    let average = equant::trend::sma(&close, 20)?;
    let strength = equant::momentum::rsi(&close, 14)?;
    let range = equant::volatility::atr(&high, &low, &close, 14)?;
    let macd = equant::trend::macd(&close, 12, 26, 9)?;

    let last = close.len() - 1;
    println!("SMA: {:.2}", average[last]);
    println!("RSI: {:.2}", strength[last]);
    println!("ATR: {:.2}", range[last]);
    println!("MACD histogram: {:.4}", macd.histogram[last]);
    Ok(())
}
```

Run the checked-in version with:

```bash
cargo run --example quickstart
```

## Data Contract

Operators consume one or more slices belonging to a single ordered series.
Related OHLCV slices must have equal length. The library does not sort,
group, resample, or mutate inputs.

Continuous outputs use these rules:

- output length equals the primary input length;
- leading warmup positions are `f64::NAN`;
- a non-finite required input invalidates the current rolling window;
- recursive averages reset and warm up again after non-finite input;
- invalid parameters and mismatched lengths return `IndicatorError`.

`na_check` returns `Vec<bool>`. TD setup and countdown return `Vec<i32>`.
Multi-output functions return structs with named `Vec<f64>` fields.

## Operator Groups

- Trend: 18 operators, including SMA, EMA, MACD, ADX, GMMA, TRIX, and KST.
- Momentum: 14 operators, including RSI, CCI, TSI, SMI, Stochastic, and KDJ.
- Volatility: 7 operators, including ATR, Bollinger, Keltner, and five
  historical volatility estimators.
- Volume: 9 operators, including OBV, CMF, VWAP, MFI, and Chaikin A/D.
- Structure: 4 operators, including pivots, Parabolic SAR, and ZigZag.
- Transform: 8 operators, including growth, lags, Aroon, regression, and TD
  counts.

See [OPERATORS.md](OPERATORS.md) for the complete searchable catalog and
formula conventions.

## Correctness

Correctness takes priority over matching a specific Python or C package.
Definitions are selected from published formulas and independently checked
implementations. Reasonable formula variants are documented instead of hidden.

Causal operators are tested for prefix invariance: appending future data must
not change already calculated history. `zigzag` is explicitly repainting and
must not be treated as a causal trading signal without delayed confirmation.

## Benchmarks

Compile or run the benchmark suite with:

```bash
cargo bench --no-run
cargo bench
```

Benchmarks cover representative operators at 1,000, 100,000, and 1,000,000
observations. Results depend on hardware and compiler settings, so this README
does not claim an unverified speedup over another library.

## Project Status

Version 0.1 focuses only on batch slice APIs. Streaming indicators, Python
bindings, DataFrame adapters, factor libraries, and backtesting are possible
future packages, not hidden commitments in this release.

## License

MIT


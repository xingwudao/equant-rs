use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use equant::{momentum, trend, volatility, volume};

fn prices(length: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let close: Vec<f64> = (0..length)
        .map(|index| 100.0 + index as f64 * 0.001 + (index as f64 / 20.0).sin())
        .collect();
    let high = close.iter().map(|value| value + 1.0).collect();
    let low = close.iter().map(|value| value - 1.0).collect();
    let volume = (0..length)
        .map(|index| 1_000.0 + index as f64 % 100.0)
        .collect();
    (high, low, close, volume)
}

fn operator_benchmarks(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("batch_operators");
    for length in [1_000usize, 100_000, 1_000_000] {
        let (high, low, close, volume) = prices(length);
        group.bench_with_input(BenchmarkId::new("sma", length), &length, |bencher, _| {
            bencher.iter(|| trend::sma(black_box(&close), 20).unwrap())
        });
        group.bench_with_input(BenchmarkId::new("rsi", length), &length, |bencher, _| {
            bencher.iter(|| momentum::rsi(black_box(&close), 14).unwrap())
        });
        group.bench_with_input(BenchmarkId::new("atr", length), &length, |bencher, _| {
            bencher.iter(|| {
                volatility::atr(black_box(&high), black_box(&low), black_box(&close), 14).unwrap()
            })
        });
        group.bench_with_input(BenchmarkId::new("mfi", length), &length, |bencher, _| {
            bencher.iter(|| {
                volume::mfi(
                    black_box(&high),
                    black_box(&low),
                    black_box(&close),
                    black_box(&volume),
                    14,
                )
                .unwrap()
            })
        });
    }
    group.finish();
}

criterion_group!(benches, operator_benchmarks);
criterion_main!(benches);

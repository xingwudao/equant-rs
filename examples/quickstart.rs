use equant::{momentum, trend, volatility};

fn main() -> Result<(), equant::IndicatorError> {
    let close: Vec<f64> = (1..=120)
        .map(|value| 100.0 + value as f64 * 0.2 + (value as f64 / 5.0).sin())
        .collect();
    let high: Vec<f64> = close.iter().map(|value| value + 1.0).collect();
    let low: Vec<f64> = close.iter().map(|value| value - 1.0).collect();

    let average = trend::sma(&close, 20)?;
    let strength = momentum::rsi(&close, 14)?;
    let range = volatility::atr(&high, &low, &close, 14)?;
    let macd = trend::macd(&close, 12, 26, 9)?;
    let last = close.len() - 1;

    println!(
        "SMA={:.4} RSI={:.4} ATR={:.4} MACD_HIST={:.4}",
        average[last], strength[last], range[last], macd.histogram[last]
    );
    Ok(())
}

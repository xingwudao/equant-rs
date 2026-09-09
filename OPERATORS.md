# Operator Catalog

All continuous results are equal-length `f64` arrays. `NaN` marks warmup or a
window invalidated by non-finite input. Unless marked otherwise, an operator is
causal and uses only current and earlier observations.

Trailing sums, extrema, WMA, Aroon, and rolling time regression use linear-time
single-pass algorithms. Recursive operators discard their state after a
required non-finite row and complete their documented warmup again.

## Trend

- `sma`: close; period; simple trailing mean; warmup `period - 1`.
- `ema`: close; period; SMA-seeded exponential mean.
- `dema`: close; period; `2 * EMA - EMA(EMA)`.
- `wma`: close; period; linear weights from 1 through period.
- `hma`: close; period; Hull combination of weighted averages.
- `zlema`: close; period; EMA of a de-lagged series.
- `alma`: close; period, offset, sigma; Gaussian weighted mean.
- `evwma`: price and volume; period; elastic volume-weighted mean.
- `vwma`: price and volume; period; trailing volume-weighted mean.
- `macd`: close; fast, slow, signal; line, signal, and histogram.
- `adx`: high, low, close; period; Wilder ADX, positive DI, and negative DI.
- `gmma`: close; standard short and long Guppy EMA ribbons.
- `tdi`: close; short and long periods; Trend Detection Index and direction.
- `trix`: close; EMA and signal periods; triple-EMA rate of change.
- `dpo`: close; period; causal lagged-price Detrended Price Oscillator.
- `vhf`: close; period; Vertical Horizontal Filter.
- `kst`: close and configuration; weighted smoothed ROC components.
- `po`: close; fast and slow periods; percentage Price Oscillator.

## Momentum

- `rsi`: close; period; Wilder RSI; output range 0 through 100.
- `cci`: high, low, close; period; typical-price CCI with constant 0.015.
- `cmo`: close; period; Chande Momentum Oscillator.
- `tsi`: close; slow and fast periods; double-smoothed True Strength Index.
- `smi`: high, low, close; range, smooth, signal periods; SMI and signal.
- `wpr`: high, low, close; period; Williams Percent Range from -100 to 0.
- `ultimate_oscillator`: high, low, close; periods; weights 4, 2, and 1.
- `roc`: close; period; percentage Rate of Change.
- `momentum`: close; period; arithmetic price difference.
- `cti`: close; period; Pearson correlation with the window time index.
- `rvi`: open, high, low, close; period; Relative Vigor Index and signal.
- `dvi`: close; long and short periods; short/long volatility percentage.
- `stoch`: high, low, close; K, D, slow periods; three stochastic lines.
- `kdj`: high, low, close; period and smoothing; K, D, and J lines.

## Volatility

- `atr`: high, low, close; period; Wilder Average True Range.
- `tr`: high, low, close; gap-aware True Range.
- `bollinger`: close; period and deviation multiple; five named outputs.
- `keltner`: high, low, close; EMA, ATR, and multiplier parameters.
- `donchian`: high and low; period; upper, lower, and middle channel.
- `pbands`: close; period and deviation multiple; percentage bands.
- `volatility`: OHLC; period, annualization, estimator; annualized result. The
  Yang-Zhang variant uses window sample variances and its period-dependent
  weight; estimators ignore OHLC fields not required by their formula.

`volatility` supports Close-to-Close, Parkinson, Garman-Klass,
Rogers-Satchell, and Yang-Zhang estimators.

## Volume

- `obv`: close and volume; segment-based On-Balance Volume.
- `cmf`: high, low, close, volume; period; Chaikin Money Flow.
- `vwap`: high, low, close, volume; period; rolling typical-price VWAP.
- `mfi`: high, low, close, volume; period; Money Flow Index.
- `emv`: high, low, volume; period; smoothed Ease of Movement.
- `clv`: high, low, close; Close Location Value from -1 to 1.
- `chaikin_ad`: high, low, close, volume; Accumulation/Distribution line.
- `chaikin_volatility`: high and low; EMA and ROC periods.
- `williams_ad`: high, low, close; Williams Accumulation/Distribution.

## Structure

- `zigzag`: high and low; threshold and unit; repainting swing detector.
- `pivots`: high, low, close; prior-bar floor pivot levels.
- `sar`: high and low; acceleration and maximum; Parabolic SAR.
- `snr`: high and low; period; rolling support, resistance, and midpoint.

`zigzag` is classified as `Repainting`. Its most recent extreme can move when
future data arrives. The remaining structure operators are causal.

## Transform

- `growth`: input; period; fractional change divided by absolute prior value.
- `adj_ratios`: close and adjusted close; pointwise adjustment ratio.
- `roll_sfm`: input; period; rolling intercept, time slope, and R-squared.
- `aroon`: high and low; period; up, down, and oscillator values.
- `td_setup`: close; signed DeMark setup counts returned as integers.
- `td_countdown`: high, low, close; signed non-consecutive countdown integers;
  begins when setup first reaches 9 and omits optional recycle/cancel variants.
- `na_check`: input; Boolean mask for NaN and infinity.
- `lags`: input; period; backward shift with NaN padding.

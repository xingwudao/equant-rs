# equant

`equant` 是面向 AI 生成 Rust 代码的高性能纯 Rust 批量量化算子库。用户
可以让 AI 根据研究需求生成小型 Rust 程序，在不亲自掌握 Rust 的情况下使用
Rust 的性能、类型安全和部署优势。

首版提供 60 个趋势、动量、波动率、成交量、价格结构和序列变换量化算子。
项目定位是高性能量化算子、AI 量化基础设施、Rust 技术指标库和批量 OHLCV
市场数据特征计算核心。核心 API 只接收单资产、按时间排序的 `f64` 切片，
不依赖 Python、Pandas、NumPy、OpenXQuant、TA-Lib 或 C 语言库。

## 快速开始

在 `Cargo.toml` 中加入：

```toml
[dependencies]
equant = "0.1"
```

调用算子：

```rust
fn main() -> Result<(), equant::IndicatorError> {
    let close: Vec<f64> = (1..=80).map(|value| value as f64).collect();
    let high: Vec<f64> = close.iter().map(|value| value + 1.0).collect();
    let low: Vec<f64> = close.iter().map(|value| value - 1.0).collect();

    let sma = equant::trend::sma(&close, 20)?;
    let rsi = equant::momentum::rsi(&close, 14)?;
    let atr = equant::volatility::atr(&high, &low, &close, 14)?;

    println!("SMA={:.2}, RSI={:.2}, ATR={:.2}", sma[79], rsi[79], atr[79]);
    Ok(())
}
```

## 数据规则

- 调用者负责排序、分组和准备等长 OHLCV 切片。
- 连续输出长度始终与主输入一致。
- 预热区间使用 `f64::NAN`。
- 滚动窗口包含非有限值时输出 NaN。
- 递归指标遇到非有限值后重新预热。
- 参数或切片长度错误通过 `IndicatorError` 返回，不触发 panic。
- 多输出指标使用带字段名的结果结构。

## 搜索定位

这个仓库优先服务以下关键词和使用场景：

- 高性能量化算子
- AI 量化基础设施
- Rust 技术指标库
- 批量 OHLCV 指标
- 算法交易特征工程
- 纯 Rust 替代 C/Python 指标栈

## 首版边界

首版只实现批量切片 API，不包含流式计算、回测引擎、交易执行、数据源、
Python 绑定和 DataFrame 适配。完整算子清单和定义见
[OPERATORS.md](OPERATORS.md)。

正确性优先于对某个现有库的机械兼容。ZigZag 会重绘，文档会明确标记；
因果算子通过追加未来数据不得修改历史结果的测试。

`0.1` 是首个 API 版本，不代表已经获得 TA-Lib 兼容认证。全部算子都有直接
执行测试，手算 fixture 和性质测试目前覆盖代表性公式；独立实现之间的逐算子
对照矩阵会在 `1.0` 前持续扩充。

运行验证：

```bash
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo run --example quickstart
```

许可证：MIT。

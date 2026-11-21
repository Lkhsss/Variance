# Variance
[![build](https://github.com/Lkhsss/Variance/actions/workflows/build.yml/badge.svg)](https://github.com/Lkhsss/Variance/actions/workflows/build.yml)

一个简单的计算方差的工具，实现了彩色输出。
由Rust编写而成。
是我学习Rust语言的练手之作。

---
#### 命令格式：
    .\variance [选项] <num1> <num2> <num3> <num4> <num5> ...
#### 例：
    .\variance 1 2 3 4 5 6 7
    .\variance --sample 1 2 3 4 5 6 7

#### 选项：
| 选项 | 说明 |
| --- | --- |
| `-s`, `--sample` | 输出无偏样本方差（÷(n-1)，Bessel 校正） |
| `-p`, `--population` | 输出总体方差（÷n，**默认**） |
| `-h`, `--help` | 显示帮助 |

---

## 方差口径说明
- **总体方差（默认）**：`σ² = Σ(x - x̄)² / n`，这是方差的数学定义（`numpy.var`、`statistics.pvariance`、Excel `VAR.P` 的默认口径）。
  输入的数据被视为"全部总体"，命令行工具默认使用该口径。
- **样本方差（`--sample`）**：`s² = Σ(x - x̄)² / (n-1)`，是总体方差的无偏估计量，用于"数据只是样本、需要推断总体"的场景。
  数据少于 2 项时结果为 `NaN`。

> 两者的离差平方和 `M2 = Σ(x - x̄)²` 完全相同，只是分母不同；增量累加使用 Welford 在线算法。

---

## 作为库使用

```toml
[dependencies]
variance = "1.2"
```

```rust
use variance::{variance, Variance};

// 一次性计算（总体方差，÷n）
let v = variance(&[1., 2., 3., 4., 5., 6., 7.]); // 4.0

// 流式累加：Constructing 可 update，Computed 只读
let mut acc = Variance::new(); // 或 Variance::from_slice(&data)
for x in [1., 2., 3.] {
    acc.update(x);
}
let stat = acc.compute();

stat.mean();                   // 均值
stat.count();                  // 样本数
stat.get_variance();           // 总体方差 ÷n
stat.get_sample_variance();    // 无偏样本方差 ÷(n-1)，n<2 时为 NaN
stat.get_standard_deviation(); // 总体标准差
```

`Variance<S>` 用 Typestate 在编译期约束状态：只有 `Variance<Constructing>` 能 `update` / `compute`，`Variance<Computed>` 只能读取，因此不会读到过期结果。

---

# CHANGELOG
## v1.0.0
1. 实现了总数，平均数，方差的计算
2. 实现了彩色输出

## v1.0.1
1. 增加了标准差的计算

## v1.0.2
1. 修改输出颜色使数据易读
2. 更改编译选项，优化程序大小

## v1.2.0
将函数部分抽象抽离并增加了新学习的`Typestate`模式！
### CHANGE
1. 算法库化：`lib.rs` 提供 `Welford` 在线算法，并用类型状态 `Variance<Constructing>` / `Variance<Computed>` 在编译期约束状态
2. 明确方差口径：默认输出总体方差（÷n），新增 `--sample` 输出无偏样本方差（÷(n-1)）
3. 修复缺陷：增量更新后可能读到过期方差、均值与方差口径不一致（`update` 现在只允许在 `Constructing` 状态调用）
4. 参数校验增强：拒绝 `NaN` / `inf` 等非有限实数，新增 `--help`，版本号由 `CARGO_PKG_VERSION` 自动生成
5. 补充算法正确性测试：两遍法与 Welford 增量结果一致、分段累加、单数据、空数据等

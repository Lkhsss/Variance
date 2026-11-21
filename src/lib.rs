use std::marker::PhantomData;

#[cfg(test)]
mod test;

/// # variance
/// Computes the **population variance** of `data` (sum of squared deviations / N).
///
/// Returns `NaN` for an empty slice and `0.0` for a single value.
/// For the unbiased sample variance (N-1), use [`Variance`] with
/// [`Variance::get_sample_variance`].
///
/// 直接计算一组数据的**总体方差**（离差平方和除以 N）。
///
/// - 空切片返回 `NaN`；
/// - 只有一个数据时返回 `0.0`（总体方差在 N=1 时是良定义的）。
///
/// 如果需要无偏样本方差（除以 N-1），请使用 [`Variance::from_slice`]
/// 配合 [`Variance::compute`] 与 [`Variance::get_sample_variance`]。
pub fn variance(data: &[f64]) -> f64 {
    if data.is_empty() {
        return f64::NAN;
    }
    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;
    data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n
}

/// Constructing state: samples may still be pushed or the result computed.
///
/// 构造中状态：可以继续累加数据（`update`）或进入计算状态（`compute`）。
pub struct Constructing;
/// Computed state: read-only, so stale values can never be returned.
///
/// 已计算状态：只能读取结果，不允许再修改，避免读到过期数据。
pub struct Computed;

/// Variance calculator using Welford's online algorithm.
///
/// The state is fixed at compile time by `S`:
/// - [`Constructing`]: `update` / `with` / `compute` are available;
/// - [`Computed`]: read-only (`mean`, `count`, variance, standard deviation).
///
/// 方差计算器，累加部分使用 Welford 在线算法：
///
/// ```text
/// n'    = n + 1
/// delta = x - mean
/// mean' = mean + delta / n'
/// M2'   = M2 + delta * (x - mean')
/// ```
///
/// 状态由类型参数 `S` 在编译期约束：
/// - [`Constructing`]：可调用 `update` / `with` / `compute`；
/// - [`Computed`]：只能读取 `mean`、`count`、方差与标准差。
#[derive(Clone, Debug)]
pub struct Variance<S> {
    n: u64,
    mean: f64,
    /// Sum of squared deviations: M2 = Σ(x - mean)²
    ///
    /// 离差平方和 M2 = Σ(x - mean)²
    m2: f64,
    _phantom: PhantomData<S>,
}

impl<S> Variance<S> {
    /// Mean of all samples processed so far (0 when empty).
    ///
    /// 截至目前为止已处理的所有样本的均值（无数据时为 0）
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Number of samples processed so far.
    ///
    /// 截至目前为止已处理的样本数量
    pub fn count(&self) -> u64 {
        self.n
    }
}

impl Variance<Constructing> {
    /// Creates an empty calculator.
    ///
    /// 创建一个空的计算器
    pub fn new() -> Variance<Constructing> {
        Variance {
            n: 0,
            mean: 0.,
            m2: 0.,
            _phantom: PhantomData,
        }
    }

    /// Builds a calculator from existing data (two-pass: mean, then M2).
    ///
    /// An empty slice equals [`Variance::new`], and [`Variance::compute`]
    /// then yields `NaN`.
    ///
    /// 用一批已有数据构造计算器（两遍法：先求均值，再求 M2）
    ///
    /// 空切片等价于 [`Variance::new`]，此时 [`Variance::compute`] 得到的结果为 `NaN`。
    pub fn from_slice(data: &[f64]) -> Variance<Constructing> {
        let n = data.len() as u64;
        if n == 0 {
            return Variance::new();
        }

        // 第一遍：均值
        let mean = data.iter().sum::<f64>() / n as f64;

        // 第二遍：离差平方和 m2 = Σ(x - mean)²
        let m2 = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>();

        Variance {
            n,
            mean,
            m2,
            _phantom: PhantomData,
        }
    }

    /// Adds one sample using Welford's recurrence.
    ///
    /// May be called after [`Variance::from_slice`]: the two-pass
    /// `(n, mean, M2)` equals Welford's state after n samples.
    ///
    /// 增量地加入一个数据（Welford 递推）。
    ///
    /// 可以在 [`Variance::from_slice`] 之后继续调用：两遍法得到的
    /// `(n, mean, M2)` 与 Welford 累加 n 个数据后的状态在数学上等价。
    pub fn update(&mut self, x: f64) {
        self.n += 1;
        let delta = x - self.mean;
        self.mean += delta / self.n as f64;
        self.m2 += delta * (x - self.mean);
    }

    /// Chained version of [`Variance::update`].
    ///
    /// [`Variance::update`] 的链式版本
    pub fn with(mut self, x: f64) -> Variance<Constructing> {
        self.update(x);
        self
    }

    /// Switches to the computed state.
    ///
    /// 进入已计算状态
    pub fn compute(&self) -> Variance<Computed> {
        Variance {
            n: self.n,
            mean: self.mean,
            m2: self.m2,
            _phantom: PhantomData,
        }
    }
}

impl Variance<Computed> {
    /// Population variance (M2 / N, the mathematical definition).
    ///
    /// Returns `NaN` when there is no data.
    ///
    /// 总体方差（除以 N，方差的数学定义）
    ///
    /// 没有任何数据时返回 `NaN`。
    pub fn get_variance(&self) -> f64 {
        if self.n == 0 {
            f64::NAN
        } else {
            self.m2 / self.n as f64
        }
    }

    /// Unbiased sample variance (M2 / (N-1), Bessel's correction).
    ///
    /// Returns `NaN` when fewer than 2 samples.
    ///
    /// 无偏样本方差（除以 N-1，Bessel 校正）
    ///
    /// 数据少于 2 项时返回 `NaN`。
    pub fn get_sample_variance(&self) -> f64 {
        if self.n < 2 {
            f64::NAN
        } else {
            self.m2 / (self.n - 1) as f64
        }
    }

    /// Population standard deviation (square root of the population variance).
    ///
    /// 总体标准差（总体方差的平方根）
    pub fn get_standard_deviation(&self) -> f64 {
        self.get_variance().sqrt()
    }

    /// Sample standard deviation (square root of the unbiased sample variance).
    ///
    /// 样本标准差（无偏样本方差的平方根）
    pub fn get_sample_standard_deviation(&self) -> f64 {
        self.get_sample_variance().sqrt()
    }
}

impl Default for Variance<Constructing> {
    fn default() -> Self {
        Self::new()
    }
}

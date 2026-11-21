//! 算法正确性测试
//!
//! 参考值说明：
//! - MICHELSON 数据集共 20 项，Σx = 18180，均值 = 909，M2 = 209180
//!   → 总体方差 = M2/20 = 10459，样本方差 = M2/19 ≈ 11009.473684210526
//! - 1..7 共 7 项，均值 = 4，M2 = 28
//!   → 总体方差 = 4，样本方差 = 28/6 ≈ 4.666666666666667

use crate::{Constructing, Variance, variance};

const MICHELSON: [f64; 20] = [
    850., 740., 900., 1070., 930., 850., 950., 980., 980., 880., 1000., 980., 930., 650., 760.,
    810., 1000., 1000., 960., 960.,
];

/// 相对误差比较（Welford 与两遍法只应相差舍入误差）
fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * b.abs().max(1.0)
}

#[test]
fn population_variance_matches_reference() {
    let v = Variance::from_slice(&MICHELSON).compute();
    assert!(
        approx(v.get_variance(), 10459.0),
        "got {}",
        v.get_variance()
    );
    assert_eq!(v.count(), MICHELSON.len() as u64);
    assert!(approx(v.mean(), 909.0));
}

#[test]
fn sample_variance_matches_reference() {
    let v = Variance::from_slice(&MICHELSON).compute();
    assert!(
        approx(v.get_sample_variance(), 11009.473684210526),
        "got {}",
        v.get_sample_variance()
    );
}

#[test]
fn standard_deviation_is_sqrt_variance() {
    let v = Variance::from_slice(&MICHELSON).compute();
    assert!(approx(v.get_standard_deviation(), v.get_variance().sqrt()));
    assert!(approx(
        v.get_sample_standard_deviation(),
        v.get_sample_variance().sqrt()
    ));
}

#[test]
fn two_pass_matches_direct_function() {
    // variance() 与 from_slice() 使用同样的两遍法，结果应一致
    let direct = variance(&MICHELSON);
    let from_slice = Variance::from_slice(&MICHELSON).compute().get_variance();
    assert!(approx(direct, from_slice), "{direct} != {from_slice}");
}

#[test]
fn welford_incremental_matches_two_pass() {
    // 真正的 Welford 增量路径：与两遍法只允许有舍入级别的差异
    let mut acc = Variance::new();
    for &x in &MICHELSON {
        acc.update(x);
    }
    let computed = acc.compute();

    assert!(
        approx(computed.get_variance(), variance(&MICHELSON)),
        "{} != {}",
        computed.get_variance(),
        variance(&MICHELSON)
    );
    assert!(
        approx(
            computed.get_sample_variance(),
            Variance::from_slice(&MICHELSON)
                .compute()
                .get_sample_variance()
        ),
        "{} != {}",
        computed.get_sample_variance(),
        Variance::from_slice(&MICHELSON)
            .compute()
            .get_sample_variance()
    );
    assert!(approx(computed.mean(), 909.0));
}

#[test]
fn with_chains_incrementally() {
    let mut acc = Variance::new();
    for &x in &[1.0, 3.0] {
        acc = acc.with(x);
    }
    let computed = acc.compute();
    // [1, 3]：均值 2，M2 = 1 + 1 = 2
    assert!(approx(computed.mean(), 2.0));
    assert!(approx(computed.get_variance(), 1.0));
    assert!(approx(computed.get_sample_variance(), 2.0));
}

#[test]
fn update_continues_from_slice_base() {
    // 先两遍法处理一半，再 Welford 累加另一半，结果应与一次性处理全部相同
    let mut acc = Variance::from_slice(&MICHELSON[..10]);
    for &x in &MICHELSON[10..] {
        acc.update(x);
    }
    let computed = acc.compute();

    assert_eq!(computed.count(), MICHELSON.len() as u64);
    assert!(
        approx(computed.get_variance(), variance(&MICHELSON)),
        "{} != {}",
        computed.get_variance(),
        variance(&MICHELSON)
    );
}

#[test]
fn simple_dataset() {
    let v = Variance::from_slice(&[1., 2., 3., 4., 5., 6., 7.]).compute();
    assert!(approx(v.mean(), 4.0));
    assert!(approx(v.get_variance(), 4.0));
    assert!(approx(v.get_sample_variance(), 28.0 / 6.0));
}

#[test]
fn single_value() {
    // 总体方差在 N=1 时为 0；样本方差需要至少 2 项
    let v = Variance::from_slice(&[42.0]).compute();
    assert_eq!(v.count(), 1);
    assert!(approx(v.get_variance(), 0.0));
    assert!(approx(v.get_standard_deviation(), 0.0));
    assert!(v.get_sample_variance().is_nan());
    assert!(v.get_sample_standard_deviation().is_nan());
}

#[test]
fn empty_data_is_nan() {
    assert!(variance(&[]).is_nan());

    let v = Variance::from_slice(&[]).compute();
    assert_eq!(v.count(), 0);
    assert!(v.get_variance().is_nan());
    assert!(v.get_standard_deviation().is_nan());
    assert!(v.get_sample_variance().is_nan());
}

#[test]
fn default_is_empty() {
    let v = Variance::<Constructing>::default();
    assert_eq!(v.count(), 0);
    assert!(v.compute().get_variance().is_nan());
}

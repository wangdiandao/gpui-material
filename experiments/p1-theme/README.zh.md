# P1-08 纯算法证据

[English](README.md) | 中文

2026-11-13，Windows 11 x64，Rust/Cargo 1.99.1，固定的 crates.io 解析。

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-theme/Cargo.toml -- --nocapture
cargo +1.99.1 tree --locked --manifest-path experiments/p1-theme/Cargo.toml --edges features
cargo +1.99.1 tree --locked --manifest-path experiments/p1-theme/Cargo.toml --edges features --invert material-color-utils
```

| 项目 | 期望 | 观察 | 状态 |
| --- | --- | --- | --- |
| 蓝色 tone 40 | `ff343dff` | `ff343dff` | pass |
| 红色 Tonal Spot，深色，默认对比度 | `primaryFixed = ffffdad4` | Spec2021 得 `ffffdad4` | pass |
| 默认对比度的固定角色 | 浅深色保持一致，主色随模式变更 | `primaryFixed`、`primaryFixedDim` 相同，`primary` 不同 | pass |
| 红色低/默认/高对比度 | `primary/onPrimary` 对比度递增 | 浅色 4.544/6.429/13.935，深色 4.535/7.750/18.427 | pass |
| 越界 tone 直接调用 | `ratioOfTones(-10, 110) = 21` | Rust 0.1.3 返回 34.20435515298331 | fail |
| 越界 tone 经适配 | 21，容差 0.001 | 在调用边界夹取到 [0, 100] 后返回 21 | pass |
| chrono 日期边界 | 闰日、跨年和范围边界正确 | 2024-01-31 加一月为 2024-02-29，2023-02-29 无效，最大/最小日期越界返回 None | pass |
| 纯依赖图 | 无 GPUI，MCU 不启用 image、serde、rayon | MCU 反向 feature 图只显示探针；包图只包含 Rust 算法及平台基础依赖 | pass |

[固定官方越界样本][vector]和[夹取语义][clamp]确认了候选库差异。最小复现为 `Contrast::ratio_of_tones(-10., 110.)`。如将来升级到提供相同夹取语义的上游发布版，先复跑参考向量再去掉适配。当前结果只覆盖候选可行性。

表中的 chrono 日期边界来自 [P1 接受提交](https://github.com/wangdiandao/gpui-material/tree/b0250f5f120ab2fbadfe41b5529612e984358295)。当前探针保留颜色参考向量，Base Calendar 的禁用闰日行为由宿主的 `theme` 探针验证。

[vector]: https://github.com/material-foundation/material-color-utilities/blob/194144d75ed7d34f06264a625e086db81a7ab9aa/dart/test/contrast_test.dart#L15-L18
[clamp]: https://github.com/material-foundation/material-color-utilities/blob/194144d75ed7d34f06264a625e086db81a7ab9aa/typescript/contrast/contrast.ts#L36-L47

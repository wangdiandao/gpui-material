# P1-08 两窗口主题投影与 Base Calendar

简体中文 · [English](08-theme.en.md)

2026-11-13，Windows 11 x64 build 26200，Rust/Cargo 1.99.1，使用此 PR 的代码与已锁定的 crates.io 依赖。

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin theme
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin theme
```

| 场景 | 观察 | 状态 |
| --- | --- | --- |
| 两窗口读取 App 投影 | App Light 时一窗跟随、一窗局部深色。显式设置 App Dark 并刷新，两窗的已绘制状态均读到 App Dark，局部策略不变。每次读完后 App global 保持设定值。 | pass |
| Base Calendar 闰日 | 禁用范围命中 2024-02-29 时拒绝选择并保留 01-31，撤销禁用后可以选择同一闰日。 | pass |
| Base 源码读取路径 | 发布版本的 Scrollbar、ResizeHandle、ScrollableMask、输入解析、文本选择、TextView 与 plot 默认值使用 App 级 Theme；局部覆盖通过显式样式传递。 | pass |
| 实窗回归 | Windows 原生二进制已构建。窗口间像素与各平台实窗待确认。 | pending |

## Windows 双窗口实机复核

2026-11-13，Windows 11 x64 build 26200，Windows GPUI 后端，100% 缩放，Rust 1.99.1，winapp 0.7.1。探针来自主线 `b050f4f` 的 `theme` 二进制。winapp 在“局部深色”窗口对「切换应用主题」执行一次 UIA Invoke，两窗口在 43 ms 内均找到 App Dark 状态。

| 窗口 | 切换前 | 切换后 |
| --- | --- | --- |
| 跟随 App | [App Light](08-follow-light.png) | [App Dark](08-follow-dark.png) |
| 局部深色 | [App Light](08-local-light.png) | [App Dark](08-local-dark.png) |

四张原生截图中，全局背景取 (250, 200)，局部块取 (25, 100)。两窗口的全局 RGB 从 (255, 255, 255) 一致变为 (10, 10, 10)；局部深色窗口的块在切换前后均为 (10, 10, 10)。Windows 两窗口主题投影通过。纯算法结果见 [P1-08 纯算法](../../p1-theme/README.md)。

# P1-08 两窗口主题投影与 Base Calendar

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

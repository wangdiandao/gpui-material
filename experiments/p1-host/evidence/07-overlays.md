# P1-07 浮层、滚动与窗口

2026-11-13，Windows 11 x64 build 26200，Rust/Cargo 1.99.1，使用此 PR 的代码与已锁定的 crates.io 依赖。

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin overlays
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin overlays
```

| 场景 | 观察 | 状态 |
| --- | --- | --- |
| 嵌套 Popover 和键盘回焦 | 窗口右侧触发外层，打开子层。两个面板均在视口右界之内。Esc 只关闭子层并回到子触发器，第二次 Esc 关闭外层并回到外触发器。 | pass |
| 模态、外点、RTL、值标签、嵌套滚动、改尺寸和拖动所有权 | 基础键盘路径已覆盖；待补相应场景与实机证据。 | pending |

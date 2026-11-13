# P1-04 原生语义

2026-11-13，Windows 11 x64 build 26200，Rust/Cargo 1.99.1，使用此 PR 的代码与已锁定的 crates.io 依赖。实窗缩放 100%，UIA 和截图工具为 winapp 0.7.1。

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin semantics
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin semantics
```

| 场景 | 观察 | 状态 |
| --- | --- | --- |
| Windows 无头绘制节点 | Button、CheckBox、Slider、Status 的角色与名称符合断言，初始复选值为 false。 | pass |
| Windows 系统 UIA 树 | `Group 操作示例` 下有 Button、CheckBox、Slider 和 StatusBar。 | pass |
| Windows Invoke 与 Toggle | 执行一次 `winapp ui invoke '执行操作' --action invoke`，StatusBar 从 0 到 1。单次 TogglePattern 后复选状态更新为 on。winapp 0.7.1 的 `toggle-on` 即时回读为 Off，紧随的系统树已为 On。[动作后截图](04-windows.png)。 | pass |
| 数值范围读写、编辑、浮层、通知播报和各平台读屏 | Windows 确认可枚举 Slider 与状态节点，Narrator、VoiceOver 和 Orca 路径待运行。 | pending |

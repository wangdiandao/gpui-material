# P1-02 激活、焦点与取消

2026-11-13，Windows 11 x64 build 26200，Rust/Cargo 1.99.1，使用此 PR 的代码与已锁定的 crates.io 依赖。实窗缩放 100%，UIA 和截图工具为 winapp 0.7.1。

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin activation
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin activation
```

| 场景 | 观察 | 状态 |
| --- | --- | --- |
| Windows 无头输入路径 | 左键按下与同一控件内释放执行一次；在控件外释放不执行。Enter 和 Space 各三次重复后释放，每键执行一次。| pass |
| Windows 无头焦点 | `Window::last_input_was_keyboard()` 在鼠标和键盘路径读数不同。Tab 到达下一个焦点，禁用后反向和正向遍历跳过禁用项。| pass |
| Windows 原生画面 | [截图](02-windows.png)和 UIA 树显示三个按钮及初始计数。 | pass |
| Roving focus、捕获丢失、禁用中断拖动、三平台真实指针与焦点 | 本次测试覆盖窗口内的移出控件及按键重复。其余场景待运行。 | pending |

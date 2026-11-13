# P1-02 激活、焦点与取消

[English](02-activation.md) | 中文

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
| Roving focus、捕获丢失、禁用中断拖动 | 渲染事件回归覆盖 roving focus 与禁用。Windows 实窗由 agent-desktop 发起拖动，由 Win32 释放本窗口捕获。 | pass |

<a id="roving-focus-and-drag-cancellation"></a>

## Roving focus 与拖动取消

2026-11-13，Windows 11 x64，Rust 1.99.1。两项单选组成一个 Tab 停靠点，方向键更改焦点和选中项；Tab 离组再 Shift+Tab 返回时，焦点回到已选项。GPUI 测试窗口连续启动三次拖动，取消顺序为 Esc、禁用、重新启用后 Esc。三次均清除活动拖动与 GPUI 命中盒捕获。

<a id="native-windows-capture-loss"></a>

## Windows 系统捕获丢失

固定 `gpui-pre-windows 0.3.12` 的鼠标按下调用 `SetCapture`，释放调用 `ReleaseCapture`，事件分派缺少 `WM_CAPTURECHANGED`。探针在本窗口 UI 线程的活动拖动期间检查 `GetCapture`，捕获离开后复用 Esc 路径清除状态，窗口停用也进入同一取消路径。Win32 调用限定在 Windows 探针目标，复用锁文件已有的 `windows 0.62.2`。退出条件为上游向 GPUI 交互层提供等价的原生捕获丢失通知。[上游 Issue 搜索](https://github.com/zed-industries/zed/issues?q=WM_CAPTURECHANGED) · [Win32 捕获变更](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-capturechanged)

```powershell
$env:P1_ACTIVATION_RELEASE_CAPTURE='1'
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin activation
```

在实窗从「拖动此处」向右拖出，继续按住至少 650 ms。自动化用 agent-desktop 的 `drag --duration 220 --drop-delay 650`。探针调用本窗口线程的 `ReleaseCapture`，在松开前检测到原生捕获离开，系统 UIA 状态显示「系统捕获已丢失」。窗口仍有焦点，计数仍为「指针 0 · 键盘 0」。同样动作连续两次均检测到丢失。单独的禁用中断结果来自上述 GPUI 测试。

<a id="upstream-patch-retest"></a>

## 上游补丁复测

在 Windows 11 x64 build 26200、Rust 1.98.1 上，Zed GPUI 的 `fix/win32-capture-loss` 分支提交 `f4f9837` 通过 `cargo check --locked -p gpui_windows` 和 `cargo test --locked -p gpui --lib window::tests::` 的全部 41 项测试。回归测试覆盖清除待定点击、取消拖动以及下一次点击。

原生复现可从 Zed 的 `crates/gpui/examples/drag_drop.rs` 开始。在首次拖动的 `on_drag` 回调中安排窗口线程在 350 ms 后调用 Win32 `ReleaseCapture`。检查 `GetCapture` 从非零变为零。在绘制阶段通过 `Window::on_mouse_event` 注册 `MouseCaptureLostEvent` 监听器，统计冒泡阶段的通知，并分别统计放下和点击。第一次拖到目标后按住 800 ms，再做一次无需强制释放的拖放，最后点击拖动源。

上一版补丁的前台输入确认左键仍按住时发生捕获丢失，收到一次通知，放下和点击均为零。下一次拖放和点击各触发一次。最终提交 `f4f9837` 通过投递 Win32 鼠标消息在 GPUI 实窗重复该序列，窗口 UI 线程调用了 `ReleaseCapture`。UIA 状态显示累计拖动开始、捕获丢失、放下和点击分别为 `1/1/0/0`、`2/1/1/0` 和 `2/1/1/1`。最终版本的前台输入和完整 Zed 应用待验证。 [跟踪 Issue #48](https://github.com/wangdiandao/gpui-material/issues/48)

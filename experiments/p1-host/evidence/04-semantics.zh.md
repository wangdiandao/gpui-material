# P1-04 原生语义

[English](04-semantics.md) | 中文

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
| 数值范围读写、编辑、浮层与读屏 | Windows UIA 验证范围、编辑和浮层。Windows 实机签收采用负责人确认，VoiceOver 和 Orca 为平台探索。 | pass |

<a id="windows-rangevalue-editing-and-overlay-relationships"></a>

## Windows RangeValue、编辑与浮层关系

2026-11-13，Windows 11 x64 build 26200，Rust 1.99.1，agent-desktop 1.2.0。UIA RangeValuePattern 的 SetValue 发出 AccessKit `SetValue(NumericValue)`。固定 Base Slider 只注册 Increment/Decrement。探针的范围元素补上 SetValue 与焦点绑定，通过 Base SliderState 的 `step_thumb` 更新并复用步进与边界规则。该适配仍在实验宿主。

- 「级别」初值 3、范围 0–10、步长 1。一次 RangeValuePattern SetValue 后窗口状态为「级别 7」，获取焦点并按右键后为 8。agent-desktop 的通用 value 回读为空并报告未验证，重新读取独立的 UIA 状态节点及[截图](04-windows-range.png)验证更新。一次按钮 Invoke 后计数从 0 到 1。
- 加入 Base SliderIndicator 记录轨道实际测量边界后，指针点击轨道 60% 位置时 UIA 状态为「级别 6」。滑块位置随状态移动，RangeValue 写入与键盘步进复测通过。
- 普通编辑框的 ValuePattern 写入 `UIA 中文` 并回读一致。密码框为 secure，无 SetValue 与明文值。
- 用 UIA 聚焦「打开浮层输入」后按 Enter。系统树显示对话框包含 `probe-popup-text` 编辑节点，ValuePattern 可写入并回读 `浮层 UIA 中文`。Esc 后该节点从系统树移除。
- Narrator 实机通过由项目负责人确认，自动化只测上述系统树和动作。见 [Windows 实机记录](../../../docs/phases/P01-evidence.zh.md#native-windows-acceptance)。

上游退出条件与最小复现见 [Issue #49](https://github.com/wangdiandao/gpui-material/issues/49)。

<a id="upstream-slider-retest"></a>

## 上游 Slider 复测

在 Windows 11 x64 build 26200 上，GPUI Kit 提交 `4a874b81` 通过 `cargo +1.99.1 test --locked -p gpui-base --lib slider::tests` 的全部 23 项测试。在上游仓库运行 `cargo +1.99.1 run --locked -p gpui-base-examples --bin components -- slider`。在原生 UIA 树找到 Slider 并获取 `RangeValuePattern`。初值为 64，范围为 0–100，SmallChange 为 1。依次调用 `SetValue` 写入 73、0、100 和 41，`RangeValuePattern.Current.Value` 每次回读均与请求值相符。 [GPUI Kit PR #3963](https://github.com/longbridge/gpui-kit/pull/3963)

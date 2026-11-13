# P1-04 Native semantics

English | [中文](04-semantics.zh.md)

Recorded on 2026-11-13 with Windows 11 x64 build 26200 and Rust/Cargo 1.99.1, using the PR code and locked crates.io dependencies. The real window used 100% scaling. UIA and screenshots used winapp 0.7.1.

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin semantics
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin semantics
```

| Scenario | Observation | Status |
| --- | --- | --- |
| Windows headless rendered nodes | Button, CheckBox, Slider, and Status matched role and name assertions. The initial checkbox value was false. | pass |
| Windows UIA tree | `Group 操作示例` contained Button, CheckBox, Slider, and StatusBar. | pass |
| Windows Invoke and Toggle | One `winapp ui invoke '执行操作' --action invoke` changed the StatusBar from 0 to 1. One TogglePattern call turned the checkbox on. winapp 0.7.1 immediately read Off for `toggle-on`, and the next system tree read On. [Screenshot after the actions](04-windows.png) | pass |
| Range read and write, editing, overlays, and screen readers | Windows UIA verified ranges, editing, and overlays. The owner confirmed native Windows acceptance. VoiceOver and Orca are exploratory. | pass |

<a id="windows-rangevalue编辑与浮层关系"></a>

## Windows RangeValue, editing, and overlay relationships

Recorded on 2026-11-13 with Windows 11 x64 build 26200, Rust 1.99.1, and agent-desktop 1.2.0. UIA RangeValuePattern SetValue emits AccessKit `SetValue(NumericValue)`. Pinned Base Slider registers Increment and Decrement. The probe adds SetValue and focus binding and updates Base SliderState through `step_thumb`, reusing stepping and bounds. The adapter is in the experimental host.

- “级别” (Level) started at 3, with range 0–10 and step 1. One RangeValuePattern SetValue changed it to 7. Focus plus Right changed it to 8. agent-desktop's generic value read was empty and reported unverified. The independent UIA status node and [screenshot](04-windows-range.png) confirmed the update. One button Invoke changed the count from 0 to 1.
- After adding Base SliderIndicator to measure track bounds, clicking at 60% showed level 6 in UIA. The thumb moved with the value. RangeValue writes and keyboard steps passed retesting.
- The ordinary edit field accepted and read back `UIA 中文` through ValuePattern. The secure password field exposed neither SetValue nor plaintext.
- Focusing “打开浮层输入” (Open overlay input) via UIA and pressing Enter opened a dialog with the `probe-popup-text` edit node. ValuePattern wrote and read back `浮层 UIA 中文`. Esc removed the node from the system tree.
- The project owner confirmed native Narrator acceptance. Automation verified the system tree and actions above. [Native Windows record](../../../docs/phases/P01-evidence.md#native-windows-acceptance)

[Issue #49](https://github.com/wangdiandao/gpui-material/issues/49) records upstream removal criteria and the minimal reproduction.

<a id="上游-slider-复测"></a>

## Upstream Slider retest

On Windows 11 x64 build 26200, GPUI Kit commit `4a874b81` passed 23 tests from `cargo +1.99.1 test --locked -p gpui-base --lib slider::tests`. Run `cargo +1.99.1 run --locked -p gpui-base-examples --bin components -- slider` in the upstream checkout. Find the native UIA Slider and get its `RangeValuePattern`. The initial value was 64, with bounds 0–100 and SmallChange 1. Calling `SetValue` with 73, 0, 100 and 41 returned the requested value each time through `RangeValuePattern.Current.Value`. [GPUI Kit PR #3963](https://github.com/longbridge/gpui-kit/pull/3963)

# P1-07 Overlays, scroll, and windows

English | [中文](07-overlays.zh.md)

Recorded on 2026-11-13 with Windows 11 x64 build 26200 and Rust/Cargo 1.99.1, using the PR code and locked crates.io dependencies.

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin overlays
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin overlays
```

| Scenario | Observation | Status |
| --- | --- | --- |
| Nested Popover and keyboard focus restoration | Opened the outer Popover near the right window edge and then the child. Both panels stayed inside the right edge. The first Esc closed only the child and focused its trigger. The second closed the outer and focused its trigger. | pass |
| Modality, outside click, RTL, value label, nested scroll, resize, and drag ownership | Four rendered event regressions passed. Windows UIA read the modal value label and confirmed focus restoration after Esc. | pass |

<a id="缩放改尺寸与外点关闭"></a>

## Scaling, resize, and outside dismissal

On 2026-11-13 with Windows 11 x64 and Rust 1.99.1, the GPUI test opened the outer Popover with a pointer, resized from 480×320 to 260×200, and set scale to 1.5. The panel stayed inside the left, right, and bottom edges. Pressing outside removed it by the next frame. This passed alongside the nested Esc test.

<a id="模态值标签和所有权"></a>

## Modality, value labels, and ownership

- Base `Dialog` keeps Tab inside the modal and wraps Shift+Tab from first to last. Clicking the scrim over the trigger dismissed the modal without sending a press to the underlying Popover. Both close paths restored focus to the modal trigger.
- Default hit testing in same-axis scrollers lets one input drive both layers. With GPUI `occlude()` on the inner layer, scrolling it −24 logical pixels left the outer at 0. Scrolling −24 on the exposed outer region left the inner at −24. An inner drag retained ownership after moving outside the scrim, with both offsets at 0. Esc cleared the drag and closed the modal.
- LTR and RTL map logical start to Base Popover physical `TopLeft` and `TopRight`. Rendered regressions check inward avoidance at mirrored viewport edges and placement below the trigger.
- In a Windows window at 100% scaling, agent-desktop opened the dialog through UIA Invoke and read “当前值 42” (Current value 42). Clicking at 75% of the Base Slider track changed the same status node to “当前值 75”. After Esc, the trigger was focused. [Native screenshot](07-windows-modal.png)

# P1-02 Activation, focus, and cancellation

English | [中文](02-activation.zh.md)

Recorded on 2026-11-13 with Windows 11 x64 build 26200 and Rust/Cargo 1.99.1, using the PR code and locked crates.io dependencies. The real window used 100% scaling. UIA and screenshots used winapp 0.7.1.

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin activation
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin activation
```

| Scenario | Observation | Status |
| --- | --- | --- |
| Windows headless input | Left press and release inside one control activated once. Releasing outside did not activate it. Three repeats followed by release activated once each for Enter and Space. | pass |
| Windows headless focus | `Window::last_input_was_keyboard()` differed for pointer and keyboard input. Tab reached the next focus target. Both traversal directions skipped disabled items. | pass |
| Windows native view | The [screenshot](02-windows.png) and UIA tree showed three buttons and their initial counts. | pass |
| Roving focus, capture loss, and disabling mid-drag | Rendered event regressions covered roving focus and disabling. agent-desktop started the native Windows drag and Win32 released capture for the window. | pass |

<a id="roving-focus-与拖动取消"></a>

## Roving focus and drag cancellation

Recorded on 2026-11-13 with Windows 11 x64 and Rust 1.99.1. Two radio options shared one Tab stop. Arrow keys changed focus and selection. Tab out followed by Shift+Tab restored focus to the selected option. The GPUI test started three drags and cancelled them with Esc, disabling, and Esc after re-enabling. All three cleared the active drag and GPUI hitbox capture.

<a id="windows-系统捕获丢失"></a>

## Native Windows capture loss

Pinned `gpui-pre-windows 0.3.12` calls `SetCapture` on mouse press and `ReleaseCapture` on release. Its event dispatch lacks `WM_CAPTURECHANGED`. During an active drag, the probe checks `GetCapture` on the window's UI thread. Losing capture or deactivating the window uses the Esc cancellation path. Win32 calls are Windows-only and reuse `windows 0.62.2` from the lockfile. Remove the adapter when upstream supplies an equivalent capture-loss notification to the GPUI interaction layer. [Upstream issue search](https://github.com/zed-industries/zed/issues?q=WM_CAPTURECHANGED) · [Win32 capture changes](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-capturechanged)

```powershell
$env:P1_ACTIVATION_RELEASE_CAPTURE='1'
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin activation
```

In the real window, drag right from “拖动此处” (Drag here) and hold for at least 650 ms. Automation used agent-desktop `drag --duration 220 --drop-delay 650`. The probe called `ReleaseCapture` on the window thread and detected capture loss before release. UIA showed “系统捕获已丢失” (System capture lost). The window kept focus and the pointer and keyboard counts stayed at 0. Two consecutive attempts detected loss. Disabling mid-drag was covered separately by the GPUI test.

<a id="上游补丁复测"></a>

## Upstream patch retest

On Windows 11 x64 build 26200 with Rust 1.98.1, Zed GPUI commit `f4f9837` on `fix/win32-capture-loss` passed `cargo check --locked -p gpui_windows` and all 41 tests from `cargo test --locked -p gpui --lib window::tests::`. The regression covers pending click and drag cancellation and the next click.

For native reproduction, use Zed's `crates/gpui/examples/drag_drop.rs`. In the first drag's `on_drag` handler, schedule Win32 `ReleaseCapture` on the window thread after 350 ms. Check that `GetCapture` goes from nonzero to zero. Register a `MouseCaptureLostEvent` listener with `Window::on_mouse_event` during paint, count notifications in the bubble phase, and count drops and clicks separately. Hold the first drag over the target for 800 ms, drag again without forced release, then click the source.

Foreground input on the preceding revision confirmed release with the left button held, one loss notification and zero drops or clicks. The next drag and click each registered once. On `f4f9837`, posted Win32 mouse messages drove the same sequence in a native GPUI window, and its UI thread called `ReleaseCapture`. The UIA status showed cumulative drag starts, losses, drops and clicks of `1/1/0/0`, `2/1/1/0`, then `2/1/1/1`. Foreground input on the final revision and the full Zed application remain unverified. [Tracking issue #48](https://github.com/wangdiandao/gpui-material/issues/48)

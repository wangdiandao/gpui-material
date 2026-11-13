# P1-03 Input and text measurement

[简体中文](03-text.md) · English

Recorded on 2026-11-13 with Windows 11 x64 build 26200 and Rust/Cargo 1.99.1, using the PR code and locked crates.io dependencies. The real window used 100% scaling. UIA and screenshots used winapp 0.7.1.

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin text
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin text
```

| Scenario | Observation | Status |
| --- | --- | --- |
| Base InputHandler preedit, commit, and cancellation | After a surrogate-pair character, the preedit range was UTF-16 2..7. Committing produced `🙂中`, which remained after cancelling the second preedit. | pass |
| Native Windows password field | Used the fixed sample `P1-sample`. The [screenshot](03-windows.png) showed masking, `winapp ui get-property probe-password -p IsPassword` returned True, and `get-value` returned the accessible name “密码” (Password). | pass |
| Windows Chinese IME | Microsoft Pinyin was installed. Both winapp click and SendInput exited on the foreground focus check. | pending |
| macOS and Linux Chinese IME, candidate positioning, blur, overlays, clipboard, and glyph metrics | Pending on the corresponding sessions and real controls. | pending |

## Additional editing regressions

On 2026-11-13 with Windows 11 x64 and Rust 1.99.1, the same test command verified the following.

- Base input selection, copy, delete, and paste round-tripped `中文🙂` through the probe clipboard. Copying a prefilled password did not put the sample in the clipboard.
- A pointer opened the Popover. Its InputHandler preedited and committed “中”. The handler's candidate anchor rectangle began after the input's start. Esc closed the overlay, restored trigger focus, and retained the input.
- “打开浮层输入…” (Open overlay input) can retest native candidates, commit, cancellation, and positioning after scroll or scaling. Headless clipboard and candidate anchors use the GPUI test platform.

## Windows DirectWrite fallback and metrics

On 2026-11-13 with Windows 11 x64 build 26200, Rust 1.99.1, and agent-desktop 1.2.0, the same probe ran with `P1_TEXT_METRICS=1`. Canvas prepaint laid out `A中文B` using Consolas at 24 px. The same `ShapedLine` drew the text, measured character positions, and placed the line-end marker.

- The Latin run used FontId(1) and the Chinese run fell back to FontId(0). The two Chinese characters began at 13.1953125 px and 37.195313 px. The line advance was 74.390625 px.
- GPUI asserted the end position matched the advance. The [Windows screenshot](03-windows-metrics.png) at 100% scaling showed both scripts and the end marker.
- The input handler regressions and [native Windows confirmation](../../../docs/phases/P01-evidence.en.md#native-windows-acceptance) established a pass for P1-03.

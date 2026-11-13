# P1-08 Two-window theme projection and Base Calendar

[简体中文](08-theme.md) · English

Recorded on 2026-11-13 with Windows 11 x64 build 26200 and Rust/Cargo 1.99.1, using the PR code and locked crates.io dependencies.

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin theme
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin theme
```

| Scenario | Observation | Status |
| --- | --- | --- |
| App theme projection in two windows | Under App Light, one window followed the app and one used local dark. Setting App Dark and refreshing let both rendered states read App Dark while retaining their local policy. The app global kept its set value after each read. | pass |
| Base Calendar leap day | Disabling a range that included 2024-02-29 rejected selection and retained 01-31. Removing the restriction allowed the same leap day. | pass |
| Base source read paths | Released Scrollbar, ResizeHandle, ScrollableMask, input parsing, text selection, TextView, and plot defaults used app-level Theme. Explicit styles carried local overrides. | pass |
| Native windows | Built the native Windows binary. Cross-window pixels and native windows on each platform were pending. | pending |

## Native Windows two-window review

Recorded on 2026-11-13 with Windows 11 x64 build 26200, the GPUI Windows backend, 100% scale, Rust 1.99.1, and winapp 0.7.1. The `theme` binary came from main commit `b050f4f`. winapp invoked “切换应用主题” (Switch app theme) once in the local-dark window. Both windows showed App Dark within 43 ms.

| Window | Before | After |
| --- | --- | --- |
| Follow app | [App Light](08-follow-light.png) | [App Dark](08-follow-dark.png) |
| Local dark | [App Light](08-local-light.png) | [App Dark](08-local-dark.png) |

In the four native screenshots, the global background was sampled at (250, 200) and the local block at (25, 100). Both windows' global RGB changed from (255, 255, 255) to (10, 10, 10). The local-dark block remained (10, 10, 10) before and after. Windows two-window theme projection passed. [P1-08 pure algorithm evidence](../../p1-theme/README.en.md)

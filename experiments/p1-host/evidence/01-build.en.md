# P1-01 Build and package identity

[简体中文](01-build.md) · English

Recorded on 2026-11-13 with Windows 11 x64 build 26200, MSVC, Rust/Cargo 1.99.1, and winapp 0.7.1. The window used the GPUI Windows backend at 100% scaling. Run the following from the repository root at the [P1 acceptance commit](https://github.com/wangdiandao/gpui-material/tree/b0250f5f120ab2fbadfe41b5529612e984358295).

```sh
python experiments/p1-host/check_graph.py +1.99.1
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin window
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin window
```

| Scenario | Expected and observed | Status |
| --- | --- | --- |
| Windows, macOS ARM64, and Linux x64 dependency graphs | Normal and test-support resolution for each target contained one crates.io Base 0.9.1, GPUI 0.3.12, and corresponding platform package 0.3.12. The build graph excluded GPUI Fast and styled crates. macOS had font-kit and runtime_shaders. Linux had X11 and Wayland. | pass |
| Windows headless interaction test window | One test passed. After one frame, the status node in Base Root matched the role, label, visibility, and width assertions. | pass |
| Windows native window | Logged `P1-01 native window: windows x86_64, scale=1`. UIA showed `Window "P1-01 窗口探针"` and `StatusBar "窗口探针 · windows · 100%"`. [Native screenshot](01-windows.png) | pass |
| macOS ARM64 native and test windows | Pending on an Apple Silicon desktop with Xcode and Metal. | pending |
| Linux native and test windows | Pending on Ubuntu 24.04 GNOME with Wayland and X11 sessions. | pending |

`check_graph.py` read `cargo metadata --filter-platform` for each target and both feature sets, then checked normal and build dependency trees. Test-support metadata lists optional gpui-fast from weak feature references. `cargo tree --edges normal,build` and the Windows build confirm it was disabled.

The current workspace uses `cargo xtask graph` for dependency regressions. Gallery's UI integration test covers Base Root rendering. The P1 window remains available for native probes. [Development and verification](../../../docs/development.en.md)

An initial launch with hidden window style opened GPUI but UIA could not enumerate a visible HWND. Launching with `cargo run` from a normal terminal exposed the system tree and allowed capture with `winapp ui screenshot -a <probe-pid>`.

These runs used the same PR commit and lockfile. [P1-01](https://github.com/wangdiandao/gpui-material/issues/10) required the three-platform results and pure algorithm dependency graph before closure.

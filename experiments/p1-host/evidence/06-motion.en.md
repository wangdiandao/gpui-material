# P1-06 Motion and system preferences

[简体中文](06-motion.md) · English

Recorded on 2026-11-13 with Windows 11 x64 build 26200 and Rust/Cargo 1.99.1, using the PR code and locked crates.io dependencies. The real window used 100% scaling. UIA and screenshots used winapp 0.7.1.

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin motion -- --nocapture
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin motion
```

| Scenario | Observation | Status |
| --- | --- | --- |
| Interruption and reentry | Width spring and opacity transition were continuous at retarget. At 60 ms the values were (145.88846, 0.8074074). After reversing for 16 ms they were (159.889, 0.6550468). Geometry retained velocity and opacity reversed immediately. Reentry started from the current values. | pass |
| Settling and reduced motion | Target was (280, 1). After settling, running the old frame callback left zero pending frames. Injecting `App::set_reduce_motion(true)` restored (80, 0.35) in the same frame. | pass |
| Initial Windows native preference | The real window logged `system reduce_motion=false`. [Initial screenshot](06-windows.png) | pass |
| System toggle and platform window baselines | Pending native evidence for the OS setting on and off, macOS and Linux preference reads, and frame performance. | pending |

## Windows system toggle and frame samples

Recorded on 2026-11-13 with Windows 11 x64 build 26200, Rust 1.99.1, and agent-desktop 1.2.0. UIA TogglePattern switched Animation effects in Settings under Accessibility → Visual effects, restarting the probe each time. InvokePattern expanded once. The original setting was restored and read back as on.

```powershell
$env:P1_MOTION_TRACE = '1'
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin motion
```

| System animation effects | App reading | First frame and settling |
| --- | --- | --- |
| On | `reduce_motion=false` | First frame 80 px / 0.350. Reached 280 px / 1.000 at 512.24 ms. 146 view renders, mean interval 3.52 ms. |
| Off | `reduce_motion=true` | First frame at 3.37 ms already reached 280 px / 1.000. One sample. |

These debug-build view render samples measured callback intervals and CPU time to construct the element tree. GPU submission and display presentation require separate measurement. The headless test also asserts zero pending animation frame callbacks after settling.

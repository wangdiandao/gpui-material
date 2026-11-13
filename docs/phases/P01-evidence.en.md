# P1 verification record

[简体中文](P01-evidence.md) · English

## Hosted verification on 2026-11-13

Both jobs passed in [GitHub Actions run 43374663131][run]. It ran commit `11d9dbb6c26fb8af88b84c33bea700f41afac20f`, using probe code and lockfiles from `e6e0223c6fad4dff778a85919af475a64881a93c`. The one-off workflow is in the verification branch's history. Earlier run 43374606947 was superseded and cancelled after integration of the P1-08 two-window probe.

| Environment | Command and observation | Result |
| --- | --- | --- |
| macOS 15.8.2 (24H232), arm64, Xcode 16.4 | Pinned Rust 1.99.1. Nine host tests, three pure theme tests, and three pure geometry tests passed. | pass |
| Ubuntu 24.04.5 LTS, x86_64, Azure kernel 7.0.0-1017 | Pinned Rust 1.99.1. The same 15 tests passed. | pass |
| Ubuntu Xvfb, X11 | Ran `window`. The log showed `P1-01 native window: linux x86_64, scale=1` and the process remained alive until the 15-second timeout. | pass |
| macOS desktop and Linux GNOME Wayland/X11 desktops | The hosted run covered builds, tests, and virtual X11. Future support evaluations can gather real desktop evidence. | exploratory |

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support
cargo +1.99.1 test --locked --manifest-path experiments/p1-theme/Cargo.toml
cargo +1.99.1 test --locked --manifest-path experiments/p1-shapes/Cargo.toml
cargo +1.99.1 build --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin window
xvfb-run -a timeout 15s experiments/p1-host/target/debug/window
```

See each job's log for the initial host test compile time on Ubuntu and macOS. [P1-01](../../experiments/p1-host/evidence/01-build.en.md) records Windows native and three-target resolution evidence. [P1 plan](P01.en.md) · [Platforms and quality requirements](../verification.en.md)

[run]: https://github.com/wangdiandao/gpui-material/actions/runs/43374663131

## Native Windows acceptance

On 2026-11-13, the project owner confirmed that Windows acceptance passed. The confirmation included no per-scenario logs, settings, or screenshots. Reproducible evidence is in the individual probe records. Windows 11 x64 build 26200 screenshots and UIA observations are available for [the window](../../experiments/p1-host/evidence/01-build.en.md), [input](../../experiments/p1-host/evidence/03-text.en.md), and [semantics](../../experiments/p1-host/evidence/04-semantics.en.md).

## macOS and Ubuntu exploratory steps

For future support evaluation, record `git rev-parse HEAD` on `main`, OS version, CPU architecture, display scaling, IME, and screen reader and its version. On Linux also record `echo "$XDG_SESSION_TYPE"` and run once on GNOME Wayland and once on GNOME on Xorg. On macOS, use Apple Silicon and a signed app bundle and record `codesign --verify --strict`. Build and pre-run the same probes with the following commands.

```sh
rustup toolchain install 1.99.1 --profile minimal
cargo +1.99.1 build --locked --manifest-path experiments/p1-host/Cargo.toml --bins
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin text
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin semantics
```

Ubuntu 24.04 can build with these dependencies.

```sh
sudo apt-get install clang libfontconfig-dev libwayland-dev libxkbcommon-x11-dev libx11-xcb-dev libssl-dev libzstd-dev libasound2-dev libvulkan1 mesa-vulkan-drivers
```

1. **IME.** Select Simplified Pinyin on macOS or the IBus Chinese Pinyin input source in GNOME. Click “中文输入” (Chinese input) and type `zhong` with a physical keyboard. When candidates appear, record the candidate box relative to the insertion point. Choose “中” to commit it. Type `wen`, press Esc, and record whether the field still contains only “中”. Start a new preedit and switch to the password field, then record the text after blur. Retest after changing scaling or moving to another display. Use test text for passwords.
2. **Screen reader.** Start VoiceOver in macOS System Settings under Accessibility, or Orca with Super+Alt+S in GNOME. Navigate to “执行操作” (Perform action) in the semantics probe and record the announced name and role. Activate it with VO+Space in VoiceOver, or Tab to it and press Enter or Space in Orca. In Orca also inspect supported actions in AT-SPI. Activate once, record the change from 0 to 1, and note whether “已执行 1 次” (Executed once) is announced automatically. Toggle “选项” (Option) and verify its state. Check the name, range, and value of “级别” (Level). In the text probe, confirm the password field conceals the test plaintext from the screen reader.
3. **Native tree.** Use macOS Accessibility Inspector or a Linux AT-SPI inspector to record parent groups, focus, buttons, checkboxes, ranges, status bars, edit fields, and password roles. Record inaccessible properties and actual announcements. Screenshots or recordings are optional.
4. **Desktop.** Run `window`, capture a real window, and record scaling and backend. Run `motion` with system reduced motion off and on, restarting the probe to record the preference and expansion behavior. Use `activation` to check pointer, Enter, Space, Tab, Shift+Tab, and cancellation. Use `overlays` to check sequential Esc and focus restoration.
5. **Report.** Record platform, session, commit, scenario, expected and observed results, and screenshot or log locations. Keep successes and failures and mark unrun items `pending`.

[Apple VoiceOver commands](https://support.apple.com/guide/voiceover/control-your-mac-with-keyboard-commands-vo2681/mac) · [GNOME Orca](https://help.gnome.org/orca/introduction.html)

## Initial-release Windows probe review

The initial platform contract was revised by [P0 repair #35](https://github.com/wangdiandao/gpui-material/pull/35). macOS and Linux [headless pixel evidence](../../experiments/p1-host/evidence/05-shapes.en.md#headless-metal-and-wgpu-pixels) and hosted results are exploratory.

Integration commit `a3b282f8d5067253356a345ffc8834b734e260ab` was rechecked on 2026-11-13 with Windows 11 x64 build 26200/MSVC and Rust/Cargo 1.99.1. All 23 tests passed, including 17 host, three pure theme, and three pure geometry tests. Formatting passed for all three manifests, as did the locked Windows binary build, all-targets host Clippy, and package identity checks for three targets. Native Windows acceptance uses the owner's earlier confirmation. Individual Pinyin and Narrator recordings and logs were not collected. Supplemental native evidence comes from real Windows windows, system settings, and UIA records.

| Probe | Acceptance evidence | Result |
| --- | --- | --- |
| P1-01 | [Locked build, native Windows window, and package identity](../../experiments/p1-host/evidence/01-build.en.md) · [PR #21](https://github.com/wangdiandao/gpui-material/pull/21) | pass |
| P1-02 | [Pointer, keyboard, roving focus, native capture loss, and disabled cancellation](../../experiments/p1-host/evidence/02-activation.en.md) · [PR #46](https://github.com/wangdiandao/gpui-material/pull/46) | pass |
| P1-03 | [UTF-16, clipboard, masking, overlay input, anchoring, and DirectWrite fallback](../../experiments/p1-host/evidence/03-text.en.md) · [PR #43](https://github.com/wangdiandao/gpui-material/pull/43) | pass |
| P1-04 | [UIA tree, single Invoke, Toggle, RangeValue, editing, and overlay relationships](../../experiments/p1-host/evidence/04-semantics.en.md) · [PR #44](https://github.com/wangdiandao/gpui-material/pull/44) · [PR #50](https://github.com/wangdiandao/gpui-material/pull/50) | pass |
| P1-05 | [Pure geometry, fill, clipping, shadow, hit testing, and ripple](../../experiments/p1-host/evidence/05-shapes.en.md) · [PR #45](https://github.com/wangdiandao/gpui-material/pull/45) | pass |
| P1-06 | [Retargeting, settling, Windows reduced motion, and rendering baseline](../../experiments/p1-host/evidence/06-motion.en.md) · [PR #42](https://github.com/wangdiandao/gpui-material/pull/42) | pass |
| P1-07 | [Nesting, focus restoration, modality, RTL, value labels, resize, and scroll/drag ownership](../../experiments/p1-host/evidence/07-overlays.en.md) · [PR #47](https://github.com/wangdiandao/gpui-material/pull/47) | pass |
| P1-08 | [Windows two-window theme projection and Calendar](../../experiments/p1-host/evidence/08-theme.en.md) · [Pure algorithms and dependency graphs](../../experiments/p1-theme/README.en.md) · [PR #38](https://github.com/wangdiandao/gpui-material/pull/38) | pass |
| P1-09 | [Architecture and dependencies](../architecture.en.md#boundaries-verified-in-p1) · [Phase status](../../status.toml) | pass |

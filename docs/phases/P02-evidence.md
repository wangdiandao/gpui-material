# P2 acceptance record

English | [中文](P02-evidence.zh.md)

Accepted on 2026-11-13. The initial engineering acceptance platform was Windows 11 x64 MSVC.

<a id="windows-实窗"></a>

## Native Windows window

Windows 11 Home build 26200, x86_64, with the GPUI Windows DirectX backend. Hardware was Ryzen 9 8945HX, 32 GiB RAM, and RTX 5070 Laptop GPU with driver 32.0.16.2114. Tools were Rust/Cargo 1.99.1, MSVC 14.51.36231, Windows SDK 10.0.26100.0, and WinApp CLI 0.7.1.

| Path | Observation | Result |
| --- | --- | --- |
| `cargo build --locked -p gpui-material-gallery`, then launch the dev build | Title bar read “Material Gallery”. Body showed “Window check” and “Display scale 100%”. Windows UI Automation reported a StatusBar with that name. | pass |
| Drag lower-right corner to minimum size | UIA bounds shrank from 536×339 to 336×239. The status text remained intact. | pass |
| Alt+F4, then title-bar Close via UIA InvokePattern on another launch | Both closed the real window and the process exited with code 0. | pass |
| `cargo test --locked --workspace --all-features` | One Gallery UI integration test and four xtask unit tests passed. | pass |
| Isolated P1 manifests, `cargo test --locked --manifest-path <manifest>` | Host with `--features test-support` passed 17 tests. Theme and shapes passed three each. | pass |

See [development and verification](../development.md) for entry points and commands. The native run used the [Workspace and Gallery merge](https://github.com/wangdiandao/gpui-material/commit/802fb8b5afacffaf562972a6d827d92079c131ed).

![Windows Gallery at the default size](p02/gallery-windows.png)

![Windows Gallery at the minimum size](p02/gallery-windows-narrow.png)

<a id="windows-构建基线"></a>

## Windows build baseline

Prefetched locked dependencies at the clean [xtask merge](https://github.com/wangdiandao/gpui-material/commit/2d4b1dcff4aa11885f3d356771d748b7394a830a) and ran `cargo xtask timings target/timings/p2-baseline`. Used normal Gallery features and the development profile. Cargo selected 32 jobs. Each cold run used an empty target directory and its warm run immediately reused it.

| Operation | Cold | Warm |
| --- | ---: | ---: |
| `cargo check` | 47.723 s | 0.585 s |
| `cargo build` | 62.664 s | 0.611 s |

Cold check and build each processed 418 compilation units. The Gallery binary unit took 0.2 s for check and 1.9 s for build, including code generation and linking. On a warm cache, `cargo rustc` directed the Gallery binary through a timing wrapper. MSVC `link.exe` took 1.257 s and Cargo took 2.22 s total. The timing command reproduces the report.

<a id="github-actions-与门禁"></a>

## GitHub Actions and merge gate

The [first hosted run](https://github.com/wangdiandao/gpui-material/actions/runs/43388732324) passed on Windows Server 2025 build 26100 x64 MSVC. Its development artifact hit a DirectX path error when launched on another machine. The pinned GPUI backend reads HLSL from the build machine's Cargo registry when debug assertions are enabled. The [artifact fix](https://github.com/wangdiandao/gpui-material/pull/63) embeds shaders at build time with Windows SDK x64 `fxc.exe`.

The [fixed Windows MSVC run](https://github.com/wangdiandao/gpui-material/actions/runs/43389985632) passed and uploaded [gallery-windows-x64-msvc](https://github.com/wangdiandao/gpui-material/actions/runs/43389985632/artifacts/14020542543). The artifact ran on Windows 11. UIA read “Display scale 100%”, and title-bar Close returned exit code 0. The executable's SHA-256 was `9ffb7016b00df18930bbc2c893700d9bd39e41d0238630e55ae8b35985673ff5`.

The [Main Windows CI ruleset](https://github.com/wangdiandao/gpui-material/rules/27184646) applied to `refs/heads/main` with no bypass roles. It required PRs, resolved review conversations, and the up-to-date GitHub Actions `Windows MSVC` check. It blocked force pushes and branch deletion. With one collaborator, the phase used itemized self-review and zero required approvals.

A [negative gate test PR](https://github.com/wangdiandao/gpui-material/pull/62) deliberately added an architecture document outside the frozen scope. The [Windows MSVC job](https://github.com/wangdiandao/gpui-material/actions/runs/43389316575) failed phase path validation and GitHub marked the PR `BLOCKED`. The PR was closed after the test.

At P2, `cargo xtask check` verified package identities and platform features on Windows, macOS, and Linux for normal and `test-support` feature sets. macOS and Linux coverage was dependency resolution. Cross-DPI, Chinese IME, and screen-reader acceptance are performed in the relevant component phase. [Platform requirements](../verification.md)

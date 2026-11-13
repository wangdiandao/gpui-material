# P1 验证记录

## 2026-11-13 托管验证

[GitHub Actions 运行 43374663131][run] 两项作业均成功。运行提交为 `11d9dbb6c26fb8af88b84c33bea700f41afac20f`，其中探针代码和锁文件来自 `e6e0223c6fad4dff778a85919af475a64881a93c`。一次性工作流保存在验证分支的该提交中。较早的运行 43374606947 在合入 P1-08 两窗口探针后被替代并取消。

| 环境 | 命令和观察 | 结果 |
| --- | --- | --- |
| macOS 15.8.2 (24H232)，arm64，Xcode 16.4 | 固定 Rust 1.99.1，宿主 9 个、纯主题 3 个、纯几何 3 个测试通过。 | pass |
| Ubuntu 24.04.5 LTS，x86_64，Azure 内核 7.0.0-1017 | 固定 Rust 1.99.1，同样 15 个测试通过。 | pass |
| Ubuntu Xvfb，X11 | 运行 `window`，日志显示 `P1-01 native window: linux x86_64, scale=1`，进程在 15 秒超时结束前持续运行。 | pass |
| macOS 桌面、Linux GNOME Wayland/X11 桌面 | 托管运行覆盖构建、测试与虚拟 X11。真实桌面证据可在未来的支持评估中采集。 | 探索 |

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support
cargo +1.99.1 test --locked --manifest-path experiments/p1-theme/Cargo.toml
cargo +1.99.1 test --locked --manifest-path experiments/p1-shapes/Cargo.toml
cargo +1.99.1 build --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin window
xvfb-run -a timeout 15s experiments/p1-host/target/debug/window
```

Ubuntu 和 macOS 的宿主首次测试编译耗时见各作业日志。此处的 Windows 原生与三目标解析证据见 [P1-01](../../experiments/p1-host/evidence/01-build.md)。冻结条件见 [P1 计划](P01.md) 和 [平台与质量要求](../verification.md)。

[run]: https://github.com/wangdiandao/gpui-material/actions/runs/43374663131

## Windows 实机验收

2026-11-13，项目负责人在会话中确认“Windows验收通过”，登记为 Windows 实机验收通过。确认未附逐项日志、系统设置或截图；相关可复现证据以各探针记录为准。Windows 11 x64 build 26200 上的已存档截图及 UIA 检查见 [窗口](../../experiments/p1-host/evidence/01-build.md)、[输入](../../experiments/p1-host/evidence/03-text.md) 和 [语义](../../experiments/p1-host/evidence/04-semantics.md)。

## macOS 和 Ubuntu 探索步骤

如后续评估平台支持，在 `main` 上记录 `git rev-parse HEAD`、OS 版本、CPU 架构、屏幕缩放、输入法、读屏工具与版本。Linux 另记 `echo "$XDG_SESSION_TYPE"`，在 GNOME Wayland 和 GNOME on Xorg 各做一次。macOS 使用 Apple Silicon 与已签名的应用包，记录 `codesign --verify --strict` 的结果。以下命令可用于构建并预跑相同探针。

```sh
rustup toolchain install 1.99.1 --profile minimal
cargo +1.99.1 build --locked --manifest-path experiments/p1-host/Cargo.toml --bins
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin text
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin semantics
```

Ubuntu 24.04 可用以下依赖构建。

```sh
sudo apt-get install clang libfontconfig-dev libwayland-dev libxkbcommon-x11-dev libx11-xcb-dev libssl-dev libzstd-dev libasound2-dev libvulkan1 mesa-vulkan-drivers
```

1. **输入法。** 在 macOS 选择“简体拼音”，在 GNOME 选择 IBus 的中文拼音输入源。点击「中文输入」，用物理键盘键入 `zhong`，候选出现时记录候选框相对插入点的位置。选「中」确认；输入 `wen` 按 Esc，记录取消后是否仍只有「中」。再次预编辑并切到密码框，记录失焦后的文本。切换显示缩放或将窗口移到另一缩放的显示器后复测。只用测试文本，不要输入真实密码。
2. **读屏。** 启动 VoiceOver（macOS 系统设置 → 辅助功能 → VoiceOver）或 Orca（GNOME 可用 Super+Alt+S）。在语义探针导航到「执行操作」，记录读到的名称和角色。VoiceOver 用 VO+空格激活；Orca 用 Tab 导航到控件后按 Enter 或 Space，另在 AT-SPI 树核对支持的动作。各激活一次，记录 0→1 和是否自动播报「已执行 1 次」。再切换「选项」并检查状态，检查「级别」的名称、范围和值，另在文本探针确认密码字段读不到测试明文。
3. **系统树。** 在 macOS Accessibility Inspector 或 Linux AT-SPI 检查器记录父组、焦点、按钮、复选框、范围、状态条、编辑框与密码角色。记录无法读取的属性以及实际播报，截图或录音可选。
4. **桌面。** 运行 `window` 截取真实窗口，记录缩放与后端；运行 `motion`，分别在系统低动效开与关时重启探针，记录窗口上的偏好及展开行为。运行 `activation` 检查指针、Enter、Space、Tab、Shift+Tab 与取消；运行 `overlays` 检查逐层 Esc 与回焦。
5. **回报。** 每次提供平台、会话、提交、场景、期望、实际结果和截图或日志位置。成功与失败都保留，未执行项标为 pending。

[Apple VoiceOver 操作](https://support.apple.com/guide/voiceover/control-your-mac-with-keyboard-commands-vo2681/mac) · [GNOME Orca](https://help.gnome.org/orca/introduction.html)

## 首版 Windows 探针复核

首版平台契约经 [P0 返修 #35](https://github.com/wangdiandao/gpui-material/pull/35) 更新。macOS 和 Linux 的[无头像素证据](../../experiments/p1-host/evidence/05-shapes.md#metal-与-wgpu-无头像素)及托管结果留作探索记录。

2026-11-13 在 Windows 11 x64 build 26200 / MSVC、Rust/Cargo 1.99.1 复核集成提交 `a3b282f8d5067253356a345ffc8834b734e260ab`。宿主 17 项、纯主题 3 项、纯几何 3 项共 23 项通过。三份清单的格式检查、锁定的 Windows 原生二进制构建、宿主全目标 Clippy 和三目标依赖身份检查通过。Windows 实机签收采用项目负责人此前确认。拼音与 Narrator 的逐项录音和日志未采集。当前补充的原生证据来自下表中的 Windows 实窗、系统设置与 UIA 记录。

| 探针 | 验收证据 | 结果 |
| --- | --- | --- |
| P1-01 | [锁定构建、Windows 实窗和包身份](../../experiments/p1-host/evidence/01-build.md) · [PR #21](https://github.com/wangdiandao/gpui-material/pull/21) | pass |
| P1-02 | [指针、键盘、roving focus、原生捕获丢失与禁用取消](../../experiments/p1-host/evidence/02-activation.md) · [PR #46](https://github.com/wangdiandao/gpui-material/pull/46) | pass |
| P1-03 | [UTF-16、剪贴板、掩码、浮层输入、锚点和 DirectWrite 回退](../../experiments/p1-host/evidence/03-text.md) · [PR #43](https://github.com/wangdiandao/gpui-material/pull/43) | pass |
| P1-04 | [UIA 树、单次 Invoke、Toggle、RangeValue、编辑与浮层关系](../../experiments/p1-host/evidence/04-semantics.md) · [PR #44](https://github.com/wangdiandao/gpui-material/pull/44) · [PR #50](https://github.com/wangdiandao/gpui-material/pull/50) | pass |
| P1-05 | [纯几何、填充、裁剪、阴影、网格命中与 ripple](../../experiments/p1-host/evidence/05-shapes.md) · [PR #45](https://github.com/wangdiandao/gpui-material/pull/45) | pass |
| P1-06 | [重定向、收敛、Windows 系统低动效和 render 基线](../../experiments/p1-host/evidence/06-motion.md) · [PR #42](https://github.com/wangdiandao/gpui-material/pull/42) | pass |
| P1-07 | [嵌套、回焦、模态、RTL、值标签、窗口变化和滚动拖动所有权](../../experiments/p1-host/evidence/07-overlays.md) · [PR #47](https://github.com/wangdiandao/gpui-material/pull/47) | pass |
| P1-08 | [Windows 双窗口主题投影和 Calendar](../../experiments/p1-host/evidence/08-theme.md) · [纯算法与依赖图](../../experiments/p1-theme/README.md) · [PR #38](https://github.com/wangdiandao/gpui-material/pull/38) | pass |
| P1-09 | [架构与依赖](../architecture.md#p1-已验证的适配边界) · [阶段登记](../../status.toml) | pass |

# P1-01 构建与包身份

[English](01-build.md) | 中文

2026-11-13，Windows 11 x64 build 26200，MSVC，Rust/Cargo 1.99.1，winapp 0.7.1。窗口使用 GPUI Windows 后端，缩放 100%。下列命令在 [P1 接受提交](https://github.com/wangdiandao/gpui-material/tree/b0250f5f120ab2fbadfe41b5529612e984358295)的仓库根目录执行。

```sh
python experiments/p1-host/check_graph.py +1.99.1
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin window
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin window
```

| 场景 | 期望和观察 | 状态 |
| --- | --- | --- |
| Windows、macOS ARM64、Linux x64 的目标依赖图 | 每目标的普通和 test-support 解析均只包含一个 crates.io Base 0.9.1、GPUI 0.3.12 和对应平台包 0.3.12；编译依赖树无 GPUI Fast 或 styled crate。macOS 后端有 font-kit、runtime_shaders；Linux 后端有 X11、Wayland。 | pass |
| Windows 无头交互测试窗口 | 1 个测试通过。完成一帧后，Base Root 中状态节点角色、标签、可见性和宽度符合断言。 | pass |
| Windows 原生窗口 | 日志为 `P1-01 native window: windows x86_64, scale=1`。winapp UIA 树含 `Window "P1-01 窗口探针"` 和 `StatusBar "窗口探针 · windows · 100%"`。[原生截图](01-windows.png) 记录标题栏与内容。 | pass |
| macOS ARM64 原生窗口与测试窗口 | 待在带 Xcode/Metal 的 Apple Silicon 桌面验证。 | pending |
| Linux 原生窗口与测试窗口 | 待在 Ubuntu 24.04 GNOME 的 Wayland 和 X11 会话验证。 | pending |

`check_graph.py` 对每个目标和两种 feature 模式读取 `cargo metadata --filter-platform`，再检查普通与构建依赖树。带 test-support 的 metadata 会列出弱 feature 引用的可选 gpui-fast 包，`cargo tree --edges normal,build` 和 Windows 编译均未启用该包。

当前 Workspace 的依赖回归使用 `cargo xtask graph`。Base Root 渲染回归由 Gallery 的 UI 集成测试覆盖，P1 窗口入口仍供原生平台探查。当前命令见[开发与验证](../../../docs/development.zh.md)。

第一次用隐藏窗口样式启动时进程成功打开 GPUI 窗口，UIA 枚举不到可见 HWND。以 `cargo run` 从普通终端启动后捕获截图和系统树。截图使用 `winapp ui screenshot -a <探针 PID>`。

这些运行以同一 PR 的代码提交及锁文件为输入。三平台运行结果及纯算法依赖图通过后才结束 [P1-01](https://github.com/wangdiandao/gpui-material/issues/10)。

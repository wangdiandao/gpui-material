# P1 验证记录

## 2026-11-13 托管验证

[GitHub Actions 运行 43374663131][run] 两项作业均成功。运行提交为 `11d9dbb6c26fb8af88b84c33bea700f41afac20f`，其中探针代码和锁文件来自 `e6e0223c6fad4dff778a85919af475a64881a93c`。一次性工作流保存在验证分支的该提交中。较早的运行 43374606947 在合入 P1-08 两窗口探针后被替代并取消。

| 环境 | 命令和观察 | 结果 |
| --- | --- | --- |
| macOS 15.8.2 (24H232)，arm64，Xcode 16.4 | 固定 Rust 1.99.1，宿主 9 个、纯主题 3 个、纯几何 3 个测试通过。 | pass |
| Ubuntu 24.04.5 LTS，x86_64，Azure 内核 7.0.0-1017 | 固定 Rust 1.99.1，同样 15 个测试通过。 | pass |
| Ubuntu Xvfb，X11 | 运行 `window`，日志显示 `P1-01 native window: linux x86_64, scale=1`，进程在 15 秒超时结束前持续运行。 | pass |
| macOS 桌面、Linux GNOME Wayland/X11 桌面 | 签名应用包、真实显示、Metal/WGPU 无头像素、IME、系统树、读屏、输入设备及系统偏好需补证。 | pending |

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support
cargo +1.99.1 test --locked --manifest-path experiments/p1-theme/Cargo.toml
cargo +1.99.1 test --locked --manifest-path experiments/p1-shapes/Cargo.toml
cargo +1.99.1 build --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin window
xvfb-run -a timeout 15s experiments/p1-host/target/debug/window
```

Ubuntu 和 macOS 的宿主首次测试编译耗时见各作业日志。此处的 Windows 原生与三目标解析证据见 [P1-01](../../experiments/p1-host/evidence/01-build.md)。冻结条件见 [P1 计划](P01.md) 和 [平台与质量要求](../verification.md)。

[run]: https://github.com/wangdiandao/gpui-material/actions/runs/43374663131

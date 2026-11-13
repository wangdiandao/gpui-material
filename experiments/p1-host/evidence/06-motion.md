# P1-06 动效与系统偏好

简体中文 · [English](06-motion.en.md)

2026-11-13，Windows 11 x64 build 26200，Rust/Cargo 1.99.1，使用此 PR 的代码与已锁定的 crates.io 依赖。实窗缩放 100%，UIA 和截图工具为 winapp 0.7.1。

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin motion -- --nocapture
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin motion
```

| 场景 | 观察 | 状态 |
| --- | --- | --- |
| 中断和重入 | 宽度 spring 与透明度 transition 在重定向瞬间连续。60 ms 时 (145.88846, 0.8074074)，反转 16 ms 后为 (159.889, 0.6550468)。几何保留速度，透明度立即沿反方向变化。重新进入同样从当前值出发。 | pass |
| 收敛和低动效 | 目标为 (280, 1)。收敛后执行旧帧回调，随后 pending frame 为 0。注入 `App::set_reduce_motion(true)` 后同帧回到 (80, 0.35)。 | pass |
| Windows 原生初始偏好 | 真实窗口启动输出 `system reduce_motion=false`，[截图](06-windows.png)显示初始偏好。 | pass |
| 系统切换及各平台窗口基线 | 待取得 OS 开启与关闭的各一次真实证据，以及 macOS 和 Linux 偏好回读、帧性能数据。 | pending |

## Windows 系统开关与帧采样

2026-11-13，Windows 11 x64 build 26200，Rust 1.99.1，agent-desktop 1.2.0。用系统设置「辅助功能 → 视觉效果 → 动画效果」的 UIA TogglePattern 切换，每次重启探针。用 InvokePattern 触发一次展开。测试后读回原设置为开启。

```powershell
$env:P1_MOTION_TRACE = '1'
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin motion
```

| 系统动画效果 | App 读数 | 首帧与收敛 |
| --- | --- | --- |
| 开启 | `reduce_motion=false` | 首帧 80 px / 0.350。512.24 ms 到达 280 px / 1.000。146 次视图 render，平均间隔 3.52 ms。 |
| 关闭 | `reduce_motion=true` | 触发后 3.37 ms 进入第一帧，已到达 280 px / 1.000。只有一次采样。 |

这些数值来自调试构建中的视图 render 入口，记录回调之间的间隔以及构建元素树的 CPU 耗时。GPU 提交与屏幕呈现耗时属于独立量测。现有无头测试另外断言收敛后待执行动画帧回调为 0。

# P1-06 动效与系统偏好

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

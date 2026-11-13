# P1-07 浮层、滚动与窗口

简体中文 · [English](07-overlays.en.md)

2026-11-13，Windows 11 x64 build 26200，Rust/Cargo 1.99.1，使用此 PR 的代码与已锁定的 crates.io 依赖。

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin overlays
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin overlays
```

| 场景 | 观察 | 状态 |
| --- | --- | --- |
| 嵌套 Popover 和键盘回焦 | 窗口右侧触发外层，打开子层。两个面板均在视口右界之内。Esc 只关闭子层并回到子触发器，第二次 Esc 关闭外层并回到外触发器。 | pass |
| 模态、外点、RTL、值标签、嵌套滚动、改尺寸和拖动所有权 | 四项渲染事件回归通过，Windows UIA 读回模态值标签与 Esc 回焦。 | pass |

## 缩放、改尺寸与外点关闭

2026-11-13，Windows 11 x64，Rust 1.99.1，GPUI 测试窗口。指针打开外层 Popover 后将视口从 480×320 缩到 260×200，缩放设为 1.5。面板仍位于视口左、右和下边界内；在面板外按下鼠标后，下一帧的面板已移除。与嵌套 Esc 测试一同通过。

## 模态、值标签和所有权

- Base `Dialog` 的焦点范围将 Tab 留在模态内，Shift+Tab 从首项绕回末项。遮罩覆盖触发按钮时点击可关闭模态，后层 Popover 没有收到按下事件。两条关闭路径都把焦点还给模态触发器。
- 同轴滚动容器的默认命中会让同一输入驱动两层。内层加 GPUI `occlude()` 后，在内层滚动 −24 逻辑像素，外层保持 0。在外层裸露区滚动 −24，内层保持 −24。内层拖动移到遮罩外后仍持有活动拖动，两层偏移均保持 0。Esc 清除活动拖动并关闭模态。
- LTR 和 RTL 分别将逻辑起点映射到 Base Popover 的物理 `TopLeft` 和 `TopRight`。渲染回归在镜像的视口边缘检查面板向内避让且位于触发器下方。
- Windows 100% 缩放实窗通过 agent-desktop 调用 UIA Invoke 打开对话框，读回「当前值 42」。点击 Base Slider 轨道 75% 位置后，同一对话框的状态节点读回「当前值 75」，Esc 后触发器再次带有 focused 状态。[实窗截图](07-windows-modal.png)。

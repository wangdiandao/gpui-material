# P1-05 形状、绘制与 ripple

简体中文 · [English](05-shapes.en.md)

2026-11-13，Windows 11 x64 build 26200，Rust/Cargo 1.99.1，使用此 PR 的代码与已锁定的 crates.io 依赖。实窗缩放 100%，UIA 和截图工具为 winapp 0.7.1。

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --bin shapes
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin shapes
cargo +1.99.1 test --locked --manifest-path experiments/p1-shapes/Cargo.toml -- --nocapture
```

| 场景 | 观察 | 状态 |
| --- | --- | --- |
| 圆角参数与空间分配 | 固定 AndroidX 参考中三角形底边的两个 anchor 都为 (0.5, 0)。polymorpher 实测误差 0。 | pass |
| 同形态 morph | 三角形在 0、0.5、1 的匹配 cubic 锚点与控制点最大误差 0。 | pass |
| 不同拓扑与圆角 morph | 三角形到正方形、正方形到圆、方形到圆角方形。每个 cubic 等步长采样 17 点，全部有限并位于半径 1 的测试边界容差内，闭合最大间隙 `5.1273855e-7`，门槛为 `1e-5` 个半径。 | pass |
| GPUI 原生填充 | `PathBuilder::cubic_bezier_to` 接收同一组曲线，三个进度均可 tessellate。[Windows 实窗](05-windows.png)显示三个填充样本。 | pass |
| 裁剪、阴影、命中、ripple 起点和高 DPI | 裁剪与命中复用填充网格，1.5× 测试通过。Windows 实窗状态层改变 3,091 个像素，未改动采样区域内的纯白背景。 | pass |

固定参考为 [CornerRoundingTest](https://android.googlesource.com/platform/frameworks/support/+/cb163bee53214245ad7a23367396a21b7f85ae82/graphics/graphics-shapes/src/commonTest/kotlin/androidx/graphics/shapes/CornerRoundingTest.kt)、[RoundedPolygonTest](https://android.googlesource.com/platform/frameworks/support/+/cb163bee53214245ad7a23367396a21b7f85ae82/graphics/graphics-shapes/src/commonTest/kotlin/androidx/graphics/shapes/RoundedPolygonTest.kt) 和 [MorphTest](https://android.googlesource.com/platform/frameworks/support/+/cb163bee53214245ad7a23367396a21b7f85ae82/graphics/graphics-shapes/src/commonTest/kotlin/androidx/graphics/shapes/MorphTest.kt)。

## Metal 与 WGPU 无头像素

2026-11-13，[修正后的托管运行](https://github.com/wangdiandao/gpui-material/actions/runs/43376009668) 通过。macOS 15 ARM64 使用 Metal，Ubuntu 24.04 x64 使用 WGPU/Vulkan。480×280 逻辑像素在 2× 缩放下得到 960×560 设备像素，三处形状内部和三处外部背景断言均通过。[Metal 截图](05-macos-metal.png) · [WGPU 截图](05-ubuntu-wgpu.png)。

```sh
P1_SHAPES_SCREENSHOT=shape-morph.png cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin shapes
```

[首轮失败](https://github.com/wangdiandao/gpui-material/actions/runs/43375384962) 显示 macOS 平台要求主线程，Ubuntu 捕获图为 960×560。修正把渲染移到二进制主线程，并以窗口缩放映射逻辑坐标和图像尺寸。此轮证据覆盖填充与选定像素，裁剪、阴影、命中和 ripple 仍待验证。

## 同一填充网格的命中与 ripple 起点

2026-11-13，Windows 11 x64 / Rust 1.99.1。三个 morph 进度复用 GPUI 填充产生的三角网格检查命中，包围盒内的空角不触发。测试窗口模拟 1.5× 缩放，通过鼠标事件对中间形态验证形内按下保存原始窗口坐标，形外按下清除起点。

## 曲线裁剪、阴影和状态层

将 64 边圆盘与填充网格的每个三角形相交。最大半径 96 逻辑像素时，圆盘轮廓的最大内缩为 0.116 逻辑像素；扩展圆盘覆盖全形状时，三个进度的裁剪结果包围盒宽、高与原网格的偏差均小于 0.001 逻辑像素。起始半径 2 包含原始指针点，距离指针 3 逻辑像素的对照点不在内。阴影采用三个同轮廓的偏移半透明填充层。常态 ripple 由 GPUI 帧请求驱动，320 ms 后消失。低动效使用静态层及一次到期刷新。

在 Windows 实窗 100% 缩放下，用 agent-desktop 点击中间样本右下方。设置 `P1_SHAPES_STILL=1` 将下一次命中的状态层固定在 45% 进度，便于比较截图。中间样本区域有 3,091 个像素改变，纯白背景改变 0 个像素。阴影采样 RGB 为 (238, 238, 238)，下方背景为 (255, 255, 255)。[Windows 实窗截图](05-windows-ripple.png)。

```powershell
$env:P1_SHAPES_STILL='1'
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin shapes
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin shapes
```

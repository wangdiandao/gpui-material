# P1-05 形状、绘制与 ripple

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
| 裁剪、阴影、命中、ripple 起点和高 DPI | 待在同一曲线上验证并量化误差。 | pending |

固定参考为 [CornerRoundingTest](https://android.googlesource.com/platform/frameworks/support/+/cb163bee53214245ad7a23367396a21b7f85ae82/graphics/graphics-shapes/src/commonTest/kotlin/androidx/graphics/shapes/CornerRoundingTest.kt)、[RoundedPolygonTest](https://android.googlesource.com/platform/frameworks/support/+/cb163bee53214245ad7a23367396a21b7f85ae82/graphics/graphics-shapes/src/commonTest/kotlin/androidx/graphics/shapes/RoundedPolygonTest.kt) 和 [MorphTest](https://android.googlesource.com/platform/frameworks/support/+/cb163bee53214245ad7a23367396a21b7f85ae82/graphics/graphics-shapes/src/commonTest/kotlin/androidx/graphics/shapes/MorphTest.kt)。

## Metal 与 WGPU 无头像素

2026-11-13，[修正后的托管运行](https://github.com/wangdiandao/gpui-material/actions/runs/43376009668) 通过。macOS 15 ARM64 使用 Metal，Ubuntu 24.04 x64 使用 WGPU/Vulkan。480×280 逻辑像素在 2× 缩放下得到 960×560 设备像素，三处形状内部和三处外部背景断言均通过。[Metal 截图](05-macos-metal.png) · [WGPU 截图](05-ubuntu-wgpu.png)。

```sh
P1_SHAPES_SCREENSHOT=shape-morph.png cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin shapes
```

[首轮失败](https://github.com/wangdiandao/gpui-material/actions/runs/43375384962) 显示 macOS 平台要求主线程，Ubuntu 捕获图为 960×560。修正把渲染移到二进制主线程，并以窗口缩放映射逻辑坐标和图像尺寸。此轮证据覆盖填充与选定像素，裁剪、阴影、命中和 ripple 仍待验证。

## 同一填充网格的命中与 ripple 起点

2026-11-13，Windows 11 x64 / Rust 1.99.1。三个 morph 进度复用 GPUI 填充产生的三角网格检查命中，包围盒内的空角不触发。测试窗口模拟 1.5× 缩放，通过实际鼠标事件对中间形态验证形内按下保存原始窗口坐标，形外按下清除起点。界面以小标记显示起点。任意曲线裁剪、阴影和完整 ripple 动画仍待对应证据。

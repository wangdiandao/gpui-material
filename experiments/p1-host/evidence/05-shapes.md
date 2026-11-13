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

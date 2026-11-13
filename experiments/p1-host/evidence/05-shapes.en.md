# P1-05 Shape, drawing, and ripple

[简体中文](05-shapes.md) · English

Recorded on 2026-11-13 with Windows 11 x64 build 26200 and Rust/Cargo 1.99.1, using the PR code and locked crates.io dependencies. The real window used 100% scaling. UIA and screenshots used winapp 0.7.1.

```sh
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --bin shapes
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin shapes
cargo +1.99.1 test --locked --manifest-path experiments/p1-shapes/Cargo.toml -- --nocapture
```

| Scenario | Observation | Status |
| --- | --- | --- |
| Corner parameters and space allocation | The two triangle base anchors were (0.5, 0) in the pinned AndroidX reference. polymorpher error was 0. | pass |
| Same-shape morph | Matching cubic anchors and control points on a triangle had maximum error 0 at 0, 0.5, and 1. | pass |
| Cross-topology and rounded morphs | Triangle to square, square to circle, and square to rounded square. All 17 evenly spaced samples per cubic were finite and within the unit-radius test bounds. Maximum closure gap was `5.1273855e-7` against a threshold of `1e-5` radii. | pass |
| GPUI native fill | `PathBuilder::cubic_bezier_to` accepted the same curves and tessellated at all three progress points. The [Windows window](05-windows.png) shows all three fills. | pass |
| Clipping, shadow, hit testing, ripple origin, and high DPI | Clip and hit testing shared the fill mesh and passed at 1.5×. In the Windows window the state layer changed 3,091 pixels and left sampled pure-white background unchanged. | pass |

Pinned references are [CornerRoundingTest](https://android.googlesource.com/platform/frameworks/support/+/cb163bee53214245ad7a23367396a21b7f85ae82/graphics/graphics-shapes/src/commonTest/kotlin/androidx/graphics/shapes/CornerRoundingTest.kt), [RoundedPolygonTest](https://android.googlesource.com/platform/frameworks/support/+/cb163bee53214245ad7a23367396a21b7f85ae82/graphics/graphics-shapes/src/commonTest/kotlin/androidx/graphics/shapes/RoundedPolygonTest.kt), and [MorphTest](https://android.googlesource.com/platform/frameworks/support/+/cb163bee53214245ad7a23367396a21b7f85ae82/graphics/graphics-shapes/src/commonTest/kotlin/androidx/graphics/shapes/MorphTest.kt).

## Headless Metal and WGPU pixels

The [corrected hosted run](https://github.com/wangdiandao/gpui-material/actions/runs/43376009668) passed on 2026-11-13. macOS 15 ARM64 used Metal and Ubuntu 24.04 x64 used WGPU/Vulkan. At 2×, 480×280 logical pixels yielded 960×560 device pixels. Three interior shape and three exterior background assertions passed. [Metal screenshot](05-macos-metal.png) · [WGPU screenshot](05-ubuntu-wgpu.png)

```sh
P1_SHAPES_SCREENSHOT=shape-morph.png cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin shapes
```

The [first run](https://github.com/wangdiandao/gpui-material/actions/runs/43375384962) failed because macOS required the main thread. Ubuntu captured 960×560. The fix moved rendering to the binary's main thread and mapped logical coordinates and image size using window scale. This run covered fills and selected pixels. Clipping, shadow, hit testing, and ripple remained pending on those platforms.

## Shared fill mesh and ripple origin

On 2026-11-13 with Windows 11 x64 and Rust 1.99.1, all three morph progress samples reused GPUI fill triangles for hit testing. Empty corners inside the bounding box did not trigger. At simulated 1.5× scale, pointer events confirmed a press inside the middle shape preserves original window coordinates and a press outside clears the origin.

## Curve clipping, shadow, and state layer

The probe intersects a 64-sided disk with each fill triangle. At the maximum radius of 96 logical pixels, the largest inward deviation was 0.116 logical pixels. When the disk covers the shape, clipped bounding widths and heights differ by less than 0.001 logical pixels from the original mesh at all three progress points. Radius 2 includes the pointer origin and excludes a point 3 logical pixels away. The shadow uses three offset translucent fills with the same outline. GPUI frame requests drive a normal ripple that disappears after 320 ms. Reduced motion uses a static layer with one expiry refresh.

At 100% scale on Windows, agent-desktop clicked the lower-right area of the middle sample. `P1_SHAPES_STILL=1` fixed the next hit's state layer at 45% for screenshot comparison. The middle sample changed 3,091 pixels and the pure-white background changed zero. The shadow sampled RGB (238, 238, 238), with (255, 255, 255) below it. [Native Windows screenshot](05-windows-ripple.png)

```powershell
$env:P1_SHAPES_STILL='1'
cargo +1.99.1 run --locked --manifest-path experiments/p1-host/Cargo.toml --bin shapes
cargo +1.99.1 test --locked --manifest-path experiments/p1-host/Cargo.toml --features test-support --bin shapes
```

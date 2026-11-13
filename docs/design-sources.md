# 设计依据与版本基线

核验日期为 2026-11-13。冻结内容包括 Material 站点快照、发布源码包、算法参考与 GPUI 版本组合。升级时先检查覆盖矩阵、Token、行为、平台、许可和依赖图，再更新本页及对应规格。

## 采用顺序

1. [Material 3](https://m3.material.io/) 的 Expressive 和适用的 M3 指南定义组件名称、结构、变体、视觉与交互语义。
2. Compose Material 3 的锁定版本用于核对 Token、状态、默认值及布局。组件矩阵给出的 Kotlin 文件位于该版本源码包。
3. 官方 Material Color Utilities 和 AndroidX graphics-shapes 用作算法与测试向量依据。
4. 固定的 GPUI 与 gpui-base 定义桌面事件、绘制和平台边界。桌面适配记录其输入路径、偏离原因及对应证据。
5. Material Components Android 的固定源码补充 Side Sheet、Docked Toolbar 等实现参照，以及弃用与替代说明。Material Web 用于历史背景。

同一主题出现差异时，以锁定的公开设计指南定范围，以锁定源码解读实现细节。组件所属阶段写明偏离、替代交互和测试证据。新增上游源码功能先进入范围审查。网站中的设备适用条件照实记录。

## Material 规范

[站点目录](https://m3.material.io/components)及各组件的 Overview、Specs、Guidelines 和 Accessibility 按发布数据版本 `2026-10-13_06-10-19` 核对。[站点快照索引](material-snapshot.md)记录各页固定 JSON 和 SHA-256。内容分块内嵌的 Token 表属于同一数据版本，具体数值、引用及生成规则归 P3 及相关组件阶段核对。此仓库保存本项目的规格、页面定位和校验值。

| 领域 | 设计范围与后续证据 | 官方入口 |
| --- | --- | --- |
| Token | Reference、System、Component 三层，原名和 Rust 字段可追溯。P3 验证引用闭合与生成可复现。 | [Design tokens](https://m3.material.io/foundations/design-tokens/overview) |
| 色彩 | 静态浅深色、角色配对、品牌/动态源色、对比度与固定角色。P3 用 MCU 检验语义色和边界。 | [Color roles](https://m3.material.io/styles/color/roles)、[Dynamic color](https://m3.material.io/styles/color/dynamic) |
| 排版 | baseline 与 emphasized 角色、字重、行高、字距与可覆盖字体回退。P3 比对角色，P4 验证实际字形及度量。 | [Typography](https://m3.material.io/styles/typography/overview) |
| 形状 | 角形阶梯、状态形状、有限圆角多边形及 morph。P3 对照几何参数，P4 对照绘制、裁剪和命中。 | [Shape](https://m3.material.io/styles/shape/overview-principles)、[Shape morph](https://m3.material.io/styles/shape/shape-morph) |
| 动效 | Standard/Expressive motion scheme，区分空间与颜色/透明度等效果。P4 测重定向、减少动态效果、终止重绘。 | [Motion](https://m3.material.io/styles/motion/overview/specs) |
| 层级 | Surface 角色、tonal elevation、阴影和 scrim。浅深色分别验证。 | [Elevation](https://m3.material.io/styles/elevation/overview) |
| 交互 | hover、focus-visible、pressed、dragged、disabled 和组件值状态。P4 验证输入取消和状态来源。 | [States](https://m3.material.io/foundations/interaction/states/overview)、[Inputs](https://m3.material.io/foundations/interaction/inputs) |
| 布局 | 可用窗口尺寸、密度、RTL、字体缩放与视觉/命中区域。P7 验证模式切换时的状态延续。 | [Breakpoints](https://m3.material.io/foundations/layout/breakpoints/overview)、[RTL](https://m3.material.io/foundations/layout/bidirectionality-rtl) |
| 无障碍 | 名称、角色、值、状态、动作、关系与通知。P1 验证平台桥接，组件阶段验证各语义及真实操作。 | [Assistive technology](https://m3.material.io/foundations/overview/assistive-technology) |

## 源码与算法参考

| 依据 | 固定版本或 revision | 校验或参考路径 |
| --- | --- | --- |
| Compose Material 3 common | `androidx.compose.material3:material3:1.5.0-rc01` | [Google Maven 源码包][compose] · `38e0515354240bf1e15b6e8891bc02608973f08dc0b8da8cd87a51dfe9ab1031` |
| Compose Material 3 Android | `androidx.compose.material3:material3-android:1.5.0-rc01` | [Google Maven 源码包][android] · `0e9c5b37e75203b44b4987b3ccc0db41637303103ca21739691d7533cbe7cbb1` |
| Adaptive Navigation Suite | `androidx.compose.material3:material3-adaptive-navigation-suite:1.5.0-rc01` | [Google Maven 源码包][nav-suite] · `fdbc86bff0c7f9f4ca8fdf9097515297af39200f65a82129d269f4a4fe11de64` |
| Adaptive Layout | `androidx.compose.material3.adaptive:adaptive-layout:1.3.0` | [Google Maven 源码包][adaptive] · `bd87db4bdbf51d7df02872b0c7e95ae82a3b7c61b7f2e021c407d30a899c22ca` |
| Adaptive Navigation | `androidx.compose.material3.adaptive:adaptive-navigation:1.3.0` | [Google Maven 源码包][adaptive-nav] · `c64c0ddec14fed5daae373c535c8e81e1f0ef1922aa258b624828448de18346d` |
| AndroidX graphics-shapes | `androidx.graphics:graphics-shapes:1.1.0`，发布变更范围终点 `cb163bee53214245ad7a23367396a21b7f85ae82` | [Google Maven 源码包][shapes] · `50586aa6c7b3a542a51b2abaf659f7a6c33d8c5d637d8f1a57aaf518dcb74f3d`；[同 revision 测试][shapes-tests] |
| Google Material Color Utilities | [`194144d75ed7d34f06264a625e086db81a7ab9aa`][mcu] | `typescript/hct/hct_test.ts`、`typescript/scheme/dynamic_scheme_test.ts`、`typescript/dynamiccolor/{dynamic_color,fixed_dynamic_color}_test.ts`、`dart/test/{contrast,palettes}_test.dart` |
| Material Components Android | [`66b0eeb53afeb6025ffbbd37734b08e5bce197ed`][mdc] | `docs/components/` 与对应 Java 源码。矩阵记录参考用途。 |
| Material Symbols | [`10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`][symbols] | `symbols/`，按实际使用项登记图标、样式、轴值及许可。 |

[Compose 发布说明][compose-release]列出稳定版 1.4.0 和候选版 1.5.0-rc01。后者的源码包提供 Expressive 实现基线，其发布变更范围终点是 [`1f549b6f5ba84cb3553be9c79110c4c5f6413039`][compose-tree]。源码以版本化包及校验值定位，Android 实现仅用于澄清交互。自适应与形状版本分别由[自适应发布说明][adaptive-release]和 [graphics-shapes 发布说明][shapes-release]核对。

P1 形状参考向量从锁定的 `RoundedPolygon.kt`、`CornerRounding.kt` 与 `Morph.kt` 建立。以 `MorphTest.cubicsTest` 的端点还原、`RoundedPolygonTest.roundingSpaceUsageTest` 的半径空间分配和 `CornerRoundingTest.cornerRoundingTest` 的参数为起点。用圆角三角形、正方形、圆、同拓扑与不同顶点数的多边形覆盖进度 0、0.5、1，记录半径、smoothing、Cubic 端点/控制点、闭合、边界与采样误差。

MCU 的固定向量包括蓝色 `0xff0000ff` 的色板 tone 40 得 `0xff343dff`，红色 `0xffff0000` 在 Tonal Spot、深色、对比度 0 下的 `primaryFixed` 为 `0xffffdad4`，tone −10 与 110 的夹取后对比度为 21（容差 0.001）。P3 增补源色往返、浅深模式、低/默认/高对比度和固定角色，记录版本、算法规格、预期值与容差。

## GPUI 与 Base

上游正式版为 [GPUI Kit v0.9.1][kit-release]，其 tag 和 crates.io 中 `gpui-base` 的包元数据均指向 [`39ab849a93b5f15507b969164d49891db8d5b470`][kit]. [固定工作区清单][kit-manifest]与两个发布包的元数据核实了下表。P0 固定版本契约，P1 探针和 P2 正式 Workspace 在各自阶段写入 manifest 与 lockfile。

| Cargo 包 | 精确版本 | crates.io SHA-256 |
| --- | --- | --- |
| `gpui-base` | `=0.9.1` | `f3e9ae5237737cfd110d0ff513fcd9c20f97d98ebabd0a73164068177f0aca4f` |
| `gpui-pre`（Rust 导入名 `gpui`） | `=0.3.12` | `29db89613527079737f53752eab27fb65c9a7dd94979256c65a0e590c9fd34d8` |
| `gpui-pre-platform`（Rust 导入名 `gpui_platform`） | `=0.3.12` | `d2b785bdcbd229f762096aebf4f9ae913bdaa9db49794907cabaf516eb55227b` |

两个 `gpui-pre` 发布包的 `package.metadata.gpui-pre.zed-rev` 都是 [`d2a4fb6ded7c470be25d1541281da3882495b65b`][zed]，发布记录见 [GPUI Kit Actions][gpui-publish]。按上游清单使用同一来源与精确版本，Cargo 解析结果须验证 GPUI 核心和平台分别只有一个包身份。Base `Theme` 在该版本实现 App `Global`，主题访问范围由 [theme.rs][base-theme]核实。

[compose]: https://dl.google.com/dl/android/maven2/androidx/compose/material3/material3/1.5.0-rc01/material3-1.5.0-rc01-sources.jar
[android]: https://dl.google.com/dl/android/maven2/androidx/compose/material3/material3-android/1.5.0-rc01/material3-android-1.5.0-rc01-sources.jar
[nav-suite]: https://dl.google.com/dl/android/maven2/androidx/compose/material3/material3-adaptive-navigation-suite/1.5.0-rc01/material3-adaptive-navigation-suite-1.5.0-rc01-sources.jar
[adaptive]: https://dl.google.com/dl/android/maven2/androidx/compose/material3/adaptive/adaptive-layout/1.3.0/adaptive-layout-1.3.0-sources.jar
[adaptive-nav]: https://dl.google.com/dl/android/maven2/androidx/compose/material3/adaptive/adaptive-navigation/1.3.0/adaptive-navigation-1.3.0-sources.jar
[shapes]: https://dl.google.com/dl/android/maven2/androidx/graphics/graphics-shapes/1.1.0/graphics-shapes-1.1.0-sources.jar
[shapes-tests]: https://android.googlesource.com/platform/frameworks/support/+/cb163bee53214245ad7a23367396a21b7f85ae82/graphics/graphics-shapes/src/commonTest/kotlin/androidx/graphics/shapes/
[mcu]: https://github.com/material-foundation/material-color-utilities/tree/194144d75ed7d34f06264a625e086db81a7ab9aa
[mdc]: https://github.com/material-components/material-components-android/tree/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components
[symbols]: https://github.com/google/material-design-icons/tree/10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7/symbols
[compose-release]: https://developer.android.com/jetpack/androidx/releases/compose-material3#1.5.0-rc01
[compose-tree]: https://github.com/androidx/androidx/tree/1f549b6f5ba84cb3553be9c79110c4c5f6413039/compose/material3
[adaptive-release]: https://developer.android.com/jetpack/androidx/releases/compose-material3-adaptive#1.3.0
[shapes-release]: https://developer.android.com/jetpack/androidx/releases/graphics#graphics-shapes-1.1.0
[kit-release]: https://github.com/longbridge/gpui-kit/releases/tag/v0.9.1
[kit]: https://github.com/longbridge/gpui-kit/tree/39ab849a93b5f15507b969164d49891db8d5b470
[kit-manifest]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/Cargo.toml
[zed]: https://github.com/zed-industries/zed/commit/d2a4fb6ded7c470be25d1541281da3882495b65b
[gpui-publish]: https://github.com/longbridge/gpui-kit/actions/runs/42539746081
[base-theme]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/crates/base/src/theme.rs

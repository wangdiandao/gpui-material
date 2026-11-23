# 设计依据

[English](design-sources.md) | 中文

最后同步日期为 2026-11-13。锁定内容包括 Material 站点快照、发布源码包、算法参考与 GPUI 版本组合。Material 规范版本升级遵循[贡献流程](../CONTRIBUTING.zh.md#设计规范)，经维护者同意采纳后更新本页及受影响的规格。

<a id="source-precedence"></a>

## 采用顺序

1. [Material 3](https://m3.material.io/) 的 Expressive 和适用的 M3 指南定义组件名称、结构、变体、视觉与交互语义。
2. 锁定版本的 Compose Material 3 作为 Token、状态、默认值及布局的实现参考。
3. 官方 Material Color Utilities 和 AndroidX graphics-shapes 作为算法与几何参考。
4. 固定的 GPUI 与 `gpui-base` 定义桌面事件、绘制和平台边界。桌面适配及其原因记在对应规格中。
5. Material Components Android 的固定源码补充 Side Sheet、Docked Toolbar 等实现参照，以及弃用与替代说明。Material Web 用于历史背景。

同一主题出现差异时，以锁定的公开设计指南定范围，以锁定源码解读实现细节。获准的桌面适配与替代交互记在对应的组件或架构文档中。

<a id="material-specification"></a>

## Material 规范

[站点目录](https://m3.material.io/components)及各组件的 Overview、Specs、Guidelines 和 Accessibility 锁定到数据版本 `2026-10-13_06-10-19`。[站点快照索引](material-snapshot.zh.md)记录各页固定 JSON 链接和 SHA-256 校验和。内嵌 Token 表来自同一数据版本，源数据变更时重新生成并更新仓库中的生成物。

| 领域 | 设计范围 | 官方入口 |
| --- | --- | --- |
| Token | Reference、System、Component 三层，原名和 Rust 字段可追溯。 | [Design tokens](https://m3.material.io/foundations/design-tokens/overview) |
| 色彩 | 静态浅深色、角色配对、品牌/动态源色、对比度与固定角色。 | [Color roles](https://m3.material.io/styles/color/roles)、[Dynamic color](https://m3.material.io/styles/color/dynamic) |
| 排版 | baseline 与 emphasized 角色、字重、行高、字距与可覆盖字体回退。 | [Typography](https://m3.material.io/styles/typography/overview) |
| 形状 | 角形阶梯、状态形状、全部 35 种 Expressive 圆角多边形、自定义圆角多边形、通用 morph、几何裁剪和形状感知命中检测。 | [Shape](https://m3.material.io/styles/shape/overview-principles)、[Shape morph](https://m3.material.io/styles/shape/shape-morph) |
| 动效 | Standard/Expressive motion scheme，区分空间与颜色/透明度等效果，支持重定向与减少动态效果。 | [Motion](https://m3.material.io/styles/motion/overview/specs) |
| 层级 | 浅深色下的 Surface 角色、tonal elevation、阴影和 scrim。 | [Elevation](https://m3.material.io/styles/elevation/overview) |
| 交互 | hover、focus-visible、pressed、dragged、disabled 和组件值状态。 | [States](https://m3.material.io/foundations/interaction/states/overview)、[Inputs](https://m3.material.io/foundations/interaction/inputs) |
| 布局 | 可用窗口尺寸、密度、RTL、字体缩放与视觉/命中区域。 | [Breakpoints](https://m3.material.io/foundations/layout/breakpoints/overview)、[RTL](https://m3.material.io/foundations/layout/bidirectionality-rtl) |
| 无障碍 | 名称、角色、值、状态、动作、关系、通知与平台桥接。 | [Assistive technology](https://m3.material.io/foundations/overview/assistive-technology) |

<a id="source-code-and-algorithm-references"></a>

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
| Material Components Android | [`66b0eeb53afeb6025ffbbd37734b08e5bce197ed`][mdc] | `docs/components/` 与对应 Java 源码 |
| Material Symbols | [`10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`][symbols] | `symbols/`（内置图标、样式、轴值及许可登记在资源清单中） |

[Compose 发布说明][compose-release]列出稳定版 1.4.0 和候选版 1.5.0-rc01；后者源码包提供 Expressive 实现参考，发布变更范围终点为 [`1f549b6f5ba84cb3553be9c79110c4c5f6413039`][compose-tree]。源码以版本化包和校验值定位。[自适应发布说明][adaptive-release] · [graphics-shapes 发布说明][shapes-release]

几何与色彩算法分别在 [foundation 形状模块](../crates/foundation/src/shape.rs)（对照 AndroidX `MorphTest` 与 `RoundedPolygonTest`）和 [theme 色彩参考测试](../crates/theme/tests/color_reference.rs)（对照 Material Color Utilities）中与锁定上游保持一致。

<a id="gpui-and-base"></a>

## GPUI 与 Base

上游正式版为 [GPUI Kit v0.10.0][kit-release]，其 tag 和 crates.io 中 `gpui-base` 的包元数据均指向 [`f697c3133d55c64fa8a5b67926147fb3dadd276d`][kit]。下表版本与[固定工作区清单][kit-manifest]及发布包元数据一致。

| Cargo 包 | 精确版本 | crates.io SHA-256 |
| --- | --- | --- |
| `gpui-base` | `=0.10.0` | `4f4e5aff50636223b268a687ad5788ec54fa77c8e57ca9e53d158f19e0425d95` |
| `gpui-pre`（Rust 导入名 `gpui`） | `=0.3.12` | `29db89613527079737f53752eab27fb65c9a7dd94979256c65a0e590c9fd34d8` |
| `gpui-pre-platform`（Rust 导入名 `gpui_platform`） | `=0.3.12` | `d2b785bdcbd229f762096aebf4f9ae913bdaa9db49794907cabaf516eb55227b` |

两个 `gpui-pre` 发布包的 `package.metadata.gpui-pre.zed-rev` 均为 [`d2a4fb6ded7c470be25d1541281da3882495b65b`][zed]，发布记录见 [GPUI Kit Actions][gpui-publish]。使用同一上游来源的精确版本，确保 Cargo 解析出单一的 GPUI 核心和平台包身份。Base `Theme` 在该版本实现 App `Global`（见 [theme.rs][base-theme]）。

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
[kit-release]: https://github.com/longbridge/gpui-kit/releases/tag/v0.10.0
[kit]: https://github.com/longbridge/gpui-kit/tree/f697c3133d55c64fa8a5b67926147fb3dadd276d
[kit-manifest]: https://github.com/longbridge/gpui-kit/blob/f697c3133d55c64fa8a5b67926147fb3dadd276d/Cargo.toml
[zed]: https://github.com/zed-industries/zed/commit/d2a4fb6ded7c470be25d1541281da3882495b65b
[gpui-publish]: https://github.com/longbridge/gpui-kit/actions/runs/42539746081
[base-theme]: https://github.com/longbridge/gpui-kit/blob/f697c3133d55c64fa8a5b67926147fb3dadd276d/crates/base/src/theme.rs

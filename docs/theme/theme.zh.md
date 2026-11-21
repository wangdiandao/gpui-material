# 纯主题

[English](theme.md) | 中文

`gpui-material-theme` 为应用提供 `MaterialTheme`。应用决定浅色或深色外观、颜色覆盖、字体家族、动效方案和密度。渲染层每次按可用逻辑宽度计算布局，并传入当前文字缩放比例与减少动态效果偏好。

| 领域 | 默认值和使用约定 |
| --- | --- |
| 色彩 | `ColorScheme::baseline` 使用 Compose 的浅色或深色角色。通过 `color` 取单个角色，`pair` 取前景和背景，`set_color` 覆盖单个角色。覆盖背景后由调用者检查前景对比度。容器角色提供基准层级，另可通过 `surface_at_elevation` 应用 Compose 的色调叠加公式。 |
| 源色 | `dynamic-color` 启用 `MaterialTheme::from_seed` 和 `from_seed_with`。默认使用 Tonal Spot Spec2021，忽略输入 alpha。对比度夹取到 −1 至 1，NaN 使用 0。 |
| 排版 | `TypeRole::style` 提供 15 个角色的 baseline 和 emphasized Token，`Typography::style` 按主题比例缩放字号、行高和字距。用 `Typography::with_text_scale` 设置比例，默认 1.0，范围为 1.0–2.0，NaN 使用 1.0。控件已有的字体缩放与主题比例相乘。Brand 和 Plain 都优先请求 Roboto，依次回退到 SF Pro Text、Segoe UI、Noto Sans、PingFang SC、Microsoft YaHei UI 和 Noto Sans CJK SC。 |
| 形状 | `CornerShape` 使用逻辑角顺序，按方向映射并限制相邻角总和。满圆角取短边的一半。`MaterialShape` 命名与 Compose `MaterialShapes` 对应的 35 种 Expressive 圆角多边形，`PolygonRole` 保留由 Token 提供的 Circle、Square 和 Triangle 源参数。渲染层负责规范化、轮廓插值、绘制、裁剪及命中检测。 |
| 动效 | 默认 Standard，可选 Expressive。空间与效果各有 Fast、Default 和 Slow 弹簧。效果弹簧采用临界阻尼。减少动态效果偏好将规格解析为立即完成。固定时长和 Bézier 曲线可通过 Token 查询。 |
| 布局 | 可用宽度的分界为 600、840、1200、1600 逻辑单位。阈值归入较大的尺寸级。逻辑起止边按 LTR 或 RTL 映射。`LayoutPolicy::scale_text` 用传入的倍率换算文字尺寸。 |
| 密度 | 桌面策略对组件可视尺寸使用 Compact −4、Standard 0、Comfortable +4 逻辑单位。交互尺寸保留 48 逻辑单位下限。修改组件密度时核验布局与命中。 |
| 序列化 | `serde` 独立启用主题值的序列化。颜色覆盖以语义角色为键，缺省角色从当前外观的基准表解析。 |

启用 `dynamic-color` 后，`ColorScheme::from_seed_with` 接受 `DynamicVariant` 和 `SpecVersion`。Tonal Spot、Vibrant、Expressive 和 Neutral 支持 Spec2025。Fidelity、Content 和 Monochrome 按上游规则使用 Spec2021。`ThemePreset` 提供基准、命名动态变体、高对比度和参考 [md3.io](https://md3.io/docs/theming/#the-shipped-themes) 的源色预设。

Info、Success 和 Warning 各有一组前景配色和一组容器配色。源色方案将状态色相向源色协调，并使用 MCU 的错误角色对比度策略解析配色。`ColorScheme::blend` 在 sRGB 中插值所有角色及其 alpha，外观在中点切换，两个端点返回原始色谱。

<a id="token-data"></a>

## Token 数据

[唯一数值源](../../tokens/material.json)按固定来源分组。[生成索引](token-index.zh.md)保留 Material 名称、Rust 常量和精确上游符号。Compose 中只暴露 Kotlin 符号的表使用 Material 三层命名规则映射，颜色静态表在角色下保留 light/dark 键。Filled Button 和 Small Button 的键用于校验 Component 层覆盖颜色、形状、尺寸和状态。扩充组件族时，将对应的锁定 Token 数据加入此文件。

Reference 层存放色板和字族等基础值。System 层定义语义和公共尺度。Component 层可引用 System 或 Reference，颜色引用保留 `ColorRole`，通过调用者当前的主题解析。同层别名支持静态色和组合排版的内部引用。生成器检查重名、Rust 标识冲突、单位、值域、层级、缺失目标与循环。

```powershell
cargo xtask tokens
cargo xtask tokens --check
cargo xtask docs --write docs/theme/token-index.md
```

生成器使用仓库内数据，写出 Rust 表和中英文 Markdown 索引。核对两份索引后再更新配对记录。日常构建直接编译提交的表。CI 通过 `cargo xtask check` 校验生成物和依赖图。

<a id="sources-and-desktop-decisions"></a>

## 来源与桌面裁决

- 静态色、色板、排版数值、角形、状态层、层级与动效取固定的 Compose Material 3 1.5.0-rc01。网站将 Roboto 定义为默认家族。Compose 使用平台无衬线抽象，桌面通过可配置家族列表表达这个选择。
- 原有 49 个动态色角色沿用固定的 Tonal Spot Spec2021 参考向量。静态状态色采用项目的桌面色板。
- `MaterialShape` 命名固定版本 Compose `MaterialShapes.kt` 中的全部 35 种 Expressive 圆角多边形，`PolygonRole` 保留由 Token 提供的 Circle、Square 和 Triangle 三种参数。Circle 参数采用 AndroidX 的最终圆半径，Square 参数为矩形宽高，Triangle 参数采用外接圆半径。参数与固定 Compose `MaterialShapes.kt` 及 AndroidX graphics-shapes 1.1.0 对照。圆角矩形的相邻角统一缩放属于桌面几何策略。
- 可用宽度来自固定 Material Breakpoints 表。48 逻辑单位取 Compose `LocalMinimumInteractiveComponentSize`。密度偏移、字体回退顺序和立即完成的低动效策略是本项目的桌面约定。

固定版本和校验值见[设计依据](../design-sources.zh.md)。

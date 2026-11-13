# 纯主题

`gpui-material-theme` 为应用提供 `MaterialTheme`。应用决定浅色或深色外观、颜色覆盖、字体家族、动效方案和密度。渲染层每次按可用逻辑宽度计算布局，并传入当前文字缩放比例与减少动态效果偏好。

| 领域 | 默认值和使用约定 |
| --- | --- |
| 色彩 | `ColorScheme::baseline` 使用 Compose 的浅色或深色角色。通过 `color` 取单个角色，`pair` 取前景和背景，`set_color` 覆盖单个角色。覆盖背景后由调用者检查前景对比度。容器角色提供基准层级，另可通过 `surface_at_elevation` 应用 Compose 的色调叠加公式。 |
| 源色 | `dynamic-color` 启用 `MaterialTheme::from_seed`。品牌色或上层取得的系统源色共用 Tonal Spot Spec2021，输入的 alpha 被忽略。对比度夹取到 −1 至 1，NaN 使用默认值 0。 |
| 排版 | `TypeRole::style` 为 15 个角色分别解析 baseline 和 emphasized。字号、行高及字距使用 sp。Brand 和 Plain 都优先请求 Roboto，桌面默认回退依次为 Segoe UI、Microsoft YaHei UI 和 Noto Sans CJK SC。应用可按角色家族替换列表，字体解析和字形回退由宿主负责。 |
| 形状 | `CornerShape` 使用逻辑角顺序，按方向映射并限制相邻角总和。满圆角取短边的一半。Circle、Square 和 Triangle 提供固定源参数作为 morph 端点，渲染层负责规范化、轮廓插值、绘制及命中。 |
| 动效 | 默认 Standard，可选 Expressive。空间与效果各有 Fast、Default 和 Slow 弹簧。效果弹簧采用临界阻尼。减少动态效果偏好将规格解析为立即完成。固定时长和 Bézier 曲线可通过 Token 查询。 |
| 布局 | 可用宽度的分界为 600、840、1200、1600 逻辑单位。阈值归入较大的尺寸级。逻辑起止边按 LTR 或 RTL 映射。字号、行高和字距由宿主提供的文字缩放比例换算。 |
| 密度 | 桌面策略对组件可视尺寸使用 Compact −4、Standard 0、Comfortable +4 逻辑单位。交互尺寸保留 48 逻辑单位下限。组件在各自阶段应用该策略并核验布局与命中。 |
| 序列化 | `serde` 独立启用主题值的序列化。颜色覆盖以语义角色为键，缺省角色从当前外观的基准表解析。 |

## Token 数据

[唯一数值源](../../tokens/material.json)按固定来源分组。[生成索引](token-index.md)保留 Material 名称、Rust 常量和精确上游符号。Compose 中只暴露 Kotlin 符号的表使用 Material 三层命名规则映射，颜色静态表在角色下保留 light/dark 键。Filled Button 和 Small Button 的键用于校验 Component 层覆盖颜色、形状、尺寸和状态。各组件族在归属阶段扩充数据。

Reference 层存放色板和字族等基础值。System 层定义语义和公共尺度。Component 层可引用 System 或 Reference，颜色引用保留 `ColorRole`，通过调用者当前的主题解析。同层别名支持静态色和组合排版的内部引用。生成器检查重名、Rust 标识冲突、单位、值域、层级、缺失目标与循环。

```powershell
cargo xtask tokens
cargo xtask tokens --check
```

生成器使用仓库内数据，写出 Rust 表与 Markdown 索引。日常构建直接编译提交的表。CI 通过 `cargo xtask check` 校验两份生成物和依赖图。

## 来源与桌面裁决

- 静态色、色板、排版数值、角形、状态层、层级与动效取固定的 Compose Material 3 1.5.0-rc01。网站将 Roboto 定义为默认家族。Compose 使用平台无衬线抽象，桌面通过可配置家族列表表达这个选择。
- 动态色选用 P0/P1 已验证的 Tonal Spot Spec2021。49 个语义角色包括 legacy background/surface-variant、inverse、fixed、outline、surface containers、scrim 及 MCU 的 shadow。
- 圆角多边形限制在 P1 已验证的三类拓扑。Circle 参数采用 AndroidX 的最终圆半径，Square 参数为矩形宽高，Triangle 参数采用外接圆半径。参数与固定 Compose `MaterialShapes.kt` 及 AndroidX graphics-shapes 1.1.0 对照。圆角矩形的相邻角统一缩放属于桌面几何策略。
- 可用宽度来自固定 Material Breakpoints 表。48 逻辑单位取 Compose `LocalMinimumInteractiveComponentSize`。密度偏移、字体回退顺序和立即完成的低动效策略是本项目的桌面约定。

固定版本和校验值见[设计依据](../design-sources.md)。

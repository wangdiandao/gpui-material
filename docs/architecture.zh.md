# 架构与依赖

[English](architecture.md) | 中文

GPUI Material 提供 Material 的 Token、主题、视觉状态、布局和组合。GPUI Base 提供可复用的行为、焦点、输入、滚动、状态和基础设施。准确版本见[设计依据](design-sources.zh.md#gpui-and-base)。

<a id="crate-dependency-direction"></a>

## Crate 依赖方向

箭头从调用方指向依赖。下表说明各层的依赖与职责。

| Crate | 依赖与责任 |
| --- | --- |
| `gpui-material-theme` | 纯 Rust。维护 Token、语义色、排版、形状描述、动效参数和布局策略。公开纯值类型。 |
| `gpui-material-foundation` | → theme、gpui-base、同源 GPUI。提供 MaterialRoot、主题投影、绘制、状态层、ripple、几何与动效适配。 |
| `gpui-material-components` | → foundation，必要时 → theme。提供轻量核心展示原语及按组划分的核心组件特性（`actions`、`selection`、`inputs`、`surfaces`、`feedback`、`navigation`；`default = ["full"]`）。 |
| `gpui-material-extras` | → components（仅启用各组件族所需的组件分组）、foundation 和所需的 theme API。按组件族启用，独立声明额外依赖。 |
| `gpui-material-symbols` | 提供完整 Material Symbols 目录的配套库，从可变字形轮廓生成 SVG。应用将结果传给 `Icon::svg_data`。[集成方法](integration.zh.md#material-symbols) |
| `gpui-material` | → 已交付的公开实现 crate，转发核心组件分组（`default = ["full"]`）及可选的 extras 和主题特性。 |
| `gpui-material-gallery` | → 已交付的门面。同一个包提供 `gpui_material_story` 库和桌面可执行程序，排除在 Workspace 默认构建之外。 |
| `xtask` | 文档与依赖检查、构建计时和 Token 离线生成。保持 GPUI 无关。 |

controls/display 可被 inputs、overlays、navigation 使用。TextField 的编辑模型向组合层提供编辑命令，ExposedDropdown 和编辑菜单由组合层衔接。navigation 使用 slot 接入搜索和工具条。实现 crate 通过底层接口共享代码，门面仅负责用户入口。生产依赖链使用 gpui-base，`gpui-component` 作为上游集成参考。

宿主可以用 `default-features = false` 依赖 `apps/gallery`，直接使用 `gpui_material_story`。打开第一个窗口前调用 `init_gallery`。在 `gpui_base::Root` 下用 `Gallery::new` 放置完整目录，或用 `Story::new_view` 放置单个组件示例。URL 可以使用 Story 标识符，并通过 `Story::find` 查找对应示例。

<a id="state-and-theme"></a>

## 状态与主题

Search 和 exposed dropdown 保留完整结果标识及滚动状态，由 GPUI 可变高度列表构建可见行和有限的预渲染行。Search 按结果 ID 保留焦点句柄；dropdown 将焦点留在 Base 编辑器，并保留活动选项 ID。内容、宽度、字体、已安装字体快照、rem 大小及下拉动画缩放变化时，相应测量缓存失效。键盘跳向尚未测量的行时使用逻辑行锚点，无需先布局全部前置行。轻量 `List` 继续适用于小集合；仅向它传入可见切片不能实现完整数据模型的键盘导航。

Dropdown 在 Base Input 和 Popup 之上保留处理 IME 的窄键盘适配层。固定版本的 Base Select/Combobox 根节点会增加自己的语义和焦点所有者，且无法在输入法组合期间否决动作处理，直接包装现有可编辑 TextField 会改变契约。按钮和选择控件共享一个呈现事件适配器；Base 仍负责激活与受控值，Material 保留涟漪时间和形状过渡。

- 调用者拥有业务值和稳定标识。Base 持有交互所需的状态、输入、选择和焦点。Material 持有主题、状态呈现策略及新增组合状态。受控组件通过同一路径请求变更，文档注明回调的执行次序及同步重入边界。
- 组件按语义公开 variant、size、disabled、selected、on_change 等必要契约，开放与组成部分对应的 slot。共享接口以具体类型为主，第三方颜色/形状类型留在适配层。
- `MaterialTheme` 是项目的主题源。固定版本的 Base `Theme` 通过 App `Global` 读取，包含 appearance、语义 Token、滚动条、分割调整、plot 与光标闪烁默认值。应用主题向 Base 投影色彩和排版。Material 在元素布局时解析窗口和子树主题，并将当前作用域传给浮层内容。
- foundation 将 Base 交互状态映射为 Material 呈现。禁用会压制 hover 和 active 样式。改动时验证禁用、取消、键盘和焦点可见行为。
- Token 用受控数据文件作唯一数值源。xtask 生成 Rust 表和索引并提交生成物，日常构建使用仓库内数据。Foundation 统一封装 `polymorpher` 以提供全部 35 种 Expressive Material 形状、通用变形、几何裁剪、外部遮罩与形状感知命中检测，使各组件共享同一套形状管线而无需直接依赖 `polymorpher`。

Foundation 为各组件族共享排版、涟漪反馈和桌面阴影投影。按压涟漪与悬停、焦点、拖动状态层分离；减少动效时使用静态按压提示。单行文字用同一份含字距的字形布局测量和绘制。换行文字将断行交给 GPUI，应用 Material 字体族、字重、字号和行高；固定版本的原生换行接口不开放自定义字距。GPUI 的双层阴影投影属于桌面适配，并非逐像素复刻 Android 的平台阴影渲染器。

<a id="dependency-policy"></a>

## 依赖规则

按[版本基线](design-sources.zh.md#gpui-and-base)锁定 Base 与 GPUI。提议变更依赖时，在 Issue 或 PR 记录用途、启用的 feature、解析后的依赖图、维护情况、许可、替换边界和验证结果。

| 依赖或候选 | 用途与约束 | 验证与替换边界 |
| --- | --- | --- |
| `gpui-base =0.10.0` / `gpui-pre =0.3.12` / `gpui-pre-platform =0.3.12` | 固定 crates.io 来源，沿用上游平台的 font-kit、x11、wayland、runtime_shaders 配对；test-support 供验收，inspector 可选。Apache-2.0。 | `cargo metadata` 和 feature 图核对单一核心/平台身份，更新锁定版本时执行相关[平台检查](verification.zh.md)。Base 公共接口通过 foundation 封装。 |
| [`material-color-utils 0.1.3`][mcu-rust] | HCT、色板和动态语义色。关闭默认 feature，避免 image、serde、rayon；直接依赖 bon、indexmap、papaya、thiserror。发布元数据使用 license-file，包内 LICENSE 为 Apache-2.0。 | 库类型由 theme 内部适配。 |
| [`polymorpher 0.1.4`][polymorpher] | 圆角多边形和 morph，保留默认空 feature，只使用基础几何；直接依赖 euclid。Apache-2.0。 | 对照 AndroidX，检查输出曲线、边界、拓扑、裁剪、命中和性能。GPUI 路径转换集中在 foundation。 |
| [Material Symbols][symbols] | 使用固定版本的必要资源，应用可替换图标。Apache-2.0。 | 验证资源映射、图标 fallback 与 RTL。逐项登记分发资产和许可证。 |
| `chrono 0.4.45` 与 [Base Calendar][calendar] | `date-time` 特性在 extras 中启用直接 chrono 依赖，并关闭其默认特性。该依赖与现有 Base 和 GPUI 依赖图解析到同一个包。MIT 或 Apache-2.0。 | Base 管理日历及日期范围的草稿状态。Chrono 提供有边界检查的日历运算和日期时间类型。应用通过格式化和解析接口控制本地化显示与输入。 |
| [`rust-i18n 4.2.4`][i18n] | 可选应用适配。MIT。初始项目使用可覆盖文案和 formatter，候选引入宏与 support crate。 | 评估编译成本和依赖图，验证字符串覆盖及真实布局。翻译库类型由应用适配封装。 |

extras 按实际需要定义 expressive-groups、search、date-time、carousel 等组件族。主题支持静态数据，dynamic-color 与 serde 按需启用，图片提色优先留在工具层。组件所需键盘、语义和动效属于完整行为。

CI 和日常开发用包级命令隔离主题、组件及 Gallery。依据构建计时和运行测量调整架构，优先使用具体内部类型及按需的类型擦除。

<a id="desktop-adapter-boundaries"></a>

## 桌面适配边界

验证适配边界时使用锁定的 Base 和 GPUI 版本。[foundation 形状测试](../crates/foundation/src/shape.rs)随 Workspace 测试运行，验证 Material 几何。

| 机制 | 适配与边界 |
| --- | --- |
| 激活与拖动 | Base 管理按钮激活与焦点，组合层保留 roving 焦点策略。Windows 捕获丢失、禁用和 Esc 汇入取消路径。固定版本的 Windows GPUI 缺少捕获丢失事件，Foundation 因此在拖动期间于 UI 线程检查 Win32 捕获所有权。 |
| 文本和语义 | Base 持有编辑状态、剪贴板、掩码和输入处理器，GPUI 持有原生桥接、字形回退和布局。同一 shaped line 用于度量及绘制。Base 0.10.0 的 Slider 处理 Windows UI Automation RangeValue 的 SetValue 动作。Material 为每个自定义滑块提供独立的原生范围节点，将动作交给 Base SliderState 中对应的滑块。 |
| 绘制与动效 | foundation 复用 GPUI 的填充网格，为凸、凹多边形统一提供精确的路径/矩形/圆盘裁剪、EvenOdd 外部背景遮罩与形状感知命中检测。非矩形多边形表面以三层半透明填充近似同形阴影。几何重定向、颜色和透明度沿用 Base spring/transition 的公共调度，系统低动效和静止停止调度均有验证。 |
| 浮层与滚动 | Base Popover 和 Dialog 持有放置、遮罩和焦点范围。组合层映射逻辑对齐，管理开关、回焦与拖动取消。独占内层滚动可通过 GPUI 命中遮蔽实现。值标签订阅 Base SliderState。 |
| 主题和日期 | 应用色彩过渡通过帧回调投影到 Base 全局主题。窗口和子树主题为 Material 及其 Base 控件提供局部样式。日期边界和禁用行为复用 Base Calendar。 |
| Windows 标题栏 | extras 中的可选适配负责布局、按钮绘制和无障碍控件。GPUI 和 Windows 负责命中测试、窗口移动、缩放和贴靠布局。窗口选项与关闭策略由应用管理。 |

`Icon::new(IconName::Favorite)` 通过 GPUI 绘制内嵌的官方 Material Symbols SVG；`.filled(true)` 选择官方填充版本。具名图标继承控件图标槽的文字尺寸和前景色，使尺寸变体、禁用颜色和颜色动画保持一致。图标槽外使用 `.size(px(24.))` 明确尺寸，`.color(ColorRole::Primary)` 可显式覆盖颜色。图标默认仅作装饰，只有设置 `.label(id, name)` 才暴露独立图像语义；纯图标按钮仍由按钮提供无障碍名称。方向图标通过 `.mirror_in_rtl(true)` 显式启用 RTL 镜像，并可与 `.rotate(...)` 组合。原有 `Icon::svg` 和 `Icon::custom` 保留 24dp/OnSurfaceVariant 默认值。内嵌图标不需要应用注册资源，也不依赖字体。[资源清单](../crates/components/assets/material-symbols/manifest.toml) 固定未经修改的上游文件和哈希，并随附[许可证](../crates/components/assets/material-symbols/LICENSE)。内置组件图标不使用字符占位或自绘静态路径，选择控件的动态几何仍由所属组件负责。

[mcu-rust]: https://github.com/RuurdBijlsma/material-color-utilities-rs/tree/d39ff59a1fca34bdc60e124de2f9f7541fff3c39
[polymorpher]: https://github.com/Aiving/polymorpher/tree/aaeb6f69b0ced6cc7079713994832b5d7ee3ff50
[symbols]: design-sources.zh.md#source-code-and-algorithm-references
[calendar]: https://github.com/longbridge/gpui-kit/blob/f697c3133d55c64fa8a5b67926147fb3dadd276d/crates/base/src/calendar.rs
[i18n]: https://github.com/longbridge/rust-i18n/tree/5f3ea5ba03395f2f2908e3587e01788cfdfd3555

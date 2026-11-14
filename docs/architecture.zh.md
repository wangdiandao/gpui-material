# 架构与依赖

[English](architecture.md) | 中文

gpui-material 提供 Material 的 Token、主题、视觉状态、布局和组合。gpui-base 提供可复用的行为、焦点、输入、滚动、状态和基础设施。P1 用[探针](phases/P01.zh.md)确认实际能力、必要适配与依赖图。准确版本见[设计依据](design-sources.zh.md#gpui-and-base)。

<a id="crate-dependency-direction"></a>

## Crate 依赖方向

箭头从调用方指向依赖。下表定义各层首次交付及后续可增加的模块。

| Crate | 首次交付 | 依赖与责任 |
| --- | --- | --- |
| `gpui-material-theme` | P3 | 纯 Rust。维护 Token、语义色、排版、形状描述、动效参数和布局策略。公开纯值类型。 |
| `gpui-material-foundation` | P4 | → theme、gpui-base、同源 GPUI。提供 MaterialRoot、主题投影、绘制、状态层、ripple、几何与动效适配。 |
| `gpui-material-components` | P5 | → foundation，必要时 → theme。P5 加 controls/display，P6 加 inputs/overlays，P7 加 navigation。 |
| `gpui-material-extras` | P8 | → components、foundation 和所需的 theme API。按组件族启用，独立声明额外依赖。 |
| `gpui-material` | P5 | → 已交付的公开实现 crate，按需转发 feature。extras 为可选依赖。 |
| `gpui-material-gallery` | P2 工程容器，P5 起逐项演示 | → 已交付的门面和宿主平台入口。按组件族构建，排除在日常默认构建集合之外。 |
| `xtask` | P2 工程工具，P3 加 Token 生成 | 阶段与构建记录检查、来源索引、离线生成。保持 GPUI 无关。 |

controls/display 可被 inputs、overlays、navigation 使用。TextField 的编辑模型向组合层提供编辑命令，ExposedDropdown 和编辑菜单由组合层衔接。navigation 使用 slot 接入搜索和工具条。实现 crate 通过底层接口共享代码，门面仅负责用户入口。生产依赖链使用 gpui-base，`gpui-component` 作为上游集成参考。

<a id="state-and-theme"></a>

## 状态与主题

- 调用者拥有业务值和稳定标识。Base 持有交互所需的状态、输入、选择和焦点。Material 持有主题、状态呈现策略及新增组合状态。受控组件通过同一路径请求变更，文档注明回调的执行次序及同步重入边界。
- 组件按语义公开 variant、size、disabled、selected、on_change 等必要契约，开放与组成部分对应的 slot。共享接口以具体类型为主，第三方颜色/形状类型留在适配层。
- `MaterialTheme` 是项目的主题源。固定版本的 Base `Theme` 通过 App `Global` 读取，包含 appearance、语义 Token、滚动条、分割调整、plot 与光标闪烁默认值。项目主题切换由 App 显式提交并同步投影和刷新。窗口或子树的局部主题由调用者显式传入样式，详见双窗口探针。
- P4 集中将 Base 的真实交互状态转成 Material 呈现。禁用会压制 hover/active 样式。P1 验证禁用、取消、键盘与焦点可见状态的公共读取路径。
- Token 用受控数据文件作唯一数值源。xtask 生成 Rust 表和索引并提交生成物，日常构建使用仓库内数据。形状适用范围由探针覆盖的圆角多边形限定。

Foundation 为各组件族共享排版、涟漪反馈和桌面阴影投影。按压涟漪与悬停、焦点、拖动状态层分离；减少动效时使用静态按压提示。单行文字用同一份含字距的字形布局测量和绘制。换行文字将断行交给 GPUI，应用 Material 字体族、字重、字号和行高；固定版本的原生换行接口不开放自定义字距。GPUI 的双层阴影投影属于桌面适配，并非逐像素复刻 Android 的平台阴影渲染器。

<a id="candidate-dependencies"></a>

## 候选依赖

P0 确定 Base 与 GPUI 来源。其余候选由归属阶段验证。准入记录包含用途、启用 feature、实际传递图、维护情况、许可、可替换接口和验证结果。

| 候选 | 准入阶段 | 功能与初始约束 | 验证与替换边界 |
| --- | --- | --- | --- |
| `gpui-base =0.9.1` / `gpui-pre =0.3.12` / `gpui-pre-platform =0.3.12` | P1 探针，P2 Workspace | 固定 crates.io 来源，沿用上游平台的 font-kit、x11、wayland、runtime_shaders 配对；test-support 供验收，inspector 可选。Apache-2.0。 | `cargo metadata` 和 feature 图核对单一核心/平台身份，Windows 运行必需探针。macOS 与 Linux 探针供探索。Base 公共接口通过 foundation 封装。 |
| [`material-color-utils 0.1.3`][mcu-rust] | P3 已准入 | HCT、色板和动态语义色。关闭默认 feature，避免 image、serde、rayon；直接依赖 bon、indexmap、papaya、thiserror。发布元数据使用 license-file，包内 LICENSE 为 Apache-2.0。 | 库类型由 theme 内部适配。见[主题 PR](https://github.com/wangdiandao/gpui-material/pull/78)。 |
| [`polymorpher 0.1.4`][polymorpher] | P1 可行性，P4 准入 | 圆角多边形和 morph，保留默认空 feature，只使用基础几何；直接依赖 euclid。Apache-2.0。 | 对照 AndroidX，检查输出曲线、边界、拓扑、裁剪、命中和性能。GPUI 路径转换集中在 foundation。 |
| [Material Symbols][symbols] | P4/P5 | 使用固定版本的必要资源，应用可替换图标。Apache-2.0。 | 验证资源映射、图标 fallback 与 RTL。逐项登记分发资产和许可证。 |
| `chrono` 与 [Base Calendar][calendar] | P1 验证行为，P8 准入日期族 | Base 清单为 `0.4.38` 兼容范围，固定上游 lockfile 解析为 0.4.45。MIT 或 Apache-2.0。 | 核对统一版本、日历语义与边界。日期运算由 Base/chrono 复用，locale 格式走独立 formatter。 |
| [`rust-i18n 4.2.4`][i18n] | P4 定义本地化入口，出现目录需求时再评估 | 可选应用适配。MIT。初始项目使用可覆盖文案和 formatter，候选引入宏与 support crate。 | 评估编译成本和依赖图，验证字符串覆盖及真实布局。翻译库类型由应用适配封装。 |

extras 按实际需要定义 expressive-groups、search、date-time、carousel 等组件族。主题支持静态数据，dynamic-color 与 serde 按需启用，图片提色优先留在工具层。组件所需键盘、语义和动效属于完整行为。

CI 和日常开发用包级命令隔离主题、组件及 Gallery。依据构建计时和运行测量调整架构，优先使用具体内部类型及按需的类型擦除。

<a id="boundaries-verified-in-p1"></a>

## P1 已验证的适配边界

以 Windows 11 x64/MSVC 验收。P2 可沿用锁定的 Base 0.9.1 和 GPUI 0.3.12 同源组合。纯主题算法和形状候选的准入分别在 P3、P4 执行。macOS 和 Linux 的托管与像素样本留作平台探索。见 [P1 验收 PR](https://github.com/wangdiandao/gpui-material/pull/51)。

| 机制 | 适配与边界 |
| --- | --- |
| 激活与拖动 | Base 管理按钮激活与焦点，组合层保留 roving 焦点策略。Windows 捕获丢失、禁用和 Esc 汇入取消路径。固定 Windows GPUI 缺少捕获丢失事件，探针以 UI 线程的 Win32 捕获检查验证过渡方案。替换时复测原生捕获离开与下一次拖动。[Issue #48](https://github.com/wangdiandao/gpui-material/issues/48) |
| 文本和语义 | Base 持有编辑状态、剪贴板、掩码和输入处理器，GPUI 持有原生桥接、字形回退和布局。同一 shaped line 用于度量及绘制。Windows UIA RangeValue 的 SetValue 经 GPUI 语义动作转给 Base SliderState。上游 Slider 直接处理此动作后移除适配并复测。[Issue #49](https://github.com/wangdiandao/gpui-material/issues/49) |
| 绘制与动效 | foundation 可复用 GPUI 的填充网格完成命中和状态层裁剪。Windows 探针的同形阴影由三层半透明填充近似。几何重定向、颜色和透明度沿用 Base spring/transition 的公共调度，系统低动效和静止停止调度均有验证。 |
| 浮层与滚动 | Base Popover 和 Dialog 持有放置、遮罩和焦点范围。组合层映射逻辑对齐，管理开关、回焦与拖动取消。独占内层滚动可通过 GPUI 命中遮蔽实现。值标签订阅 Base SliderState。 |
| 主题和日期 | Base 的 Theme 是 App 级 Global。Material 通过显式投影和局部样式支持窗口策略，纯颜色与日期探针可独立运行。日期边界和禁用行为复用 Base Calendar。 |

`Icon::new(IconName::Favorite)` 通过 GPUI 绘制内嵌的官方 Material Symbols SVG；`.filled(true)` 选择官方填充版本。具名图标继承控件图标槽的文字尺寸和前景色，使尺寸变体、禁用颜色和颜色动画保持一致。图标槽外使用 `.size(px(24.))` 明确尺寸，`.color(ColorRole::Primary)` 可显式覆盖颜色。图标默认仅作装饰，只有设置 `.label(id, name)` 才暴露独立图像语义；纯图标按钮仍由按钮提供无障碍名称。方向图标通过 `.mirror_in_rtl(true)` 显式启用 RTL 镜像，并可与 `.rotate(...)` 组合。原有 `Icon::svg` 和 `Icon::custom` 保留 24dp/OnSurfaceVariant 默认值。内嵌图标不需要应用注册资源，也不依赖字体。[资源清单](../crates/components/assets/material-symbols/manifest.toml) 固定未经修改的上游文件和哈希，并随附[许可证](../crates/components/assets/material-symbols/LICENSE)。内置组件图标不使用字符占位或自绘静态路径，选择控件的动态几何仍由所属组件负责。

[mcu-rust]: https://github.com/RuurdBijlsma/material-color-utilities-rs/tree/d39ff59a1fca34bdc60e124de2f9f7541fff3c39
[polymorpher]: https://github.com/Aiving/polymorpher/tree/aaeb6f69b0ced6cc7079713994832b5d7ee3ff50
[symbols]: design-sources.zh.md#source-code-and-algorithm-references
[calendar]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/crates/base/src/calendar.rs
[i18n]: https://github.com/longbridge/rust-i18n/tree/5f3ea5ba03395f2f2908e3587e01788cfdfd3555

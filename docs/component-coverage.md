# 组件覆盖矩阵

本表覆盖 [Material 3 组件目录](https://m3.material.io/components)和项目所需的桌面组合。官方组件链接对应[固定站点快照](material-snapshot.md)。Kotlin 文件名对应[固定 Compose 源码包](design-sources.md#源码与算法参考)的 `commonMain/androidx/compose/material3/`，标注 Android 或 Adaptive 的条目使用相应发布包。

类别为 **规范**、**适配**、**实验**、**兼容**。规范和适配条目是首个完整候选版的必需范围。实验项在 P8 记录纳入或暂缓的结论。兼容项由后续串行阶段安排。类型为 `component`、`primitive` 或 `pattern`。表中生产交付均处于 `planned`，其平台验收以[平台矩阵](verification.md)为准。

每项在归属阶段补齐适用的尺寸、Token、视觉、交互、动效、语义和证据。首版平台列使用 **桌面**（Windows 11 x64，覆盖键盘/指针、浅深色、RTL、高 DPI 及适用的读屏）、**文本**（桌面加 Windows 中文 IME）、**触控**（桌面加实际触摸或触控板证据）。缺少触控设备的项目记录独立证据状态。macOS 与 Linux 可按同样分类记录探索结果。

## P4 基础原语

| 交付 | 类型 / 类别 | 规范与源码 | 范围和主要验收 | 平台 |
| --- | --- | --- | --- | --- |
| MaterialRoot、主题投影、Surface、scrim | primitive / 适配 | [Elevation](https://m3.material.io/styles/elevation/overview)，`Surface.kt`、`Scrim.kt` | App 主题、组件覆盖、明暗层级、遮罩及窗口刷新；Base 交互设施同步读取必要角色。 | 桌面 |
| 状态层、焦点环、ripple、形状与动效 | primitive / 规范 | [States](https://m3.material.io/foundations/interaction/states/overview)、[Motion](https://m3.material.io/styles/motion/overview/specs)，`Ripple.kt`、`MotionScheme.kt` | 输入来源、禁用与取消优先级、形状裁剪和命中、连续重定向、减少动态效果、动画结束停止重绘。 | 桌面 |
| 焦点、浮层、拖拽、滚动、值标签、文本度量和语义 | primitive / 适配 | [Inputs](https://m3.material.io/foundations/interaction/inputs)、[GPUI Base][base]，`Label.kt`、`PrecisionPointer.kt` | 捕获及取消、嵌套滚动归属、roving focus、浮层避让、字体与本地化入口、语义动作。Slider 值标签属于这里。 | 文本 |

## P5 基础组件

| 交付 | 类型 / 类别 | 规范与源码 | 必需变体、状态和验收重点 | 平台 |
| --- | --- | --- | --- | --- |
| Button | component / 规范 | [Buttons](https://m3.material.io/components/buttons/overview)，`Button.kt` | Elevated、Filled、Filled Tonal、Outlined、Text，XS–XL、圆/方及可用图标槽。覆盖焦点、悬停、按下 morph、拖出与禁用，三种语义激活路径各触发一次。 | 桌面 |
| ToggleButton | component / 规范 | [Buttons](https://m3.material.io/components/buttons/overview)，`ToggleButton.kt` | 可用的 Elevated、Filled、Filled Tonal、Outlined，XS–XL、圆/方及选中形变。受控值、按下叠加和读屏切换状态一致。 | 桌面 |
| IconButton / IconToggleButton | component / 规范 | [Icon buttons](https://m3.material.io/components/icon-buttons/overview)，`IconButton.kt`、`IconButtonDefaults.kt` | Standard、Filled、Tonal、Outlined，适用尺寸、宽度与形状，图标选中轮廓/实心。验证可访问名称、可视尺寸与命中区、焦点和受控切换。 | 桌面 |
| FAB | component / 规范 | [FAB](https://m3.material.io/components/floating-action-button/overview)，`FloatingActionButton.kt` | 当前 FAB、Medium、Large，按规范色组和悬浮层级。焦点、滚动中位置、窗口避让与激活一致。 | 桌面 |
| Extended FAB | component / 规范 | [Extended FAB](https://m3.material.io/components/extended-fab/overview)，`FloatingActionButton.kt` | Small、Medium、Large，标签及图标槽、长文本及适用的收展。保持语义名称、焦点和命中连续。 | 桌面 |
| Checkbox | component / 规范 | [Checkbox](https://m3.material.io/components/checkbox/overview)，`Checkbox.kt` | 未选、选中、混合及对应错误、禁用。验证 Space、可访问混合状态和父子聚合请求。 | 桌面 |
| Radio / RadioGroup | component / 规范 | [Radio button](https://m3.material.io/components/radio-button/overview)，`RadioButton.kt` | 互斥、禁用与受控选择，方向键移动及选择，验证组关系、边界项及 RTL。 | 桌面 |
| Switch | component / 规范 | [Switch](https://m3.material.io/components/switch/overview)，`Switch.kt` | 开关、禁用及适用拇指图标。验证按下取消、拇指形态、焦点与读屏开关值。 | 桌面 |
| Chips | component / 规范 | [Chips](https://m3.material.io/components/chips/overview)，`Chip.kt` | Assist、Filter、Input、Suggestion 及适用 elevated、选中、移除。验证独立删除动作、焦点归位与禁用。 | 桌面 |
| Slider / RangeSlider | component / 规范 | [Sliders](https://m3.material.io/components/sliders/overview)，`Slider.kt` | Standard、Centered、Range，适用 XS–XL、水平/垂直与内嵌图标，连续/离散值。验证重叠拇指、卡点精度、捕获/取消、键盘、语义范围和父滚动隔离。 | 桌面 |
| Text / Icon | primitive / 适配 | [Typography](https://m3.material.io/styles/typography/overview)、[Icons](https://m3.material.io/styles/icons/overview)，`Text.kt`、`Icon.kt` | baseline/emphasized、字体回退、行高、截断、可替换图标和装饰/有意义语义。验证中文、缩放及镜像策略。 | 桌面 |
| Card | component / 规范 | [Cards](https://m3.material.io/components/cards/overview)，`Card.kt` | Elevated、Filled、Outlined，静态与可操作卡片。验证嵌套动作路由、焦点、层级及禁用视觉。 | 桌面 |
| ListItem / List 分组 | component / 规范 | [Lists](https://m3.material.io/components/lists/overview)，`ListItem.kt`、`ListItemDefaults.kt` | Standard、Segmented，单/多行、首尾槽、首/中/末组形状、交互与选中。验证长列表身份、焦点及滚动、独立子动作。 | 桌面 |
| Divider | component / 规范 | [Divider](https://m3.material.io/components/divider/overview)，`Divider.kt` | 水平、垂直与适用 inset。验证缩放下线宽、对齐及装饰语义。 | 桌面 |
| Badge | component / 规范 | [Badges](https://m3.material.io/components/badges/overview)，`Badge.kt` | 小标记、大型内容/计数、溢出显示。验证 trailing 锚点、RTL 与宿主合并名称。 | 桌面 |
| Linear / Circular Progress | component / 规范 | [Progress indicators](https://m3.material.io/components/progress-indicators/overview)，`ProgressIndicator.kt`、`WavyProgressIndicator.kt` | 线形/圆形、确定/不确定、适用平直/波形。验证范围、切换连续性、值语义和低动效。 | 桌面 |
| LoadingIndicator | component / 规范 | [Loading indicator](https://m3.material.io/components/loading-indicator/overview)，`LoadingIndicator.kt` | 默认/contained 及形变序列。验证加载通知、停止重绘和减少动态效果。 | 桌面 |

## P6 输入与浮层

| 交付 | 类型 / 类别 | 规范与源码 | 必需变体、状态和验收重点 | 平台 |
| --- | --- | --- | --- | --- |
| TextField / 安全输入 | component / 规范 | [Text fields](https://m3.material.io/components/text-fields/overview)，`TextField.kt`、`OutlinedTextField.kt`、`SecureTextField.kt` | Filled/Outlined，单行、多行、错误、辅助文字、readonly、disabled 与安全模式。验证组合文本、选区、撤销、光标、隐私、滚动及名称/描述关系。 | 文本 |
| Tooltip | component / 规范 | [Tooltips](https://m3.material.io/components/tooltips/overview)，`Tooltip.kt` | Plain/Rich，适用标题和动作。验证焦点/悬停触发、Rich 内操作、计时、避让和 Esc。 | 桌面 |
| Menu | component / 规范 | [Menus](https://m3.material.io/components/menus/overview)，`Menu.kt`、Android `AndroidMenu.android.kt` | Anchored/Context、分组、可选/可勾选、子菜单。验证次键、键盘打开、方向导航、关闭链、RTL 位置及焦点恢复。 | 桌面 |
| Dialog | component / 规范 | [Dialogs](https://m3.material.io/components/dialogs/overview)，`AlertDialog.kt` | Basic、Full-screen 适配窗口内容区，标题/说明/确认取消。验证默认动作、模态焦点、Esc、嵌套 IME 和触发点回焦。 | 文本 |
| Snackbar | component / 规范 | [Snackbar](https://m3.material.io/components/snackbar/overview)，`Snackbar.kt`、`SnackbarHost.kt` | 定时及持续、动作、关闭和队列。验证播报、定时策略、可达动作与关闭竞态。 | 桌面 |
| Bottom Sheet | component / 规范 | [Bottom sheets](https://m3.material.io/components/bottom-sheets/overview)，`BottomSheet.kt`、`BottomSheetScaffold.kt`、`ModalBottomSheet.kt` | Standard/Modal、展开/折叠及适用拖柄。验证非手势操作、拖动取消、嵌套滚动和模态回焦。 | 桌面 |
| Side Sheet | component / 适配 | [Side sheets](https://m3.material.io/components/side-sheets/overview)、[MDC SideSheet][side-sheet] | Standard/Modal，固定窗格与覆盖层、RTL 对应边。验证宽度变化、Scrim、滚动、焦点及 Esc。 | 桌面 |
| ExposedDropdown | component / 适配 | [Menus](https://m3.material.io/components/menus/overview)、[Text fields](https://m3.material.io/components/text-fields/overview)，`ExposedDropdownMenu.kt`、Android 实现 | TextField 与 Menu 完成后组合。验证可编辑及只选模式、过滤、锚点、提交/取消、IME 和展开状态播报。 | 文本 |

## P7 导航与自适应

| 交付 | 类型 / 类别 | 规范与源码 | 必需变体、状态和验收重点 | 平台 |
| --- | --- | --- | --- | --- |
| Tabs | component / 规范 | [Tabs](https://m3.material.io/components/tabs/overview)，`Tab.kt`、`TabRow.kt` | Primary/Secondary、固定/滚动、文本/图标。验证焦点、受控选中、选中项入视和 RTL。 | 桌面 |
| AppBar | component / 规范 | [App bars](https://m3.material.io/components/app-bars/overview)，`AppBar.kt` | Small 含可用居中布局、Medium/Large Flexible；为 P8 提供搜索 slot。验证收展、滚动色与长标题、导航命令。 | 桌面 |
| NavigationBar | component / 规范 | [Navigation bar](https://m3.material.io/components/navigation-bar/overview)，`ShortNavigationBar.kt`、`NavigationBar.kt` | Expressive 短式在紧凑窗口用垂直条目，中等窗口用水平条目。验证缩放、禁用、键盘移动和可访问目的地。 | 桌面 |
| NavigationRail / Expanded Rail | component / 规范 | [Navigation rail](https://m3.material.io/components/navigation-rail/overview)，`NavigationRail.kt`、`WideNavigationRail.kt` | 折叠/展开，非模态/模态、折叠时可隐藏，标题和 FAB slot。验证溢出、焦点迁移、目的地保持与 RTL。 | 桌面 |
| Scaffold | pattern / 适配 | [Scaffold](https://m3.material.io/foundations/layout/scaffold/overview)，`Scaffold.kt` | AppBar、导航、内容、FAB、Snackbar 等 slot，验证内容边界、层级和滚动所有权。 | 桌面 |
| 自适应导航 | pattern / 适配 | [Breakpoints](https://m3.material.io/foundations/layout/breakpoints/overview)，Adaptive Suite `NavigationSuiteScaffold.kt` | 依可用宽度在 Bar 和 Rail 间切换，保留目的地、状态和焦点；覆盖断点两侧及窗口往返缩放。 | 桌面 |
| 列表详情 / 支持窗格 / Feed | pattern / 适配 | [Canonical layouts](https://m3.material.io/foundations/layout/canonical-examples/overview)，Adaptive `ListDetailPaneScaffold.kt`、`SupportingPaneScaffold.kt` | 单/多窗格、列表详情返回路径、主/支持窗格、Feed 列数和滚动。验证窄宽往返、选中和滚动身份、焦点及 RTL。 | 桌面 |

## P8 复合组件与实验模式

| 交付 | 类型 / 类别 | 规范与源码 | 变体、状态和验收重点 | 平台 |
| --- | --- | --- | --- | --- |
| ButtonGroup | component / 规范 | [Button groups](https://m3.material.io/components/button-groups/overview)，`ButtonGroup.kt` | Standard/Connected，XS–XL、动作组、单选/多选/必选、适用溢出。验证相邻按压及选中形变、窄宽、方向键和选择约束。 | 桌面 |
| SplitButton | component / 规范 | [Split button](https://m3.material.io/components/split-button/overview)，`SplitButton.kt` | XS–XL、适用色组，主动作与次级菜单。验证各自标签和焦点，任一半激活时只分发对应动作，菜单回焦。 | 桌面 |
| FAB Menu | component / 规范 | [FAB menu](https://m3.material.io/components/fab-menu/overview)，`FloatingActionButtonMenu.kt` | 2–6 个动作、适用 FAB 尺寸和色组、展开/折叠。验证键盘、Esc、焦点恢复与退出动效。 | 桌面 |
| Floating / Docked Toolbar | component / 规范 | [Toolbars](https://m3.material.io/components/toolbars/overview)，`FloatingToolbar.kt`、`AppBar.kt`、[MDC Toolbar][toolbar] | Floating 横/竖、Docked、Standard/Vibrant、适用 FAB 组合及滚动收展。验证顺序、RTL、溢出时的键盘路径和导航互斥。 | 桌面 |
| SearchBar / SearchView | component / 规范 | [Search](https://m3.material.io/components/search/overview)、[App bars](https://m3.material.io/components/app-bars/overview)，`SearchBar.kt`、Android 实现 | 收起、contained 样式的 docked/full-screen 展开及搜索 AppBar 组合。验证空/加载/错误/结果、过期异步响应、IME、提交及焦点恢复。 | 文本 |
| DatePicker / DateRange | component / 规范 | [Date pickers](https://m3.material.io/components/date-pickers/overview)，`DatePicker.kt`、`DateRangePicker.kt`、`DateInput.kt`、`DateRangeInput.kt` | Docked、Modal、Modal Input，单日期/区间及禁用日期。验证跨月和边界、无效文本、地区格式、日历键盘与播报。 | 文本 |
| TimePicker / TimeInput | component / 规范 | [Time pickers](https://m3.material.io/components/time-pickers/overview)，`TimePicker.kt`、`TimePickerDialog.kt` | Dial/Input、12/24 小时、适用横竖布局。验证键盘编辑、地区格式、范围校验、确认与取消。 | 文本 |
| Carousel | component / 规范 | [Carousel](https://m3.material.io/components/carousel/overview)，`carousel/Carousel.kt`、`carousel/Strategy.kt` | Multi-browse、Uncontained、Uncontained Multi-aspect Ratio、Hero、Centered Hero、Full-screen。Compose 参考已有布局；其余以官网和几何证据验收。验证吸附、缩放、焦点项可见、RTL、键盘与低动效。 | 桌面 |
| PullToRefresh | pattern / 实验 | [Gestures](https://m3.material.io/foundations/interaction/gestures)、[Loading indicator](https://m3.material.io/components/loading-indicator/overview)，`pulltorefresh/PullToRefresh.kt` | 有实际触摸/触控板产品需求时验证滚动边界、取消、单次刷新、显式键盘动作。证据不足时维持实验状态。 | 触控 |
| SwipeToDismiss | pattern / 实验 | [Gestures](https://m3.material.io/foundations/interaction/gestures)，`SwipeToDismissBox.kt` | 可关闭对象的拖动阈值、取消、RTL 与撤销；提供同一关闭命令的键盘及语义入口。 | 触控 |
| TimeScroll | component / 实验 | `TimePicker.kt` 的 `TimeScroll`；[Time pickers](https://m3.material.io/components/time-pickers/overview)列出 Dial/Input | 独立评估滚轮、键盘、值边界和语义。首版以官网两种变体验收。 | 桌面 |

## 后续兼容及范围边界

| 项目 | 类型 / 类别 | 来源 | 后续范围与验收方向 |
| --- | --- | --- | --- |
| Segmented Buttons | component / 兼容 | [Segmented buttons](https://m3.material.io/components/segmented-buttons/overview)、[MDC ToggleButtonGroup][toggle-group]，`SegmentedButton.kt` | 后续有迁移需求时评估单选/多选与旧布局，当前迁移目的地为 P8 Connected ButtonGroup。桌面。 |
| Navigation Drawer | component / 兼容 | [Navigation drawer](https://m3.material.io/components/navigation-drawer/overview)、[MDC NavigationDrawer][drawer]，`NavigationDrawer.kt` | 后续按需验证 standard/modal 导航迁移及焦点。当前迁移目的地为 P7 Expanded Rail。桌面。 |
| Baseline Bottom App Bar、Medium/Large AppBar | component / 兼容 | [App bars](https://m3.material.io/components/app-bars/overview)、[Toolbars](https://m3.material.io/components/toolbars/overview)，`AppBar.kt` | 后续评估旧布局与行为的迁移。当前用 Docked Toolbar 或 Flexible AppBar 对应组合。桌面。 |
| Baseline NavigationBar/Rail、Small/Surface FAB、Divided Search | component / 兼容 | [Navigation bar](https://m3.material.io/components/navigation-bar/overview)、[Navigation rail](https://m3.material.io/components/navigation-rail/overview)、[FAB](https://m3.material.io/components/floating-action-button/overview)、[Search](https://m3.material.io/components/search/overview) | 后续根据迁移需求逐项验证旧尺寸、颜色、标签、选中及位置。Extended FAB 的 Expressive Small 属于 P5。桌面。 |
| XR 变体 | pattern / 实验 | [Material XR components](https://m3.material.io/foundations/xr/components/overview) | 首版平台限定为平面桌面窗口。空间交互、距离和输入设备需独立验证环境及阶段。 |

All buttons 是导航索引。独立公开的组件按其规范单列，DragHandle、Label、定位或布局 helper 在所属原语或组件验收中覆盖。通用应用导航、结果数据源和业务校验由调用者提供。

[base]: https://github.com/longbridge/gpui-kit/tree/39ab849a93b5f15507b969164d49891db8d5b470/crates/base
[side-sheet]: https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/SideSheet.md
[toolbar]: https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/DockedToolbar.md
[toggle-group]: https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/ToggleButtonGroup.md
[drawer]: https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/NavigationDrawer.md

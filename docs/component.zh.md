# 组件

[English](component.md) | 中文

下列组件和布局可从 `gpui_material` 使用。[可选组件](#可选组件)需要启用对应特性。安装方式和平台支持见[集成指南](integration.zh.md)，也可以运行 `cargo gallery` 体验交互示例。

项目以[锁定的 Material 3 规范](design-sources.zh.md#material-specification)为设计依据。下表链接指向官方指南，项目采用的版本见[站点快照索引](material-snapshot.zh.md)。

<a id="actions-and-selection"></a>

## 操作与选择

| 组件 | 支持的变体与行为 | Material 3 |
| --- | --- | --- |
| `Button`、`ToggleButton` | Elevated、Filled、Filled Tonal、Outlined。`Button` 还提供 Text 变体。两者支持 XS–XL 尺寸、圆形和方形外观、自定义内容和样式，以及可选的交互观察。 | [按钮](https://m3.material.io/components/buttons/overview) |
| `IconButton`、`IconToggleButton` | Standard、Filled、Filled Tonal、Outlined，可选尺寸、宽度及形状。 | [图标按钮](https://m3.material.io/components/icon-buttons/overview) |
| `Fab`、`ExtendedFab` | Small、Medium、Large 尺寸和多种颜色角色。`ExtendedFab` 支持文字标签和图标。 | [悬浮操作按钮](https://m3.material.io/components/floating-action-button/overview) · [扩展悬浮操作按钮](https://m3.material.io/components/extended-fab/overview) |
| `Checkbox` | 未选中、选中和混合状态。 | [复选框](https://m3.material.io/components/checkbox/overview) |
| `Radio`、`RadioGroup` | 互斥选择，支持组内键盘导航。 | [单选按钮](https://m3.material.io/components/radio-button/overview) |
| `Switch` | 开启和关闭状态，可选滑块图标。 | [开关](https://m3.material.io/components/switch/overview) |
| `Chip` | Assist、Filter、Input、Suggestion，适用的变体可选中或移除。 | [标签组件](https://m3.material.io/components/chips/overview) |
| `Slider` | 单值、居中和范围滑块，支持横向或纵向布局，以及连续值或步进值。 | [滑块](https://m3.material.io/components/sliders/overview) |

使用 `.content(element)` 自定义显示的标签。构造器提供的无障碍名称和前后图标插槽仍然生效。内容中应避免放置独立控件，由按钮负责激活和焦点。可通过 `.shapes(ButtonShapes)`、`.colors(ButtonColors)`、`.border(ButtonBorder)`、`.content_padding(ButtonPadding)` 和 `.elevations(ButtonElevations)` 配置单个实例。未指定的形状、颜色和 elevation 字段沿用对应状态的尺寸或变体默认值。`ButtonPadding` 随窗口文本方向调整。零宽度边框会移除默认描边。

调用方持有 `Entity<InteractionSource>`，通过 `.interaction_source(&source)` 观察按钮。同一窗口内共享一个源的控件应使用各自的元素 ID。用 `InteractionSource::state` 读取当前状态，或订阅 `InteractionUpdate` 逐次接收延后派发的变更，包括两次绘制之间发生的按下和释放。控件移除后，其状态快照会清除。Base 管理输入，应用持有切换值。

<a id="content-and-feedback"></a>

## 内容与反馈

| 组件 | 支持的变体与行为 | Material 3 |
| --- | --- | --- |
| `Text`、`Icon` | 主题排版、可替换图标和随库提供的 Material Symbols。 | [排版](https://m3.material.io/styles/typography/overview) · [图标](https://m3.material.io/styles/icons/overview) |
| `Surface` | 为应用内容提供主题背景和色调层级。 | [层级](https://m3.material.io/styles/elevation/overview) |
| `Card` | Elevated、Filled、Outlined，支持静态或可操作的卡片。 | [卡片](https://m3.material.io/components/cards/overview) |
| `List`、`ListItem` | Standard 和 Segmented 列表，支持前置、辅助及尾部内容。 | [列表](https://m3.material.io/components/lists/overview) |
| `Divider`、`Badge` | 横向与纵向分隔线支持逻辑内缩、粗细和单独配色。徽标支持圆点、内容及计数。 | [分隔线](https://m3.material.io/components/divider/overview) · [徽标](https://m3.material.io/components/badges/overview) |
| `LinearProgress`、`CircularProgress` | 确定和不确定进度，支持直线和波浪样式。 | [进度指示器](https://m3.material.io/components/progress-indicators/overview) |
| `LoadingIndicator` | 默认样式和带容器的加载指示器。 | [加载指示器](https://m3.material.io/components/loading-indicator/overview) |
| `Snackbar`、`SnackbarHost` | 消息排队、可选操作、定时或持续显示以及关闭。 | [消息条](https://m3.material.io/components/snackbar/overview) |

### 分隔线

`Divider::horizontal()` 和 `Divider::vertical()` 默认使用窗口主题的 `OutlineVariant` 颜色，厚度为一个逻辑像素。通过 `.thickness(px(2.))` 设置逻辑像素厚度，通过 `.thickness(px(0.))` 绘制一个物理像素的细线。无效的厚度值回退到默认值。纵向分隔线填满父布局的可用高度。

使用 `.color(ColorRole::Primary)` 按窗口主题解析颜色角色，或使用 `.color_override(hsla)` 指定固定颜色。颜色设置以最后一次调用为准。`.inset(start, end)` 在横向布局中随窗口文字方向调整左右内缩，在纵向布局中分别设置顶部和底部内缩。分隔线是装饰元素，不增加键盘焦点。

<a id="input-and-overlays"></a>

## 输入与浮层

| 组件 | 支持的变体与行为 | Material 3 |
| --- | --- | --- |
| `TextField` | Filled、Outlined，支持单行、多行、安全输入、只读和禁用。支持标签位置、装饰和说明插槽、必填与无效状态以及字符计数。 | [文本字段](https://m3.material.io/components/text-fields/overview) |
| `ExposedDropdown` | 锚定到文本字段的下拉列表，支持可编辑筛选和仅选择模式。 | [菜单](https://m3.material.io/components/menus/overview) · [文本字段](https://m3.material.io/components/text-fields/overview) |
| `Menu` | 锚定菜单和上下文菜单，支持分组、勾选项及子菜单。 | [菜单](https://m3.material.io/components/menus/overview) |
| `Tooltip` | 普通和富内容提示，通过鼠标悬停或键盘焦点触发。 | [提示](https://m3.material.io/components/tooltips/overview) |
| `Scrim` | 独立遮罩，支持主题配色、自定义颜色、透明度和可选关闭操作。 | [遮罩](https://developer.android.com/reference/kotlin/androidx/compose/material3/Scrim.composable) |
| `Dialog` | 基本对话框和全屏任务对话框。后者在紧凑布局中占用窗口内容区。 | [对话框](https://m3.material.io/components/dialogs/overview) |
| `Sheet` | 标准和模态底部面板，以及在逻辑起始侧或末尾侧停靠或覆盖的侧面板。 | [底部面板](https://m3.material.io/components/bottom-sheets/overview) · [侧面板](https://m3.material.io/components/side-sheets/overview) |

`TextField` 为调用方持有的 Base 编辑器提供外观。使用 `label_content`、`placeholder_content` 和 `supporting_content` 时，纯文本分别提供无障碍名称、提示和说明。首尾元素与紧邻输入值的前后缀分开。标签收起时显示前后缀。Filled 默认使用内部标签，Outlined 默认使用缺口标签。可通过 Above 将标签放在容器上方，也可设置水平对齐方式，让内部或缺口标签始终保持收起。字符计数按 Unicode 标量值统计，输入长度仍由编辑器处理。安全字段隐藏计数和无障碍值。可独立于窗口主题设置实例配色、容器形状、内容内边距和输入文字样式。`on_text_layout` 在帧结束后报告编辑器与文本的布局边界和换行后的行数。通过 `interaction_source` 共享悬停、键盘焦点和禁用状态的观测结果。Base 实体继续管理编辑、选区、校验、滚动和输入法组合。

### 菜单开关和菜单项内容

`Menu::new` 默认自行管理显示状态。使用 `.open(open)` 和 `.on_open_change(|requested, window, cx| ...)` 时，由调用者持有状态。调用者可接受请求、更新状态并重新渲染。调用者主动更新 `open` 会同步显示和焦点，不会触发变更回调。`.on_dismiss(|window, cx| ...)` 报告用户操作发出的关闭请求。`Menu::composed` 向自定义触发控件提供当前显示状态及激活回调。

为每个 `MenuItem` 提供稳定的 ID 和无障碍标签。`.content`、`.leading`、`.supporting`、`.trailing` 和 `.selection_icon` 接收可重复创建视觉元素的闭包。键盘和指针操作由菜单行处理。用 `.checked(value)` 提供勾选状态，通过 `.on_checked_change` 处理请求。选中项继续用 `.selected(value)` 和 `.on_activate`。

使用 `.color(MenuColor::Vibrant)` 启用第三色菜单配色。`MenuItem::section(id, name, rows)` 绘制分组表面，方向键仍按菜单顺序遍历其中的项目。`Menu::content` 在菜单项前添加调用方提供的内容。分组的 `.content` 可以替换可见标题。

使用 `MenuSurfaceStyle` 配合 `Menu::appearance` 或 `MenuItem::group_appearance` 设置表面形状、内边距、间距、边框和高度。`MenuItemStyle` 可覆盖菜单行的形状、间距、内边距和颜色角色。使用 `.anchor` 和 `.offset` 相对触发控件定位，或用 `.position` 指定窗口坐标。`.track_scroll` 接收调用方持有的滚动句柄。

### 遮罩

在已定位的容器中，将 `Scrim::new(id)` 放在背景内容之后、前景表面之前。遮罩填满容器，默认使用当前窗口主题的 Scrim 色彩角色和 32% 不透明度。使用 `.color(color)` 指定自定义颜色，用 `.opacity(value)` 乘以该颜色的 alpha 值。数值限制在 0–1，NaN 使用默认值。

默认遮罩仅作装饰，降低内容亮度，并保留底下内容的指针操作和无障碍遍历。使用 `.on_click(|window, cx| ...)` 后，遮罩拦截指向底下内容的指针事件，并响应指针或辅助技术的激活操作。遮罩不占用键盘 Tab 焦点。使用 `.accessibility_label(label)` 为辅助技术提供操作名称，默认名称为 “Dismiss overlay”。显示状态、关闭操作，以及模态内容的焦点与无障碍范围均由调用方管理。模态内容需要提供自己的键盘关闭操作。

<a id="navigation-and-adaptive-layout"></a>

## 导航与自适应布局

| 组件 | 支持的变体与行为 | Material 3 |
| --- | --- | --- |
| `AppBar` | Small、Centered、Medium Flexible 和 Large Flexible 顶部应用栏。Flexible 变体可随滚动收起。 | [应用栏](https://m3.material.io/components/app-bars/overview) |
| `Tabs` | Primary、Secondary，可固定或滚动。 | [标签页](https://m3.material.io/components/tabs/overview) |
| `NavigationBar`、`NavigationRail` | 短导航栏支持纵向或横向排列选项。导航轨道支持折叠、展开、隐藏和模态呈现。 | [导航栏](https://m3.material.io/components/navigation-bar/overview) · [导航轨道](https://m3.material.io/components/navigation-rail/overview) |
| `AdaptiveNavigation` | 根据可用宽度在导航栏和导航轨道之间切换。 | [断点](https://m3.material.io/foundations/layout/breakpoints/overview) |
| `Scaffold` | 组合导航、应用栏、内容、悬浮操作按钮和消息条。 | [Scaffold](https://m3.material.io/foundations/layout/scaffold/overview) |
| `ListDetail`、`SupportingPane`、`Feed` | 列表与详情、辅助窗格及响应式信息流布局。 | [典型布局](https://m3.material.io/foundations/layout/canonical-examples/overview) |

<a id="optional-components"></a>

## 可选组件

在 `gpui-material` 依赖上启用 feature。`DatePicker` 和 `TimePicker` 从 `gpui_material::date_time` 导入。

| Feature | 组件 | 支持的变体与行为 | Material 3 |
| --- | --- | --- | --- |
| `expressive-groups` | `ButtonGroup`、`SplitButton` | Standard 和 Connected 按钮组，以及主操作与次操作独立的拆分按钮。 | [按钮组](https://m3.material.io/components/button-groups/overview) · [拆分按钮](https://m3.material.io/components/split-button/overview) |
| `toolbars` | `FabMenu`、`Toolbar` | 可展开的悬浮操作菜单。工具栏支持悬浮或停靠，以及横向或纵向布局。 | [悬浮操作菜单](https://m3.material.io/components/fab-menu/overview) · [工具栏](https://m3.material.io/components/toolbars/overview) |
| `search` | `SearchBar`、`SearchView` | 收起的搜索栏，以及停靠或全屏的搜索视图。 | [搜索](https://m3.material.io/components/search/overview) |
| `date-time` | `DatePicker`、`TimePicker` | 单日期和日期范围选择，支持 Docked、Modal、Modal Input。时间选择支持 Dial、Input 和 12、24 小时制。 | [日期选择](https://m3.material.io/components/date-pickers/overview) · [时间选择](https://m3.material.io/components/time-pickers/overview) |
| `carousel` | `Carousel` | Multi-browse、Uncontained、Uncontained Multi-aspect Ratio、Hero、Centered Hero、Full-screen。 | [轮播](https://m3.material.io/components/carousel/overview) |

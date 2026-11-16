# 组件

[English](component.md) | 中文

下列组件和布局可从 `gpui_material` 使用。[可选组件](#可选组件)需要启用对应特性。安装方式和平台支持见[集成指南](integration.zh.md)，也可以运行 `cargo gallery` 体验交互示例。

项目以[锁定的 Material 3 规范](design-sources.zh.md#material-specification)为设计依据。下表链接指向官方指南，项目采用的版本见[站点快照索引](material-snapshot.zh.md)。

<a id="actions-and-selection"></a>

## 操作与选择

| 组件 | 支持的变体与行为 | Material 3 |
| --- | --- | --- |
| `Button`、`ToggleButton` | Elevated、Filled、Filled Tonal、Outlined。`Button` 还提供 Text 变体。两者支持 XS–XL 尺寸、圆形和方形外观、自定义内容和样式，以及可选的交互观察。 | [按钮](https://m3.material.io/components/buttons/overview) |
| `IconButton`、`IconToggleButton` | Standard、Filled、Filled Tonal、Outlined，可选尺寸和宽度，可为单个实例配置形状、颜色和描边，并可观察交互。 | [图标按钮](https://m3.material.io/components/icon-buttons/overview) |
| `Fab`、`ExtendedFab` | Small、Medium、Large 尺寸和多种颜色角色。`ExtendedFab` 支持文字标签和图标。 | [悬浮操作按钮](https://m3.material.io/components/floating-action-button/overview) · [扩展悬浮操作按钮](https://m3.material.io/components/extended-fab/overview) |
| `Checkbox` | 未选中、选中、混合、错误和禁用状态，可配置实例颜色和描边并观察交互。 | [复选框](https://m3.material.io/components/checkbox/overview) |
| `Radio`、`RadioGroup` | 互斥选择、组内键盘导航、单项颜色配置和可选的交互观察。 | [单选按钮](https://m3.material.io/components/radio-button/specs) |
| `Switch` | 开启和关闭状态，可选滑块图标。 | [开关](https://m3.material.io/components/switch/overview) |
| `Chip` | Assist、Filter、Input、Suggestion，支持内容插槽和自定义外观，适用的变体支持独立选中和移除。 | [标签组件](https://m3.material.io/components/chips/overview) |
| `Slider` | 单值、居中和范围滑块，支持横向或纵向布局，以及连续值或步进值。 | [滑块](https://m3.material.io/components/sliders/overview) |

使用 `.content(element)` 自定义显示的标签。构造器提供的无障碍名称和前后图标插槽仍然生效。内容中应避免放置独立控件，由按钮负责激活和焦点。可通过 `.shapes(ButtonShapes)`、`.colors(ButtonColors)`、`.border(ButtonBorder)`、`.content_padding(ButtonPadding)` 和 `.elevations(ButtonElevations)` 配置单个实例。未指定的形状、颜色和 elevation 字段沿用对应状态的尺寸或变体默认值。`ButtonPadding` 随窗口文本方向调整。零宽度边框会移除默认描边。

构造 `IconButton` 或 `IconToggleButton` 时传入装饰性图标元素和无障碍名称。图标按钮默认使用 Small 尺寸、Uniform 宽度、Round 形状和 Standard 配色。使用 `.shapes(ButtonShapes)` 设置静止和按下时的圆角，以及切换按钮选中时的圆角。使用 `.colors(ButtonColors)` 设置启用、禁用和选中状态下的容器与图标颜色。省略的字段沿用当前尺寸和变体的默认值。`.border(ButtonBorder)` 替换默认描边，宽度为零时移除描边。`IconToggleButton` 根据调用方传入的值渲染选中状态，并在选中时使用可选的 `.selected_icon(element)`。

调用方持有 `Entity<InteractionSource>`，通过 `.interaction_source(&source)` 观察按钮或图标按钮。同一窗口内共享一个源的控件应使用各自的元素 ID。用 `InteractionSource::state` 读取当前状态，或订阅 `InteractionUpdate` 逐次接收延后派发的变更，包括两次绘制之间发生的按下和释放。控件移除后，其状态快照会清除。Base 管理输入，应用持有切换值。

### 悬浮操作按钮

`Fab::new` 接收一个图标和无障碍名称，默认使用 56 dp 的 Small 尺寸和主色容器配色。可通过 `.shape(CornerShape)`、`.colors(ButtonColors)` 和 `.elevations(ButtonElevations)` 分别配置每个 Small、Medium 或 Large 实例。用启用和禁用状态的容器与内容字段设置颜色，省略的字段沿用对应尺寸和配色的默认值。用 `.interaction_source(&source)` 连接由调用方持有的交互源。

应用持有显示状态，每次渲染时通过 `.visible(visible)` 传入。更新时保持元素 ID 不变，缩放和透明度即可从当前进度反向播放。`FabVisibilityAnimation` 默认以中心为缩放原点，隐藏比例为 `0.2`，并分别使用主题的快档空间动效和效果动效。用 `FabAlignment` 的逻辑位置指定缩放所围绕的边或角，起始侧与末尾侧随窗口文本方向镜像。将 `target_scale` 设为有限非负值，并分别设置 `scale_motion` 和 `alpha_motion`。无效比例（包括导致缩放后尺寸溢出的值）回退到 `0.2`。启用减少动态效果时直接使用最终状态。

FAB 开始隐藏时立即退出焦点导航并停止响应输入。淡出期间保留布局空间，透明度降到零时释放空间。初始隐藏的 FAB 不占空间。图标仅用于装饰，由 FAB 自己管理激活和焦点。需要随动效缩放的自定义图标应采用继承的文字大小或相对尺寸。

### 复选框

`Checkbox::new(id, label, state)` 渲染调用方传入的状态。指针、键盘和无障碍激活通过 `.on_change` 请求下一状态。激活混合状态的复选框会请求 `Checked`。由调用方处理请求，并在下一次渲染传入当前值。`.error(true)` 公开校验错误状态，默认使用 `Error` 和 `OnError`。禁用外观优先于错误外观。

通过 `.colors(CheckboxColors)` 为未选中、选中和混合状态分别提供 `CheckboxStateColors`，也可配置对应的错误和禁用状态。每种状态的 `checkmark`、`box_color` 和 `border` 可独立覆盖。省略的字段沿用窗口主题默认值。未选中状态的标记颜色用于标记收回的动效。

`.checkmark_stroke(gpui::StrokeOptions)` 设置勾选和混合标记的等宽描边，包括宽度、端点和连接样式。用 `.outline_stroke(width)` 设置圆角方框轮廓的逻辑像素宽度。两者默认均为两个逻辑像素。零宽度移除对应描边，负数或非有限宽度恢复默认值，过大的宽度以指示图形尺寸为限。使用调用方持有的 `Entity<InteractionSource>` 和 `.interaction_source(&source)` 观察悬停、按下、键盘焦点和禁用状态。各控件使用不同的 ID 即可共享交互源，选中值仍由调用方持有。

### 单选按钮

使用 `Radio::colors(RadioColors)` 分别覆盖选中、未选中、禁用且选中、禁用且未选中时的指示器颜色。省略的字段沿用当前窗口配色。选中默认使用 Primary，未选中使用 On Surface Variant，两种禁用状态默认均使用 38% 不透明度的 On Surface。

传入 `.interaction_source(&source)` 可观察独立的单选按钮或 `RadioGroup` 中的单项。组内快照使用渲染标识 `ElementId::from((group_id, option_id.to_string()))`，不同组可共享同一个 `Entity<InteractionSource>`。移除选项或更换其交互源时，原交互源中的快照会清除。选择状态由调用方持有，并根据 `.on_select(...)` 请求更新。每组有一个 Tab 焦点，导航会跳过禁用选项，并在 RTL 中镜像横向方向键的行为。

### 标签组件

`Chip::new(id, label)` 将标签用作无障碍名称。所有变体均可用 `.content(element)` 替换显示标签，用 `.leading_icon(element)` 设置前置图标。Input 可用 `.avatar(element)` 设置 24 dp 圆形头像插槽，头像优先于前置图标。`.trailing(element)` 为 Assist、Filter 或 Input 添加尾部图形。Filter 或 Input 设置 `.on_remove(...)` 后，该图形由移除按钮显示，移除操作仍有独立的无障碍名称和焦点。未提供自定义尾部图形时，移除按钮显示 Close 图标。各视觉插槽应避免嵌套独立控件。

Filter 和 Input 渲染调用方的 `.selected(value)`，并通过 `.on_change` 请求变更。Assist 和 Suggestion 使用 `.on_activate`。移除只调用移除回调。`.disabled(true)` 同时禁用两个操作。前置和尾部内容以及移除按钮在 RTL 布局中镜像。键盘先访问主操作，再访问移除操作。

`.tonal(true)` 为 Filter、Elevated Filter 和 Input 选用 Expressive Tonal 默认外观。静止圆角为 Medium，选中为 Full，按下为 Small，未选中的前置图标使用主题的 `OnSurfaceVariant`。普通构造器沿用现有的平面 Assist 和 Small 圆角。用 `.shapes(ButtonShapes)` 按状态设置圆角。用 `.colors(ChipColors)` 为启用、选中、禁用或禁用且选中的外观传入 `ChipStateColors`，每组可设置容器、标签、前置、尾部和边框颜色。省略的字段沿用所选变体默认值，禁用且选中的颜色优先回退到禁用覆盖值。

`.border(ButtonBorder)` 设置轮廓宽度和回退颜色。零宽度隐藏轮廓。`.elevations(ChipElevations)` 可覆盖静止、按下、焦点、悬停、拖动和禁用时的阴影层级。`.dragged(bool)` 显示由应用持有的拖动状态，同时取消尚未完成的指针激活；手势由应用管理。`.spacing(width)` 设置内容插槽之间的水平间距，`.content_padding(ButtonPadding)` 设置逻辑内边距。默认间距为 8 dp，纯标签边缘为 16 dp，图标侧为 8 dp，头像起始侧为 4 dp。

为各标签提供不同的 ID，再用调用方持有的 `Entity<InteractionSource>` 和 `.interaction_source(&source)` 按该 ID 观察主操作。交互源报告悬停、按下、键盘焦点、拖动和禁用反馈；选中与移除仍由调用方控制。

<a id="content-and-feedback"></a>

## 内容与反馈

| 组件 | 支持的变体与行为 | Material 3 |
| --- | --- | --- |
| `Text`、`Icon` | 主题排版、可替换图标和随库提供的 Material Symbols。 | [排版](https://m3.material.io/styles/typography/overview) · [图标](https://m3.material.io/styles/icons/overview) |
| `Surface` | 为应用内容提供主题背景和色调层级。 | [层级](https://m3.material.io/styles/elevation/overview) |
| `Card` | Elevated、Filled、Outlined，支持静态或可操作的卡片及独立外观配置。 | [卡片](https://m3.material.io/components/cards/specs) |
| `List`、`ListItem` | Standard 和 Segmented 列表，支持前置、辅助及尾部内容。 | [列表](https://m3.material.io/components/lists/overview) |
| `SwipeToDismiss` | 横向滑动关闭，内容、背景及操作由调用方提供。 | [列表](https://m3.material.io/components/lists/guidelines) · [Compose SwipeToDismissBox](https://developer.android.com/reference/kotlin/androidx/compose/material3/SwipeToDismissBox.composable) |
| `Divider`、`Badge` | 横向与纵向分隔线支持逻辑内缩、粗细和单独配色。徽标支持独立或附着显示，可使用圆点、文本、计数、自定义内容和单独配色。 | [分隔线](https://m3.material.io/components/divider/overview) · [徽标](https://m3.material.io/components/badges/overview) |
| `LinearProgress`、`CircularProgress` | 确定和不确定进度，支持直线和波浪样式。 | [进度指示器](https://m3.material.io/components/progress-indicators/overview) |
| `LoadingIndicator` | Default 和 Contained 样式，支持确定与不确定进度、自定义形状序列和实例配色。 | [加载指示器](https://m3.material.io/components/loading-indicator/overview) |
| `Snackbar`、`SnackbarHost` | 消息排队、可选操作、定时或持续显示以及关闭。 | [消息条](https://m3.material.io/components/snackbar/overview) |

### 卡片

通过 `Card::new(id).variant(CardVariant::Elevated)`、`Filled` 或 `Outlined` 选择当前 Material 变体。静态卡片将内容分组，不占用键盘焦点。使用 `.on_activate(name, callback)` 可添加有名称的卡片操作及键盘焦点，子控件仍各自处理操作。`.disabled(true)` 禁用卡片操作，并为卡片和继承内容提供禁用颜色。

使用 `.shape(CornerShape)` 设置容器、描边、阴影和交互反馈的形状，圆角随窗口书写方向调整。通过 `.border(CardBorder { width, color })` 自定义描边，宽度为零时移除描边。`.colors(CardColors)` 独立覆盖启用和禁用时的容器色与内容色。`.elevations(CardElevations)` 独立覆盖静止、按下、聚焦、悬停、拖动和禁用时的阴影高度。未指定的值沿用变体默认值，包括 Medium 圆角、Outlined 描边以及 Outlined 悬停时的平坦外观。禁用高度优先，其次依次为拖动、按下、键盘聚焦、悬停和静止。

调用方可根据拖动生命周期设置 `.dragged(bool)`，并在完成或取消时清除该状态。拖动会中断尚未完成的指针激活。传入 `.interaction_source(&source)` 可通过共享的 `Entity<InteractionSource>` 观察卡片。同一窗口中每张受观察的卡片需要有独立标识。

### 分隔线

`Divider::horizontal()` 和 `Divider::vertical()` 默认使用窗口主题的 `OutlineVariant` 颜色，厚度为一个逻辑像素。通过 `.thickness(px(2.))` 设置逻辑像素厚度，通过 `.thickness(px(0.))` 绘制一个物理像素的细线。无效的厚度值回退到默认值。纵向分隔线填满父布局的可用高度。

使用 `.color(ColorRole::Primary)` 按窗口主题解析颜色角色，或使用 `.color_override(hsla)` 指定固定颜色。颜色设置以最后一次调用为准。`.inset(start, end)` 在横向布局中随窗口文字方向调整左右内缩，在纵向布局中分别设置顶部和底部内缩。分隔线是装饰元素，不增加键盘焦点。

### 徽标

`Badge::dot(host)`、`Badge::label(host, text)` 和 `Badge::count(host, count, maximum)` 将徽标附着在宿主的右上角，RTL 布局下则附着在左上角。渲染时传入当前计数和上限。计数超过上限时，显示上限并追加 `+`。`Badge::standalone()` 在当前布局中独立绘制圆点。两种放置方式均可通过 `.content(id, element)` 提供简短的自定义视觉元素。请为同一父容器中的自定义徽标提供稳定标识。

徽标容器和内容默认使用窗口主题的 `Error` 和 `OnError` 颜色角色，可通过 `.container_color(color)` 和 `.content_color(color)` 分别覆盖。自定义内容应为非交互元素，附着徽标的信息由宿主的无障碍名称提供。独立传达信息的徽标可使用 `.accessible_label(id, label)` 提供无障碍名称，无需增加键盘焦点。

### 滑动关闭

为每项保留独立的 `Entity<SwipeToDismissState>`，传给 `SwipeToDismiss::new(id, &state, content)`。状态默认从初始位置开始。恢复已滑出条目时，可用 `.initial_value(value)` 指定初始锚点。通过 `.background(element)` 设置填满容器的背景，显示在移动的前景内容后面。观察状态后，可用逻辑方向 `dismiss_direction()`、带符号的逻辑像素偏移 `offset()` 或从 0 到 1 的滑出进度 `progress()` 更新背景。`current_value()` 表示最接近的锚点，`target_value()` 表示预计停留的锚点，`settled_value()` 在移动完成后更新。

默认启用两个逻辑方向和滑动手势。使用 `.enable_start_to_end(bool)`、`.enable_end_to_start(bool)` 和 `.gestures_enabled(bool)` 可分别设置。窗口的书写方向决定实际滑动方向。默认阈值为 56 个逻辑像素。使用 `SwipeToDismissState::positional_threshold(|width| ...)` 按布局宽度计算阈值。有限值限制在可滑动距离内，无效值使用默认阈值。纵向拖动和滚动仍交给周围布局处理。

`dismiss(direction, window, cx)` 以动画滑至启用的方向，`reset(window, cx)` 以动画回到初始位置，`snap_to(value, window, cx)` 立即移动。减少动态效果时立即定位。移动停在新的关闭方向时，`.on_dismiss(|direction, window, cx| ...)` 执行一次，前景同时退出键盘遍历和无障碍树。数据变更由应用负责，例如移除条目。应用也可调用 `reset` 重新显示。为每个操作提供有名称且能获得键盘焦点的按钮或菜单项。激活时调用同一个 `dismiss` 方法，在手势禁用时也可以使用。

### 加载指示器

`LoadingIndicator::new(id, label)` 占用 48 dp 的空间，默认循环播放七种形状，表示不确定进度。用 `.contained(true)` 添加圆形容器。Default 的指示形状使用窗口的 `Primary` 色；Contained 使用 `OnPrimaryContainer` 绘制指示形状，使用 `PrimaryContainer` 绘制容器。两种样式均可用 `.indicator_color(color)` 覆盖指示形状的颜色。Contained 可用 `.container_color(color)` 和 `.container_shape(CornerShape)` 配置容器。起始侧和末尾侧的圆角随窗口布局方向调整。

确定进度由应用持有，每次渲染时用 `.value(fraction)` 传入。进度值限制在 `0..=1`，NaN 按零处理。默认序列在零时显示圆形，在一时显示 Soft Burst。用至少两个 `LoadingIndicatorShape` 构造可复用的 `LoadingIndicatorShapes`，再用 `.shapes(&shapes)` 传引用。使用 `LoadingIndicatorShape::circle()`，或用 `LoadingIndicatorShape::rounded_polygon(&vertices, radius)` 传入 GPUI 顶点和同单位的半径。多边形坐标或半径非有限、形状退化或自相交时返回 `None`。形状少于两个时，序列构造器返回 `None`。确定进度从序列首形状变换到末形状，不确定进度还会从末形状变回首形状。

标签为进度指示器提供无障碍名称。确定模式公开数值百分比并直接绘制传入的进度。不确定模式提供礼貌播报的状态，无数值。减少动态效果时，不确定模式停留在首形状，确定模式继续响应进度变化。`.active(false)` 保留布局空间，同时隐藏图形和无障碍进度信息。只有活动窗口内处于活动状态的不确定指示器才会请求动画帧。

<a id="input-and-overlays"></a>

## 输入与浮层

| 组件 | 支持的变体与行为 | Material 3 |
| --- | --- | --- |
| `TextField` | Filled、Outlined，支持单行、多行、安全输入、只读和禁用。支持标签位置、装饰和说明插槽、必填与无效状态以及字符计数。 | [文本字段](https://m3.material.io/components/text-fields/overview) |
| `ExposedDropdown` | 锚定到文本字段的下拉列表，支持可编辑筛选和仅选择模式。 | [菜单](https://m3.material.io/components/menus/overview) · [文本字段](https://m3.material.io/components/text-fields/overview) |
| `Menu` | 锚定菜单和上下文菜单，支持分组、勾选项及子菜单。 | [菜单](https://m3.material.io/components/menus/overview) |
| `Tooltip` | 临时普通提示、临时富内容提示和持久富内容提示。调用方可持有显示状态并自定义内容、定位及外观。 | [提示](https://m3.material.io/components/tooltips/overview) |
| `Scrim` | 独立遮罩，支持主题配色、自定义颜色、透明度和可选关闭操作。 | [遮罩](https://developer.android.com/reference/kotlin/androidx/compose/material3/Scrim.composable) |
| `Dialog` | 基本对话框和全屏任务对话框。后者在紧凑布局中占用窗口内容区。 | [对话框](https://m3.material.io/components/dialogs/overview) |
| `Sheet` | 标准和模态底部面板，以及在逻辑起始侧或末尾侧停靠或覆盖的侧面板。 | [底部面板](https://m3.material.io/components/bottom-sheets/overview) · [侧面板](https://m3.material.io/components/side-sheets/overview) |

`TextField` 为调用方持有的 Base 编辑器提供外观。使用 `label_content`、`placeholder_content` 和 `supporting_content` 时，纯文本分别提供无障碍名称、提示和说明。首尾元素与紧邻输入值的前后缀分开。标签收起时显示前后缀。Filled 默认使用内部标签，Outlined 默认使用缺口标签。可通过 Above 将标签放在容器上方，也可设置水平对齐方式，让内部或缺口标签始终保持收起。字符计数按 Unicode 标量值统计，输入长度仍由编辑器处理。安全字段隐藏计数和无障碍值。可独立于窗口主题设置实例配色、容器形状、内容内边距和输入文字样式。`on_text_layout` 在帧结束后报告编辑器与文本的布局边界和换行后的行数。通过 `interaction_source` 共享悬停、键盘焦点和禁用状态的观测结果。Base 实体继续管理编辑、选区、校验、滚动和输入法组合。

### 菜单开关和菜单项内容

`Menu::new` 默认自行管理显示状态。使用 `.open(open)` 和 `.on_open_change(|requested, window, cx| ...)` 时，由调用者持有状态。调用者可接受请求、更新状态并重新渲染。调用者主动更新 `open` 会同步显示和焦点，不会触发变更回调。`.on_dismiss(|window, cx| ...)` 报告用户操作发出的关闭请求。`Menu::composed` 向自定义触发控件提供当前显示状态及激活回调。

为每个 `MenuItem` 提供稳定的 ID 和无障碍标签。`.content`、`.leading`、`.supporting`、`.trailing` 和 `.selection_icon` 接收可重复创建视觉元素的闭包。键盘和指针操作由菜单行处理。用 `.checked(value)` 提供勾选状态，通过 `.on_checked_change` 处理请求。选中项继续用 `.selected(value)` 和 `.on_activate`。

使用 `.color(MenuColor::Vibrant)` 启用第三色菜单配色。`MenuItem::section(id, name, rows)` 绘制分组表面，方向键仍按菜单顺序遍历其中的项目。`Menu::content` 在菜单项前添加调用方提供的内容。分组的 `.content` 可以替换可见标题。

使用 `MenuSurfaceStyle` 配合 `Menu::appearance` 或 `MenuItem::group_appearance` 设置表面形状、内边距、间距、边框和高度。`MenuItemStyle` 可覆盖菜单行的形状、间距、内边距和颜色角色。使用 `.anchor` 和 `.offset` 相对触发控件定位，或用 `.position` 指定窗口坐标。`.track_scroll` 接收调用方持有的滚动句柄。

### 提示

`Tooltip::plain` 和 `Tooltip::rich` 在指针悬停后显示，也可通过 `.track_focus(&focus)` 在触发控件获得键盘焦点时显示。对于持久富内容提示，用 `cx.new(|_| TooltipState::new())` 创建 `Entity<TooltipState>`，再调用 `.persistent(&state)`。点击或用键盘激活触发控件可以显示提示，调用方也可执行 `state.show(cx)`。触发按钮负责自己的无障碍 Invoke 操作，在其 `on_activate` 回调中执行 `state.update(cx, |state, cx| state.show(cx))` 即可让该操作显示提示。指针离开后，持久提示仍保持显示。调用 `state.dismiss(cx)` 可主动关闭，`is_visible()` 返回已请求的显示状态。

`.enable_user_input(false)` 将激活操作交给调用方。焦点位于触发控件或提示内容时，可以按 Escape 关闭。点击持久提示外部也会关闭它。使用 `.on_dismiss_request(callback)` 时，持久提示的调用方通过 `dismiss(cx)` 接受关闭请求。临时提示在按 Escape 时关闭，并通过同一回调报告请求。`.focus_target(&focus)` 允许用 Tab 从触发控件进入调用方提供的控件。`.focusable(true)` 会在打开时将焦点移入提示，默认保留焦点。关闭时若焦点位于提示内，则返回触发控件或原来拥有焦点的控件。

用 `.title_content`、`.body_content` 和 `.action_content` 提供 GPUI 元素。构造器的文本用作无障碍说明。`.link(label, callback)` 创建链接，由回调负责导航。`.placement(TooltipPlacement)` 选择物理方向或逻辑起始侧 `Start`、末尾侧 `End`，`.offset(pixels)` 设置间距。默认首选 `Above`、间隔 4 px，空间不足时会翻转。`.positioner(...)` 可以根据测得的触发控件边界、视口和文字方向返回 `TooltipPositioner`。

使用 `.caret(TooltipCaret::default())` 显示 Material 默认的 16 × 8 px 指示尖角，也可以自定义尺寸和路径。默认不显示尖角。通过 `.max_width`、`.shape`、`.colors`、`.tonal_elevation` 和 `.shadow_elevation` 配置表面。普通提示和富内容提示默认最大宽度分别为 200 px 和 320 px，并受窗口可用空间限制。未指定的颜色和形状采用窗口的 Material 配色与圆角角色。默认色调层级为零，阴影层级为普通提示 Level 0、富内容提示 Level 2。

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
| `Scaffold` | 组合导航、应用栏、内容、可定位的悬浮操作按钮和消息条，并根据已占空间计算内容内边距。 | [Scaffold](https://m3.material.io/foundations/layout/scaffold/overview) |
| `ListDetail`、`SupportingPane`、`Feed` | 列表与详情、辅助窗格及响应式信息流布局。 | [典型布局](https://m3.material.io/foundations/layout/canonical-examples/overview) |
| `VerticalDragHandle` | 用于横向调整窗格尺寸的纵向把手，可分别设置静止、按下和拖动外观。 | [布局要素](https://m3.material.io/foundations/layout/understanding-layout/parts-of-layout#314a4c32-be52-414c-8da7-31f059f1776d) · [Compose VerticalDragHandle](https://developer.android.com/reference/kotlin/androidx/compose/material3/VerticalDragHandle.composable) |

### 纵向拖动把手

将 `VerticalDragHandle::new(id)` 放在两个窗格之间。用 `.sizes(DragHandleSizes)`、`.colors(DragHandleColors)` 和 `.shapes(DragHandleShapes)` 分别覆盖静止、按下和拖动状态的值。默认静止标记为 4 × 48 的胶囊，使用当前窗口主题的 Outline 角色。按下和拖动标记为 12 × 52，使用 On Surface 角色与 Medium 圆角。视觉尺寸以逻辑像素计。交互目标始终至少为 48 × 48，并能容纳所有状态中尺寸最大的标记。悬停和键盘焦点提供各自的反馈，且保留独立的交互状态。

使用 `.on_resize(label, width, min..=max, callback)` 设置分隔控件的名称、逻辑起始窗格的当前宽度、最小及最大宽度和宽度请求回调。调用方决定是否采用请求的宽度，并负责布局。分隔控件获得焦点后，左右方向键分别向左或向右移动它，Home 和 End 分别请求最小和最大宽度。用 `.keyboard_step(px(...))` 修改默认的 8 像素步长。当前无障碍桥接将原生 Splitter 角色视为只读，因此该纵向分隔控件提供带名称的可写范围语义，以 “splitter” 描述角色，数值为起始窗格的宽度。窗口主题根据 LTR 或 RTL 将鼠标及触摸的实际移动方向映射到逻辑宽度。按 Escape、失去焦点或指针操作被取消时，拖动结束，调用方保留最近一次接受的宽度。

传入 `.interaction_source(&source)` 可通过共享的 `Entity<InteractionSource>` 观察悬停、按下、拖动和可见键盘焦点。观察多个把手时，请为它们提供不同的标识。运行 `cargo gallery vertical-drag-handle` 可体验分割窗格示例。

### Scaffold 布局

`Scaffold::new(id, content)` 测量并预留独立的 `.app_bar`、`.rail` 和 `.bottom_bar` 插槽所占的空间。应用持有这些插槽、FAB 和消息条宿主。请使用稳定的 ID，并为 Scaffold 提供有界的尺寸。用 `.content_insets(ScaffoldInsets { top, start, bottom, end })` 设置可用内容距各边的最小距离，单位为逻辑像素。各边已占用的空间计入最小距离。普通构造器会在内容区内应用剩余内边距。

用 `Scaffold::with_layout(id, |layout, window, cx| ...)` 在构造内容前读取当前布局。`ScaffoldContentLayout` 提供内容区边界 `bounds`、已占空间 `occupied`、剩余内边距 `padding` 和文本方向 `direction`。只把 `padding` 应用到内容或滚动区的子元素，因为 Scaffold 已预留 `occupied`。用 `ScaffoldInsets::resolve(direction)` 将逻辑起始侧和末尾侧转换成实际左右边。内容构造闭包在布局时执行。插槽或窗口尺寸变化后，它会收到当前测量结果。

用 `.fab_position(ScaffoldFabPosition)` 将 FAB 在可用内容宽度内设为 `Start`、`Center` 或 `End`，默认是 `End`。这三种位置让 FAB 位于底栏上方，有消息条插槽时也位于消息条上方。`EndOverlay` 让 FAB 贴近 Scaffold 底部并覆盖底栏，仍遵守请求的底部内边距；消息条会避让底栏和 FAB。起始侧与末尾侧随窗口方向镜像。FAB 和消息条作为浮层呈现，不计入已占内容空间。

<a id="optional-components"></a>

## 可选组件

在 `gpui-material` 依赖上启用 feature。日期选择器、时间选择器和滚动选择字段从 `gpui_material::date_time` 导入。

| Feature | 组件 | 支持的变体与行为 | Material 3 |
| --- | --- | --- | --- |
| `expressive-groups` | `ButtonGroup`、`SplitButton` | Standard 和 Connected 按钮组，以及主操作与次操作独立的拆分按钮。 | [按钮组](https://m3.material.io/components/button-groups/overview) · [拆分按钮](https://m3.material.io/components/split-button/overview) |
| `toolbars` | `FabMenu`、`Toolbar` | 可展开的悬浮操作菜单。工具栏支持悬浮或停靠，以及横向或纵向布局。 | [悬浮操作菜单](https://m3.material.io/components/fab-menu/overview) · [工具栏](https://m3.material.io/components/toolbars/overview) |
| `search` | `SearchBar`、`SearchView` | 收起的搜索栏，以及停靠或全屏的搜索视图。 | [搜索](https://m3.material.io/components/search/overview) |
| `date-time` | `DatePicker`、`TimePicker`、`ScrollField` | 单日期和日期范围选择，支持 Docked、Modal、Modal Input。时间选择支持 Dial、Input 和 12、24 小时制。支持独立的滚动选择字段。 | [日期选择](https://m3.material.io/components/date-pickers/overview) · [时间选择](https://m3.material.io/components/time-pickers/overview) |

### 滚动选择字段

每个字段通过 `cx.new(|cx| ScrollFieldState::new(item_count, index, cx))` 创建一个 `Entity<ScrollFieldState>`，并传给 `ScrollField::new(id, name, &state)`。`selected_option()` 返回最接近居中的选项，`target_option()` 返回目标选项，`is_scroll_in_progress()` 表示仍在滚动或归位。可观察该实体来更新其他视图。`scroll_to_option(index, cx)` 立即定位，`animate_scroll_to_option(index, cx)` 遵循窗口的动效方案和系统减少动效设置。越界索引会限制在有效范围内。`set_item_count(count, cx)` 更新选项数并限制目标索引。选项数为零时，字段为空，且没有选中项或目标项。

当前选项在三行可见区域内居中。可点击某行，也可使用鼠标滚轮、触控板、上下方向键或无障碍操作改变选项。滚动到末项后会回到首项，反向操作同样循环。通过 `.item_content(|index, selected, enabled| ...)` 为每行提供装饰性 GPUI 元素。默认将索引补足为至少两位数字。构造器的名称用于标识字段，`.value_description(|index| ...)` 为辅助技术描述滚动中的目标选项和静止时的居中选项。字段名称应说明用途，并在需要时说明选项范围。

通过 `.colors(ScrollFieldColors)` 设置启用和禁用时的容器、普通选项及选中选项颜色。默认配色使用 `SurfaceContainerLowest`、`Outline` 和 `OnSurface`。`.disabled(true)` 结束当前的用户滚动并阻止后续用户操作。调用方仍可通过状态定位。使用 `.interaction_source(&source)` 接入调用方持有的 `InteractionSource`，独立于选中值观察字段交互。
| `carousel` | `Carousel` | Multi-browse、Uncontained、Uncontained Multi-aspect Ratio、Hero、Centered Hero、Full-screen。 | [轮播](https://m3.material.io/components/carousel/overview) |

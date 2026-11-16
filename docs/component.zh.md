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
| `Switch` | 开启和关闭状态，支持可选滑块图标或自定义内容、各状态配色和共享交互观察。 | [开关](https://m3.material.io/components/switch/overview) |
| `Chip` | Assist、Filter、Input、Suggestion，支持内容插槽和自定义外观，适用的变体支持独立选中和移除。 | [标签组件](https://m3.material.io/components/chips/overview) |
| `Slider` | 单值、居中和范围滑块，可自定义轨道、滑柄和指示器，支持横向及可反转的纵向布局，以及连续值或步进值。 | [滑块](https://m3.material.io/components/sliders/specs) |

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

### 开关

将应用的当前值传给 `Switch::new(id, label, checked)`，并通过 `.on_change(...)` 处理请求的新值。指针、键盘和无障碍激活都会发出相同的变更请求。`.disabled(true)` 阻止用户更改。构造器提供的标签同时用于显示和无障碍名称。

用 `.checked_icon` 和 `.unchecked_icon` 提供可选图标，也可通过 `.thumb_content(element)` 传入装饰性 GPUI 元素。在本次渲染中，自定义元素优先。调用方可使用自己的选中值决定各状态渲染的内容。自定义内容位于居中的 16 px 插槽内，并继承该状态的内容颜色。请将标签和独立控件放在滑块之外。

通过 `.colors(SwitchColors)` 设置选中、未选中、禁用且选中、禁用且未选中四种状态的 `SwitchStateColors` 配色。每种配色可分别覆盖 `track`、`border`、`thumb` 和 `content`。省略的字段沿用主题默认值。使用 `.interaction_source(&source)` 接入调用方持有的 `Entity<InteractionSource>`，观察悬停、焦点、按下和禁用变化，选中值仍由应用持有。

### 标签组件

`Chip::new(id, label)` 将标签用作无障碍名称。所有变体均可用 `.content(element)` 替换显示标签，用 `.leading_icon(element)` 设置前置图标。Input 可用 `.avatar(element)` 设置 24 dp 圆形头像插槽，头像优先于前置图标。`.trailing(element)` 为 Assist、Filter 或 Input 添加尾部图形。Filter 或 Input 设置 `.on_remove(...)` 后，该图形由移除按钮显示，移除操作仍有独立的无障碍名称和焦点。未提供自定义尾部图形时，移除按钮显示 Close 图标。各视觉插槽应避免嵌套独立控件。

Filter 和 Input 渲染调用方的 `.selected(value)`，并通过 `.on_change` 请求变更。Assist 和 Suggestion 使用 `.on_activate`。移除只调用移除回调。`.disabled(true)` 同时禁用两个操作。前置和尾部内容以及移除按钮在 RTL 布局中镜像。键盘先访问主操作，再访问移除操作。

`.tonal(true)` 为 Filter、Elevated Filter 和 Input 选用 Expressive Tonal 默认外观。静止圆角为 Medium，选中为 Full，按下为 Small，未选中的前置图标使用主题的 `OnSurfaceVariant`。普通构造器沿用现有的平面 Assist 和 Small 圆角。用 `.shapes(ButtonShapes)` 按状态设置圆角。用 `.colors(ChipColors)` 为启用、选中、禁用或禁用且选中的外观传入 `ChipStateColors`，每组可设置容器、标签、前置、尾部和边框颜色。省略的字段沿用所选变体默认值，禁用且选中的颜色优先回退到禁用覆盖值。

`.border(ButtonBorder)` 设置轮廓宽度和回退颜色。零宽度隐藏轮廓。`.elevations(ChipElevations)` 可覆盖静止、按下、焦点、悬停、拖动和禁用时的阴影层级。`.dragged(bool)` 显示由应用持有的拖动状态，同时取消尚未完成的指针激活；手势由应用管理。`.spacing(width)` 设置内容插槽之间的水平间距，`.content_padding(ButtonPadding)` 设置逻辑内边距。默认间距为 8 dp，纯标签边缘为 16 dp，图标侧为 8 dp，头像起始侧为 4 dp。

为各标签提供不同的 ID，再用调用方持有的 `Entity<InteractionSource>` 和 `.interaction_source(&source)` 按该 ID 观察主操作。交互源报告悬停、按下、键盘焦点、拖动和禁用反馈；选中与移除仍由调用方控制。

### 滑块

调用方持有 `SliderState` 实体，范围与步进由 Base 管理。`SliderEvent::Change` 报告数值编辑，`SliderEvent::Release` 报告指针手势结束。使用 `Slider::range` 时传入范围值和两个无障碍名称。单值滑块使用 `.kind(SliderKind::Centered)` 从中点开始填充。`.axis(Axis::Vertical)` 默认从下往上递增。设置 `.reverse_vertical(true)` 后，布局、指针移动和上下方向键改为从上往下递增。横向布局和方向键随窗口方向调整。两种方向下，无障碍的值和边界都遵循数值顺序。

`.track_style(SliderTrackStyle)` 可按逻辑像素设置 `thickness`、`outer_radius`、`inner_radius` 和 `thumb_gap`。省略时沿用对应尺寸的默认轨道厚度和外侧圆角、2 dp 内侧圆角，以及距滑柄边缘 6 dp 的间隙。使用 `.thumb_style(RangeThumb, SliderThumbStyle)` 分别为滑柄设置 `size`、`pressed_width`、`color`、`disabled_color` 和 `focused_color`。`size` 的第一维沿数值轴，第二维垂直于数值轴。滑柄沿数值轴默认宽 4 dp，拖动时缩到 2 dp，另一维随当前尺寸变化。非有限数值和非正尺寸使用默认值。圆角和间隙可设为零。单值滑块使用 `RangeThumb::End`。

`.colors(SliderColors)` 可配置滑柄、激活和未激活轨道、激活和未激活刻度在启用和禁用时的颜色。省略的字段沿用窗口配色。启用时滑柄和激活轨道使用 Primary，未激活轨道使用 Secondary Container，激活和未激活刻度分别使用 On Primary 和 On Secondary Container。禁用时，滑柄、激活轨道和刻度默认使用 38% 不透明度的 On Surface，未激活轨道使用 12% 不透明度的 On Surface。单个滑柄的配色优先于滑块配色。键盘焦点默认使用 Secondary。通过 `.thumb(RangeThumb, element)` 和 `.track(element)` 用装饰性内容替换默认滑柄和轨道。输入、焦点和无障碍语义由滑块处理，插槽内应避免放置独立控件。使用自定义轨道时，按需绘制内嵌图标、刻度和端点标记。

`.tick_intervals(count)` 设置可见刻度的区间数，Base 的步进单独配置。`.draw_ticks(...)` 和 `.draw_stop_indicators(...)` 接收带窗口坐标、默认颜色和默认直径的 `SliderMarker`，默认直径为 4 dp。设置 `.show_stop_indicators(false)` 可隐藏端点标记。拖动或通过键盘聚焦滑柄时默认显示数值指示器，保留两位小数。通过 `.value_indicator(false)` 关闭指示器，或传入 `.format_value(...)` 格式化当前显示值。

`.interaction_source(RangeThumb, &source)` 可独立观察每个滑柄。查询键为 `ElementId::from((slider_id, "start"))` 和 `ElementId::from((slider_id, "end"))`。两个滑柄可共用一个交互源，也可分别观察。移除滑柄或更换交互源时，原快照会清除。

<a id="content-and-feedback"></a>

## 内容与反馈

| 组件 | 支持的变体与行为 | Material 3 |
| --- | --- | --- |
| `Text`、`Icon` | 主题排版、可替换图标和随库提供的 Material Symbols。 | [排版](https://m3.material.io/styles/typography/overview) · [图标](https://m3.material.io/styles/icons/overview) |
| `Surface` | 支持静态、可点击、可选及可勾选表面，提供主题内容颜色与层级。 | [层级](https://m3.material.io/styles/elevation/overview) |
| `Card` | Elevated、Filled、Outlined，支持静态或可操作的卡片及独立外观配置。 | [卡片](https://m3.material.io/components/cards/specs) |
| `List`、`ListItem` | Standard 和 Segmented 列表，支持前置、辅助及尾部内容。 | [列表](https://m3.material.io/components/lists/overview) |
| `SwipeToDismiss` | 横向滑动关闭，内容、背景及操作由调用方提供。 | [列表](https://m3.material.io/components/lists/guidelines) · [Compose SwipeToDismissBox](https://developer.android.com/reference/kotlin/androidx/compose/material3/SwipeToDismissBox.composable) |
| `Divider`、`Badge` | 横向与纵向分隔线支持逻辑内缩、粗细和单独配色。徽标支持独立或附着显示，可使用圆点、文本、计数、自定义内容和单独配色。 | [分隔线](https://m3.material.io/components/divider/overview) · [徽标](https://m3.material.io/components/badges/overview) |
| `DisplayScrollbar` | 根据视口状态绘制任一方向的滚动位置，支持自定义尺寸、配色和淡出。 | [Compose 显示型滚动条](https://developer.android.com/reference/kotlin/androidx/compose/material3/nonInteractiveScrollbar.composable) |
| `LinearProgress`、`CircularProgress` | 确定和不确定进度，支持可配置的直线和波浪样式。 | [进度指示器](https://m3.material.io/components/progress-indicators/overview) |
| `LoadingIndicator` | Default 和 Contained 样式，支持确定与不确定进度、自定义形状序列和实例配色。 | [加载指示器](https://m3.material.io/components/loading-indicator/overview) |
| `Snackbar`、`SnackbarHost` | 定时或持续显示的排队消息，支持自定义内容和布局、可选操作与关闭，以及完成结果。 | [消息条](https://m3.material.io/components/snackbar/specs) |

### 表面

`Surface::new(role)` 提供静态背景。自定义控件可使用 `Surface::clickable(id, label, role)` 或 `Surface::selectable(id, label, role, selected)`，并通过 `.on_activate(...)` 处理激活。通过 `Surface::toggleable(id, label, role, checked)` 和 `.on_checked_change(...)` 处理勾选请求。标签提供无障碍名称，由调用方在每次渲染时传入当前选中或勾选状态。指针、回车、空格和无障碍操作调用同一处理函数。`.disabled(true)` 禁止激活，Tab 导航会跳过该表面。为每个表面提供稳定 ID，为独立操作使用独立表面。同组可选表面可放在具有无障碍名称的 `gpui::Role::ListBox` 中。

表面默认使用与主题背景角色配对的前景色。没有配对前景色的背景角色，以及通过 `.container_color(color)` 设置的容器颜色，沿用当前前景色。`.inherit_content_color(true)` 可让有配对色的背景角色也沿用当前前景，`.content_color(color)` 可覆盖内容颜色。颜色通过 GPUI 文本样式传递给子元素，包括未明确设置颜色的 Material 文字和图标。

通过 `.shape(CornerShape)` 设置逻辑方向圆角，通过 `.border(SurfaceBorder { width, color })` 设置以逻辑像素表示的描边。背景、外框和反馈使用这些设置。其他 GPUI Styled 布局与绘制接口仍可使用。GPUI 使用矩形内容区域裁剪溢出的子元素。`.tonal_elevation(Elevation)` 为使用主题 Surface 角色的背景着色，`.shadow_elevation(Elevation)` 独立设置阴影。

交互表面显示 Material 状态层、有边界的涟漪和键盘焦点环。`.feedback(SurfaceFeedback)` 配置单个表面的效果和颜色。通过 `.interaction_source(&source)` 观察悬停、按下、键盘焦点和禁用状态，`.track_focus(&focus)` 接入调用方持有的焦点句柄。减少动态效果设置使用静态按下反馈。

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

### 进度指示器

创建 `LinearProgress` 或 `CircularProgress` 时提供稳定的 ID 和无障碍名称。调用方持有确定进度值，通过 `.value(fraction)` 传入。有限值限制在 `0..=1`。省略数值或调用 `.indeterminate()` 可显示不确定进度，非有限值也会选用该模式。确定模式的无障碍进度条公开当前目标百分比，不确定模式无数值。

通过 `.shape(ProgressShape::Wavy)` 选用波浪形进度线。各模式均支持 `.active_stroke(ProgressStroke)`、`.track_stroke(ProgressStroke)` 和 `.gap(size)`。两条线可分别设置线宽、颜色和 `ProgressCap` 端部样式（Round、Butt 或 Square）。默认使用 4 dp 圆端描边、4 dp 间隔，以及窗口主题的 `Primary` 和 `SecondaryContainer` 色。线宽为零时隐藏该描边。负值或非有限线宽及间隔采用默认值。绘制时将描边限制在可用边界内，间隔也不会超出轨道长度。线性指示器随窗口书写方向镜像，环形指示器从顶部顺时针绘制。

使用 `.wave_amplitude(fraction)` 或 `.wave_amplitude_by(|progress| …)` 设置波浪在可用空间内的振幅。回调在确定模式下接收正在绘制的进度，在不确定模式下接收零。振幅限制在 `0..=1`，NaN 按零处理。确定模式默认在进度不超过 10% 或达到 95% 时采用平线，不确定模式的默认振幅为最大值。确定进度为零或 100% 时为平线。`.wavelength(length)` 以逻辑像素设置波峰间距，环形绘制会在一圈内排列整数个波形。`.wave_speed(length_per_second)` 的单位为逻辑像素每秒，默认每秒移动一个波长，速度为零时保持当前相位。无效的波长或速度采用默认值。减少动效时保持静态形状，并即时应用确定进度的变化。

线性确定进度默认在轨道逻辑末端绘制 4 dp 的终点标记。用 `.stop_size(size)` 调整尺寸，零会隐藏标记，无效尺寸采用默认值。轨道与周围表面的对比度低于 3:1 时应保留可见标记。通过 `.draw_stop_indicator(|stop, window, cx| …)` 替换默认绘制。`ProgressStop` 提供标记边界、进度线颜色和端部样式，以及布局方向。仅在标记可见时调用绘制函数，按标记边界裁切，且不添加无障碍节点。

### 加载指示器

`LoadingIndicator::new(id, label)` 占用 48 dp 的空间，默认循环播放七种形状，表示不确定进度。用 `.contained(true)` 添加圆形容器。Default 的指示形状使用窗口的 `Primary` 色；Contained 使用 `OnPrimaryContainer` 绘制指示形状，使用 `PrimaryContainer` 绘制容器。两种样式均可用 `.indicator_color(color)` 覆盖指示形状的颜色。Contained 可用 `.container_color(color)` 和 `.container_shape(CornerShape)` 配置容器。起始侧和末尾侧的圆角随窗口布局方向调整。

确定进度由应用持有，每次渲染时用 `.value(fraction)` 传入。进度值限制在 `0..=1`，NaN 按零处理。默认序列在零时显示圆形，在一时显示 Soft Burst。用至少两个 `LoadingIndicatorShape` 构造可复用的 `LoadingIndicatorShapes`，再用 `.shapes(&shapes)` 传引用。使用 `LoadingIndicatorShape::circle()`，或用 `LoadingIndicatorShape::rounded_polygon(&vertices, radius)` 传入 GPUI 顶点和同单位的半径。多边形坐标或半径非有限、形状退化或自相交时返回 `None`。形状少于两个时，序列构造器返回 `None`。确定进度从序列首形状变换到末形状，不确定进度还会从末形状变回首形状。

标签为进度指示器提供无障碍名称。确定模式公开数值百分比并直接绘制传入的进度。不确定模式提供礼貌播报的状态，无数值。减少动态效果时，不确定模式停留在首形状，确定模式继续响应进度变化。`.active(false)` 保留布局空间，同时隐藏图形和无障碍进度信息。只有活动窗口内处于活动状态的不确定指示器才会请求动画帧。

### 消息条

为窗口保留一个 `Entity<SnackbarHost>`，将其挂载到布局或传给 `Scaffold::snackbar`。通过 `push` 将 `Snackbar::new(message)` 添加到先进先出的队列。普通消息默认显示四秒。设置 `.action(label, callback)` 后，消息默认持续显示。可在 `.action(...)` 之后通过 `.timeout(Some(duration))` 覆盖时长，或使用 `.timeout(None)` 等待关闭。每条消息成为当前项时开始计时。

使用 `.message_content(|window, cx| ...)` 替换可见消息，`Snackbar::new` 仍提供无障碍状态文本。将 `.action_content(...)` 与 `.action(...)` 一同使用，可替换有名称的操作内容。`.dismiss_content(label, |window, cx| ...)` 替换有名称的关闭控件内容。默认显示关闭图标，设置 `.show_dismiss(false)` 可隐藏该控件。插槽内容应为装饰元素，由宿主负责焦点、激活和完成流程。`.action_on_new_line(true)` 将操作和可选的关闭控件放在消息下方，并将这一行对齐到逻辑尾侧。

使用 `.shape(CornerShape)` 设置容器形状，通过 `.colors(SnackbarColors)` 分别设置容器、消息、操作和关闭内容的颜色。默认圆角为 4 dp。窗口默认使用 Inverse Surface 作为容器色，使用 Inverse On Surface 作为消息和关闭内容色，使用 Inverse Primary 作为操作色。宿主以礼貌模式的实时状态播报消息，保留当前键盘焦点。具名控件可以正常通过 Tab 遍历并激活。消息条的控件获得焦点时，可以按 Escape 关闭消息条。消息条持有焦点时完成操作，焦点将回到仍可用的原目标。

需要 `SnackbarId` 时用 `enqueue`。`current()` 返回当前项的标识和消息，`dismiss()` 关闭当前项。`dismiss_message(id, window, cx)` 和 `perform_action(id, window, cx)` 只对当前项生效，且 `perform_action` 要求存在操作。通过 `.on_result(|result, window, cx| ...)` 观察 `SnackbarResult::ActionPerformed` 或 `SnackbarResult::Dismissed`。超时、显式关闭和程序关闭均报告 `Dismissed`，并在提供 `.on_dismiss(...)` 时调用它。每条消息只交付一次结果。队列先推进，再执行回调，因此回调可以继续操作同一个宿主的队列。释放宿主会取消计时并丢弃剩余消息。

使用 `SnackbarHost::renderer(|message, parts, window, cx| ...)` 可在自定义容器中布局 `SnackbarParts`。放置 `parts.message` 可保留实时状态，放置 `parts.action` 和 `parts.dismiss` 可保留具名控件。计时、完成、动效和退场内容仍由宿主管理。退场期间，插槽以静态内容显示。渲染器可读取 `scale` 让自定义尺寸随动效缩放。

### 显示型滚动条

将已绑定视口的 `ScrollHandle`、`UniformListScrollHandle` 或 `ListState` 传给 `DisplayScrollbar::new(id, axis, &handle)`。这些类型均实现 Base 的 `ScrollbarHandle`。将滚动条放在视口之后，作为同一相对定位父元素的叠加子元素。滚动状态、焦点和输入由视口管理，指示器只负责绘制。横向进度从逻辑起始侧移动到尾侧，纵向轨道位于逻辑尾侧，两者随窗口的 RTL 方向镜像。

通过 `.style(DisplayScrollbarStyle)` 设置 `thumb_color`、`track_color`、`thickness`、`thumb_min_length`、`thumb_max_length_fraction`、`main_axis_track_inset` 和 `cross_axis_track_inset`。滑块默认使用不透明度为 70% 的窗口 Outline，轨道默认透明。默认厚度为 4 个逻辑像素，滑块最短为 24 个逻辑像素，最长为扣除内缩后的轨道长度的 90%。主轴两端各内缩 2 个逻辑像素，交叉轴默认无内缩。内容可完全显示或轨道无法容纳最短滑块时隐藏。通过 `.offset(point(dx, dy))` 按物理方向和逻辑像素移动绘制位置，视口仍负责裁剪。

`.fade(DisplayScrollbarFade { enabled, delay, duration })` 控制滚动位置变化后的显示。默认启用淡出，静止后等待 400 毫秒，再用 250 毫秒淡出。设置 `enabled: false` 可在内容溢出时持续显示。减少动态效果模式保留等待时间，等待结束后立即隐藏。视口在偏移或范围变化时发起常规重绘。指示器在等待结束时唤醒一次，并在淡出期间请求帧。

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

### 公开式下拉菜单

每个下拉菜单创建一个 `ExposedDropdownState`，由它保存待提交的查询、已提交的选择和列表位置。用 `is_open()` 读取当前显示状态，用 `selected_option_id()` 读取最近提交的选项 ID。`with_editor(&editor, editable, window, cx)` 使用调用方持有的 `TextFieldState`。`.editor(|field| ...)` 可以配置或包装传入的 `TextField`，包括标签、装饰和编辑器布局。将这个字段保留在返回元素中，可保留无障碍名称以及原生编辑、光标和输入法行为。

默认可编辑字段在输入或激活尾部触发器时展开，并保留编辑器焦点。默认仅选择字段通过点击或空格展开，同样保留字段焦点。`.primary_anchor(DropdownAnchor::PrimaryEditable)` 让整个字段可以触发菜单并保留编辑器焦点。仅选择状态使用 `PrimaryNotEditable` 可在展开时将焦点移至列表，关闭时回到字段。`.secondary_anchor(label, |open| ...)` 自定义尾部触发器及其无障碍名称。使用指针激活次要锚点时仍可继续编辑，通过无障碍操作激活可编辑字段的次要锚点时焦点进入选项列表。`.primary_anchor_enabled(false)` 和 `.secondary_anchor_enabled(false)` 可分别关闭相应锚点的菜单激活，仍可使用编辑器。

通过 `.open(open)` 和 `.on_open_change(|requested, window, cx| ...)` 控制菜单显示。由调用方处理请求并重新渲染。`.on_dismiss(...)` 报告用户发出的关闭请求。程序更新在渲染时生效，丢弃草稿时保留已提交值。上下方向键浏览选项，回车提交，Escape 在输入法组合结束后关闭。从菜单关闭时，焦点返回仍可用的触发锚点。已获取焦点的外部控件保留焦点。

重新创建或调整选项顺序时保留各自的 ID。`.option_content(|option, highlighted, window, cx| ...)` 提供装饰性行内容，原始标签用于无障碍名称和筛选。可通过 `scroll_state()` 读取或移动保留的 GPUI `ListState`，或用 `set_scroll_state(&scroll, cx)` 接入调用方持有的状态。列表仅构造可见行和预加载区域，包括自定义内容。菜单默认跟随字段宽度。用 `.match_anchor_width(false)` 选用独立宽度，默认 240 px，`.popup_width(width)` 可覆盖该值。菜单宽高受窗口可用区域限制。`.appearance(MenuSurfaceStyle)` 配置菜单形状、主题角色、描边和高程。`.container_color(color)` 可覆盖容器颜色。

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
| `AppBar`、`AppBarRow`、`AppBarColumn` | Small、Centered、Medium Flexible、Large Flexible 及可自定义的 TwoRows 顶部应用栏，操作可随可用空间移入溢出菜单。 | [应用栏](https://m3.material.io/components/app-bars/overview) |
| `Tabs` | Primary、Secondary，可固定或滚动。 | [标签页](https://m3.material.io/components/tabs/overview) |
| `NavigationBar`、`NavigationRail` | 短导航栏支持纵向或横向排列选项。导航轨道支持折叠、展开、隐藏和模态呈现。 | [导航栏](https://m3.material.io/components/navigation-bar/overview) · [导航轨道](https://m3.material.io/components/navigation-rail/overview) |
| `AdaptiveNavigation` | 根据可用宽度在导航栏和导航轨道之间切换。 | [断点](https://m3.material.io/foundations/layout/breakpoints/overview) |
| `Scaffold` | 组合导航、应用栏、内容、可定位的悬浮操作按钮和消息条，并根据已占空间计算内容内边距。 | [Scaffold](https://m3.material.io/foundations/layout/scaffold/overview) |
| `ListDetail`、`SupportingPane`、`Feed` | 列表与详情、辅助窗格及响应式信息流布局。 | [典型布局](https://m3.material.io/foundations/layout/canonical-examples/overview) |
| `VerticalDragHandle` | 用于横向调整窗格尺寸的纵向把手，可分别设置静止、按下和拖动外观。 | [布局要素](https://m3.material.io/foundations/layout/understanding-layout/parts-of-layout#314a4c32-be52-414c-8da7-31f059f1776d) · [Compose VerticalDragHandle](https://developer.android.com/reference/kotlin/androidx/compose/material3/VerticalDragHandle.composable) |

### 应用栏

`AppBar::new(id, title)` 默认创建 Small 应用栏，标题同时是工具栏的无障碍名称。`.title_alignment(AppBarAlignment)` 和 `.subtitle_alignment(AppBarAlignment)` 分别设置逻辑 Start、Center 或 End 对齐。Centered 变体的两项默认值都是 Center。用 `.title_max_lines(lines)` 和 `.subtitle_max_lines(lines)` 设置文本换行上限，默认单行。通过 `.title_content(element)` 或 `.subtitle_content(element)` 提供富内容标题。导航、操作和搜索插槽各自布局，搜索取代首行标题。

`.logo(element)` 可在 Small 或 Centered 中替换文本，在 Flexible 中置于展开标题上方，在搜索布局中置于搜索之前。`AppBarVariant::TwoRows` 将展开标题放在导航与操作的下方。`.title_for_rows(|expanded| ...)` 和 `.subtitle_for_rows(|expanded| ...)` 为当前显示的行构造标题。`.second_row(element)` 为展开标题添加补充内容。标题、标志及补充内容用于呈现信息，命令应放在导航或操作插槽中。

`.colors(AppBarColors)` 分别覆盖容器、滚动后容器、导航、标题、副标题和操作的配色。插槽内容继承对应颜色，独立设色的控件沿用自身颜色。`.content_padding(AppBarPadding)` 设置每行的逻辑内边距。以逻辑像素设置 `.collapsed_height(height)` 和 `.expanded_height(height)`。展开高度包含两行，且至少等于收起高度。首行可增高以容纳插槽。默认收起高度为 64 dp，Flexible 变体采用 Material 的展开尺寸。无效尺寸回退到默认值。

通过 `.track_scroll(&handle)` 接入 GPUI 内容滚动句柄，或通过 `.scroll_state(&state)` 接入保留的 `Entity<AppBarScrollState>`。调用方也可用 `set_content_offset(offset, cx)` 设置内容离顶部的非负距离。接入滚动句柄时由句柄提供该距离。可从状态读取 `content_offset()`、`height_offset()`、`height_offset_limit()`、`collapsed_fraction()` 和 `overlapped_fraction()`。未指定策略时，Flexible 栏按内容离顶部的距离收起。用 `.scroll_behavior(AppBarScrollBehavior::Pinned)` 固定高度并按滚动切换容器色，`EnterAlways` 在滚动反向时立即展开，`ExitUntilCollapsed` 在内容回到顶部之前保持收起。内容区继续持有滚动及其输入。用 `.scroll_enabled(false)` 暂停滚动驱动的更新。用 `set_collapsed_fraction(fraction, cx)` 设置位置，或在滚动结束时调用 `settle(velocity, cx)`。正速度选择收起，负速度在策略允许时选择展开，零选择最近端点。`.scroll_motion(MotionSpec)` 覆盖归位动效，减少动效时即时归位。

`AppBarRow::new(id, items)` 和 `AppBarColumn::new(id, items)` 水平或垂直排列操作，将放不下的项移入 Material 菜单。用 `AppBarItem::clickable(id, label, icon_factory)` 创建行内和溢出共用的命令，并通过 `.on_activate(...)` 和 `.disabled(bool)` 设置行为。通过 `AppBarItem::custom(id, inline, |menu| ...)` 提供自定义行内元素和对应的 `MenuItem`。为各项提供稳定的 ID。`.overflow_indicator(|menu| ...)` 可自定义内置键盘按钮的图形，用 `.overflow_label(label)` 设置按钮名称。`AppBarMenuState` 提供 `is_showing()`、`show(window, cx)` 和 `dismiss(window, cx)`。`.max_item_count(count)` 限制包括触发按钮在内的可见项数。横排在 RTL 中镜像，溢出操作在菜单打开后可访问。

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
| `expressive-groups` | `ButtonGroup`、`SplitButton` | Standard 和 Connected 按钮组，以及内容、外观和激活行为各自独立的拆分按钮。 | [按钮组](https://m3.material.io/components/button-groups/overview) · [拆分按钮](https://m3.material.io/components/split-button/overview) |
| `toolbars` | `FabMenu`、`Toolbar` | 可展开的悬浮操作菜单。工具栏支持悬浮或停靠，以及横向或纵向布局。 | [悬浮操作菜单](https://m3.material.io/components/fab-menu/overview) · [工具栏](https://m3.material.io/components/toolbars/overview) |
| `search` | `SearchBar`、`SearchView` | 收起的搜索栏，以及停靠或全屏的搜索视图。 | [搜索](https://m3.material.io/components/search/overview) |
| `date-time` | `DatePicker`、`TimePicker`、`ScrollField` | 单日期和日期范围选择，支持 Docked、Modal、Modal Input。时间选择支持 Dial、Input 和 12、24 小时制。支持独立的滚动选择字段。 | [日期选择](https://m3.material.io/components/date-pickers/overview) · [时间选择](https://m3.material.io/components/time-pickers/overview) |
| `carousel` | `Carousel`、`CarouselParallax` | Multi-browse、Uncontained、Uncontained Multi-aspect Ratio、Hero、Centered Hero、Full-screen。 任意 GPUI 内容可使用独立滚动视差效果。 | [轮播](https://m3.material.io/components/carousel/overview) |

### 按钮组

`ButtonGroup::new(id, name, items)` 使用稳定的成员标识，并提供一个 Tab 焦点。左右方向键移动焦点，保留选中状态。导航会跳过禁用成员，视觉顺序和方向键随窗口书写方向调整。`.variant(ButtonGroupVariant::Connected)` 连接相邻成员。Standard 和 Connected 默认间距分别为 12 和 2 个逻辑像素。可通过 `.spacing(width)` 覆盖间距，使用 `.alignment(ButtonGroupAlignment)` 设置逻辑横向排列，使用 `.vertical_alignment(ButtonGroupVerticalAlignment)` 设置纵向对齐，默认 Top。成员可以单独设置纵向对齐。

`ButtonGroupItem::new(id, name)` 在启用组选择时使用 `.selection(mode, selected_ids)` 和 `.on_change(...)`。Single 和 Multiple 允许取消选中，Required 模式会保留最后一个选中项。应用处理请求并提供当前选择。`.action(callback)` 在任一组内建立独立命令，`.toggle(selected, callback)` 建立单独受控的切换控件。`.on_activate(...)` 为 Actions 模式下的成员提供命令。

使用 `.leading_icon(|| ...)` 在标签旁加入图标，用 `.icon_only(|| ...)` 以图标替代标签，或通过 `.content(width, || ...)` 提供带首选逻辑宽度的装饰内容。构造器名称用于无障碍和默认溢出菜单行。`.icon_width(IconButtonWidth)` 选择纯图标尺寸。`.appearance(ButtonVariant)` 和 `.style(ButtonGroupItemStyle)` 分别覆盖成员的外观预设，以及静止、按下、选中形状、配色、描边、阴影和逻辑内边距。通过 `.shape(ButtonShape::Square)` 可将默认 Round 外侧形状改为方形。`.connected_shapes(ButtonGroupConnectedShapes)` 为首段、中间段、尾段和单一段设置各状态形状。成员设置的形状字段优先。

`.weight(weight)` 按比例分配非加权成员与最小目标宽度剩余的空间。默认权重为零。`.expanded_ratio(ratio)` 默认为 `0.15`。按下的成员向左右可见邻项借用宽度。`.compression_limit(px(...))` 限制成员可让出的宽度，默认保留至少 24 个逻辑像素。`.interaction_source(&source)` 使用 `ElementId::from((group_id, item_id))` 观察可见成员，宽度动画读取同一按压状态。减少动态效果时立即应用目标宽度。

无法显示的成员进入溢出菜单，保留各自的名称、可用与选中状态以及回调。`.overflow_trigger(width, |open| ...)` 替换触发器的显示内容，`.overflow_label(...)` 保留其无障碍名称。`.overflow_content(|| ...)` 在自动生成的菜单行之前添加装饰内容，`.overflow_item_content(|| ...)` 自定义对应成员的菜单行。使用 `.overflow_open(open)` 时由应用持有显示状态，并应用 `.on_overflow_open_change(...)` 的请求。省略打开值时，由按钮组管理菜单显示。

### 拆分按钮

`SplitButton::new(id, leading_name, trailing_name, menu_items)` 提供两段各自的无障碍名称。前段命令使用 `.on_activate(...)`。通过 `.leading_icon(element)` 在标签旁加入图标，`.leading_icon_only(element)` 用图标替代标签，`.leading_content(element)` 提供自定义内容。尾段默认显示菜单箭头。使用 `.trailing_icon(element)` 可按当前尺寸设置图标，或用 `.trailing_content(element)` 提供自定义内容。每段以最后一次内容设置为准。各插槽仅用于呈现，由对应段负责焦点、名称和激活。

尾段默认打开指定的菜单，菜单关闭时焦点返回触发按钮。通过 `.on_trailing_activate(callback)` 设置独立命令，或通过 `.trailing_toggle(selected, callback)` 设置受控切换。尾段命令与切换按最后一次设置生效，并取代构造器提供的菜单。激活切换时请求下一状态，选中值仍由调用方持有。`.disabled(true)` 禁用前段命令，`.trailing_disabled(true)` 单独禁用尾段。菜单为空时，默认菜单触发按钮禁用。两段在 RTL 中镜像，键盘先访问前段再访问尾段。

Filled、Filled Tonal、Elevated 和 Outlined 预设为各 Material 尺寸提供基础外观。`.spacing(width)` 设置段间距，默认 2 dp。用 `.leading_style(SplitButtonSegmentStyle)` 和 `.trailing_style(SplitButtonSegmentStyle)` 单独配置每段。可通过 `ButtonShapes` 设置静止、按下、选中时的逻辑圆角，通过 `ButtonColors` 设置各状态配色，用 `ButtonBorder` 设置描边、`ButtonElevations` 设置阴影及 `ButtonPadding` 设置逻辑内边距。省略的字段沿用该段默认值。已选中的尾段切换和已打开的菜单使用选中外观，菜单触发器保留按钮语义。选中外观默认增大尾段内侧圆角。未设置选中颜色时沿用启用配色，禁用配色优先。减少动效时立即应用目标形状。

使用 `.leading_interaction_source(&source)` 和 `.trailing_interaction_source(&source)` 传入调用方持有的交互源，可分别用 `ElementId::from((id, "primary"))` 和 `ElementId::from((id, "secondary"))` 读取快照。同一个实体可以观察两个标识。更换交互源时，对应段的快照会从原交互源清除。

### 滚动选择字段

每个字段通过 `cx.new(|cx| ScrollFieldState::new(item_count, index, cx))` 创建一个 `Entity<ScrollFieldState>`，并传给 `ScrollField::new(id, name, &state)`。`selected_option()` 返回最接近居中的选项，`target_option()` 返回目标选项，`is_scroll_in_progress()` 表示仍在滚动或归位。可观察该实体来更新其他视图。`scroll_to_option(index, cx)` 立即定位，`animate_scroll_to_option(index, cx)` 遵循窗口的动效方案和系统减少动效设置。越界索引会限制在有效范围内。`set_item_count(count, cx)` 更新选项数并限制目标索引。选项数为零时，字段为空，且没有选中项或目标项。

当前选项在三行可见区域内居中。可点击某行，也可使用鼠标滚轮、触控板、上下方向键或无障碍操作改变选项。滚动到末项后会回到首项，反向操作同样循环。通过 `.item_content(|index, selected, enabled| ...)` 为每行提供装饰性 GPUI 元素。默认将索引补足为至少两位数字。构造器的名称用于标识字段，`.value_description(|index| ...)` 为辅助技术描述滚动中的目标选项和静止时的居中选项。字段名称应说明用途，并在需要时说明选项范围。

通过 `.colors(ScrollFieldColors)` 设置启用和禁用时的容器、普通选项及选中选项颜色。默认配色使用 `SurfaceContainerLowest`、`Outline` 和 `OnSurface`。`.disabled(true)` 结束当前的用户滚动并阻止后续用户操作。调用方仍可通过状态定位。使用 `.interaction_source(&source)` 接入调用方持有的 `InteractionSource`，独立于选中值观察字段交互。

### 轮播

通过 `Carousel::new(id, label, items)` 构造轮播，为每个条目提供唯一且稳定的 ID。可用 `cx.new(|_| CarouselState::new(initial_index))` 创建 `Entity<CarouselState>`，并通过 `.state(&state)` 接入。观察该实体即可读取 `current_item()`、`current_item_id()` 和相对最接近条目的带符号偏移 `current_item_offset_fraction()`。条目移动时位置随目标 ID 保持，目标被移除时回到经过边界限制的原索引。初始索引和空列表期间发出的请求会在条目到达后生效。通过 `scroll_to_item(index, cx)` 立即定位，通过 `animate_scroll_to_item(index, cx)` 按窗口动效设置移动。两种方法都会限制索引范围。每个实体对应一个轮播。未接入实体时，轮播按元素 ID 保留自身位置。

通过 `.alignment(CarouselAlignment::Start)` 或 `.alignment(CarouselAlignment::Center)` 设置 Multi-browse、Hero 和 Centered Hero 的对齐方式。Centered Hero 默认居中，其他内含布局默认起始侧对齐。六种布局均支持 `.item_spacing(px(...))` 和 `.content_padding(CarouselPadding { start, end, top, bottom })`。`start` 和 `end` 是逻辑方向边。横向布局默认使用 8 px 间距、16 px 起始侧内边距和 8 px 交叉轴内边距。内含布局还有 16 px 末尾侧内边距。Full-screen 默认使用 16 px 间距和零内边距。`.small_item_widths(min, max)` 设置小条目的正数宽度上下限，默认范围为 40–56 px。视口过窄时布局可简化为一个焦点条目。

`.user_scroll_enabled(false)` 关闭滚轮、触控板、键盘导航及无障碍滚动，条目仍可激活，应用仍可定位。内含布局和 Full-screen 默认吸附到最近的定位线，非内含布局默认自由滚动，可通过 `.snap_scrolling(true)` 启用吸附。`.fling_behavior(...)` 可选择自由滚动、就近吸附、单条目推进，或根据 `CarouselFlingContext` 自定义最终条目。逐格滚轮与无障碍滚动请求按条目移动。定位动画和吸附遵循减少动态效果设置。

在可滚动布局中，用 `CarouselParallax::new(content)` 包装任意 GPUI 内容。效果默认使用 GPUI 当前裁剪区域作为视口，也可通过 `.viewport(bounds)` 传入窗口坐标下的视口。纵向布局使用 `.axis(Axis::Vertical)`。条目进出视口时，内容逐步裁剪和平移，布局尺寸保持固定。减少动态效果设置保留视口裁剪，并跳过附加的视差平移。通过 `.shape(CornerShape)` 和 `.border(CarouselBorder)` 配置匹配的圆角和描边。GPUI 使用矩形区域裁剪内容，圆角处通过 Material Surface 颜色覆盖，也可通过 `.backdrop(color)` 设置与均匀父背景匹配的不透明色。图案或透明背景可使用直角保留原背景。

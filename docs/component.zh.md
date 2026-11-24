# 组件

[English](component.md) | 中文

下列组件和布局可从 `gpui_material` 使用。[可选组件](#可选组件)需要启用对应特性。安装方式和平台支持见[集成指南](integration.zh.md)，也可以运行 `cargo gallery` 体验交互示例。

项目以[锁定的 Material 3 规范](design-sources.zh.md#material-specification)为设计依据。下表链接指向官方指南，锁定的数据版本见[站点快照索引](material-snapshot.zh.md)。

<a id="actions-and-selection"></a>

## 操作与选择

| 组件 | 支持的变体与行为 | Material 3 |
| --- | --- | --- |
| `Button`、`ToggleButton` | Elevated、Filled、Filled Tonal、Outlined。两者均提供变体快捷构造器（`filled`、`tonal`、`elevated`、`outlined`，`Button` 还提供 `text` 变体与快捷构造器）以及受控的 `.loading(bool)` 忙碌状态，支持 XS–XL 尺寸、圆形和方形外观、`SemanticTone` 语义快捷配色（`Default`、`Danger`、`Success`、`Warning`、`Info`）、自定义内容和样式，以及可选的交互观察。 | [按钮](https://m3.material.io/components/buttons/overview) |
| `IconButton`、`IconToggleButton` | Standard、Filled、Filled Tonal、Outlined。两者均提供变体快捷构造器（`standard`、`filled`、`tonal`、`outlined`）及受控的 `.loading(bool)` 忙碌状态，可选尺寸和宽度，支持 `SemanticTone` 语义快捷配色，可为单个实例配置形状、颜色和描边，并可观察交互。 | [图标按钮](https://m3.material.io/components/icon-buttons/overview) |
| `Fab`、`ExtendedFab` | Small、Medium、Large 尺寸，可配置实例外观和控制显示状态。`ExtendedFab` 支持文字标签、可选图标和自定义内容。 | [悬浮操作按钮](https://m3.material.io/components/floating-action-button/overview) · [扩展悬浮操作按钮](https://m3.material.io/components/extended-fab/overview) |
| `Checkbox` | 未选中、选中、混合、错误和禁用状态，可配置实例颜色和描边并观察交互。 | [复选框](https://m3.material.io/components/checkbox/overview) |
| `Radio`、`RadioGroup` | 互斥选择、组内键盘导航、单项颜色配置和可选的交互观察。 | [单选按钮](https://m3.material.io/components/radio-button/specs) |
| `Switch` | 开启和关闭状态，支持可选滑块图标或自定义内容、各状态配色和共享交互观察。 | [开关](https://m3.material.io/components/switch/overview) |
| `Chip`、`ChipGroup` | Assist、Filter、Input、Suggestion 标签组件，支持内容插槽、自定义外观以及独立的选中与移除；受控 `ChipGroup` 集合支持 `Single`、`SingleRequired`、`Multiple` 选择模式、自动换行或横向滚动布局及漫游键盘焦点。 | [标签组件](https://m3.material.io/components/chips/overview) |
| `Slider` | 单值、居中和范围滑块，可自定义轨道、滑柄和指示器，支持横向及可反转的纵向布局，以及连续值或步进值。 | [滑块](https://m3.material.io/components/sliders/specs) |

使用 `.content(element)` 自定义显示的标签。构造器提供的无障碍名称和前后图标插槽仍然生效。内容中应避免放置独立控件，由按钮负责激活和焦点。可使用 `Button::filled`、`Button::tonal`、`Button::elevated`、`Button::outlined` 或 `Button::text`（以及 `ToggleButton::filled`、`ToggleButton::tonal`、`ToggleButton::elevated` 或 `ToggleButton::outlined`）作为 `.variant(...)` 的快捷构造器。在 `Button`、`ToggleButton`、`IconButton` 或 `IconToggleButton` 上设置 `.loading(true)` 可进入受控忙碌状态：阻止指针、键盘与无障碍激活并暴露忙碌语义，除非同时设置 `.disabled(true)` 否则保持启用配色，保留静止宽度避免布局抖动，并在按钮中央渲染紧凑的内联 Material 环形旋转指示器，同时遵循窗口/视口可见性与减少动态效果（回退为静态圆弧）。可通过 `.tone(SemanticTone)` 在所有变体与切换状态上快捷应用 Material 3 语义色板（`Default`、`Danger`、`Success`、`Warning`、`Info`）。可通过 `.shapes(ControlShapes)`、`.colors(ButtonColors)`、`.border(ControlBorder)`、`.content_padding(LogicalEdges)` 和 `.elevations(ControlElevations)` 配置单个实例；显式传入的 `.colors` 和 `.border` 始终优先于语义色板默认值。未指定的形状、颜色和 elevation 字段沿用对应状态的尺寸、变体和语义色调默认值。`LogicalEdges` 随窗口文本方向调整。零宽度边框会移除默认描边。

构造 `IconButton::new(id, name, icon)`（或变体快捷构造器 `IconButton::standard`、`IconButton::filled`、`IconButton::tonal`、`IconButton::outlined`）或 `IconToggleButton::new(id, name, icon, selected)`（或 `IconToggleButton::standard`、`IconToggleButton::filled`、`IconToggleButton::tonal`、`IconToggleButton::outlined`）时传入无障碍名称和装饰性图标元素。图标按钮默认使用 Small 尺寸、Uniform 宽度、Round 形状和 Standard 配色。使用 `.shapes(ControlShapes)` 设置静止和按下时的圆角，以及切换按钮选中时的圆角。使用 `.colors(ButtonColors)` 设置启用、禁用和选中状态下的容器与图标颜色。省略的字段沿用当前尺寸和变体的默认值。`.border(ControlBorder)` 替换默认描边，宽度为零时移除描边。`IconToggleButton` 根据调用方传入的值渲染选中状态，并在选中时使用可选的 `.selected_icon(element)`。

常规按钮与图标按钮使用 `.on_activate(handler)` 处理指针、键盘和无障碍激活。当 `Button` 或 `IconButton` 放在已接管指针激活的触发器容器内（如 `Menu::composed` 或 `gpui_base::Popover`）时，改用 `.on_accessible_activate(handler)`：仅注册 `AccessibleAction::Click` 而不挂载 `on_click`，既保留按压视觉反馈与无障碍激活，又避免与外层 `on_mouse_down` 冲突。由于 `.on_accessible_activate(...)` 会跳过按钮自身的 `on_click`，直接使用 `gpui_base::Popover` 且需要键盘 `Enter`/`Space` 触发时，应在外层触发器容器上处理 `Confirm` 动作（与 `Menu::composed` 一致）。`.on_activate(...)` 与 `.on_accessible_activate(...)` 互相覆盖先前注册的回调。

调用方持有 `Entity<InteractionSource>`，通过 `.interaction_source(&source)` 观察按钮或图标按钮。同一窗口内共享一个源的控件应使用各自的元素 ID。用 `InteractionSource::state` 读取当前状态，或订阅 `InteractionUpdate` 逐次接收延后派发的变更，包括两次绘制之间发生的按下和释放。控件移除后，其状态快照会清除。Base 管理输入，应用持有切换值。

### 悬浮操作按钮

`Fab::new` 接收一个图标和无障碍名称，默认使用 56 dp 的 Small 尺寸和主色容器配色。可通过 `.shape(CornerShape)`、`.colors(ButtonColors)` 和 `.elevations(ControlElevations)` 分别配置每个 Small、Medium 或 Large 实例。用启用和禁用状态的容器与内容字段设置颜色，省略的字段沿用对应尺寸和配色的默认值。用 `.interaction_source(&source)` 连接由调用方持有的交互源。

应用持有显示状态，每次渲染时通过 `.visible(visible)` 传入。更新时保持元素 ID 不变，缩放和透明度即可从当前进度反向播放。`FabVisibilityAnimation` 默认以中心为缩放原点，隐藏比例为 `0.2`，并分别使用主题的快档空间动效和效果动效。用 `FabAlignment` 的逻辑位置指定缩放所围绕的边或角，起始侧与末尾侧随窗口文本方向镜像。将 `target_scale` 设为有限非负值，并分别设置 `scale_motion` 和 `alpha_motion`。无效比例（包括导致缩放后尺寸溢出的值）回退到 `0.2`。启用减少动态效果时直接使用最终状态。

FAB 开始隐藏时立即退出焦点导航并停止响应输入。淡出期间保留布局空间，透明度降到零时释放空间。初始隐藏的 FAB 不占空间。图标仅用于装饰，由 FAB 自己管理激活和焦点。需要随动效缩放的自定义图标应采用继承的文字大小或相对尺寸。

### 扩展悬浮操作按钮

使用 `ExtendedFab::new(id, label, icon)` 创建带图标和标签的操作，或使用 `ExtendedFab::text(id, label)` 创建纯文字操作。`.content(element)` 替换显示的标签，同时保留构造器提供的无障碍名称和可选图标。图标和自定义内容只用于展示，由按钮负责激活和焦点。内置标签受可用宽度约束，完全展开后过长的文字显示省略号。自定义内容按标签可用空间布局，超出部分会裁剪。

Small、Medium、Large 的默认高度分别为 56、80、96 dp。通过 `.expanded(expanded)` 传入调用方持有的展开状态。带图标的按钮收起时过渡为只显示图标的方形，无图标的按钮在两种状态下均保留文字或自定义内容。展开和收起共用一个操作和焦点句柄。在从右到左的窗口中，前置图标移至右侧。启用减少动态效果时直接应用目标状态。

可使用与 `Fab` 相同的 `.shape`、`.colors`、`.elevations`、`.interaction_source`、`.visible` 和 `.visibility_animation` 方法。渲染时保持元素 ID 不变，展开和显示状态可以独立变化。显示动效缩放期间保留原有布局空间，透明度降为零时释放空间。开始隐藏时立即退出焦点导航和无障碍树。内置标签和图标插槽随按钮缩放。自定义标签内容在缩放后的插槽内布局，超出插槽的固定尺寸内容会被裁剪。

### 复选框

`Checkbox::new(id, label, state)` 渲染调用方传入的状态。指针、键盘和无障碍激活通过 `.on_change` 请求下一状态。激活混合状态的复选框会请求 `Checked`。由调用方处理请求，并在下一次渲染传入当前值。`.error(true)` 公开校验错误状态，默认使用 `Error` 和 `OnError`。禁用外观优先于错误外观。

通过 `.colors(CheckboxColors)` 为未选中、选中和混合状态分别提供 `CheckboxStateColors`，也可配置对应的错误和禁用状态。每种状态的 `checkmark`、`box_color` 和 `border` 可独立覆盖。省略的字段沿用窗口主题默认值。未选中状态的标记颜色用于标记收回的动效。

`.checkmark_stroke(gpui::StrokeOptions)` 设置勾选和混合标记的等宽描边，包括宽度、端点和连接样式。用 `.outline_stroke(width)` 设置圆角方框轮廓的逻辑像素宽度。两者默认均为两个逻辑像素。零宽度移除对应描边，负数或非有限宽度恢复默认值，过大的宽度以指示图形尺寸为限。设置 `.show_label(false)` 可隐藏可见标签及尾部间距，同时保留构造器标签作为无障碍名称；传入 `.label_content(element)` 可替换可见标签并保留无障碍名称。使用调用方持有的 `Entity<InteractionSource>` 和 `.interaction_source(&source)` 观察悬停、按下、键盘焦点和禁用状态。各控件使用不同的 ID 即可共享交互源，选中值仍由调用方持有。传入 `.track_focus(&focus)` 可接入调用方持有的 `FocusHandle`。

### 单选按钮

使用 `Radio::colors(RadioColors)` 分别覆盖选中、未选中、禁用且选中、禁用且未选中时的指示器颜色。省略的字段沿用当前窗口配色。选中默认使用 Primary，未选中使用 On Surface Variant，两种禁用状态默认均使用 38% 不透明度的 On Surface。

设置 `.show_label(false)` 可隐藏可见标签与尾部间距并保留无障碍名称，传入 `.label_content(element)` 可自定义可见标签。传入 `.interaction_source(&source)` 可观察独立的单选按钮或 `RadioGroup` 中的单项。组内快照使用渲染标识 `ElementId::from((group_id, option_id.to_string()))`，不同组可共享同一个 `Entity<InteractionSource>`。移除选项或更换其交互源时，原交互源中的快照会清除。选择状态由调用方持有，并根据 `.on_select(...)` 请求更新。每组有一个 Tab 焦点，导航会跳过禁用选项，并在 RTL 中镜像横向方向键的行为。独立渲染的 `Radio` 以及 `RadioGroup` 内的各 `Radio` 选项均可通过 `.track_focus(&focus)` 接入调用方持有的 `FocusHandle`，`RadioGroup` 会保留并将其用于组内漫游焦点导航。

### 开关

将应用的当前值传给 `Switch::new(id, label, checked)`，并通过 `.on_change(...)` 处理请求的新值。指针、键盘和无障碍激活都会发出相同的变更请求。`.disabled(true)` 阻止用户更改。构造器提供的标签同时用于显示和无障碍名称；使用 `.show_label(false)` 可隐藏可见标签与尾部间距并保留无障碍名称，使用 `.label_content(element)` 可替换可见标签。

用 `.checked_icon` 和 `.unchecked_icon` 提供可选图标，也可通过 `.thumb_content(element)` 传入装饰性 GPUI 元素。在本次渲染中，自定义元素优先。调用方可使用自己的选中值决定各状态渲染的内容。自定义内容位于居中的 16 px 插槽内，并继承该状态的内容颜色。请将标签和独立控件放在滑块之外。

通过 `.colors(SwitchColors)` 设置选中、未选中、禁用且选中、禁用且未选中四种状态的 `SwitchStateColors` 配色。每种配色可分别覆盖 `track`、`border`、`thumb` 和 `content`。省略的字段沿用主题默认值。使用 `.interaction_source(&source)` 接入调用方持有的 `Entity<InteractionSource>`，观察悬停、焦点、按下和禁用变化，选中值仍由应用持有。传入 `.track_focus(&focus)` 可接入调用方持有的 `FocusHandle`。

### 标签组件

`Chip::new(id, label)`（或变体快捷构造器 `Chip::assist`、`Chip::filter`、`Chip::input` 和 `Chip::suggestion`）将标签用作无障碍名称。所有变体均可用 `.content(element)` 替换显示标签，用 `.leading_icon(element)` 设置前置图标。Input 可用 `.avatar(element)` 设置 24 dp 圆形头像插槽，头像优先于前置图标。`.trailing(element)` 为 Assist、Filter 或 Input 添加尾部图形。Filter 或 Input 设置 `.on_remove(...)` 后，该图形由移除按钮显示，移除操作仍有独立的无障碍名称和焦点。未提供自定义尾部图形时，移除按钮显示 Close 图标。各视觉插槽应避免嵌套独立控件。

Filter 和 Input 渲染调用方的 `.selected(value)`，并通过 `.on_change` 请求变更。Assist 和 Suggestion 使用 `.on_activate`。移除只调用移除回调；当启用状态的 Filter 或 Input 标签配置了 `.on_remove(...)` 时，主操作获得焦点时按下无修饰键的 `Backspace` 或 `Delete` 同样会触发移除回调并停止事件冒泡。`.disabled(true)` 同时禁用两个操作。前置和尾部内容以及移除按钮在 RTL 布局中镜像。键盘先访问主操作，再访问移除操作。

`.tonal(true)` 为 Filter、Elevated Filter 和 Input 选用 Expressive Tonal 默认外观。静止圆角为 Medium，选中为 Full，按下为 Small，未选中的前置图标使用主题的 `OnSurfaceVariant`。普通构造器沿用现有的平面 Assist 和 Small 圆角。可通过 `.tone(SemanticTone)` 在启用与选中状态下应用 Material 3 语义容器、标签、图标和边框色，同时保留禁用样式及显式 `.colors(ChipColors)` / `.border(ControlBorder)` 的优先级。用 `.shapes(ControlShapes)` 按状态设置圆角。用 `.colors(ChipColors)` 为启用、选中、禁用或禁用且选中的外观传入 `ChipStateColors`，每组可设置容器、标签、前置、尾部和边框颜色。省略的字段沿用所选变体默认值，禁用且选中的颜色优先回退到禁用覆盖值。

`.border(ControlBorder)` 设置轮廓宽度和回退颜色。零宽度隐藏轮廓。`.elevations(ControlElevations)` 可覆盖静止、按下、焦点、悬停、拖动和禁用时的阴影层级。`.dragged(bool)` 显示由应用持有的拖动状态，同时取消尚未完成的指针激活；手势由应用管理。`.spacing(width)` 设置内容插槽之间的水平间距，`.content_padding(LogicalEdges)` 设置逻辑内边距。默认间距为 8 dp，纯标签边缘为 16 dp，图标侧为 8 dp，头像起始侧为 4 dp。

为各标签提供不同的 ID，再用调用方持有的 `Entity<InteractionSource>` 和 `.interaction_source(&source)` 按该 ID 观察主操作。交互源报告悬停、按下、键盘焦点、拖动和禁用反馈；选中与移除仍由调用方控制。使用 `.track_focus(&focus)` 为主操作接入调用方持有的 `FocusHandle`，例如作为相邻标签 `.on_remove(...)` 的焦点恢复目标。

使用 `ChipGroup::new(id, name, items)` 与 `ChipGroupItem` 条目构建受控标签集合（`Role::Group`），默认采用 Material 3 8 dp 间距、RTL 镜像与单一漫游 Tab 停靠点（左右键、`Home`/`End`，自动跳过禁用项）。通过 `.selection(ChipGroupSelection, selected)`（`Single`、`SingleRequired` 或默认的 `Multiple`）与 `.on_change(|ids, window, cx| ...)` 管理选中状态，也可使用 `.single_select(selected, |id, window, cx| ...)` 作为 `SingleRequired` 的快捷写法。可通过 `.variant(ChipVariant)` 选择变体（默认 `Filter`，同样支持 `Input`），通过组级默认值 `.tonal(bool)` 与 `.elevated(bool)` 统一配置成员外观（单项可覆盖），通过 `.disabled(bool)` 整体禁用组内所有条目（与单项禁用取逻辑或），通过 `.scrollable(true)`（及可选 `.scroll_handle(&handle)`）切换为单行横向滚动而非换行布局，并通过 `.spacing(px)` 与 `.row_spacing(px)` 调整间距。每个 `ChipGroupItem` 支持 `.leading_icon`、`.avatar`、`.trailing`、`.disabled`、`.tonal`、`.elevated`、`.tone` 与 `.on_remove(label, handler)`。

### 滑块

调用方持有 `SliderState` 实体，范围与步进由 Base 管理。`SliderEvent::Change` 报告数值编辑，`SliderEvent::Release` 报告指针手势结束。键盘或无障碍操作改变数值时，先发出 `Change`，再发出 `Release`。数值未变时不会发出这两个事件。使用 `Slider::range` 时传入范围值和两个无障碍名称。单值滑块使用 `.kind(SliderKind::Centered)` 从中点开始填充。`.axis(Axis::Vertical)` 默认从下往上递增。设置 `.reverse_vertical(true)` 后，布局、指针移动和上下方向键改为从上往下递增。横向布局和方向键随窗口方向调整。聚焦滑柄时，`Home` 和 `End` 跳转到数值下限或上限，`PageUp` 和 `PageDown` 按较大步长（量程的 10% 且不小于一个无障碍步进）调整。拖动期间按 `Escape` 可取消手势并回滚到拖动前初始值。两种方向下，无障碍的值和边界都遵循数值顺序。

`.track_style(SliderTrackStyle)` 可按逻辑像素设置 `thickness`、`outer_radius`、`inner_radius` 和 `thumb_gap`。省略时沿用对应尺寸的默认轨道厚度和外侧圆角、2 dp 内侧圆角，以及距滑柄边缘 6 dp 的间隙。使用 `.thumb_style(RangeThumb, SliderThumbStyle)`（单值滑块可直接用 `.single_thumb_style(SliderThumbStyle)`）分别为滑柄设置 `size`、`pressed_width`、`color`、`disabled_color` 和 `focused_color`。`size` 的第一维沿数值轴，第二维垂直于数值轴。滑柄沿数值轴默认宽 4 dp，拖动时缩到 2 dp，另一维随当前尺寸变化。非有限数值和非正尺寸使用默认值。圆角和间隙可设为零。单值滑块调用范围接口时使用 `RangeThumb::End`。

`.colors(SliderColors)` 可配置滑柄、激活和未激活轨道、激活和未激活刻度在启用和禁用时的颜色。省略的字段沿用窗口配色。启用时滑柄和激活轨道使用 Primary，未激活轨道使用 Secondary Container，激活和未激活刻度分别使用 On Primary 和 On Secondary Container。禁用时，滑柄、激活轨道和刻度默认使用 38% 不透明度的 On Surface，未激活轨道使用 12% 不透明度的 On Surface。单个滑柄的配色优先于滑块配色。键盘焦点默认使用 Secondary。通过 `.thumb(RangeThumb, element)`（或 `.single_thumb(element)`）和 `.track(element)` 用装饰性内容替换默认滑柄和轨道。输入、焦点和无障碍语义由滑块处理，插槽内应避免放置独立控件。使用自定义轨道时，按需绘制内嵌图标、刻度和端点标记。

`.tick_intervals(count)` 设置可见刻度的区间数，Base 的步进单独配置。`.draw_ticks(...)` 和 `.draw_stop_indicators(...)` 接收带窗口坐标、默认颜色和默认直径的 `SliderMarker`，默认直径为 4 dp。设置 `.show_stop_indicators(false)` 可隐藏端点标记。拖动或通过键盘聚焦滑柄时默认显示数值指示器，保留两位小数。通过 `.value_indicator(false)` 关闭指示器，或传入 `.format_value(...)` 格式化当前显示值。

`.interaction_source(RangeThumb, &source)`（或 `.single_interaction_source(&source)`）可独立观察每个滑柄。查询键为 `ElementId::from((slider_id, "start"))` 和 `ElementId::from((slider_id, "end"))`。两个滑柄可共用一个交互源，也可分别观察。移除滑柄或更换交互源时，原快照会清除。传入 `.track_focus(RangeThumb, &focus)`（或 `.single_track_focus(&focus)`）可为指定滑柄接入调用方持有的 `FocusHandle`。

<a id="content-and-feedback"></a>

## 内容与反馈

| 组件 | 支持的变体与行为 | Material 3 |
| --- | --- | --- |
| `Text`、`Icon` | 主题排版（提供全部 15 个 Material 字阶快捷构造器 `display_large`–`label_small`、`.muted()`、`.align(TextAlign)`、`.max_lines(usize)` 与 `.color_override(Hsla)`）、可替换图标（支持 `.inherit_color()`、`.color_override(Hsla)`）和随库提供的 Material Symbols。 | [排版](https://m3.material.io/styles/typography/overview) · [图标](https://m3.material.io/styles/icons/overview) |
| `Surface` | 支持静态、可点击、可选及可勾选表面，提供主题内容颜色与层级。 | [层级](https://m3.material.io/styles/elevation/overview) |
| `Card` | Elevated、Filled、Outlined，支持静态或可操作的卡片及独立外观配置。 | [卡片](https://m3.material.io/components/cards/specs) |
| `List`、`ListItem`、`ListSection`、`SwitchTile`、`CheckboxTile`、`RadioTile`、`SliderTile`、`ChoiceTile` | Standard 和 Segmented 列表及带标题的列表分组，支持自定义插槽、受控行操作、各状态外观，以及开关、复选框、单选框、滑块与锚定选项菜单等设置项快捷行。 | [列表](https://m3.material.io/components/lists/overview) |
| `DropTarget` | Material 风格拖放容器，同时支持外部系统文件拖放（`ExternalPaths`）与应用内类型化负载，提供校验反馈及可选点击/键盘激活。 | [拖放交互](https://m3.material.io/foundations/interaction/gestures) |
| `SwipeToDismiss` | 横向滑动关闭，内容、背景及操作由调用方提供。 | [列表](https://m3.material.io/components/lists/guidelines) · [Compose SwipeToDismissBox](https://developer.android.com/reference/kotlin/androidx/compose/material3/SwipeToDismissBox.composable) |
| `Divider`、`Badge`、`Avatar`、`AvatarGroup` | 横向与纵向分隔线支持逻辑内缩、粗细和单独配色。徽标支持独立或附着显示，可使用圆点、文本、计数、自定义内容和单独配色。头像支持圆形或圆角，可呈现首字母、图标、带回退内容的图片或自定义元素；`AvatarGroup` 支持重叠堆叠、表面色镂空环与 `+N` 溢出计数。 | [分隔线](https://m3.material.io/components/divider/overview) · [徽标](https://m3.material.io/components/badges/overview) |
| `EmptyView` | 纯展示型的 Material 3 Expressive 空状态布局外壳，支持可选装饰图标或自定义视觉插槽（`MaterialShape` 或 `CornerShape` 衬底）、标题、辅助说明及居中操作槽位。 | [空状态](https://m3.material.io/foundations/content-design/overview) |
| `Skeleton` | 基于 `SurfaceContainerHigh`/`SurfaceContainerHighest` 的 M3 脉冲骨架占位（`rect`、`text_line`、`circle` 与 `shape`），视口外零帧调度并在低动效下保持静态。 | [加载与进度](https://m3.material.io/components/progress-indicators/overview) |
| `DisplayScrollbar` | 根据视口状态绘制任一方向的滚动位置，支持自定义尺寸、配色和淡出。 | [Compose 显示型滚动条](https://developer.android.com/reference/kotlin/androidx/compose/material3/nonInteractiveScrollbar.composable) |
| `LinearProgress`、`CircularProgress` | 确定和不确定进度，支持可配置的直线和波浪样式。 | [进度指示器](https://m3.material.io/components/progress-indicators/overview) |
| `LoadingIndicator` | Default 和 Contained 样式，支持确定与不确定进度、自定义形状序列和实例配色。 | [加载指示器](https://m3.material.io/components/loading-indicator/overview) |
| `PullToRefresh` | 支持手势与无障碍操作的受控刷新，提供可配置的指示器和滚动接管状态。 | [加载指示器](https://m3.material.io/components/loading-indicator/guidelines) |
| `Banner` | 流式 Material 3 横幅提示表面，支持 `SemanticTone` 色板、可选图标、标题、正文、1–2 个语义匹配操作或自定义操作插槽、可选关闭按钮、RTL 镜像与 `Role::Status`/`Role::Alert` 语义。 | [横幅](https://m3.material.io/components) |
| `Snackbar`、`SnackbarHost` | 定时或持续显示的排队消息，支持自定义内容和布局、可选操作与关闭，以及完成结果。 | [消息条](https://m3.material.io/components/snackbar/specs) |

### 表面

`Surface::new(role)` 提供静态背景。自定义控件可使用 `Surface::clickable(id, label, role)` 或 `Surface::selectable(id, label, role, selected)`，并通过 `.on_activate(...)` 处理激活。通过 `Surface::toggleable(id, label, role, checked)` 和 `.on_checked_change(...)` 处理勾选请求。标签提供无障碍名称，由调用方在每次渲染时传入当前选中或勾选状态。指针、回车、空格和无障碍操作调用同一处理函数。`.disabled(true)` 禁止激活，Tab 导航会跳过该表面。为每个表面提供稳定 ID，为独立操作使用独立表面。同组可选表面可放在具有无障碍名称的 `gpui::Role::ListBox` 中。

表面默认使用与主题背景角色配对的前景色。没有配对前景色的背景角色，以及通过 `.container_color(color)` 设置的容器颜色，沿用当前前景色。`.inherit_content_color(true)` 可让有配对色的背景角色也沿用当前前景，`.content_color(color)` 可覆盖内容颜色。颜色通过 GPUI 文本样式传递给子元素，包括未明确设置颜色的 Material 文字和图标。

通过 `.shape(CornerShape)` 设置逻辑方向圆角，或通过 `.polygon_shape(...)` 传入任意 `MaterialShape`、`PolygonRole`、自定义 `ShapePolygon` 或 `ShapeMorph` 采样轮廓。`.shape_fit(ShapeFit)` 可选保持宽高比的 `Contain`（多边形默认）或拉伸填充的 `Stretch`。通过 `.border(SurfaceBorder { width, color })` 设置以逻辑像素表示的描边。背景、外框和反馈使用这些设置。其他 GPUI Styled 布局与绘制接口仍可使用。GPUI 使用矩形内容区域裁剪溢出的子元素；在纯色背景上可传入 `.shape_clip_backdrop(color)`，以 EvenOdd 外部遮罩将溢出子元素视觉裁剪到形状轮廓内。`.tonal_elevation(Elevation)` 为使用主题 Surface 角色的背景着色，`.shadow_elevation(Elevation)` 独立设置阴影。

交互表面显示 Material 状态层、有边界的涟漪和键盘焦点环。`.feedback(SurfaceFeedback)` 配置单个表面的效果和颜色。多边形表面默认启用形状感知指针命中检测，使指针悬停、按下和点击忽略形状轮廓外的区域；可通过 `.shape_hit_test(bool)` 为任意表面配置该行为。键盘与无障碍激活不受影响。通过 `.interaction_source(&source)` 观察悬停、按下、键盘焦点和禁用状态，`.track_focus(&focus)` 接入调用方持有的焦点句柄。减少动态效果设置使用静态按下反馈。

### 卡片

通过 `Card::new(id).variant(CardVariant::Elevated)`、`Filled` 或 `Outlined`（或变体快捷构造器 `Card::elevated(id)`、`Card::filled(id)` 和 `Card::outlined(id)`）选择当前 Material 变体。静态卡片将内容分组（`Role::Group`）而不占用键盘焦点，支持通过 `.accessible_label(label)` 命名该分组。使用 `.on_activate(callback)` 可添加卡片操作及键盘焦点（可通过 `.accessible_label(label)` 提供无障碍名称），子控件仍各自处理操作。`.disabled(true)` 禁用卡片操作，并为卡片和继承内容提供禁用颜色。

使用 `.shape(CornerShape)` 设置容器、描边、阴影和交互反馈的形状；子内容按卡片圆角裁剪并随窗口书写方向调整。可通过 `.content_padding(LogicalEdges)`（默认 16 dp）与 `.gap(Pixels)`（默认 8 dp）自定义内部留白与子项间距。通过 `.border(ControlBorder { width, color })` 自定义描边，宽度为零时移除描边。`.colors(CardColors)` 独立覆盖启用和禁用时的容器色与内容色。`.elevations(ControlElevations)` 独立覆盖静止、按下、聚焦、悬停、拖动和禁用时的阴影高度。未指定的值沿用变体默认值，包括 Medium 圆角、Outlined 描边以及 Outlined 悬停时的平坦外观。禁用高度优先，其次依次为拖动、按下、键盘聚焦、悬停和静止。

调用方可根据拖动生命周期设置 `.dragged(bool)`，并在完成或取消时清除该状态。拖动会中断尚未完成的指针激活并在拖动期间阻止点击激活。传入 `.interaction_source(&source)` 可通过共享的 `Entity<InteractionSource>` 观察卡片。同一窗口中每张受观察的卡片需要有独立标识。可操作的卡片可通过 `.track_focus(&focus)` 接入调用方持有的 `FocusHandle`。

### 列表

使用 `List::new(id).item(row)` 或 `.items(rows)` 创建 Standard 列表，或用 `.variant(ListVariant::Segmented)` 设置分组圆角和 2 像素行间距，支持在 `Role::List` 容器上通过 `.accessible_label(label)` 设置无障碍名称。为每个 `ListItem::new(id, label)` 提供稳定的 ID 和无障碍名称。`.content(element)`、`.overline_content(element)` 和 `.supporting_content(element)` 接收任意元素。行操作的无障碍名称仍来自标签。`.overline(text)` 和 `.supporting(text)` 使用 Material 字体样式，辅助文本会换行。`.leading(element)` 和 `.trailing(element)` 随窗口书写方向排列。内嵌控件独立接收焦点和激活。

用 `.on_activate(...)` 设置行操作。`.selected(bool)` 在统一的具名行无障碍节点上控制选中样式和语义。`.checked(bool).on_checked_change(...)` 独立提供勾选值和请求反转值的回调。具备勾选值的行提供复选框角色。`.expanded_content(expanded, element).on_expanded_change(...)` 提供展开状态，通过激活该行请求切换。展开内容渲染在行下方，可包含独立操作的控件。各状态更新由应用负责。列表的游标式 Tab 停靠点可用上下方向键以及 `Home` / `End` 在启用的行操作间移动。`.track_focus(&focus)` 与 `.tab_stop(bool)` 允许虚拟列表或调用方提供自定义 `FocusHandle` 和漫游 Tab 停靠点，`List` 与 `ListSection` 均会保留调用方提供的句柄。

用 `.on_long_press(label, callback)` 设置具名次要操作。在行的前置、主体或尾部任意区域按住指针、桌面端右键点击（`MouseButton::Right`）、`Shift+F10`、`contextmenu` 键或平台支持的无障碍上下文操作均可触发。提前松开、开始拖动或中断输入都会取消长按。`.long_press_button(id)` 在逻辑尾部添加使用相同名称和回调的可见按钮，适用于缺少次要上下文操作的平台。长按完成后会消费对应的行点击。

用 `.swipe_to_dismiss(&state, |swipe| ...)` 将现有的 `SwipeToDismiss` 与行组合，使用其保留状态及配置方法。调用方处理关闭结果，并按下文提供具名且可用键盘操作的命令。纵向滚动仍可用。已滑动关闭的行离开焦点遍历，直到重置或移除。

`.shapes(ListItemShapes)` 覆盖静止、选中、按下、聚焦、悬停和拖动状态的逻辑圆角。`.tone(SemanticTone)` 使用 Material 3 语义色板着色启用与选中态的标签、前置与尾部角色，同时保留禁用样式与显式 `.colors(ListItemColors)` 覆写优先级。`.colors(ListItemColors)` 通过 `ListItemStateColors` 分别配置普通、禁用、选中和拖动状态下的容器、主体、前置、尾部、上方标签及辅助内容颜色。省略的字段沿用窗口默认值。`.vertical_alignment(ListItemAlignment)` 可选 Auto、Start、Center 或 End。Auto 将较矮的行内容居中，将较高文本行的内容置顶。`.content_padding(LogicalEdges)` 指定逻辑起始侧、尾侧、顶部和底部的内边距（默认水平 16 逻辑单位、垂直 10 逻辑单位，三行内容垂直 12 逻辑单位），行最小高度随 `theme.layout.density` 缩放。`.elevations(ControlElevations)` 覆盖静止及交互状态的阴影高程。`.dragged(bool)` 提供由调用方管理的拖动状态，`.interaction_source(&source)` 可观察行的悬停、按下、键盘焦点、拖动和禁用状态。

### 设置项快捷行与列表分组

`SwitchTile::new(id, label, checked)` 将 `ListItem` 与状态同步的尾部视觉 `Switch` 组合（启用 `settings-tiles` 特性时可用）。点击整行任意区域或按 Enter / Space 均通过 `.on_change(...)` 单次切换，仅占一个焦点停靠点并暴露 `Role::Switch` 语义，避免双重焦点或重复切换。可通过 `.supporting`、`.overline`、`.leading`、`.disabled`、`.variant(ListVariant)`、`.colors(ListItemColors)`、`.switch_colors(SwitchColors)`、`.track_focus(&focus)` 和 `.interaction_source(&source)` 独立配置行和开关外观及状态观察。

`CheckboxTile::new(id, label, state)`（或 `CheckboxTile::bool(id, label, checked)`）将 `ListItem` 与状态同步的装饰性 `Checkbox` 组合（需要 `settings-tiles`）。点击整行或按 Enter / Space 通过 `.on_change(...)` 请求下一个 `CheckboxState`（`Unchecked` 或 `Indeterminate` → `Checked`；`Checked` → `Unchecked`），仅占一个焦点停靠点并暴露 `Role::CheckBox` 语义。支持 `.supporting`、`.overline`、`.leading`、`.trailing(element)`（当 `.trailing_checkbox(false)` 将复选框置于前侧时显示）、`.trailing_checkbox(bool)`（默认置于尾侧）、`.error(bool)`、`.disabled(bool)`、`.variant(ListVariant)`、`.colors(ListItemColors)`、`.checkbox_colors(CheckboxColors)`、`.track_focus(&focus)` 与 `.interaction_source(&source)`。

`RadioTile::new(id, label, selected)` 将 `ListItem` 与状态同步的装饰性 `Radio` 组合（需要 `settings-tiles`）。点击未选中的整行或按 Enter / Space 触发 `.on_select(...)`，仅占一个焦点停靠点并暴露 `Role::RadioButton` 语义。支持 `.supporting`、`.overline`、`.leading`、`.trailing(element)`（当 `.trailing_radio(false)` 将单选框置于前侧时显示）、`.trailing_radio(bool)`、`.disabled(bool)`、`.variant(ListVariant)`、`.colors(ListItemColors)`、`.radio_colors(RadioColors)`、`.track_focus(&focus)` 与 `.interaction_source(&source)`。

`SliderTile::new(id, label, &state)` 将 `ListItem` 行头（支持 `.overline`、`.leading`、`.supporting` 及尾侧通过 `.format_value(...)` 实时格式化的当前值）与绑定到调用方 `Entity<SliderState>` 的内嵌 `Slider` 组合（需要 `settings-tiles`）。支持 `.disabled(bool)`、`.variant(ListVariant)`、`.colors(ListItemColors)`、`.track_focus(&focus)`、`.interaction_source(&source)` 与 `.slider(|slider| ...)` 配置闭包，无需重复暴露全部滑块方法。

`ChoiceTile::new(id, label, options, selected)` 将 `ListItem` 与由 `ChoiceOption` 构成的锚定 Material `Menu` 组合（需要 `settings-tiles`）。行尾显示当前选中项的标签（或 `.placeholder(...)`）与下拉箭头，并通过行的无障碍描述向辅助技术暴露该当前值；菜单中以勾选标记标识当前选项，选中后通过 `.on_select(...)` 报告选项 ID 并在关闭时恢复焦点。可传入外部 `.menu_state(&state)`（`Entity<DialogState>`）进行受控管理，或由组件保留内部的键控 `DialogState`，并支持 `.track_focus(&focus)` 与 `.interaction_source(&source)`。

`ListSection::new(id)`（或 `ListSection::segmented(id)`，需要 `surfaces` 或 `settings-tiles`）在可选的 `.title(...)` 分组标题（`LabelMedium` 与逻辑水平缩进）下通过 `.item(...)` / `.items(...)`（`ListSectionItem`）混排任意 `ListItem`、`SwitchTile`、`CheckboxTile`、`RadioTile`、`SliderTile` 和 `ChoiceTile`。在 `ListVariant::Segmented` 模式下应用 Material 3 Expressive 分组圆角几何（首项 `[16, 16, 4, 4]`、中间项 `[4, 4, 4, 4]`、末项 `[4, 4, 16, 16]`、单项 `[16, 16, 16, 16]` 及 2 dp 间距），并在交互行之间接入统一的纵向漫游焦点（`Up`/`Down`/`Home`/`End`，跳过禁用行，并保留调用方提供的 `.track_focus(...)` 句柄），同时保留 `SliderTile` 内嵌滑块的原生键盘调节。

### 拖放目标

`DropTarget::files(id)` 接收外部系统文件拖放（`gpui::ExternalPaths`），`DropTarget::new::<T>(id)` 接收任意应用内 GPUI 类型化拖拽负载 `T: 'static`。可通过 `.title`、`.supporting` 和 `.icon` 呈现默认占位拖放区，或通过 `.content(element)` 包裹自定义内容。`.can_drop(|payload, window, cx| ...)` 在悬停和释放时校验负载；通过校验的负载使用主色高亮，被拒绝的负载显示错误反馈且不触发 `.on_drop(...)`。设置 `.on_activate(...)` 可支持点击和键盘激活（在同时设置 `.disabled(true)` 时保持禁用态的 `Role::Button`），`.shape(CornerShape)` 按实际布局边界解析圆角，`.colors(DropTargetColors)` 覆盖静止/有效/无效配色，`.disabled(true)` 禁用拖放注册和激活，`.track_focus(&focus)` 与 `.interaction_source(&source)` 接入焦点与状态观察，`.accessible_label(...)` 提供无障碍名称。

### 分隔线

`Divider::horizontal()` 和 `Divider::vertical()` 默认使用窗口主题的 `OutlineVariant` 颜色，厚度为一个逻辑像素。通过 `.thickness(px(2.))` 设置逻辑像素厚度，通过 `.thickness(px(0.))` 绘制一个物理像素的细线。无效的厚度值回退到默认值。纵向分隔线填满父布局的可用高度。

使用 `.color(ColorRole::Primary)` 按窗口主题解析颜色角色，或使用 `.color_override(hsla)` 指定固定颜色。颜色设置以最后一次调用为准。`.inset(start, end)` 在横向布局中随窗口文字方向调整左右内缩，在纵向布局中分别设置顶部和底部内缩。分隔线是装饰元素，不增加键盘焦点。

### 徽标

`Badge::dot(host)`、`Badge::label(host, text)` 和 `Badge::count(host, count, maximum)` 将徽标附着在宿主的右上角，RTL 布局下则附着在左上角。渲染时传入当前计数和上限。计数超过上限时，显示上限并追加 `+`。`Badge::standalone()`、`Badge::standalone_label(text)` 和 `Badge::standalone_count(count, maximum)` 在当前布局中独立绘制圆点、文本或上限计数徽标。可通过 `.visible(false)` 隐藏徽标（附着模式仅渲染宿主，独立模式不渲染内容）、`.offset(Point<Pixels>)` 微调附着偏移，以及 `.content(id, element)` 为两种放置方式提供简短的自定义视觉元素。请为同一父容器中的自定义徽标提供稳定标识。任意 `IntoElement` 也可通过 `MaterialElementExt` 以后缀链式方式附加徽标（`.badge_dot()`、`.badge_label(text)`、`.badge_count(count, maximum)`）。

徽标容器和内容默认使用窗口主题的 `Error` 和 `OnError` 颜色角色。可使用 `.tone(SemanticTone)` 应用 Material 语义状态配色（`Default` 与 `Danger` 使用 `Error`/`OnError`；`Success`、`Warning`、`Info` 使用对应语义色对），或通过 `.color_role(container, content)` 指定主题颜色角色。显式 `.container_color(color)` 与 `.content_color(color)` 优先于 `.color_role(...)`，而 `.color_role(...)` 优先于 `.tone(...)`。自定义内容应为非交互元素，附着徽标的信息由宿主的无障碍名称提供。独立传达信息的徽标可使用 `.accessible_label(id, label)` 提供无障碍名称，无需增加键盘焦点。

### 头像

`Avatar::initials(text)`、`Avatar::icon(icon)`、`Avatar::image(source)` 和 `Avatar::custom(content)` 以 `ExtraSmall`（24 px）、`Small`（32 px）、`Medium`（40 px，默认）、`Large`（48 px）、`ExtraLarge`（64 px）或 `Custom(Pixels)` 尺寸呈现用户或实体头像，通过 `.size(AvatarSize)` 切换。首字母模式自动裁剪、转为大写并按尺寸匹配 Material 字阶。图片模式以 `ObjectFit::Cover` 裁剪至 `AvatarShape::Circle`（默认）或 `AvatarShape::Rounded(CornerShape)`，并在加载中或加载失败时显示可选的 `.fallback(|| element)`。支持 `.color_role(container, content)`（默认 `PrimaryContainer` / `OnPrimaryContainer`）、`.colors(AvatarColors)`、`.border(width)`，以及暴露 `Role::Image` 而不增加焦点停靠点的 `.accessible_label(id, label)`。

使用 `AvatarGroup::new(id, avatars)` 渲染横向重叠的 `Avatar` 堆叠。`.size(AvatarSize)` 为成员头像与溢出徽章应用统一尺寸（默认 `Medium`）。`.max_visible(count)` 在数量超限时截断并追加使用 `SurfaceContainerHighest` / `OnSurfaceVariant` 配色的 `+N` 溢出头像，可通过 `.overflow_colors(AvatarColors)` 或 `.overflow_content(|count| ...)` 自定义。可通过 `.overlap(pixels)` 配置相邻重叠距离（默认直径的 25%，并限制在严格小于直径的非负范围内），并通过 `.ring_width(pixels)`（默认 2 px）、`.ring_role(ColorRole)`（默认 `Surface`）或 `.ring_color(color)` 配置与表面匹配的镂空描边环。堆叠顺序与负外边距在 RTL 窗口中自动镜像。`.accessible_label(label)` 暴露汇总的 `Role::Group` 无障碍标签且不增加额外焦点停靠点。

### 空状态

`EmptyView::new(id, title)` 提供纯展示型的 Material 3 Expressive 空状态布局外壳，不包含内置加载或重试状态机。通过 `.icon(element)` 在装饰性 `Surface` 衬底中渲染图标（默认使用 `MaterialShape::Cookie7Sided` 与 `SecondaryContainer` / `OnSecondaryContainer`），可用 `.icon_container_shape(shape)`（接收任意 `ShapePolygon`、`MaterialShape` 或 `PolygonRole`）、`.shape(CornerShape)` 和 `.icon_container_color_role(container, content)` 自定义；也可通过 `.visual(element)` 用任意自定义插画替换图标容器。通过 `.supporting(text)`（`BodyMedium` / `OnSurfaceVariant`）或 `.supporting_content(element)` 提供辅助说明，用 `.max_width(pixels)`（默认 360 dp）和 `.title_role(TypeRole)`（默认 `TitleLarge`）调整内容宽度与标题层级，并将调用方控件放入 `.primary_action(element)` 与 `.secondary_action(element)`（在居中横向行中以 12 dp 间距和 24 dp 顶边距排列，并在 RTL 下镜像）。图标容器为纯装饰元素不占焦点，标题、辅助文本及调用方按钮保持各自正常的无障碍语义与键盘焦点。

### 骨架屏占位

`Skeleton::rect(id, width, height)`、`Skeleton::text_line(id)`、`Skeleton::circle(id, diameter)` 与 `Skeleton::shape(id, width, height, shape)` 在无可选特性依赖的核心中渲染 Material 3 加载占位。`text_line` 默认铺满宽度、高 16 dp 并使用 `CornerRole::Small`，支持 `.width(...)`、`.height(...)`、`.role(TypeRole)` 与 `.full_width()`；`shape` 接收任意 `ShapePolygon`（包含 `MaterialShape::Cookie7Sided` 等表达性形状）。可通过 `.corner(CornerShape)` 或 `.corner_role(CornerRole)` 配置圆角，并通过 `.colors(SkeletonColors)` 覆写底色与高亮色（默认为 `SurfaceContainerHigh` 与 `SurfaceContainerHighest`）。占位默认纯装饰且不占 Tab 焦点，按需通过 `.accessible_label(label)` 暴露 `Role::Status` 语义。动效由 `foundation::LoopClock` 驱动并受 `is_bounds_visible` 门控：离屏或被裁剪时调度零帧动画，`.animated(false)` 或系统开启减少动态效果时以静态底色渲染且调度零帧动画。

### 滑动关闭

为每项保留独立的 `Entity<SwipeToDismissState>`，传给 `SwipeToDismiss::new(id, &state, content)`。状态默认从初始位置开始。恢复已滑出条目时，可用 `.initial_value(value)` 指定初始锚点。通过 `.background(element)` 设置填满容器的背景，显示在移动的前景内容后面。观察状态后，可用逻辑方向 `dismiss_direction()`、带符号的逻辑像素偏移 `offset()` 或从 0 到 1 的滑出进度 `progress()` 更新背景。`current_value()` 表示最接近的锚点，`target_value()` 表示预计停留的锚点，`settled_value()` 在移动完成后更新。

默认启用两个逻辑方向和滑动手势。使用 `.enable_start_to_end(bool)`、`.enable_end_to_start(bool)` 和 `.gestures_enabled(bool)` 可分别设置。窗口的书写方向决定实际滑动方向。默认阈值为 56 个逻辑像素。使用 `SwipeToDismissState::positional_threshold(|width| ...)` 按布局宽度计算阈值。有限值限制在可滑动距离内，无效值使用默认阈值。纵向拖动和滚动仍交给周围布局处理。

`dismiss(direction, window, cx)` 以动画滑至启用的方向，`reset(window, cx)` 以动画回到初始位置，`snap_to(value, window, cx)` 立即移动。减少动态效果时立即定位。移动停在新的关闭方向时，`.on_dismiss(|direction, window, cx| ...)` 执行一次，前景同时退出键盘遍历和无障碍树。数据变更由应用负责，例如移除条目。应用也可调用 `reset` 重新显示。为每个操作提供有名称且能获得键盘焦点的按钮或菜单项。激活时调用同一个 `dismiss` 方法，在手势禁用时也可以使用。

### 进度指示器

创建 `LinearProgress` 或 `CircularProgress` 时提供稳定的 ID 和无障碍名称。调用方持有确定进度值，通过 `.value(fraction)` 传入。有限值限制在 `0..=1`。省略数值或调用 `.indeterminate()` 可显示不确定进度，非有限值也会选用该模式。确定模式的无障碍进度条公开当前目标百分比，不确定模式无数值。

通过 `.shape(ProgressShape::Wavy)` 选用波浪形进度线。各模式均支持 `.tone(SemanticTone)`、`.active_stroke(ProgressStroke)`、`.track_stroke(ProgressStroke)` 和 `.gap(size)`。两条线可分别设置线宽、颜色和 `ProgressCap` 端部样式（Round、Butt 或 Square）。默认使用 4 dp 圆端描边、4 dp 间隔，以及窗口主题的 `Primary` 和 `SecondaryContainer` 色；语义色调将进度线映射到对应主题色角色（`Error`、`Success`、`Warning`、`Info`）并将轨道映射到其容器角色（`*Container`），显式传入的 `.active_stroke` 和 `.track_stroke` 颜色始终优先。线宽为零时隐藏该描边。负值或非有限线宽及间隔采用默认值。绘制时将描边限制在可用边界内，间隔也不会超出轨道长度。线性指示器随窗口书写方向镜像，环形指示器从顶部顺时针绘制。

使用 `.wave_amplitude(fraction)` 或 `.wave_amplitude_by(|progress| …)` 设置波浪在可用空间内的振幅。回调在确定模式下接收正在绘制的进度，在不确定模式下接收零。振幅限制在 `0..=1`，NaN 按零处理。确定模式默认在进度不超过 10% 或达到 95% 时采用平线，不确定模式的默认振幅为最大值。确定进度为零或 100% 时为平线。`.wavelength(length)` 以逻辑像素设置波峰间距，环形绘制会在一圈内排列整数个波形。`.wave_speed(length_per_second)` 的单位为逻辑像素每秒，默认每秒移动一个波长，速度为零时保持当前相位。无效的波长或速度采用默认值。减少动效时保持静态形状，并即时应用确定进度的变化。

线性确定进度默认在轨道逻辑末端绘制 4 dp 的终点标记。用 `.stop_size(size)` 调整尺寸，零会隐藏标记，无效尺寸采用默认值。轨道与周围表面的对比度低于 3:1 时应保留可见标记。通过 `.draw_stop_indicator(|stop, window, cx| …)` 替换默认绘制。`ProgressStop` 提供标记边界、进度线颜色和端部样式，以及布局方向。仅在标记可见时调用绘制函数，按标记边界裁切，且不添加无障碍节点。

### 加载指示器

`LoadingIndicator::new(id, label)` 占用 48 dp 的空间，默认循环播放七种形状，表示不确定进度。用 `.contained(true)` 添加圆形容器。Default 的指示形状使用窗口的 `Primary` 色；Contained 使用 `OnPrimaryContainer` 绘制指示形状，使用 `PrimaryContainer` 绘制容器。两种样式均可用 `.indicator_color(color)` 覆盖指示形状的颜色。Contained 可用 `.container_color(color)` 和 `.container_shape(CornerShape)` 配置容器。起始侧和末尾侧的圆角随窗口布局方向调整。

确定进度由应用持有，每次渲染时用 `.value(fraction)` 传入。进度值限制在 `0..=1`，NaN 按零处理。默认序列在零时显示圆形，在一时显示 Soft Burst。用至少两个 `LoadingIndicatorShape` 构造可复用的 `LoadingIndicatorShapes`，再用 `.shapes(&shapes)` 传引用。可使用 `LoadingIndicatorShape::circle()`、`LoadingIndicatorShape::material(MaterialShape)`、`LoadingIndicatorShape::from(ShapePolygon)`，或用 `LoadingIndicatorShape::rounded_polygon(&vertices, radius)` 传入 GPUI 顶点和同单位的半径。多边形坐标或半径非有限、形状退化或自相交时返回 `None`。形状少于两个时，序列构造器返回 `None`。确定进度从序列首形状变换到末形状，不确定进度还会从末形状变回首形状。

标签为进度指示器提供无障碍名称。确定模式公开数值百分比并直接绘制传入的进度。不确定模式提供礼貌播报的状态，无数值。减少动态效果时，不确定模式停留在首形状，确定模式继续响应进度变化。`.active(false)` 保留布局空间，同时隐藏图形和无障碍进度信息。只有活动窗口内处于活动状态的不确定指示器才会请求动画帧。

### 下拉刷新

每个视口创建一个 `Entity<PullToRefreshState>`，并在渲染组件的视图中观察它。使用 `PullToRefresh::new(id, &state, refreshing, on_refresh)`，再通过 GPUI 的 `ParentElement` API 添加子元素。容器创建垂直滚动视口，保留可通过 `scroll_handle()` 获取的滚动句柄。需要包装已有滚动控件时，用 `.track_scroll(&handle)` 连接滚动句柄，或用 `.scroll_position(|window, cx| ...)` 返回距滚动起点的正向距离，单位为逻辑像素。子元素保留滚动行为，外层在滚动到顶部时接管下拉。

触屏向下拖动或带阶段的精确滚动到达顶部后可拉出指示器。反向拖动时先收回指示器，再恢复垂直滚动。水平移动和普通滚轮继续执行滚动。默认阈值为 80 逻辑像素，到达阈值前指示器以拖动速度的一半移动。用 `.threshold(pixels)` 设置阈值，用 `.indicator_distance(pixels)` 独立设置指示器移动距离，默认值为 80 逻辑像素。`.enabled(false)` 会取消当前下拉并禁用刷新操作。取消手势会舍弃尚未触发的释放操作。

应用持有 `refreshing` 参数。超过阈值后释放，或激活具名刷新按钮，均只发出一次请求。按钮支持键盘与辅助技术，它的名称由 `.refresh_label(label)` 指定。在应用传入 `refreshing = true` 接受请求，或调用 `reject_refresh(cx)` 拒绝请求前，组件暂停发送后续请求。刷新期间无法再次刷新。操作完成后重新传入 `refreshing = false`。

居中的 Contained 加载指示器跟随下拉进度，刷新期间切换为不确定进度。用 `.indicator(|progress, window, cx| ...)` 提供自定义装饰内容。`PullToRefreshProgress` 提供距离比例、请求等待或刷新状态、指示器偏移与动画状态。可通过保留状态读取 `distance_fraction()`、`is_animating()` 和 `is_refresh_pending()`。`snap_to(fraction, cx)`、`animate_to_threshold(cx)`、`animate_to_hidden(cx)` 与 `reset(cx)` 可以独立于应用刷新值控制呈现。减少动态效果时，指示器直接移动到目标位置，不确定进度保持为静态形状。

### 横幅

`Banner::new(id, message)` 渲染非阻塞的流式 Material 3 横幅提示表面（`feedback` 特性），可见性完全由调用方持有，不含内部隐藏状态。可通过 `.title(text)` 添加可选标题行，通过 `.icon(element)` 添加前置装饰图标（自动匹配色调前景色），通过 `.action(label, handler)` 与 `.secondary_action(label, handler)` 渲染最多两个匹配语义色调的 Material 文本按钮（或通过 `.actions_content(element)` 提供自定义操作区），并通过 `.on_dismiss(handler)` 附加尾侧关闭 `IconButton`（无障碍名称由 `.dismiss_label(label)` 指定，默认为 `"Dismiss"`）。`.tone(SemanticTone)` 在 `Default`（`SurfaceContainerLow` 容器、`OnSurface` 文案、`Primary` 图标、`OutlineVariant` 描边）与 `Danger`/`Success`/`Warning`/`Info` 语义色对之间切换，显式 `.colors(BannerColors)` 始终优先于 `.tone`。可通过 `.shape(CornerShape)`（默认 `CornerRole::Medium`）调整圆角，通过 `.alert(bool)` 选择 `Role::Alert` 或 `Role::Status`（`SemanticTone::Danger` 默认为 `Role::Alert`）。图标、文字对齐、操作行与关闭按钮均随 RTL 镜像。

### 消息条

为窗口保留一个 `Entity<SnackbarHost>`，将其挂载到布局或传给 `Scaffold::snackbar`。通过 `push` 将 `Snackbar::new(message)` 添加到先进先出的队列。普通消息默认显示四秒。设置 `.action(label, callback)` 后，消息默认持续显示。可在 `.action(...)` 之后通过 `.timeout(Some(duration))` 覆盖时长，或使用 `.timeout(None)` 等待关闭。每条消息成为当前项时开始计时。

使用 `.message_content(|window, cx| ...)` 替换可见消息，`Snackbar::new` 仍提供无障碍状态文本。将 `.action_content(...)` 与 `.action(...)` 一同使用，可替换有名称的操作内容。`.dismiss_label(label)` 可在保留默认关闭图标的前提下自定义关闭按钮的无障碍标签，`.dismiss_content(label, |window, cx| ...)` 则替换有名称的关闭控件内容。默认显示关闭图标，设置 `.show_dismiss(false)` 可隐藏该控件。插槽内容应为装饰元素，由宿主负责焦点、激活和完成流程。`.action_on_new_line(true)` 将操作和可选的关闭控件放在消息下方，并将这一行对齐到逻辑尾侧。

使用 `.shape(CornerShape)` 设置容器形状，通过 `.tone(SemanticTone)` 选用 Material 语义状态色板，通过 `.colors(SnackbarColors)` 分别设置容器、消息、操作和关闭内容的颜色。默认圆角为 4 dp。`SemanticTone::Default` 使用 Inverse Surface 作为容器色、Inverse On Surface 作为消息和关闭内容色、Inverse Primary 作为操作色；语义色调（`Danger`、`Success`、`Warning`、`Info`）分别使用对应 `*Container`、`On*Container` 和基础语义角色，显式传入的 `SnackbarColors` 始终优先。宿主以礼貌模式的实时状态播报消息，保留当前键盘焦点。定时消息条在指针悬停或内部控件持有焦点时会暂停自动关闭计时，并在悬停与焦点移开后重新计时。具名控件可以正常通过 Tab 遍历并激活。消息条的控件获得焦点时，可以按 Escape 关闭消息条。消息条持有焦点时完成操作，焦点将回到仍可用的原目标。

需要 `SnackbarId` 时用 `enqueue`（纯消息快捷方法为 `push_message(message, window, cx)`）。调用 `replace(snackbar, window, cx)` 会干净地关闭当前显示的消息条（如有，并触发其关闭完成回调），立即将新消息条置于待展示队列之前并显示。`current()` 返回当前项的标识和消息，`pending_len()` 返回排在当前项之后等待展示的数量，`clear_queue(cx)` 清空待展示队列而不影响当前项，`dismiss()` 关闭当前项。`dismiss_message(id, window, cx)` 和 `perform_action(id, window, cx)` 只对当前项生效，且 `perform_action` 要求存在操作。通过 `.on_result(|result, window, cx| ...)` 观察 `SnackbarResult::ActionPerformed` 或 `SnackbarResult::Dismissed`。超时、显式关闭和程序关闭均报告 `Dismissed`，并在提供 `.on_dismiss(...)` 时调用它。每条消息只交付一次结果。队列先推进，再执行回调，因此回调可以继续操作同一个宿主的队列。释放宿主会取消计时并丢弃剩余消息。

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
| `Label` | 锚定控件的静态标签，支持共享交互状态和持续显示。 | [Compose Label](https://developer.android.com/reference/kotlin/androidx/compose/material3/Label.composable) |
| `Scrim` | 独立遮罩，支持主题配色、自定义颜色、透明度和可选关闭操作。 | [遮罩](https://developer.android.com/reference/kotlin/androidx/compose/material3/Scrim.composable) |
| `Dialog` | 结构化基本对话框、自适应全屏任务对话框、自定义内容的模态外壳，以及按需打开的原生对话框窗口。 | [对话框](https://m3.material.io/components/dialogs/overview) |
| `Sheet` | 标准和模态底部面板，以及在逻辑起始侧或末尾侧停靠或覆盖的侧面板。 | [底部面板](https://m3.material.io/components/bottom-sheets/overview) · [侧面板](https://m3.material.io/components/side-sheets/overview) |

`TextField` 为调用方持有的 Base 编辑器提供外观。可使用 `TextField::filled(id, label, &state)` 或 `TextField::outlined(id, label, &state)` 作为 `TextField::new(id, label, &state).variant(...)` 的快捷构造器。使用 `label_content`、`placeholder_content` 和 `supporting_content` 时，纯文本分别提供无障碍名称、提示和说明。首尾元素与紧邻输入值的前后缀分开。标签收起时显示前后缀，且前后缀不进入存储值。设置 `.show_label(false)` 可省略视觉标签及垂直标签预留高度，同时保留构造器标签作为无障碍名称并使占位符与前后缀立即显示。默认容器最小高度随 `theme.layout.density` 调整（Compact 52 dp、Standard 56 dp、Comfortable 60 dp），也可通过 `.min_height(Pixels)` 覆盖最小高度直至贴合文本与内边距（负数或非有限值将被忽略）。使用 `.indicator_width(unfocused, focused)`（默认未聚焦 1 px、聚焦 2 px，负数或非有限值将被忽略）可自定义 Filled 底部指示条或 Outlined 边框宽度，传入 `px(0.)` 可干净移除。设置 `.clearable(true)`（可用 `.clear_label(label)` 覆盖默认标签 `"Clear"`）后，在字段可编辑且内容非空时自动显示尾部清除按钮（`IconName::Close`），激活后通过 Base `replace_all("")` 清空并保持编辑器焦点，且指针按下时不会触发中间失焦；同时配置 `.clearable(true)` 与自定义 `.trailing(...)`（或 `.visibility_toggle(...)`）时，内置控件按逻辑顺序排在自定义尾元素之前。点击容器或标签内会聚焦编辑器，点击容器和标签之外（包括底部辅助文本行与非可聚焦空白区）会使编辑器失焦，除非打开中的 `ExposedDropdown` 弹出层接管该次点击；已聚焦的编辑器被禁用时也会主动失焦。Filled 默认使用内部标签，Outlined 默认使用缺口标签。可通过 Above 将标签放在容器上方，也可设置水平对齐方式，让内部或缺口标签始终保持收起。字符计数按 `value()` 的 Unicode 标量值统计，输入长度仍由编辑器处理。可通过 `.disabled(bool)` 在组件侧禁用输入字段（与编辑器状态中的禁用标志取逻辑或）。可独立于窗口主题设置实例配色、容器形状、逻辑内容内边距（`LogicalEdges`）和输入文字样式。`on_text_layout` 在帧结束后报告编辑器与文本的布局边界和换行后的行数。通过 `interaction_source` 共享悬停、键盘焦点和禁用状态的观测结果。Base 实体继续管理编辑、选区、校验、滚动和输入法组合。

安全字段请使用以 `.masked(true)` 初始化的独立 `TextFieldState` 并搭配 `TextField::secure` 渲染。可通过 `.obfuscation_mode(TextFieldObfuscationMode::Hidden | Visible)`（默认 `Hidden`）手动控制显隐，或调用 `.visibility_toggle(visible, on_toggle)`（可用 `.visibility_labels(show, hide)` 覆盖默认的 `"Show password"` / `"Hide password"`）自动同步掩码模式并渲染使用内置 `IconName::Visibility` 与 `IconName::VisibilityOff` 符号的尾部显隐切换按钮。`Hidden` 使用 Base 固定的圆点（`•`）遮罩，禁用复制与剪切，并隐藏无障碍值、无障碍 `SetValue` 与字符计数；`Visible` 切换为普通明文输入，与桌面和 Web 原生密码切换惯例保持一致。作为桌面适配边界，不提供自定义掩码字符及移动端短暂显露末字符的计时模式。

如需在提交前拦截非法输入，使用 Base 的 `TextFieldState::validate` 或 `pattern`，覆盖键入、粘贴、输入法组合和无障碍 `SetValue`。结构化格式与数字分组使用 `TextFieldMaskPattern` 配合 `TextFieldState::mask_pattern` 和 `unmask_value()`；固定纯展示前后缀使用 `prefix` 和 `suffix`。任意内容规范化（如大小写转换）在应用的 `TextFieldEvent::Change` 或 `Blur` 处理中完成；提交后的改写会形成独立的撤销步骤，不再额外维护独立的仅显示偏移映射层。

### 公开式下拉菜单

每个下拉菜单创建一个 `ExposedDropdownState`，由它保存待提交的查询、已提交的选择和列表位置。用 `is_open()` 读取当前显示状态，用 `selected_option_id()` 读取最近提交的选项 ID。`with_editor(&editor, editable, window, cx)` 使用调用方持有的 `TextFieldState`。`.editor(|field| ...)` 可以配置或包装传入的 `TextField`，包括标签、装饰和编辑器布局。将这个字段保留在返回元素中，可保留无障碍名称以及原生编辑、光标和输入法行为。

默认可编辑字段在输入或激活尾部触发器时展开，并保留编辑器焦点。默认仅选择字段通过点击或空格展开，同样保留字段焦点。`.primary_anchor(DropdownAnchor::PrimaryEditable)` 让整个字段可以触发菜单并保留编辑器焦点。仅选择状态使用 `PrimaryNotEditable` 可在展开时将焦点移至列表，关闭时回到字段。`.secondary_anchor(label, |open| ...)` 自定义尾部触发器及其无障碍名称。使用指针激活次要锚点时仍可继续编辑，通过无障碍操作激活可编辑字段的次要锚点时焦点进入选项列表。`.primary_anchor_enabled(false)` 和 `.secondary_anchor_enabled(false)` 可分别关闭相应锚点的菜单激活，仍可使用编辑器。

通过 `.open(open)` 和 `.on_open_change(|requested, window, cx| ...)` 控制菜单显示，并可通过 `.disabled(bool)` 同时禁用输入与弹出触发器。由调用方处理请求并重新渲染。`.on_dismiss(...)` 报告用户发出的关闭请求。程序更新在渲染时生效，丢弃草稿时保留已提交值。上下方向键与 `Home`/`End` 浏览选项，回车提交，`Tab` 移出焦点时关闭弹出层，Escape 在输入法组合结束后关闭。从菜单关闭时，焦点返回仍可用的触发锚点。已获取焦点的外部控件保留焦点。

重新创建或调整选项顺序时保留各自的 ID。使用 `ExposedDropdown::with_shared_options(id, label, &state, options)` 传入保留的 `Rc<[DropdownOption]>`（或任意 `Into<Rc<[DropdownOption]>>`），可在多次渲染间共享选项切片，避免逐帧收集与线性切片比对。`.option_content(|option, highlighted, window, cx| ...)` 提供装饰性行内容，原始标签用于无障碍名称和筛选。可通过 `scroll_state()` 读取或移动保留的 GPUI `ListState`，或用 `set_scroll_state(&scroll, cx)` 接入调用方持有的状态。列表仅构造可见行和预加载区域，包括自定义内容。菜单默认跟随字段宽度。用 `.match_anchor_width(false)` 选用独立宽度，默认 240 px，`.popup_width(width)` 可覆盖该值。菜单宽高受窗口可用区域限制。`.appearance(MenuSurfaceStyle)` 配置菜单形状、主题角色、描边和高程。`.container_color(color)` 可覆盖容器颜色。

### 菜单开关和菜单项内容

`Menu::new` 默认自行管理显示状态，`Menu::new_with(id, trigger, |window, cx| ...)` 仅在菜单展开时延迟构造菜单项。任意 `IntoElement` 也可通过 `MaterialElementExt::context_menu(id, items)` 或延迟形式 `MaterialElementExt::context_menu_with(id, |window, cx| ...)` 以后缀方式附加右键上下文菜单。调用 `.context()` 后，右键点击会在指针坐标处弹出并自动限制在视口内（除非显式调用 `.position(...)`），在触发区域内再次右键会平滑更新位置，同时不劫持触发控件自身的 `Enter`/`Space` 主操作，并支持通过 `Shift+F10` 或 `ContextMenu` 键锚定触发控件打开。使用 `.disabled(true)` 可仅渲染触发控件而不启用菜单。使用 `.open(open)` 和 `.on_open_change(|requested, window, cx| ...)` 时，由调用者持有状态。调用者可接受请求、更新状态并重新渲染。调用者主动更新 `open` 会同步显示和焦点，不会触发变更回调。`.on_dismiss_request(|window, cx| ...)` 报告用户操作发出的关闭请求。`Menu::composed` 向自定义触发控件提供当前显示状态及激活回调，可连接到 `Button::on_accessible_activate` 或 `IconButton::on_accessible_activate` 以提供 `AccessibleAction::Click` 而不与菜单的指针处理冲突。

为每个 `MenuItem` 提供稳定的 ID 和无障碍标签。`.content`、`.leading`、`.supporting`、`.trailing` 和 `.selection_icon` 接收可重复创建视觉元素的闭包。可通过 `.tone(SemanticTone)` 或 `.destructive()`（`SemanticTone::Danger` 的快捷方法）以语义角色为标签和前置图标着色，同时保留禁用样式和显式 `MenuItemStyle` 的优先级。通过 `.shortcut(text)` 可在尾部插槽以 `LabelSmall` 与 `OnSurfaceVariant`（禁用时使用禁用色）渲染桌面快捷键提示，显式 `.trailing(...)` 优先。使用 `MenuItem::divider(id)` 可在平铺菜单中插入不参与交互和键盘导航的水平分隔线行。键盘和指针操作由菜单行处理。用 `.checked(value)` 提供勾选状态，通过 `.on_checked_change` 处理请求。选中项继续用 `.selected(value)` 和 `.on_activate`。

使用 `.color(MenuColor::Vibrant)` 启用第三色菜单配色。`MenuItem::section(id, name, rows)` 绘制分组表面，方向键仍按菜单顺序遍历其中的项目。`Menu::content` 在菜单项前添加调用方提供的内容。分组的 `.content` 可以替换可见标题。

使用 `MenuSurfaceStyle` 配合 `Menu::appearance` 或 `MenuItem::group_appearance` 设置表面形状、逻辑 `LogicalEdges` 内边距、间距、边框和高度。`MenuItemStyle` 可覆盖菜单行的形状、间距、逻辑 `LogicalEdges` 内边距和颜色角色，行高随 `theme.layout.density` 适配。使用 `.anchor` 和 `.offset` 相对触发控件定位，或用 `.position` 指定窗口坐标。`.track_scroll` 接收调用方持有的滚动句柄。

### 提示

`Tooltip::plain`（或后缀形式的 `MaterialElementExt::tooltip(id, text)`）和 `Tooltip::rich` 在指针悬停后显示（默认 500 ms 延迟显示、200 ms 延迟隐藏，可通过 `.show_delay(Duration)` 与 `.hide_delay(Duration)` 覆盖），也可通过 `.track_focus(&focus)` 在触发控件获得键盘焦点时显示。对于持久富内容提示，用 `cx.new(|_| TooltipState::new())` 创建 `Entity<TooltipState>`，再调用 `.persistent(&state)`。点击或用键盘激活触发控件可以显示提示，调用方也可执行 `state.show(cx)`。触发按钮负责自己的无障碍 Invoke 操作，在其 `on_activate` 回调中执行 `state.update(cx, |state, cx| state.show(cx))` 即可让该操作显示提示。指针离开后，持久提示仍保持显示。调用 `state.dismiss(cx)` 可主动关闭，`is_visible()` 返回已请求的显示状态。

`.enable_user_input(false)` 将激活操作交给调用方。焦点位于触发控件或提示内容时，可以按 Escape 关闭。点击持久提示外部也会关闭它。使用 `.on_dismiss_request(callback)` 时，持久提示的调用方通过 `dismiss(cx)` 接受关闭请求。临时提示在按 Escape 时关闭，并通过同一回调报告请求。`.focus_target(&focus)` 允许用 Tab 从触发控件进入调用方提供的控件。`.focusable(true)` 会在打开时将焦点移入提示，默认保留焦点。关闭时若焦点位于提示内，则返回触发控件或原来拥有焦点的控件。

用 `.title_content`、`.body_content` 和 `.action_content` 提供 GPUI 元素。使用 `.shortcut(text)` 可在普通提示文本旁或富内容提示的标题/正文处附加柔和格式的桌面快捷键提示，同时保持构造器文本作为无障碍说明。`.link(label, callback)` 创建链接，由回调负责导航。`.placement(TooltipPlacement)` 选择物理方向或逻辑起始侧 `Start`、末尾侧 `End`，`.offset(pixels)` 设置间距。默认首选 `Above`、间隔 4 px，空间不足时会翻转。`.positioner(...)` 可以根据测得的触发控件边界、视口和文字方向返回 `TooltipPositioner`。

使用 `.caret(TooltipCaret::default())` 显示 Material 默认的 16 × 8 px 指示尖角，也可以自定义尺寸和路径。默认不显示尖角。通过 `.max_width`、`.shape`、`.colors`、`.tonal_elevation` 和 `.shadow_elevation` 配置表面。普通提示和富内容提示默认最大宽度分别为 200 px 和 320 px，并受窗口可用空间限制。未指定的颜色和形状采用窗口的 Material 配色与圆角角色。默认色调层级为零，阴影层级为普通提示 Level 0、富内容提示 Level 2。

### 标签

使用 `Label::new(id, anchor, description)` 为控件或其他 GPUI 元素附加浮层标签。`description` 提供无障碍文本，`.content(element)` 可为标签表面提供自定义装饰内容。锚点保留自身角色、焦点和激活行为，标签表面下方的控件仍可接收指针输入。

通过 `.interaction_source(&source, control_id)` 指定控件使用的交互源和对应的 ID。非持续标签在控件悬停、按下、拖动或获得键盘可见焦点时显示。控件禁用或移除时，交互触发的标签会隐藏。未指定交互源时标签保持隐藏。传入 `.persistent(true)` 可在锚点可见时持续显示标签，后续渲染更改该值即可恢复由交互驱动的显示行为。Escape 和外部输入仍由周围控件处理。

标签使用普通 Tooltip 的表面和定位方式，默认优先在锚点上方，间隔 4 px，并根据窗口可用空间调整。通过 `.placement(TooltipPlacement)`、`.offset(pixels)`、`.max_width(pixels)` 或 `.caret(TooltipCaret)` 配置定位和表面。可选的指示尖角会跟随最终位置。

### 遮罩

在已定位的容器中，将 `Scrim::new(id)` 放在背景内容之后、前景表面之前。遮罩填满容器，默认使用当前窗口主题的 Scrim 色彩角色和 32% 不透明度。使用 `.color(color)` 指定自定义颜色，用 `.opacity(value)` 乘以该颜色的 alpha 值。数值限制在 0–1，NaN 使用默认值。

默认遮罩仅作装饰，降低内容亮度，并保留底下内容的指针操作和无障碍遍历。使用 `.on_click(|window, cx| ...)` 后，遮罩拦截指向底下内容的指针事件，并响应指针或辅助技术的激活操作。遮罩不占用键盘 Tab 焦点。使用 `.accessibility_label(label)` 为辅助技术提供操作名称，默认名称为 “Dismiss overlay”。显示状态、关闭操作，以及模态内容的焦点与无障碍范围均由调用方管理。模态内容需要提供自己的键盘关闭操作。

### 对话框

每个对话框创建一个 `Entity<DialogState>`。调用 `open(window, cx)` 与 `close(window, cx)` 控制窗口内对话框。用 `Dialog::new(id, &state, title)` 创建采用 Material 默认布局的对话框，或用 `Dialog::alert(id, &state, title, description)` 快速创建带说明文本且隐藏默认取消按钮的基础提示对话框。`DialogVariant::FullScreen` 在内容宽度小于 600 逻辑像素时填满窗口内容区，在较宽窗口中采用基本布局。Tab 和 Shift+Tab 在模态范围内切换焦点。在同一窗口中嵌套时，为内层对话框设置更高的 `.layer(index)`，并从当前对话框内打开它。如果关闭时焦点仍在对话框内，则恢复此前的焦点。

用 `.icon(element)`、`.title_content(element)`、`.description(text)` 和 `.body(element)` 构造图标、标题、辅助文字和正文。构造器的标题用于无障碍名称。用 `.show_title(false)` 隐藏可见标题。`.confirm(label, validate)` 提供默认确认操作，也能在焦点位于对话框容器时用 Enter 触发；校验返回 false 时，对话框保持打开。`.on_confirm(label, action)` 是无需校验的快捷方法，执行 `action` 后总是关闭对话框；`.confirm_tone(SemanticTone)` 为基本和全屏布局中的默认确认按钮应用语义色调；`.confirm_disabled(bool)` 与 `.confirm_loading(bool)` 可禁用默认确认按钮或显示内嵌加载指示，同时抑制 Enter 确认。`DialogState::focus_handle()` 与 `DialogState::scroll_handle()` 暴露保留的容器焦点句柄与正文滚动句柄。`.confirm_content(|confirm| ...)` 与 `.dismiss_content(|dismiss| ...)` 分别构造确认和取消控件。使用 `Button` 等控件，将指针、键盘及辅助技术的激活操作连接到传入的 `DialogAction::activate(window, cx)`。自定义确认控件可复用同一校验函数，并自行处理快捷键。`.show_cancel(false)` 省略默认的取消按钮或紧凑布局中的关闭图标。

用 `Dialog::unstructured(id, &state, accessible_name).body(element)` 自定义对话框结构。外壳提供焦点范围、无障碍名称、受限视口与关闭策略。调用方指定内边距、滚动、标题、操作和表面。`.body_content(|confirm, dismiss| ...)` 将两个激活句柄传给自定义内容。`.width(pixels)` 设置居中对话框的最大宽度，并受可用空间限制。用 `.shape`、`.colors(DialogColors)`、`.tonal_elevation` 和 `.shadow_elevation` 配置表面及插槽样式。用 `.scrim_color` 和 `.scrim_opacity` 自定义窗口内遮罩。透明遮罩仍拦截指向对话框后方的指针输入。

`DialogProperties` 分别控制 Escape 和点击外部是否关闭，默认均允许。`.on_dismiss_request(|reason, window, cx| ...)` 返回 false 可拒绝 Escape、外部点击和系统窗口关闭请求。`.on_closed(|reason, window, cx| ...)` 对一次已接受的用户关闭操作只回调一次。原因包括 `Confirm`、`Cancel`、`Escape`、`OutsideClick` 或 `WindowClose`。程序主动关闭时，可从状态的 `last_close_reason()` 读取 `Programmatic`。回调接收对话框所在窗口。状态切换和焦点恢复独立于回调。

在活动父窗口中调用 `Dialog::open_window(&state, parent, options, cx, |state, window, cx| ...)`，可请求独立的 GPUI 对话框窗口并在其中构造内容。`WindowOptions` 指定窗口边界、标题栏和是否可调整大小。父窗口的模态行为、窗口装饰及原生尺寸调整由系统处理。Windows 会停用当前活动父窗口，macOS 使用窗口附属表单，Linux 的模态行为由合成器决定。Escape 和系统关闭按钮沿用同样的关闭策略。外部点击关闭和遮罩用于窗口内对话框。需要跨平台一致的单窗口模态行为时，使用窗口内对话框。

### 底部面板

使用 `BottomSheetState::new(initial_value, cx)` 创建一个 `Entity<BottomSheetState>`，再用 `Sheet::bottom(id, &state, name, content)` 渲染。`BottomSheetValue` 提供 `Hidden`、`PartiallyExpanded` 和 `Expanded`。构建状态时可用 `.enabled_values(values)` 和 `.confirm_value_change(|target| ...)` 配置停靠点和切换校验。目标不可用或切换被拒绝时，面板停留在当前停靠点。

调用 `show(window, cx)` 打开面板，有可用的部分展开停靠点时优先使用该位置。使用 `partial_expand(window, cx)`、`expand(window, cx)` 和 `hide(window, cx)` 动画切换到指定位置，也可用 `animate_to(value, window, cx)` 传入停靠点。用 `snap_to(value, window, cx)` 直接切换。通过 `available_values()`、`current_value()`、`target_value()`、`is_animating()`、`visible_height()` 和 `offset()` 读取状态。偏移量是距完全展开位置的向下距离，单位为逻辑像素。减少动态效果时会直接停靠。

标准底部面板加入调用方布局。`Scaffold::bottom_sheet(sheet)` 将标准面板放在主内容下方和底部导航上方。面板适应实测的内容列尺寸，其可见高度计入 Scaffold 底部已占空间。其余正文内边距继续使用 `LogicalEdges` 的最小值规则。模态底部面板覆盖窗口，关闭前将焦点限制在内部。用户关闭受 `.properties(DialogProperties)`、`.on_dismiss_request` 和状态切换校验控制。`.on_closed` 对一次已接受的用户关闭报告一次。程序主动隐藏时会恢复打开控件的焦点。

用 `.extent(height)` 设置完全展开的高度，用 `.peek_height(height)` 设置部分展开的可见高度。标准面板默认露出 56 逻辑像素。模态面板默认将部分展开高度限制在视口的一半以内。默认宽度上限为 640 逻辑像素，可用 `.max_width(width)` 调整。通过 `.content_insets(LogicalEdges)` 为可滚动的面板正文添加逻辑内边距。形状、颜色、遮罩和海拔沿用 Sheet 的构建方法。

具名调整手柄支持指针拖动，也可由键盘或辅助技术激活，循环切换可用停靠点。上键和下键在停靠点之间移动，Home 和 End 选择最低或最高的可用位置。用 `.handle_label(label)` 指定无障碍名称，用 `.handle_content(element)` 替换手柄装饰，或用 `.show_handle(false)` 改用文本操作。`.drag_enabled(false)` 保留显式调整操作。`.drag_thresholds(BottomSheetThresholds)` 配置位置阈值和释放速度阈值，默认为 56 逻辑像素和每秒 125 逻辑像素。拖拽中按 Escape、失去指针捕获或窗口调整大小时，面板回到可用的停靠点。正文滚动与手柄拖动相互独立。

### 侧边栏

使用 `Sheet::new(id, &state, name, content)` 创建侧边栏，并用 `.edge(SheetEdge::Start)` 或 `.edge(SheetEdge::End)` 选择逻辑侧边。侧边随窗口方向调整。标准侧边栏加入调用方布局，旁边的内容仍可交互。请把它放在主内容对应的逻辑侧边。模态侧边栏覆盖窗口，并在关闭前将焦点限制在其内部。保留一个 `DialogState`，调用 `open(window, cx)` 和 `close(window, cx)` 控制模态侧边栏的显示。嵌套模态侧边栏时，为内层指定更高的 `.layer(index)`。

用 `.placement(SheetPlacement::Detached)` 使侧边栏与窗口或布局区域的边缘留出间距。分离布局默认使用 16 逻辑像素的边距，可用 `.margins(LogicalEdges)` 覆盖。停靠布局贴合边缘。`.extent(width)` 设置侧边栏的宽度，默认上限为 400 逻辑像素（可通过 `.max_width(width)` 调整），并受可用空间限制。默认宽度为 320 逻辑像素。用 `.edge_divider(true)` 分隔停靠的标准侧边栏与相邻主内容。

用 `.header_content(element)` 替换可见标题，或在底部面板与侧边栏上用 `.show_header(false)` 省略可见标题行；构造器中的名称继续用于无障碍。`.back(label, callback)` 提供具名的返回按钮，`.back_content(element)` 接收调用方自定义的控件。返回操作默认保留侧边栏，由回调决定是否关闭。模态侧边栏带有关闭操作，可用 `.close_label(label)` 设置名称，或用 `.show_close(false)` 隐藏。用 `.close_content(|close| ...)` 构建自定义具名控件，并在激活时调用 `close.activate(window, cx)`。标准侧边栏可通过 `.on_closed` 启用默认关闭按钮，由调用方移除侧边栏。用 `.footer(element)` 在滚动正文下添加固定操作区，用 `.footer_divider(true)` 将其与正文分开。

`.properties(DialogProperties)` 控制模态侧边栏是否响应 Escape 和外部点击。`.on_dismiss_request(|reason, window, cx| ...)` 可拒绝用户发起的关闭，包括关闭按钮。`.on_closed(|reason, window, cx| ...)` 对已接受的模态关闭只回调一次，也可接收标准侧边栏的关闭操作。原因包括 `Cancel`、`Escape` 或 `OutsideClick`。程序主动关闭模态侧边栏后，`DialogState::last_close_reason()` 返回 `Programmatic`。关闭时若焦点仍在模态范围内，则恢复此前的焦点。遮罩即使透明或禁用外部点击关闭，也会拦截背景的指针和滚动输入。

用 `.shape`、`.container_role`、`.content_role` 和 `.colors(SheetColors)` 配置表面、正文、标题、分隔线和遮罩。`.tonal_elevation`、`.shadow_elevation` 和 `.scrim_opacity` 均可独立设置。标准侧边栏使用 Surface 色，模态侧边栏使用 Surface Container Low 色。停靠的模态侧边栏在内侧使用圆角，分离布局在四角使用圆角。自定义插槽控件保留各自的样式和无障碍语义。

<a id="navigation-and-adaptive-layout"></a>

## 导航与自适应布局

| 组件 | 支持的变体与行为 | Material 3 |
| --- | --- | --- |
| `AppBar`、`AppBarRow`、`AppBarColumn`、`BottomAppBar` | Small、Centered、Medium Flexible、Large Flexible 及可自定义的 TwoRows 顶部应用栏（操作可移入溢出菜单），以及 80 dp 底部应用栏（支持操作按钮、内嵌 FAB 插槽与滚动隐藏）。 | [应用栏](https://m3.material.io/components/app-bars/overview) · [底部应用栏](https://m3.material.io/components/bottom-app-bar/overview) |
| `Tabs` | Primary、Secondary，可固定或滚动，支持前置图标、自定义内容与徽标、指示器配置和保留的滚动状态。 | [标签页](https://m3.material.io/components/tabs/overview) |
| `NavigationBar`、`NavigationRail`、`NavigationDrawer` | 短导航栏支持等宽或居中排列、纵向或横向图标、自定义标签和徽标。导航轨道支持折叠、展开、隐藏和模态呈现。独立导航抽屉支持标准与模态模式、分组标题、分隔线和胶囊指示器。 | [导航栏](https://m3.material.io/components/navigation-bar/overview) · [导航轨道](https://m3.material.io/components/navigation-rail/overview) · [导航抽屉](https://m3.material.io/components/navigation-drawer/overview) |
| `Stepper` | 横向与纵向多步骤进度与线性/非线性步骤导航，支持进行中、已完成、待进行、错误和禁用状态。 | [步骤条](https://m3.material.io/components) |
| `Breadcrumbs` | 桌面端路径导航，支持调用方持有的路径段、RTL 镜像分隔符、当前页语义，以及宽度受限时将中间路径自动折叠至锚定 Material `Menu`。 | [导航](https://m3.material.io/components) |
| `AdaptiveNavigation` | 适配导航类型与动画可见性，支持自定义插槽和样式。 | [断点](https://m3.material.io/foundations/layout/breakpoints/overview) |
| `Scaffold` | 组合导航、应用栏、内容、可定位的悬浮操作按钮和消息条，并根据已占空间计算内容内边距。 | [Scaffold](https://m3.material.io/foundations/layout/scaffold/overview) |
| `ListDetail`、`SupportingPane`、`Feed` | 列表、详情、辅助窗格及响应式信息流的典型布局，支持自适应三窗格容器。 | [典型布局](https://m3.material.io/foundations/layout/canonical-examples/overview) |
| `VerticalDragHandle` | 用于横向调整窗格尺寸的纵向把手，可分别设置静止、按下和拖动外观。 | [布局要素](https://m3.material.io/foundations/layout/understanding-layout/parts-of-layout#314a4c32-be52-414c-8da7-31f059f1776d) · [Compose VerticalDragHandle](https://developer.android.com/reference/kotlin/androidx/compose/material3/VerticalDragHandle.composable) |

### 自适应导航

用 `AdaptiveNavigation::new(scaffold, items, width)` 传入可用的逻辑宽度。默认在 600 px 以下使用项目纵向排列的 `ShortBarCompact`，从 600 px 到 839 px 使用项目横向排列的 `ShortBarMedium`，从 840 px 起使用 `RailCollapsed`。导航占用现有 Scaffold 的底部或逻辑起始侧。请保持导航集合和目标 ID 稳定，通过 `NavigationItems::on_select` 接收选择请求，再由调用方用 `.selected(id)` 提交。

用 `.navigation_type(AdaptiveNavigationType)` 选择上述呈现方式，也可选择 `RailExpanded` 或 `None`。用 `.navigation_policy(|info| ...)` 根据可用尺寸、宽度类别和窗口方向选择导航类型。当 Scaffold 的有界尺寸与视口不同时，用 `.available_size(size)` 传入实际尺寸。选择 `None` 时，导航套件保留 Scaffold 已有的插槽。

用 `AdaptiveNavigationVisibility::Shown` 或 `Hidden` 初始化 `Entity<AdaptiveNavigationState>`，并通过 `.state(&state)` 传给导航套件。用 `current_value()`、`target_value()`、`is_animating()` 和 `progress()` 读取当前值、目标值、动画状态及可见比例。调用 `show(cx)`、`hide(cx)` 或 `toggle(cx)` 从当前可见比例开始过渡，调用 `snap_to(value, cx)` 立即到达目标。默认过渡为 240 ms，遵循减少动态效果设置，可用 `.transition_duration(...)` 调整时长。导航从底部或逻辑起始侧滑入，Scaffold 预留当前可见部分的空间。隐藏完成后，插槽从焦点树和无障碍树移除。如果焦点位于隐藏的插槽内，会移到下一个可用控件。重新显示时，焦点留在当前控件；如果调用方的选择未改变，标准导航项目会恢复上一个可用的键盘停靠点。

在 `NavigationItem` 上设置单项样式、图标位置、徽标内容及交互来源。用 `.navigation_bar(|kind, bar| ...)` 或 `.navigation_rail(|kind, rail| ...)` 设置各呈现方式的布局与外观，包括保留的 `NavigationRailState`、可选的模态会话、排列、形状和颜色。用 `.navigation_content(|kind, default| ...)` 替换导航区域，或在标准内容周围加入其他元素。自定义区域自行管理无障碍名称、Tab 停靠点和焦点句柄。切换呈现方式时保持句柄和元素 ID 稳定。如果自定义内容包含模态窗口，由应用协调模态会话和导航的可见状态。

### 自适应多窗格布局

每个布局用 `AdaptivePaneState::new(PaneDestination::new(role))` 创建并保留一个 `Entity<AdaptivePaneState>`。自定义布局使用 `AdaptivePaneLayout::new(id, size, &state)`，典型顺序使用 `ListDetail::adaptive(...)` 和 `SupportingPane::adaptive(...)`。传入可用视口的逻辑尺寸，并为各窗格提供稳定 ID。用 `.pane(PaneRole, AdaptivePane)` 注册窗格，额外窗格使用 `Tertiary`。业务状态、焦点句柄和滚动句柄由调用方保留。现有 `ListDetail`、`SupportingPane` 和 `Feed` 构造器沿用原行为；`Feed` 暴露 `Role::List` 并支持 `.accessible_label(label)`，按 `WidthClass` 断点排布 1、2、3 或 4 列并在列数变更时保持 `ScrollHandle` 滚动锚点。

`PaneLayoutDirective::for_size(size)` 在紧凑和中等宽度下横向显示一个窗格，在展开宽度下最多显示两个，在大型宽度下最多显示三个。单列视口足够高时，还允许第二个纵向分区。用 `.directive(directive)` 指定横纵分区数上限、间距、默认首选宽高及逻辑宽度排除区间。`AdaptivePane` 可覆盖首选宽高、最小宽度和逻辑 `LogicalEdges` 边距。布局约束在可用视口内，RTL 下镜像逻辑排列，并将窗格内容裁剪到自身边界。

超出布局容量的窗格可以隐藏，或在单列布局有剩余纵向空间时 `Reflow` 到可见窗格下方。`Levitate` 使用首选尺寸、可用视口和逻辑 `PaneOverlayAlignment`，将当前窗格悬浮在布局上方。用 `.adapt_with(...)` 按视口和指令选择策略。悬浮窗格接收自身范围内的指针输入。`.levitate_scrim(element)` 添加拦截输入的遮罩，并将 Tab 遍历限制在悬浮窗格内。用 `.entry_focus(&focus)` 指定窗格的首选键盘焦点。

用 `.separator(start_role, end_role, PaneSeparator::new(label))` 为横向相邻窗格添加具名手柄。手柄支持拖动、随 RTL 映射的物理左右键、Home 和 End，以及无障碍范围调整。`.anchors(...)` 接收 `PaneExpansionAnchor::Proportion(fraction)`、`FromStart(px)` 或 `FromEnd(px)`。锚点基于扣除间距后的横向逻辑分区。完成拖动后吸附至最近锚点，拖动中断时保留最后生效的宽度。Page Up 和 Page Down 分别移至逻辑顺序中的下一个和上一个锚点。`.handle(...)` 自定义 Material 手柄，`.keyboard_step(...)` 调整输入步长。调用状态的 `set_expansion` 或 `clear_expansion` 可用程序移动或重置边界。

用 `navigate_to(PaneDestination)` 前往指定窗格，可附带内容键。`current_destination()` 和 `history()` 返回保留的目的地。返回策略可回到最近一项，或跳至布局、角色或内容键有变化的一项。给 `can_navigate_back`、`navigate_back` 和 `seek_back` 传入 `PaneBackBehavior`。调用方绑定返回按钮或按键。`set_history_aware` 可选择已访问的窗格角色是否影响响应式布局。

用 `current_value()`、`target_value()` 和 `progress()` 观察呈现状态。`snap_to`、`animate_to` 和 `seek_to` 接收明确的 `PaneLayoutValue` 布局配置。`follow_navigation` 恢复自动布局并取消返回预览。切换从当前显示的几何位置重新设定目标。默认持续 240 毫秒，可用 `.transition_duration(...)` 调整。减少动态效果时，自动切换立即完成。视口变化时仍保留业务导航历史。

### 应用栏

`AppBar::new(id, title)` 默认创建 Small 应用栏，标题同时是工具栏的无障碍名称。`.title_alignment(AppBarAlignment)` 和 `.subtitle_alignment(AppBarAlignment)` 分别设置逻辑 Start、Center 或 End 对齐。Centered 变体的两项默认值都是 Center。用 `.title_max_lines(lines)` 和 `.subtitle_max_lines(lines)` 设置文本换行上限，默认单行。通过 `.title_content(element)` 或 `.subtitle_content(element)` 提供富内容标题。导航、操作和搜索插槽各自布局，搜索取代首行标题。

`.logo(element)` 可在 Small 或 Centered 中替换文本，在 Flexible 中置于展开标题上方，在搜索布局中置于搜索之前。`AppBarVariant::TwoRows` 将展开标题放在导航与操作的下方。`.title_for_rows(|expanded| ...)` 和 `.subtitle_for_rows(|expanded| ...)` 为当前显示的行构造标题。`.second_row(element)` 为展开标题添加补充内容。标题、标志及补充内容用于呈现信息，命令应放在导航或操作插槽中。

`.colors(AppBarColors)` 分别覆盖容器、滚动后容器、导航、标题、副标题和操作的配色。插槽内容继承对应颜色，独立设色的控件沿用自身颜色。`.content_padding(LogicalEdges)` 设置每行的逻辑内边距。以逻辑像素设置 `.collapsed_height(height)` 和 `.expanded_height(height)`。展开高度包含两行，且至少等于收起高度。首行可增高以容纳插槽。默认收起高度为 64 dp（随 `theme.layout.density` 适配），Flexible 变体采用 Material 的展开尺寸。无效尺寸回退到默认值。

通过 `.track_scroll(&handle)` 接入 GPUI 内容滚动句柄，或通过 `.scroll_state(&state)` 接入保留的 `Entity<AppBarScrollState>`。调用方也可用 `set_content_offset(offset, cx)` 设置内容离顶部的非负距离。接入滚动句柄时由句柄提供该距离。可从状态读取 `content_offset()`、`height_offset()`、`height_offset_limit()`、`collapsed_fraction()` 和 `overlapped_fraction()`。未指定策略时，Flexible 栏按内容离顶部的距离收起。用 `.scroll_behavior(AppBarScrollBehavior::Pinned)` 固定高度并按滚动切换容器色，`EnterAlways` 在滚动反向时立即展开，`ExitUntilCollapsed` 在内容回到顶部之前保持收起。内容区继续持有滚动及其输入。用 `.scroll_enabled(false)` 暂停滚动驱动的更新。用 `set_collapsed_fraction(fraction, cx)` 设置位置，或在滚动结束时调用 `settle(velocity, cx)`。正速度选择收起，负速度在策略允许时选择展开，零选择最近端点。`.scroll_motion(MotionSpec)` 覆盖归位动效，减少动效时即时归位。

`AppBarRow::new(id, items)` 和 `AppBarColumn::new(id, items)` 水平或垂直排列操作，将放不下的项移入 Material 菜单。用 `AppBarItem::clickable(id, label, icon_factory)` 创建行内和溢出共用的命令，并通过 `.on_activate(...)` 和 `.disabled(bool)` 设置行为。通过 `AppBarItem::custom(id, inline, |menu| ...)` 提供自定义行内元素和对应的 `MenuItem`。为各项提供稳定的 ID。`.overflow_indicator(|menu| ...)` 可自定义内置键盘按钮的图形，用 `.overflow_label(label)` 设置按钮名称。`AppBarMenuState` 提供 `is_showing()`、`show(window, cx)` 和 `dismiss(window, cx)`。`.max_item_count(count)` 限制包括触发按钮在内的可见项数。横排在 RTL 中镜像，溢出操作在菜单打开后可访问。

### 标签页

`Tabs::new(NavigationItems::new(id, name, items))` 默认创建固定的 Primary 标签页。用 `.variant(TabVariant::Secondary)` 选择 Secondary 样式，用 `.scrollable(true)` 启用横向滚动。为每个 `NavigationItem` 提供稳定 ID。应用通过 `.on_select(...)` 接收请求并通过 `NavigationItems::selected(id)` 提交选中状态。左右方向键按逻辑顺序移动焦点，Home 和 End 移到首个和末个可用标签，激活时请求选中。键盘导航跳过禁用标签。

通过 `NavigationItem::icon_position(NavigationItemIconPosition::Start)` 将图标放在逻辑起始侧，在 RTL 窗口中镜像。默认 `Top` 将图标放在标签上方。`.label_content(element)` 可替换可见标签，无图标时可提供整项内容。构造器仍提供标签页的无障碍名称。`.badge(Badge::standalone())` 将徽标贴在图标或标签上方的逻辑末尾侧。徽标设有 `.accessible_label(...)` 时，其内容用作该标签项的无障碍描述。插槽仅用于呈现，焦点和激活由标签项处理。

通过 `.colors(TabsColors)` 设置行容器、共用内容色和内置分隔线色。使用 `NavigationItem::colors(NavigationItemColors)` 为单项设置选中、未选中和禁用时的图标和标签色。选中标签可按 Top 与 Start 分别配色，也可设置单项选中指示器色。用 `NavigationItem::interaction_source(&source)` 接入由调用方持有的 `Entity<InteractionSource>`，通过 `ElementId::from((collection_id, destination_id))` 读取状态。标签重新排序或移除时，ID 和交互监听随标签项更新。

用 `.indicator_style(TabIndicatorStyle)` 设置高度、逻辑形状、颜色和动效。`TabIndicatorWidth::Default` 保留预设宽度，`Tab` 覆盖所选标签，`Content` 使用测得的图标与标签宽度，`Fixed(width)` 使用不超过标签宽度的逻辑像素值。可用空间允许时，内容宽度至少为 24 dp。切换选中项时，可配置指示器的位置和宽度会过渡到目标。启用减少动态效果时直接应用目标状态。`.indicator_content(|layout: TabIndicatorLayout| ...)` 替换可见指示器，可读取选中 ID、计算后的边界、内容宽度和颜色。内容会裁剪到指示器边界和行视口内。`.divider(element)` 替换横跨视口的底部分隔线。向任一自定义插槽传入 `gpui::Empty` 可省略对应装饰。

滚动行可通过 `.edge_padding(pixels)` 设置逻辑两端留白，通过 `.min_tab_width(pixels)` 设置最小标签宽度，默认分别为 52 和 90 dp。无效长度沿用默认值，较大的自定义值受窗口宽度约束。用 `.scroll_handle(&handle)` 接入外部滚动句柄，也可由标签行保留自己的句柄。选中项或焦点项变化时，标签行将对应项滚入视口，并处理重新排序和视口或标签尺寸变化。普通重绘保留调用方或用户当前的滚动偏移。

### 短导航栏

`NavigationBar::new(items)` 默认采用 `.layout(NavigationBarLayout::Vertical)` 和 `.arrangement(NavigationBarArrangement::EqualWeight)`，各导航项等宽。`Centered` 将各项集中放在栏的中间，标签较长的项可以增宽，最多占据等宽布局中单项的宽度。布局决定每项的默认图标位置。可用 `.icon_position(NavigationItemIconPosition::Top)` 将图标放在标签上方，或用 `Start` 放在逻辑起始侧。同一栏可以混用两种位置，`Start` 在从右到左的窗口中镜像。

创建 `NavigationItem::new(id, label)` 时提供稳定 ID 和无障碍名称。可用 `.label_content(element)` 替换可见标签，内容用作装饰，无障碍名称仍然保留。用 `.badge(Badge::standalone())` 提供圆点，或为独立徽标设置自定义内容。徽标覆盖图标的上方末尾侧，无图标时贴合标签。徽标的 `.accessible_label(...)` 用作导航项的无障碍描述。导航项保留自己的名称和一个焦点停靠点。

用 `.colors(NavigationBarColors)` 配置导航栏容器色和共用内容色。可在导航项上调用 `.colors(NavigationItemColors)` 分别覆盖选中与未选中时的图标和标签色、禁用时的图标和标签色，以及选中指示器色。选中标签可按 Top 和 Start 位置分别配色。导航项覆盖值优先于导航栏配色，省略的值沿用导航栏的共用配色或窗口主题。图标和标签的禁用色优先于选中色。

使用 `.interaction_source(&source)` 让导航项接入由调用方持有的 `Entity<InteractionSource>`。通过 `ElementId::from((collection_id, destination_id))` 读取该项状态，集合 ID 为 `ElementId`，导航项 ID 为 `SharedString`。导航项移除后释放其状态。整栏只有一个 Tab 停靠点，方向键跳过禁用项，在从右到左的窗口中反转左右键行为。Home 和 End 分别移至首个和末个可用项。使用指针、键盘或辅助技术激活时，`NavigationItems::on_select` 报告目标项，由调用方通过 `.selected(id)` 提交选中状态。

### 侧边导航栏

使用 `NavigationRail::new(id, items)` 和受控的 `NavigationItems`。收起时默认宽 96 个逻辑像素。`.mode(RailMode::Expanded)` 默认宽 240，`.expanded_width(width)` 接收 220–360 个逻辑像素。`RailMode::Hidden` 释放布局空间。宽度受窗口约束，逻辑起始侧和 Start 图标位置随 RTL 镜像。

用初始 `RailMode` 创建并保留一个 `Entity<NavigationRailState>`，通过 `.state(&state)` 接入。可读取 `current_value()`、`target_value()`、`is_animating()` 和 `progress()`。用 `expand(cx)`、`collapse(cx)`、`toggle(cx)`、`hide(cx)` 或 `animate_to(mode, cx)` 设置目标。改变目标时从当前显示宽度接续。`snap_to(mode, cx)` 和减少动态效果模式立即到达目标。可在状态上用 `.transition_duration(...)` 配置默认的 240 毫秒过渡。

用 `.header(element)`、`.headline(text)`、`.fab(element)` 或 `.destination_content(|mode| ...)` 传入自定义内容。自定义导航区域负责名称、选中状态和键盘行为。标准集合可用 `.divider_after(destination_id, element)`、`NavigationItem::label_content` 和 `NavigationItem::badge` 添加内容和具名徽标描述。方向键在启用的导航项间移动焦点。`.icon_position(...)` 覆盖 Top 或 Start 图标位置。`.indicator_padding(collapsed, expanded)` 用逻辑 `LogicalEdges` 调整指示器内边距，保留导航项的交互目标。

`.arrangement(NavigationRailArrangement)` 分配标准导航项的剩余纵向空间。用 `.destination_spacing(...)`、`.content_padding(LogicalEdges)`、`.header_spacing(...)` 和 `.expanded_header_top_padding(...)` 配置逻辑内边距和间距。`.shapes(NavigationRailShapes)` 覆盖收起、展开和模态形状。`.colors(NavigationRailColors)` 配置普通和模态容器及内容色，以及遮罩颜色和不透明度。

用 `.modal_state(&dialog)` 接入保留的 `DialogState`，通过侧边面板会话叠加展开导航栏。关闭后回到停靠模式。`.modal_options(RailModalOptions)` 与 `.modal_properties(DialogProperties)` 配置 Escape、点击外部关闭、层级、放置方式、悬浮边距以及收起时隐藏。`.on_dismiss_request(...)` 可拒绝用户关闭请求。`.on_closed(...)` 对已接受的用户关闭报告一次原因。模态区域限制焦点并阻止背景指针和滚动输入。关闭时若焦点仍由模态区域持有，则恢复入口焦点。

### 底部应用栏

`BottomAppBar::new(id)` 提供 80 dp 高的 Material 3 底部应用栏（`SurfaceContainer`、`Elevation::Level2`，随 `theme.layout.density` 适配），可直接放入 `Scaffold::bottom_bar(...)` 或独立布局中使用。通过 `.action(element)` 或 `.actions(iter)` 添加最多四个逻辑起始侧操作，通过 `.fab(element)` 在逻辑末尾侧放置内嵌悬浮操作按钮。可使用 `.height(pixels)`、`.padding(LogicalEdges)`、`.elevation(Elevation)`、`.colors(BottomAppBarColors)` 和 `.accessible_label(label)`（地标角色 `Role::Toolbar`）自定义尺寸和样式。接入 `.scroll_behavior(AppBarScrollBehavior, &state)`（及可选的 `.track_scroll(&handle)`）可复用 `AppBarScrollState` 实现滚动隐藏（`EnterAlways`、`ExitUntilCollapsed` 或 `Pinned`）。布局与逻辑内边距随 RTL 镜像。

### 导航抽屉

`NavigationDrawer::new(id, items)` 渲染独立的 Material 3 导航抽屉，默认宽 360 dp（可通过 `.width(pixels)` 配置）。使用 `.mode(NavigationDrawerMode::Standard)` 渲染停靠式 `SurfaceContainerLow` 面板，或使用 `.mode(NavigationDrawerMode::Modal)` 配合 `.modal_state(&dialog_state)` 通过模态 `Sheet`（`SheetEdge::Start`）展示，完整复用遮罩、焦点锁定、Escape/外部点击关闭、`.modal_properties`、`.on_dismiss_request`、`.on_closed` 和焦点恢复。可通过 `NavigationItems`、`.section(header, items)`、`.entries(...)`、`.divider_before(item_id)`、`.headline(text)`、`.header(element)` 和 `.footer(element)` 组织内容。导航项在纵向滚动列表中以随密度适配的 56 dp 胶囊指示器与尾部徽标呈现，支持单一漫游 Tab 停靠点（上下键、`Home`/`End`、`Enter`/`Space`）与 RTL 镜像。

### 步骤条

`Stepper::new(id, steps)` 根据 `StepItem` 列表以横向（`StepperOrientation::Horizontal`，默认）或纵向（`StepperOrientation::Vertical`）展示多步骤流程。通过 `.active_step(index)` 可快捷将此前步骤标为 `Completed`、当前步骤标为 `Active`、后续步骤标为 `Upcoming`，单个步骤上显式设置的 `StepItem::status(StepStatus)` 优先。每个步骤支持 `.supporting(text)`、`.icon(element)`、`.editable(bool)`，以及纵向模式下在当前步骤下方展开的 `.content(element)`。提供 `.on_select(|index, window, cx| ...)` 启用步骤切换；线性模式允许激活已完成、进行中、错误或可编辑步骤，`.non_linear(true)` 允许激活任意非禁用步骤。步骤暴露包含标题与状态的无障碍名称，遵循 RTL 镜像，并支持漫游键盘导航（横向左右键、纵向上下键、`Home`/`End`、`Enter`/`Space`）。

### 面包屑导航

`Breadcrumbs::new(id, items)` 根据调用方持有的 `BreadcrumbItem`（支持 `.icon(|| element)`、`.disabled(bool)`）渲染桌面端路径导航（`navigation` 特性，地标 `Role::Navigation`，默认 `.accessible_label("Breadcrumb")`）。上级路径段为键盘可访问按钮，激活后通过 `.on_select(|id, window, cx| ...)` 报告对应项 ID；末尾当前段默认不可导航并暴露当前页语义。分隔符默认使用 `Icon::new(IconName::ChevronRight).size(px(16.)).mirror_in_rtl(true)`，可通过 `.separator(|| ...)` 自定义，通过 `.colors(BreadcrumbsColors)` 覆写上级、当前项、分隔符和禁用颜色。当路径段超出 `.max_items(count)` 或可用宽度受限时，中间路径段自动折叠到由省略号（`…`）按钮触发的锚定 Material `Menu` 中，同时保留首段与末尾/当前段；从菜单中选择折叠项同样触发 `.on_select`。路径段顺序与分隔符均随 RTL 镜像。

### 纵向拖动把手

将 `VerticalDragHandle::new(id)` 放在两个窗格之间。用 `.sizes(DragHandleSizes)`、`.colors(DragHandleColors)` 和 `.shapes(DragHandleShapes)` 分别覆盖静止、按下和拖动状态的值。默认静止标记为 4 × 48 的胶囊，使用当前窗口主题的 Outline 角色。按下和拖动标记为 12 × 52，使用 On Surface 角色与 Medium 圆角。视觉尺寸以逻辑像素计。交互目标始终至少为 48 × 48，并能容纳所有状态中尺寸最大的标记。悬停和键盘焦点提供各自的反馈，且保留独立的交互状态。

使用 `.on_resize(label, width, min..=max, callback)` 设置分隔控件的名称、逻辑起始窗格的当前宽度、最小及最大宽度和宽度请求回调。调用方决定是否采用请求的宽度，并负责布局。分隔控件获得焦点后，左右方向键分别向左或向右移动它，Home 和 End 分别请求最小和最大宽度。用 `.keyboard_step(px(...))` 修改默认的 8 像素步长。当前无障碍桥接将原生 Splitter 角色视为只读，因此该纵向分隔控件提供带名称的可写范围语义，以 “splitter” 描述角色，数值为起始窗格的宽度。窗口主题根据 LTR 或 RTL 将鼠标及触摸的实际移动方向映射到逻辑宽度。按 Escape、失去焦点或指针操作被取消时，拖动结束，调用方保留最近一次接受的宽度。

传入 `.interaction_source(&source)` 可通过共享的 `Entity<InteractionSource>` 观察悬停、按下、拖动和可见键盘焦点。观察多个把手时，请为它们提供不同的标识。运行 `cargo gallery vertical-drag-handle` 可体验分割窗格示例。

### Scaffold 布局

`Scaffold::new(id, content)` 测量并预留独立的 `.app_bar`、`.rail` 和 `.bottom_bar` 插槽所占的空间。应用持有这些插槽、FAB 和消息条宿主。请使用稳定的 ID，并为 Scaffold 提供有界的尺寸。用 `.content_insets(LogicalEdges)` 设置可用内容距各边的最小距离，单位为逻辑单位。各边已占用的空间计入最小距离。普通构造器会在内容区内应用剩余内边距。

用 `Scaffold::with_layout(id, |layout, window, cx| ...)` 在构造内容前读取当前布局。`ScaffoldContentLayout` 提供内容区边界 `bounds`、已占空间 `occupied`、剩余内边距 `padding` 和文本方向 `direction`。只把 `padding` 应用到内容或滚动区的子元素，因为 Scaffold 已预留 `occupied`。用 `LogicalEdges::resolve(direction)`（或 `foundation::resolve_edges`）将逻辑起始侧和末尾侧转换成实际左右边。内容构造闭包在布局时执行。插槽或窗口尺寸变化后，它会收到当前测量结果。

用 `.fab_position(ScaffoldFabPosition)` 将 FAB 在可用内容宽度内设为 `Start`、`Center` 或 `End`，默认是 `End`。这三种位置让 FAB 位于底栏上方，有消息条插槽时也位于消息条上方。`EndOverlay` 让 FAB 贴近 Scaffold 底部并覆盖底栏，仍遵守请求的底部内边距；消息条会避让底栏和 FAB。起始侧与末尾侧随窗口方向镜像。FAB 和消息条作为浮层呈现，不计入已占内容空间。

<a id="optional-components"></a>

## 可选组件

核心组件默认通过 `full` 全部启用。在 `gpui-material`（或 `gpui-material-components`）上关闭默认特性后，仅编译轻量核心（`Text`、`Icon`、`IconName`、`Avatar`、`AvatarGroup`、`EmptyView`、`Skeleton`、`Badge`、`Divider`、`DisplayScrollbar`、`ControlSize`、`LogicalEdges`、`SemanticTone`、`MaterialElementExt` 及 `init_components`）和所选分组：

| 核心特性 | 组件 | 隐含分组 |
| --- | --- | --- |
| `actions` | `Button`、`ToggleButton`、`IconButton`、`IconToggleButton`、`Fab`、`ExtendedFab` 及控件样式 | — |
| `selection` | `Checkbox`、`Radio`、`RadioGroup`、`Switch`、`Chip`、`ChipGroup`、`Slider` | — |
| `inputs` | `TextField`、`TextFieldState`、`TextAreaState`、`ExposedDropdown`、`ExposedDropdownState` | `actions` |
| `surfaces` | `Card`、`DropTarget`、`List`、`ListItem`、`ListSection`、`SwipeToDismiss`、`VerticalDragHandle` | `actions` |
| `settings-tiles` | `SwitchTile`、`CheckboxTile`、`RadioTile`、`SliderTile`、`ChoiceTile`、`ChoiceOption`、`ListSection` | `surfaces`、`selection`、`feedback` |
| `feedback` | `Banner`、`CircularProgress`、`LinearProgress`、`LoadingIndicator`、`PullToRefresh`、`Dialog`、`Sheet`、`BottomSheet*`、`Menu`、`Snackbar`、`SnackbarHost`、`Tooltip`、`Label` | `actions` |
| `navigation` | `AppBar`、`AppBarRow`、`AppBarColumn`、`BottomAppBar`、`Breadcrumbs`、`Tabs`、`NavigationBar`、`NavigationRail`、`NavigationDrawer`、`Stepper`、`AdaptiveNavigation`、`Scaffold`、`ListDetail`、`SupportingPane`、`Feed`、`AdaptivePane` | `actions`、`feedback` |
| `full` | 启用上述全部核心组件分组（默认） | 全部核心分组 |

在 `gpui-material` 依赖上启用可选组件族 feature，各组件族仅激活其依赖的核心组件分组。日期选择器、时间选择器和滚动选择字段从 `gpui_material::date_time` 导入。

| Feature | 组件 | 支持的变体与行为 | Material 3 |
| --- | --- | --- | --- |
| `expressive-groups` | `ButtonGroup`、`SplitButton` | Standard 和 Connected 按钮组，以及内容、外观和激活行为各自独立的拆分按钮。 | [按钮组](https://m3.material.io/components/button-groups/overview) · [拆分按钮](https://m3.material.io/components/split-button/overview) |
| `toolbars` | `FabMenu`、`Toolbar` | 悬浮操作菜单支持受控展开、触发器和操作项插槽及水平对齐。悬浮和停靠工具栏支持任意控件、外观和滚动行为配置。 | [悬浮操作菜单](https://m3.material.io/components/fab-menu/overview) · [工具栏](https://m3.material.io/components/toolbars/overview) |
| `search` | `SearchBar`、`SearchView` | 收起的搜索栏及可自定义的 Docked、Full-screen 和 Contained 搜索视图。 | [搜索](https://m3.material.io/components/search/overview) |
| `date-time` | `DatePicker`、`TimePicker`、`TimePickerDialog`、`VibrantTimePickerDialog`、`TimePickerPanel`、`TimeScroll`、`ScrollField` | 单日期和日期范围选择，支持 Docked、Modal、Modal Input 和自适应 FullScreen。可组合独立面板或对话框。时间选择支持表盘、文本输入、循环滚动和 12、24 小时制。支持独立的滚动选择字段。 | [日期选择](https://m3.material.io/components/date-pickers/overview) · [时间选择](https://m3.material.io/components/time-pickers/overview) |
| `carousel` | `Carousel`、`CarouselParallax` | Multi-browse、Uncontained、Uncontained Multi-aspect Ratio、Hero、Centered Hero、Full-screen。 任意 GPUI 内容可使用独立滚动视差效果。 | [轮播](https://m3.material.io/components/carousel/overview) |

<a id="window-chrome"></a>

### 跨平台窗口标题栏

启用可选特性 `window-chrome` 后，可在 Windows、macOS 和 Linux 上从 `gpui_material::window_chrome` 导入 `TitleBar`、`CaptionButtons`、`CaptionButtonLabels`、`ChromePlatform`、`TitleBarColors`、`CaptionButtonColors`、`title_bar_window_options` 和 `title_bar_window_options_with_height`。

把 `title_bar_window_options(options)`（或 `title_bar_window_options_with_height(options, height)`）的结果传给 `MaterialRoot::open_window`，让 GPUI 在标题栏区域内绘制。该函数保留传入的窗口标题、边界、最小尺寸和能力选项：在 Windows 和 macOS 上启用 `appears_transparent`；在 macOS 上当未指定 `traffic_light_position` 时按标题栏高度自动垂直居中计算交通灯位置；在 Linux 上当未指定 `window_decorations` 时请求 `WindowDecorations::Client`。

在窗口视图顶部渲染 `TitleBar::new(id)`。默认高度为 48 逻辑像素，沿用当前窗口 Material 主题排版（`TypeRole::TitleSmall` 及已解析的主题字体族），自动读取实时窗口标题，并提供拖动区与适配各平台的窗口装饰：

- **Windows**：绘制跨平台矢量最小化、最大化/还原和关闭按钮，对接 GPUI 原生 `WindowControlArea` 命中测试（含 Windows 贴靠布局）与 Win32 关闭/还原路由，完整尊重 `Window::on_window_should_close`。
- **macOS**：非全屏时在起始侧预留可配置的交通灯留白（默认 78 逻辑像素，全屏时自动收起），默认隐藏自定义控制按钮，并将拖动与标题栏双击交由 GPUI 原生窗口钩子处理。
- **Linux**：尊重 `Window::window_decorations()`（`Decorations::Client` 与 `Decorations::Server`）、`Window::window_controls()` 及 `Window::client_inset()`；在客户端装饰模式下渲染支持指针与键盘激活的矢量控制按钮，通过 `Window::start_window_move()` 启动拖动，并在支持时右键呼出窗口菜单。

可通过 `.height(px(...))`、`.title(text)`、`.title_role(TypeRole)`、`.title_font(Font)`、`.title_font_family(name)`、`.title_font_size(px(...))`、`.leading(element)`、`.content(element)`、`.trailing(element)`、`.traffic_light_padding(px(...))`、`.colors(TitleBarColors)`、`.caption_colors(CaptionButtonColors)`、`.caption_labels(CaptionButtonLabels)`、`.caption_button_width(px(...))`、`.show_caption_buttons(bool)`、`.on_close(handler)` 和 `.platform(ChromePlatform)` 自定义 `TitleBar`。也可直接使用 `CaptionButtons::new(id)` 配合 `.labels(CaptionButtonLabels)` 在自定义布局中放置本地化矢量窗口控制按钮。所有控制按钮均提供无障碍名称、键盘激活与 `AccessibleAction::Click`。 [Windows 标题栏指南](https://learn.microsoft.com/en-us/windows/apps/design/basics/titlebar-design)

<a id="search"></a>

### 搜索

启用 `search` feature，并为每个搜索入口保留一个 `Entity<SearchState>`。可将 `SearchBar::new(id, label, &state)` 直接放在布局中，或放入 `AppBar::search(...)`。默认使用 Docked。全窗口编辑器使用 `SearchViewVariant::FullScreen`，带内嵌编辑器的全窗口视图使用 `FullScreenContained`。`.widths(collapsed, focused)` 以逻辑像素设置触发器和聚焦编辑器宽度。未覆盖时触发器根据内容确定宽度。`.trigger_content(element)` 替换触发器的可见标签。`.leading(element)`、`.trailing(element)` 可在触发器两侧添加独立控件或头像。可通过 `SearchBar::track_focus(&focus)` 绑定调用方持有的触发器句柄，或调用 `SearchState::trigger_focus()` 读取保留的默认触发器焦点句柄。

`.expanded_view(|view| ...)` 配置也可单独组合的 `SearchView`。`.input(|field| ...)` 提供保留编辑器的 `TextField`，可配置前后操作、自定义提示、前后缀、配色和交互源。返回的元素应包含该字段，以保留编辑、焦点、输入法和搜索提供方。用 `SearchState::with_editor(...)` 接入调用方持有的 `TextFieldState`，也可用 `editor()` 获取默认实体。

`.docked_size(width, height)` 默认使用 440 × 424 逻辑像素，尺寸受视口约束。`.result_gap(px(...))` 设置 Docked 编辑器和结果表面之间的间距。`.popup_options(SearchPopupOptions)` 配置停靠弹出层的逻辑锚点、触发器下方的间距、关闭行为和可选遮罩。`.window_options(SearchWindowOptions)` 为全窗口模式配置逻辑内边距、关闭行为和遮罩。这些选项配置编辑器所在 GPUI 窗口的弹出层或覆盖层。`.style(SearchStyle)` 分别覆盖触发器、容器、内容、分隔线、输入、形状和高程。省略的选项沿用窗口默认值。

提供方的请求包含版本。通过 `complete(&request, outcome, cx)` 返回结果，由状态筛选有效请求。搜索分别发出提交或选择事件，应用管理导航。用 `SearchResult` 的 `.group(label)` 和 `.icon(IconName)` 为连续分组添加标题和前置图标。`.result_row(|row, result, window, cx| ...)` 自定义可见的 Material 行。生成的列表保持虚拟化，并按稳定 ID 在完整模型中导航（支持 `Up`/`Down`/`Home`/`End`，以及在搜索输入框中按 `Down` 直接聚焦首个可见结果）。用 `scroll_state()` 访问当前视口，或用 `set_scroll_state(&scroll, cx)` 接入保留的 GPUI `ListState`。`.filters(element)` 在提供方结果上方添加内容。用 `.result_content(|state, window, cx| ...)` 自行渲染结果区域，用 `results()` 读取已接受的结果，用 `select_result(...)` 选择自定义结果。

`open`、`close` 立即到达目标。用 `animate_to(SearchValue, duration, window, cx)` 或 `snap_to(fraction, window, cx)` 控制过渡。`progress()`、`current_value()`、`target_value()`、`is_animating()` 返回呈现和请求状态。用 `.motion(SearchMotion)` 让触发和关闭采用动画。减少动态效果时直接应用目标。关闭后恢复入口焦点。Escape 优先处理输入法组合输入。

放入应用栏的搜索可沿用应用栏的滚动状态和行为。独立搜索栏通过 `.scroll_behavior(&state)` 使用保留的 `SearchScrollState`。可用 `set_content_offset` 提供内容偏移，或用 `.track_scroll(&handle)` 关联视口。向前滚动时收起的搜索栏随之隐藏，反向滚动即显示。活动搜索不受内容滚动影响。

### 按钮组

`ButtonGroup::new(id, name, items)` 使用稳定的成员标识，并提供一个 Tab 焦点。左右方向键移动焦点，保留选中状态。导航会跳过禁用成员，视觉顺序和方向键随窗口书写方向调整。使用 `ButtonGroup::connected(id, name, items)` 或 `.variant(ButtonGroupVariant::Connected)` 连接相邻成员。Standard 和 Connected 默认间距分别为 12 和 2 个逻辑像素。可通过 `.spacing(width)` 覆盖间距，使用 `.alignment(ButtonGroupAlignment)` 设置逻辑横向排列，使用 `.vertical_alignment(ButtonGroupVerticalAlignment)` 设置纵向对齐，默认 Top。成员可以单独设置纵向对齐。可在 `ButtonGroup` 或单个 `ButtonGroupItem` 上设置 `.tone(SemanticTone)`，同步应用到可见按钮与溢出 `MenuItem`。

`ButtonGroupItem::new(id, name)` 在启用组选择时使用 `.selection(mode, selected_ids)` 和 `.on_change(...)`。针对常见的单选场景，`.single_select(selected, |id, window, cx| ...)` 配置 `ButtonGroupSelection::SingleRequired` 并直接传入新选中的 `SharedString`，`.single_optional_select(selected, |id, window, cx| ...)` 则配置可取消选中的 `ButtonGroupSelection::Single` 并传入 `Option<SharedString>`。Single 和 Multiple 允许取消选中，Required 模式会保留最后一个选中项。应用处理请求并提供当前选择。`.action(callback)` 在任一组内建立独立命令，`.toggle(selected, callback)` 建立单独受控的切换控件。`.on_activate(...)` 为 Actions 模式下的成员提供命令。

使用 `.leading_icon(|| ...)` 在标签旁加入图标，用 `.icon_only(|| ...)` 以图标替代标签，或通过 `.content(width, || ...)` 提供带首选逻辑宽度的装饰内容。构造器名称用于无障碍和默认溢出菜单行。`.icon_width(IconButtonWidth)` 选择纯图标尺寸。`.appearance(ButtonVariant)` 和 `.style(ButtonGroupItemStyle)` 分别覆盖成员的外观预设，以及静止、按下、选中形状、配色、描边、阴影和逻辑内边距。通过 `.shape(ButtonShape::Square)` 可将默认 Round 外侧形状改为方形。`.connected_shapes(ButtonGroupConnectedShapes)` 为首段、中间段、尾段和单一段设置各状态形状。成员设置的形状字段优先。

`.weight(weight)` 按比例分配非加权成员与最小目标宽度剩余的空间。默认权重为零。`.expanded_ratio(ratio)` 默认为 `0.15`，仅作用于 `Standard` 按钮组：按下的成员会向左右可见邻项借用宽度，而 `Connected` 按钮组保持各段宽度不变。`.compression_limit(px(...))` 限制成员可让出的宽度，默认保留至少 24 个逻辑像素。带内置标签的按钮被压缩窄于首选宽度时，会优先收缩左右水平内边距（每侧至少保留 4 个逻辑像素，或调用方设定的更小值），空间仍不足时再单行省略。`.interaction_source(&source)` 使用 `ElementId::from((group_id, item_id))` 观察可见成员，宽度动画读取同一按压状态。减少动态效果时立即应用目标宽度。

无法显示的成员进入溢出菜单，保留各自的名称、可用与选中状态以及回调。`.overflow_trigger(width, |open| ...)` 替换触发器的显示内容，`.overflow_label(...)` 保留其无障碍名称。`.overflow_content(|| ...)` 在自动生成的菜单行之前添加装饰内容，`.overflow_item_content(|| ...)` 自定义对应成员的菜单行。使用 `.overflow_open(open)` 时由应用持有显示状态，并应用 `.on_overflow_open_change(...)` 的请求。省略打开值时，由按钮组管理菜单显示。

### 拆分按钮

`SplitButton::new(id, leading_name, trailing_name, menu_items)` 提供两段各自的无障碍名称。前段命令使用 `.on_activate(...)`。通过 `.leading_icon(element)` 在标签旁加入图标，`.leading_icon_only(element)` 用图标替代标签，`.leading_content(element)` 提供自定义内容。尾段默认显示菜单箭头。使用 `.trailing_icon(element)` 可按当前尺寸设置图标，或用 `.trailing_content(element)` 提供自定义内容。每段以最后一次内容设置为准。各插槽仅用于呈现，由对应段负责焦点、名称和激活。

尾段默认打开指定的菜单，菜单关闭时焦点返回触发按钮。通过 `.on_trailing_activate(callback)` 设置独立命令，或通过 `.trailing_toggle(selected, callback)` 设置受控切换。尾段命令与切换按最后一次设置生效，并取代构造器提供的菜单。激活切换时请求下一状态，选中值仍由调用方持有。`.disabled(true)` 禁用前段命令，`.trailing_disabled(true)` 单独禁用尾段。菜单为空时，默认菜单触发按钮禁用。两段在 RTL 中镜像，键盘先访问前段再访问尾段。

Filled、Filled Tonal、Elevated 和 Outlined 预设为各 Material 尺寸提供基础外观。`.tone(SemanticTone)`、`.leading_tone(SemanticTone)` 与 `.trailing_tone(SemanticTone)` 为整组或各分段独立应用语义色板。`.spacing(width)` 设置段间距，默认 2 dp。用 `.leading_style(SplitButtonSegmentStyle)` 和 `.trailing_style(SplitButtonSegmentStyle)` 单独配置每段。可通过 `ControlShapes` 设置静止、按下、选中时的逻辑圆角，通过 `ButtonColors` 设置各状态配色，用 `ControlBorder` 设置描边、`ControlElevations` 设置阴影及 `LogicalEdges` 设置逻辑内边距。省略的字段沿用该段默认值。已选中的尾段切换和已打开的菜单使用选中外观，菜单触发器保留按钮语义。选中外观默认增大尾段内侧圆角。未设置选中颜色时沿用启用配色，禁用配色优先。减少动效时立即应用目标形状。

使用 `.leading_interaction_source(&source)` 和 `.trailing_interaction_source(&source)` 传入调用方持有的交互源，可分别用 `ElementId::from((id, "primary"))` 和 `ElementId::from((id, "secondary"))` 读取快照。同一个实体可以观察两个标识。更换交互源时，对应段的快照会从原交互源清除。

### 悬浮操作菜单

`FabMenu::new(id, open_name, icon, items)` 默认保留自己的展开状态。应用持有状态时，通过 `.expanded(expanded)` 传入当前值，并通过 `.on_expanded_change(|requested, window, cx| ...)` 接收请求。触发器请求切换展开状态。按 Escape、在菜单外按下指针或激活操作项都会请求 `false`。即使应用保持菜单展开，操作项也会执行一次自身命令。展开生效后，焦点移至首个可用项。关闭生效时，如果焦点仍属于菜单，则回到触发器。`.track_focus(&handle)` 接收调用方持有的触发器焦点句柄。

用 `.alignment(FabMenuAlignment)` 将弹出层和胶囊操作项对齐到逻辑 Start、Center 或 End，默认是 End。Start 和 End 随窗口文本方向镜像。可用 `.open_icon(element)` 和 `.close_icon(element)` 设置自定义图标，或用 `.open_content(element)` 和 `.close_content(element)` 设置任意装饰内容。同一状态下最后一次内容设置生效。构造器提供收起状态的无障碍名称，`.close_label(name)` 设置展开状态的名称，两种状态使用同一个触发器。

用 `.trigger_style(FabMenuTriggerStyle)` 分别为 `collapsed` 和 `expanded` 提供 `FabMenuTriggerAppearance`。每个端点可设置尺寸、逻辑形状、图标尺寸、容器色、内容色和高度层级。省略的字段沿用所选 FAB 尺寸和配色。触发器在端点之间插值，正的有限尺寸受可用空间约束，无效尺寸使用默认值。启用减少动态效果时直接应用目标端点。触发器插槽只用于呈现，由按钮管理焦点和激活。

`FabMenuItem::new(id, name, icon)` 创建带图标和标签的操作项，`FabMenuItem::custom(id, name, content)` 接收任意呈现内容。`.icon(element)` 替换图标，`.content(element)` 替换可见标签。操作项继续使用构造器提供的无障碍名称。`.colors(FabMenuItemColors)` 设置启用和禁用的容器色和内容色。`.disabled(true)` 保留操作项的显示状态，并在激活和键盘移动时跳过该项。

通过 `.visible(visible)` 独立控制触发器是否显示，通过 `.visibility_animation(FabVisibilityAnimation)` 设置缩放、透明度和逻辑缩放原点。开始隐藏时，触发器和菜单操作立即退出焦点导航和无障碍树，淡出完成后释放触发器的布局空间。触发器再次显示时，受控的 `.expanded(true)` 会重新生效。

### 工具栏

`Toolbar::new(id, name, actions)` 默认创建采用 Standard 配色的横向悬浮工具栏。通过 `ToolbarAction::new(id, label, IconName)` 或 `ToolbarAction::with_icon(id, label, || ...)`（自定义图标）构造操作项，行内按钮与收起的溢出 `MenuItem` 均会渲染对应图标。`.content(element)` 在生成的操作项前插入任意内容，包括可编辑文本框。`.leading_content(element)` 和 `.trailing_content(element)` 在该区域前后加入插槽。插槽默认随 `.expanded(expanded)` 与内容滚动展开或收起。可通过 `.leading_expanded(bool)` 或 `.trailing_expanded(bool)` 单独设置。拥有焦点的插槽保留至其中的控件失去焦点。嵌入控件管理自身的键盘和输入行为。生成的操作项继续支持方向键导航，收起的次要操作移入有名称的溢出菜单。

通过 `.placement(ToolbarPlacement::Docked)` 使用停靠工具栏。横向停靠栏有空余空间时，可用 `.arrangement(ToolbarArrangement)` 按逻辑 Start、Center、End、SpaceBetween、SpaceAround 或 SpaceEvenly 排列。`.spacing(pixels)` 设置最小间距，默认 4 dp。`.expanded_height(pixels)` 设置横向停靠栏展开时的固定高度。省略设置时沿用现有布局。横向顺序和逻辑内边距在 RTL 中镜像，纵向插槽保持从上到下的顺序。为工具栏提供有界的主轴尺寸，以便空间缩小时滚动到获得焦点的控件。

通过 `.padding(ToolbarPadding { top, start, bottom, end })`、`.shape(CornerShape)`、`.colors(ToolbarColors)` 和 `.elevations(ToolbarElevations)` 设置实例外观。悬浮栏仍默认使用圆角，停靠栏仍默认使用直角。工具栏配色包括容器色、继承的内容色及选中操作色。高度层级随展开状态变化，默认为零。`.fab(...)` 在悬浮栏旁放置拥有独立名称和焦点的悬浮操作按钮。通过 `.fab_color(FabColor)` 和 `.fab_colors(ButtonColors)` 单独配置该按钮。`.fab_content(position, element)` 可在相同位置放入自定义控件。停靠栏的主操作由内容或操作列表提供。

使用 `.track_scroll(&handle)` 读取滚动视图的纵向偏移，或通过 `.scroll_state(&state)` 接入保留的 `Entity<ToolbarScrollState>`，并调用 `set_content_offset(offset, cx)`。同时设置时由滚动句柄提供偏移。使用从内容顶部起算的非负逻辑像素距离。`.scroll_behavior(ToolbarScrollBehavior)` 设置展开和收起阈值，默认从最后一次换向起各累计 40 dp。设置 `exit_direction` 后，向前滚动将工具栏移向逻辑 Start、End、Top 或 Bottom，反向滚动使其返回。工具栏保留布局占位，到达隐藏端点时移除其中的控件，内部控件持有焦点时保持可见。相邻的悬浮操作按钮保持可用。省略退出方向时，滚动只改变展开状态。

可从保留的状态读取 `content_offset()`、`offset()`、`offset_limit()` 和 `collapsed_fraction()`。`offset()` 是朝指定边缘移动的非负距离。`set_collapsed_fraction(fraction, cx)` 立即应用位置。应用管理的手势结束时可调用 `settle(velocity, cx)`。正速度朝退出方向移动。惯性配置在指定时间内投影速度，并归位到最近的端点。用 `snap_motion` 和 `ToolbarFling::motion` 设置归位动效。新的滚动输入从当前显示位置中断归位。`.transition_motion(MotionSpec)` 控制插槽展开与高度层级动效。启用减少动态效果时直接应用目标状态。

### 日期选择器

每个选择器保留一个 `Entity<DatePickerState>`。`DatePicker::new(id, label, &state, value)` 提供打开控件（支持 `.disabled(true)` 禁用触发控件，并通过 `DatePickerState::trigger_focus()` 读取默认触发器焦点句柄）。应用持有已提交的 `DateSelection`，每次打开时用当前值初始化草稿。停靠式选择器在选择完整后立即提交，模态选择器在验证并确认后提交。`DatePickerVariant::FullScreen` 在紧凑窗口中填满内容区，在较宽窗口中使用基本对话框。`ModalInput` 打开时使用保留的文本编辑器。

用 `DatePicker::panel(id, label, &state)` 将日历或编辑器嵌入其他布局，初始草稿与模式由保留状态控制。`DatePicker::dialog(id, label, &state, value)` 为调用方的触发控件提供模态对话框。用 `state.open(value, variant, &locale, window, cx)` 打开，用 `close(window, cx)` 主动关闭。`.focus_target(&focus)` 为内置打开控件指定初始焦点，默认聚焦日历或首个编辑器。调用方控制打开操作时，也可在打开后设置焦点。

构建状态时用 `.initial_month(date)`、`.date_bounds(range)`、`.disabled_dates(predicate)` 和 `.disabled_years(predicate)` 配置日期约束。显示的月份归一到月首，并限制在日期范围内。日历可显示禁用年份，但这些日期无法选取，年份列表也会禁用对应项。用 `displayed_month()` 和 `set_displayed_month(date, cx)` 读写当前月份。用 `draft(cx)` 读取草稿，或用 `set_draft(DatePickerDraft, window, cx)` 更新。日期范围可暂时只有开始日期。对草稿调用 `selection()` 可获取完整的候选值。无效或禁用的端点会被拒绝。`display_mode()` 和 `set_display_mode(mode, window, cx)` 在 `Calendar` 与 `Input` 间切换。用 `.show_mode_toggle(false)` 隐藏切换控件。通过 `input_editors(cx)` 可获取开始日期的保留编辑器，以及范围选择时的结束日期编辑器。

用 `.docked_anchor(|field| ...)` 自定义停靠式 `TextField`。保留传入的字段，以继续使用 Base 编辑器、无障碍语义与日历触发控件。末尾操作打开日历。输入法组合完成后，按 Enter 可接受有效的输入。日期范围输入用 “ – ” 分隔两个端点。`DatePickerLocale` 配置输入格式、解析、每周首日与日历标签。用 `.accessible_date_format(|date| ...)` 独立设置日期和选择状态的播报格式。

用 `.title_content` 和 `.headline_content` 替换可见标题与选择摘要，打开控件与日历状态仍保留无障碍描述。用 `.confirm_content(|confirm| ...)` 和 `.dismiss_content(|dismiss| ...)` 提供模态操作，并将激活连接到传入的 `DialogAction`。确认会验证当前草稿，未完成或无效时保持打开。`.properties(DialogProperties)` 控制 Escape 和外部点击关闭。`.on_dismiss_request` 可拒绝 Escape 和外部点击关闭，`.on_close` 对一次接受的关闭操作报告一次原因。程序主动关闭会更新 `last_close_reason(cx)`。用 `.colors(DatePickerColors)` 配置容器、标题、摘要、导航、年份与日期状态以及输入编辑器。

### 时间选择器

`TimePicker::new(id, label, &state, value)` 提供打开标准对话框的触发按钮，每次打开都用应用已提交的值创建精确到分钟的草稿。默认使用表盘、24 小时制和自动布局。通过 `.variant(TimePickerVariant::Input)` 或 `.variant(TimePickerVariant::Scroll)` 设置初始模式，使用 `.format(TimeFormat::TwelveHour)` 启用上午和下午选择，支持 `.disabled(bool)`、`.show_mode_toggle(bool)`、`.colors(DialogColors)`、`.properties(DialogProperties)`、`.on_dismiss_request(...)`、`.on_close(...)` 和 `.on_confirm(...)` 配置对话框并将确认的草稿应用到程序状态中；通过 `TimePickerState::trigger_focus()` 读取触发器焦点句柄。表盘支持自动、横向和纵向布局。两种窗口方向下表盘都按顺时针排列，字段顺序按窗口方向镜像。

每个选择器通过 `cx.new(|cx| TimePickerState::new(window, cx))` 创建一个 `Entity<TimePickerState>`。独立挂载时，`TimePickerPanel::new(id, &state)` 使用状态中的模式，`TimePickerPanel::dial(id, &state)` 和 `TimePickerPanel::input(id, &state)` 分别固定显示表盘和输入面板，`TimeScroll::new(id, &state)` 显示两个鲜明配色的滚动字段。每个状态只挂载一个面板。用 `.layout(...)` 设置表盘布局，或用 `.mode_toggle_content(...)` 在独立输入或滚动字段旁放置切换控件。小时和分钟各自循环。每个字段有独立的无障碍名称、当前值、焦点和方向键操作。滚轮归位后将选中的值写入草稿。启用减少动效时立即归位。

通过实体读取最近的有效 `value()`、`hour()`（0–23）、`minute()`（0–59）、`segment()`、`mode()` 和 `format()`。使用 `set_value`、`set_hour` 或 `set_minute` 从外部修改时间。越界数值会被拒绝，保留当前编辑值。使用 `set_segment`、`set_mode`、`set_format` 和 `set_locale` 改变呈现方式或活动字段。`TimePickerLocale` 可设置数字格式与解析、分隔符、字段与时段名称及验证消息。输入过程中，`hour_input(cx)` 和 `minute_input(cx)` 可以包含未完成的文本。用 `set_hour_input` 和 `set_minute_input` 可从外部写入本地化原始文本。每次有效编辑会更新对应的草稿分量。`is_pm()` 返回当前是否为下午，`is_hour_input_valid(cx)`、`is_minute_input_valid(cx)` 和 `is_input_valid(cx)` 分别检查小时、分钟和两者。切换到另一模式会从草稿恢复有效文本。输入法处于组合态时，确认和模式切换会等到组合结束。选择器收到 Escape 时会先结束组合，保持对话框打开。

应用自定义触发器可挂载 `TimePickerDialog::new(id, title, &state)` 或 `VibrantTimePickerDialog::new(id, name, &state)`，并在触发时调用 `state.open(value, window, cx)`。鲜明配色对话框采用表层容器色和大圆角。即使标题在视觉上隐藏，辅助技术也能读出其名称。两种对话框默认都根据保留的模式显示对应面板。通过 `.body(...)` 放入固定面板时，让保留的模式与面板匹配，使焦点和验证对应。用 `.body(...)` 放入自定义控件，`.title_content(...)` 替换可见标题，或用 `.mode_toggle_content(|action| ...)` 替换模式切换控件。调用该操作的 `activate(window, cx)` 切换模式并转移焦点。`.show_mode_toggle(false)` 和 `.show_dismiss(false)` 隐藏对应的默认控件。通过 `.confirm_content(|action| ...)` 和 `.dismiss_content(|action| ...)` 将自定义控件接入同一对话框流程。用 `.properties(...)` 配置 Escape 与外部点击关闭。`.on_dismiss_request(...)` 可以拒绝这些请求，`.on_close(...)` 在接受用户关闭时收到一次关闭原因。取消、Escape 和外部点击都不会确认。确认时如果存在无效输入，对话框保持打开，将对应字段标记为无效并聚焦首个无效字段。接受关闭后，Material 对话框会恢复先前的焦点。调用 `state.close(window, cx)` 可通过代码关闭，无需触发用户关闭回调。

通过选择器、面板或对话框的 `.style(TimePickerStyle)` 覆盖时间字段与时段选择器的形状、面板和表盘的配色、表盘指针和标签、时间及上午下午的选中与未选中颜色、时段描边、描边文本字段的配色和滚动字段配色。省略的颜色及形状采用预设以及当前窗口主题。用对话框的 `.shape(...)` 和 `.colors(DialogColors)` 独立调整外壳。

### 滚动选择字段

每个字段通过 `cx.new(|cx| ScrollFieldState::new(item_count, index, cx))` 创建一个 `Entity<ScrollFieldState>`，并传给 `ScrollField::new(id, name, &state)`。`selected_option()` 返回最接近居中的选项，`target_option()` 返回目标选项，`is_scroll_in_progress()` 表示仍在滚动或归位。可观察该实体来更新其他视图。`scroll_to_option(index, cx)` 立即定位，`animate_scroll_to_option(index, cx)` 遵循窗口的动效方案和系统减少动效设置。越界索引会限制在有效范围内。`set_item_count(count, cx)` 更新选项数并限制目标索引。选项数为零时，字段为空，且没有选中项或目标项。

当前选项在三行可见区域内居中。可点击某行，也可使用鼠标滚轮、触控板、上下方向键或无障碍操作改变选项。滚动到末项后会回到首项，反向操作同样循环。通过 `.item_content(|index, selected, enabled| ...)` 为每行提供装饰性 GPUI 元素。默认将索引补足为至少两位数字。构造器的名称用于标识字段，`.value_description(|index| ...)` 为辅助技术描述滚动中的目标选项和静止时的居中选项。字段名称应说明用途，并在需要时说明选项范围。

通过 `.colors(ScrollFieldColors)` 设置启用和禁用时的容器、普通选项及选中选项颜色。默认配色使用 `SurfaceContainerLowest`、`Outline` 和 `OnSurface`。`.disabled(true)` 结束当前的用户滚动并阻止后续用户操作。调用方仍可通过状态定位。使用 `.interaction_source(&source)` 接入调用方持有的 `InteractionSource`，独立于选中值观察字段交互。

### 轮播

通过 `Carousel::new(id, label, items)` 构造轮播，为每个条目提供唯一且稳定的 ID。可用 `cx.new(|_| CarouselState::new(initial_index))` 创建 `Entity<CarouselState>`，并通过 `.state(&state)` 接入。观察该实体即可读取 `current_item()`、`current_item_id()` 和相对最接近条目的带符号偏移 `current_item_offset_fraction()`。条目移动时位置随目标 ID 保持，目标被移除时回到经过边界限制的原索引。初始索引和空列表期间发出的请求会在条目到达后生效。通过 `scroll_to_item(index, cx)` 立即定位，通过 `animate_scroll_to_item(index, cx)` 按窗口动效设置移动。两种方法都会限制索引范围。每个实体对应一个轮播。未接入实体时，轮播按元素 ID 保留自身位置。

通过 `.alignment(CarouselAlignment::Start)` 或 `.alignment(CarouselAlignment::Center)` 设置 Multi-browse、Hero 和 Centered Hero 的对齐方式。Centered Hero 默认居中，其他内含布局默认起始侧对齐。六种布局均支持 `.item_spacing(px(...))` 和 `.content_padding(CarouselPadding { start, end, top, bottom })`。`start` 和 `end` 是逻辑方向边。横向布局默认使用 8 px 间距、16 px 起始侧内边距和 8 px 交叉轴内边距。内含布局还有 16 px 末尾侧内边距。Full-screen 默认使用 16 px 间距和零内边距。`.small_item_widths(min, max)` 设置小条目的正数宽度上下限，默认范围为 40–56 px。视口过窄时布局可简化为一个焦点条目。

`.user_scroll_enabled(false)` 关闭滚轮、触控板、键盘导航及无障碍滚动，条目仍可激活，应用仍可定位。内含布局和 Full-screen 默认吸附到最近的定位线，非内含布局默认自由滚动，可通过 `.snap_scrolling(true)` 启用吸附。`.fling_behavior(...)` 可选择自由滚动、就近吸附、单条目推进，或根据 `CarouselFlingContext` 自定义最终条目。逐格滚轮与无障碍滚动请求按条目移动。定位动画和吸附遵循减少动态效果设置。

在可滚动布局中，用 `CarouselParallax::new(content)` 包装任意 GPUI 内容。效果默认使用 GPUI 当前裁剪区域作为视口，也可通过 `.viewport(bounds)` 传入窗口坐标下的视口。纵向布局使用 `.axis(Axis::Vertical)`。条目进出视口时，内容逐步裁剪和平移，布局尺寸保持固定。减少动态效果设置保留视口裁剪，并跳过附加的视差平移。通过 `.shape(CornerShape)` 和 `.border(CarouselBorder)` 配置匹配的圆角和描边。GPUI 使用矩形区域裁剪内容，圆角处通过 Material Surface 颜色覆盖，也可通过 `.backdrop(color)` 设置与均匀父背景匹配的不透明色。图案或透明背景可使用直角保留原背景。

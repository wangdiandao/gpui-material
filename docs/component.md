# Components

English | [中文](component.zh.md)

These components and layouts are available from `gpui_material`. Enable the named [features](#optional-components) to use the optional component families. See the [integration guide](integration.md) for setup and platform support, or run `cargo gallery` to try interactive examples.

The [pinned Material 3 specification](design-sources.md#material-specification) governs component design. The links below point to the corresponding official guidance. [Site snapshots](material-snapshot.md) record the version used by the project.

<a id="操作与选择"></a>

## Actions and selection

| Component | Supported variants and behavior | Material 3 |
| --- | --- | --- |
| `Button`, `ToggleButton` | Elevated, Filled, Filled Tonal, and Outlined. `Button` also has a Text variant. Both support XS–XL sizes, round or square shapes, custom content and appearance, and optional interaction observation. | [Buttons](https://m3.material.io/components/buttons/overview) |
| `IconButton`, `IconToggleButton` | Standard, Filled, Filled Tonal, and Outlined, with selectable sizes and widths, per-instance shapes, colors and stroke, and optional interaction observation. | [Icon buttons](https://m3.material.io/components/icon-buttons/overview) |
| `Fab`, `ExtendedFab` | Small, Medium, and Large sizes with multiple color roles. `ExtendedFab` supports a text label and an icon. | [FAB](https://m3.material.io/components/floating-action-button/overview) · [Extended FAB](https://m3.material.io/components/extended-fab/overview) |
| `Checkbox` | Unchecked, checked, and mixed states. | [Checkbox](https://m3.material.io/components/checkbox/overview) |
| `Radio`, `RadioGroup` | Mutually exclusive selection with keyboard navigation in a group. | [Radio button](https://m3.material.io/components/radio-button/overview) |
| `Switch` | On and off states with optional thumb icons. | [Switch](https://m3.material.io/components/switch/overview) |
| `Chip` | Assist, Filter, Input, and Suggestion, with selection and removal where applicable. | [Chips](https://m3.material.io/components/chips/overview) |
| `Slider` | Single-value, centered, and range sliders, with horizontal or vertical layouts and continuous or stepped values. | [Sliders](https://m3.material.io/components/sliders/overview) |

Use `.content(element)` to customize the rendered label while retaining the constructor's accessible name and any leading or trailing icon slots. Keep the content free of independent controls so the button owns activation and focus. Set `.shapes(ButtonShapes)`, `.colors(ButtonColors)`, `.border(ButtonBorder)`, `.content_padding(ButtonPadding)`, and `.elevations(ButtonElevations)` per instance. Unspecified shape, color, and elevation fields keep the size and variant defaults for that state. `ButtonPadding` follows the window's text direction. A zero-width border removes the default outline.

Pass a decorative icon element and an accessible name to `IconButton::new` or `IconToggleButton::new`. Icon buttons default to the Small size, Uniform width, Round shape, and Standard palette. Use `.shapes(ButtonShapes)` for resting and pressed corners and for checked corners on toggles. `.colors(ButtonColors)` supplies container and icon colors for enabled, disabled, and checked states. Omitted fields use the current size and variant defaults. `.border(ButtonBorder)` replaces the default stroke. Use zero width to remove it. `IconToggleButton` renders the caller's selected value and uses `.selected_icon(element)` when selected, if supplied.

Keep an `Entity<InteractionSource>` in the caller and attach it with `.interaction_source(&source)` to observe a button or icon button. Give each control sharing a source a distinct element ID within its window. Read the current state with `InteractionSource::state`, or subscribe to `InteractionUpdate` for each deferred change, including press and release between draws. A removed control clears its snapshot. Base still owns input handling and the application owns toggle values.

### Floating action button

`Fab::new` takes an icon and an accessible name. It defaults to a 56 dp Small FAB in the primary container palette. Give each Small, Medium, or Large instance its own `.shape(CornerShape)`, `.colors(ButtonColors)`, and `.elevations(ButtonElevations)`. Use the container and content fields for enabled and disabled colors. Omitted fields keep the size and palette defaults. Use `.interaction_source(&source)` to observe the action with a caller-owned source.

Keep the visibility flag in application state and pass it with `.visible(visible)` on each render. Use a stable element ID between updates so the two motion channels can reverse from their current progress. `FabVisibilityAnimation` defaults to center alignment, a hidden scale of `0.2`, and the theme's Fast Spatial and Fast Effects motion for scale and alpha. Use logical `FabAlignment` positions to scale about an edge or corner that mirrors in right-to-left windows. Set `target_scale` to a finite nonnegative value and set `scale_motion` and `alpha_motion` independently. Invalid scales, including values that overflow the scaled extent, fall back to `0.2`. Reduced motion applies the final state immediately.

A hiding FAB stops taking focus and input immediately. It occupies its layout space while fading and gives up that space at zero opacity. An initially hidden FAB takes no space. Keep the icon decorative so the FAB owns activation and focus. Use inherited text sizing or relative dimensions for custom icons that should resize during the transition.

<a id="内容与反馈"></a>

## Content and feedback

| Component | Supported variants and behavior | Material 3 |
| --- | --- | --- |
| `Text`, `Icon` | Themed typography, replaceable icons, and bundled Material Symbols. | [Typography](https://m3.material.io/styles/typography/overview) · [Icons](https://m3.material.io/styles/icons/overview) |
| `Surface` | Themed background and tonal elevation for application content. | [Elevation](https://m3.material.io/styles/elevation/overview) |
| `Card` | Elevated, Filled, and Outlined, with static and actionable forms. | [Cards](https://m3.material.io/components/cards/overview) |
| `List`, `ListItem` | Standard and Segmented lists with leading, supporting, and trailing content. | [Lists](https://m3.material.io/components/lists/overview) |
| `SwipeToDismiss` | Horizontal swipes with caller-owned content, background, and action. | [Lists](https://m3.material.io/components/lists/guidelines) · [Compose SwipeToDismissBox](https://developer.android.com/reference/kotlin/androidx/compose/material3/SwipeToDismissBox.composable) |
| `Divider`, `Badge` | Horizontal and vertical dividers with logical insets, configurable thickness, and per-divider color. Standalone and anchored badges with dot, text, count, or custom content and optional colors. | [Divider](https://m3.material.io/components/divider/overview) · [Badges](https://m3.material.io/components/badges/overview) |
| `LinearProgress`, `CircularProgress` | Determinate and indeterminate progress with straight and wavy styling. | [Progress indicators](https://m3.material.io/components/progress-indicators/overview) |
| `LoadingIndicator` | Default and Contained, with determinate and indeterminate modes, custom morph sequences and instance colors. | [Loading indicator](https://m3.material.io/components/loading-indicator/overview) |
| `Snackbar`, `SnackbarHost` | Queued messages with optional actions, timed or persistent display, and dismissal. | [Snackbars](https://m3.material.io/components/snackbar/overview) |

### Divider

`Divider::horizontal()` and `Divider::vertical()` default to one logical pixel in the window theme's `OutlineVariant` color. Use `.thickness(px(2.))` to set a different logical width or `.thickness(px(0.))` for a one-device-pixel hairline. Invalid thickness values fall back to the default. A vertical divider fills the available height of its parent.

Use `.color(ColorRole::Primary)` to resolve a role against the window theme or `.color_override(hsla)` to use a literal color. The last color setter takes precedence. `.inset(start, end)` follows the window's text direction for horizontal dividers and applies to the top and bottom for vertical dividers. The divider is decorative and adds no focus stop.

### Badges

`Badge::dot(host)`, `Badge::label(host, text)`, and `Badge::count(host, count, maximum)` anchor a mark to the host's upper trailing edge. Pass the current count and cap when rendering. Counts above the cap display the cap followed by `+`. `Badge::standalone()` renders a dot in the surrounding layout. Use `.content(id, element)` to supply a short visual element for either placement. Give each custom mark a stable ID within its parent.

Container and content colors default to the window theme's `Error` and `OnError` roles. Use `.container_color(color)` and `.content_color(color)` to set either independently. Keep custom content noninteractive and let the host's accessible name describe the anchored badge. For a standalone indicator that conveys information on its own, use `.accessible_label(id, label)` to give it a name without adding a focus stop.

### Swipe to dismiss

Keep an `Entity<SwipeToDismissState>` per item and pass it to `SwipeToDismiss::new(id, &state, content)`. The state starts at rest. Use `.initial_value(value)` when restoring an item at a dismissal anchor. Add a background with `.background(element)`. It fills the container behind the moving foreground. Observe the state to update the background with the logical `dismiss_direction()`, signed `offset()` in logical pixels, or outward `progress()` from 0 to 1. `current_value()` is the nearest anchor, `target_value()` is the expected landing anchor, and `settled_value()` changes when the movement completes.

Both logical directions and gestures are enabled by default. Use `.enable_start_to_end(bool)`, `.enable_end_to_start(bool)`, and `.gestures_enabled(bool)` to configure them. The window's writing direction determines the physical swipe direction. The default threshold is 56 logical pixels. Use `SwipeToDismissState::positional_threshold(|width| ...)` to compute a threshold from the laid-out width. Finite results are clamped to the available travel and invalid values use the default. Vertical drags and scroll continue through the surrounding layout.

`dismiss(direction, window, cx)` animates to an enabled direction, `reset(window, cx)` animates back to the resting position, and `snap_to(value, window, cx)` moves immediately. Reduced motion settles immediately. `.on_dismiss(|direction, window, cx| ...)` runs once when movement settles in a new dismissal direction. The foreground leaves keyboard traversal and the accessibility tree at that point. The application owns data changes such as removing the item, and can call `reset` to show it again. Provide a named, keyboard-focusable button or menu item for each offered action. Its activation calls the same `dismiss` method and remains available when gestures are disabled.

### Loading indicator

`LoadingIndicator::new(id, label)` uses a 48 dp footprint and a seven-shape indeterminate loop. Add `.contained(true)` for a rounded container. The Default indicator uses the window's `Primary` color. Contained uses `OnPrimaryContainer` for the indicator and `PrimaryContainer` for the container. Override either indicator with `.indicator_color(color)`. Set `.container_color(color)` and `.container_shape(CornerShape)` on Contained instances. Start and end corners follow the window's layout direction.

The application owns determinate progress. Pass it as `.value(fraction)` on each render. The fraction is clamped to `0..=1`, and NaN becomes zero. With the default sequence, zero shows a circle and one shows the Soft Burst shape. Create a reusable `LoadingIndicatorShapes` with at least two `LoadingIndicatorShape` values and pass it by reference with `.shapes(&shapes)`. Use `LoadingIndicatorShape::circle()` or `LoadingIndicatorShape::rounded_polygon(&vertices, radius)` with GPUI points and a radius in the same units. Polygon construction returns `None` for non-finite, degenerate or self-intersecting geometry. The sequence constructor returns `None` if given fewer than two shapes. A determinate sequence goes from first to last, and an indeterminate sequence also morphs from last to first.

The label names a progress indicator. Determinate mode exposes a numeric percentage and draws the supplied value directly. Indeterminate mode exposes no numeric value and uses a polite live status. Reduced motion holds the first indeterminate shape and keeps determinate progress responsive. `.active(false)` retains the footprint and suppresses visuals and progress semantics. Only an active indeterminate indicator in an active window requests animation frames.

<a id="输入与浮层"></a>

## Input and overlays

| Component | Supported variants and behavior | Material 3 |
| --- | --- | --- |
| `TextField` | Filled and Outlined, with single-line, multiline, secure, read-only, and disabled modes. Supports label placement, adornment and description slots, required and invalid states, and character counts. | [Text fields](https://m3.material.io/components/text-fields/overview) |
| `ExposedDropdown` | Editable filtering and selection-only lists anchored to a text field. | [Menus](https://m3.material.io/components/menus/overview) · [Text fields](https://m3.material.io/components/text-fields/overview) |
| `Menu` | Anchored menus and context menus, with groups, checked items, and submenus. | [Menus](https://m3.material.io/components/menus/overview) |
| `Tooltip` | Plain and rich tooltips with hover and keyboard focus activation. | [Tooltips](https://m3.material.io/components/tooltips/overview) |
| `Scrim` | Standalone backdrop with themed or custom color, adjustable opacity, and optional dismissal. | [Scrim](https://developer.android.com/reference/kotlin/androidx/compose/material3/Scrim.composable) |
| `Dialog` | Basic dialogs and full-screen task dialogs that use the window content area on compact layouts. | [Dialogs](https://m3.material.io/components/dialogs/overview) |
| `Sheet` | Standard and modal bottom sheets, and side sheets docked or overlaid at the logical start or end. | [Bottom sheets](https://m3.material.io/components/bottom-sheets/overview) · [Side sheets](https://m3.material.io/components/side-sheets/overview) |

`TextField` decorates the caller's retained Base editor. Use `label_content`, `placeholder_content`, and `supporting_content` with plain text for the accessible name, hint, and description. Leading and trailing elements are separate from the prefix and suffix next to the value. Prefix and suffix appear while the label is minimized. Labels default to Inside for Filled and Cutout for Outlined. Select Above for a label outside the container, or configure the horizontal alignments and keep an inside or cutout label minimized. The character count shows Unicode scalar values without enforcing the limit. Secure fields suppress this count along with their accessible value. Set instance colors, the container shape, content padding, or input typography independently of the window theme. `on_text_layout` reports the laid-out editor and text bounds and the wrapped line count after the frame. Set `interaction_source` to share observations of hover, keyboard focus, and disabled states. The Base entity still owns editing, selection, validation, scrolling, and IME composition.

### Menu controls and item content

`Menu::new` owns its visibility by default. Add `.open(open)` and `.on_open_change(|requested, window, cx| ...)` to let the caller own it. Apply the requested value when desired and render again. A caller update to `open` applies visibility and focus without sending change callbacks. `.on_dismiss(|window, cx| ...)` reports a close request from a user action. `Menu::composed` supplies the displayed open value and an activation callback to a custom trigger.

Keep each `MenuItem` ID stable and provide a label for accessibility. Its `.content`, `.leading`, `.supporting`, `.trailing`, and `.selection_icon` builders accept repeatable element factories for visual content. A menu row owns the keyboard and pointer action. Pass the checked value with `.checked(value)` and handle requests with `.on_checked_change`. Selected items continue to use `.selected(value)` with `.on_activate`.

Use `.color(MenuColor::Vibrant)` for the tertiary menu palette. A `MenuItem::section(id, name, rows)` draws a grouped surface and keeps its rows in the menu's arrow-key order. `Menu::content` adds caller-authored content before the rows. A section's `.content` replaces its visible heading.

Use `MenuSurfaceStyle` with `Menu::appearance` or `MenuItem::group_appearance` to configure the surface shape, padding, spacing, border and elevations. `MenuItemStyle` overrides menu-row shapes, spacing, padding, and color roles. Use `.anchor` and `.offset` for logical trigger placement or `.position` with a window-local point. `.track_scroll` accepts a caller-owned scroll handle.

### Scrim

Place `Scrim::new(id)` in a positioned container, after the background content and before the foreground surface. It fills the container and uses the current window theme's Scrim role at 32% opacity. Use `.color(color)` to supply a custom tint and `.opacity(value)` to multiply the tint's alpha. Values clamp to 0–1 and NaN uses the default.

By default the scrim is decorative. It dims the content while leaving pointer input and accessibility traversal available. Add `.on_click(|window, cx| ...)` to block pointer input behind the scrim and handle pointer or accessibility activation. The scrim has no keyboard tab stop. Use `.accessibility_label(label)` to name the action for assistive technology. The default label is “Dismiss overlay.” The caller owns visibility, dismissal, and the modal focus and accessibility scope. Give modal content its own keyboard dismissal action.

<a id="导航与自适应布局"></a>

## Navigation and adaptive layout

| Component | Supported variants and behavior | Material 3 |
| --- | --- | --- |
| `AppBar` | Small, Centered, Medium Flexible, and Large Flexible top app bars. Flexible variants can collapse on scroll. | [App bars](https://m3.material.io/components/app-bars/overview) |
| `Tabs` | Primary and Secondary tabs with fixed or scrolling layouts. | [Tabs](https://m3.material.io/components/tabs/overview) |
| `NavigationBar`, `NavigationRail` | Short navigation bar with vertical or horizontal items. Rail with collapsed, expanded, hidden, and modal presentation. | [Navigation bar](https://m3.material.io/components/navigation-bar/overview) · [Navigation rail](https://m3.material.io/components/navigation-rail/overview) |
| `AdaptiveNavigation` | Switches between the navigation bar and rail by available width. | [Breakpoints](https://m3.material.io/foundations/layout/breakpoints/overview) |
| `Scaffold` | Combines navigation, app bars, content, a FAB, and snackbars. | [Scaffold](https://m3.material.io/foundations/layout/scaffold/overview) |
| `ListDetail`, `SupportingPane`, `Feed` | Canonical layouts for list and detail views, supporting content, and responsive feeds. | [Canonical layouts](https://m3.material.io/foundations/layout/canonical-examples/overview) |

<a id="可选组件"></a>

## Optional components

Enable features on the `gpui-material` dependency. Import `DatePicker` and `TimePicker` from `gpui_material::date_time`.

| Feature | Component | Supported variants and behavior | Material 3 |
| --- | --- | --- | --- |
| `expressive-groups` | `ButtonGroup`, `SplitButton` | Standard and Connected button groups, and a split button with separate primary and secondary actions. | [Button groups](https://m3.material.io/components/button-groups/overview) · [Split button](https://m3.material.io/components/split-button/overview) |
| `toolbars` | `FabMenu`, `Toolbar` | Expandable FAB menus and floating or docked toolbars with horizontal and vertical layouts. | [FAB menu](https://m3.material.io/components/fab-menu/overview) · [Toolbars](https://m3.material.io/components/toolbars/overview) |
| `search` | `SearchBar`, `SearchView` | Collapsed search bars and docked or full-screen search views. | [Search](https://m3.material.io/components/search/overview) |
| `date-time` | `DatePicker`, `TimePicker` | Single dates and date ranges in Docked, Modal, or Modal Input pickers. Dial and Input time pickers with 12- and 24-hour formats. | [Date pickers](https://m3.material.io/components/date-pickers/overview) · [Time pickers](https://m3.material.io/components/time-pickers/overview) |
| `carousel` | `Carousel` | Multi-browse, Uncontained, Uncontained Multi-aspect Ratio, Hero, Centered Hero, and Full-screen. | [Carousel](https://m3.material.io/components/carousel/overview) |

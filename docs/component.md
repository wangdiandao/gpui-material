# Components

English | [中文](component.zh.md)

These components and layouts are available from `gpui_material`. Enable the named [features](#optional-components) to use the optional component families. See the [integration guide](integration.md) for setup and platform support, or run `cargo gallery` to try interactive examples.

The [pinned Material 3 specification](design-sources.md#material-specification) governs component design. The links below point to the corresponding official guidance. [Site snapshots](material-snapshot.md) record the version used by the project.

<a id="操作与选择"></a>

## Actions and selection

| Component | Supported variants and behavior | Material 3 |
| --- | --- | --- |
| `Button`, `ToggleButton` | Elevated, Filled, Filled Tonal, and Outlined. `Button` also has a Text variant. Both support XS–XL sizes and round or square shapes. | [Buttons](https://m3.material.io/components/buttons/overview) |
| `IconButton`, `IconToggleButton` | Standard, Filled, Filled Tonal, and Outlined, with selectable sizes, widths, and shapes. | [Icon buttons](https://m3.material.io/components/icon-buttons/overview) |
| `Fab`, `ExtendedFab` | Small, Medium, and Large sizes with multiple color roles. `ExtendedFab` supports a text label and an icon. | [FAB](https://m3.material.io/components/floating-action-button/overview) · [Extended FAB](https://m3.material.io/components/extended-fab/overview) |
| `Checkbox` | Unchecked, checked, and mixed states. | [Checkbox](https://m3.material.io/components/checkbox/overview) |
| `Radio`, `RadioGroup` | Mutually exclusive selection with keyboard navigation in a group. | [Radio button](https://m3.material.io/components/radio-button/overview) |
| `Switch` | On and off states with optional thumb icons. | [Switch](https://m3.material.io/components/switch/overview) |
| `Chip` | Assist, Filter, Input, and Suggestion, with selection and removal where applicable. | [Chips](https://m3.material.io/components/chips/overview) |
| `Slider` | Single-value, centered, and range sliders, with horizontal or vertical layouts and continuous or stepped values. | [Sliders](https://m3.material.io/components/sliders/overview) |

<a id="内容与反馈"></a>

## Content and feedback

| Component | Supported variants and behavior | Material 3 |
| --- | --- | --- |
| `Text`, `Icon` | Themed typography, replaceable icons, and bundled Material Symbols. | [Typography](https://m3.material.io/styles/typography/overview) · [Icons](https://m3.material.io/styles/icons/overview) |
| `Surface` | Themed background and tonal elevation for application content. | [Elevation](https://m3.material.io/styles/elevation/overview) |
| `Card` | Elevated, Filled, and Outlined, with static and actionable forms. | [Cards](https://m3.material.io/components/cards/overview) |
| `List`, `ListItem` | Standard and Segmented lists with leading, supporting, and trailing content. | [Lists](https://m3.material.io/components/lists/overview) |
| `Divider`, `Badge` | Horizontal and vertical dividers. Dot, content, and count badges. | [Divider](https://m3.material.io/components/divider/overview) · [Badges](https://m3.material.io/components/badges/overview) |
| `LinearProgress`, `CircularProgress` | Determinate and indeterminate progress with straight and wavy styling. | [Progress indicators](https://m3.material.io/components/progress-indicators/overview) |
| `LoadingIndicator` | Default and contained loading indicators. | [Loading indicator](https://m3.material.io/components/loading-indicator/overview) |
| `Snackbar`, `SnackbarHost` | Queued messages with optional actions, timed or persistent display, and dismissal. | [Snackbars](https://m3.material.io/components/snackbar/overview) |

<a id="输入与浮层"></a>

## Input and overlays

| Component | Supported variants and behavior | Material 3 |
| --- | --- | --- |
| `TextField` | Filled and Outlined, with single-line, multiline, secure, read-only, and disabled modes. Supports error and supporting text. | [Text fields](https://m3.material.io/components/text-fields/overview) |
| `ExposedDropdown` | Editable filtering and selection-only lists anchored to a text field. | [Menus](https://m3.material.io/components/menus/overview) · [Text fields](https://m3.material.io/components/text-fields/overview) |
| `Menu` | Anchored menus and context menus, with groups, checked items, and submenus. | [Menus](https://m3.material.io/components/menus/overview) |
| `Tooltip` | Plain and rich tooltips with hover and keyboard focus activation. | [Tooltips](https://m3.material.io/components/tooltips/overview) |
| `Dialog` | Basic dialogs and full-screen task dialogs that use the window content area on compact layouts. | [Dialogs](https://m3.material.io/components/dialogs/overview) |
| `Sheet` | Standard and modal bottom sheets, and side sheets docked or overlaid at the logical start or end. | [Bottom sheets](https://m3.material.io/components/bottom-sheets/overview) · [Side sheets](https://m3.material.io/components/side-sheets/overview) |

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

---
name: gpui-material
description: Build and integrate Material 3 Expressive desktop interfaces with the gpui-material Rust facade. Use when setting up a Material window or theme, choosing or composing gpui-material controls, wiring controlled values and retained component state, or checking Material design and desktop behavior in this project.
---

# GPUI Material

Import controls and window setup from `gpui_material`, and use `gpui` and `gpui_platform` for the runtime and platform launcher. GPUI provides window and entity lifecycles, Base provides reusable interaction behavior, and Material supplies the tokens, themed controls, and composition.

Use the [component conventions](references/conventions.md) to choose a constructor, state owner, and callback. Use the [application recipe](references/application.md) for a complete window with a controlled switch and a retained text field.

## Read the Guides First

| Guide | Read when |
| --- | --- |
| [Design sources and version baseline](../../docs/design-sources.md) | Choosing a variant, changing tokens or interaction states, or evaluating a desktop adaptation. Follow the pinned Material 3 Expressive specification and its implementation references. |
| [Architecture and dependencies](../../docs/architecture.md) | Deciding crate boundaries, state ownership, Base integration, or public API shape. |
| [Development and verification](../../docs/development.md) | Changing the library or Gallery and selecting the applicable checks. |
| [Downstream integration](../../docs/integration.md) | Adding the facade, starting a window, enabling features, or migrating an application. |

### Coding guide map

| Task | Reference |
| --- | --- |
| Choose a home for new code | [Crate dependency direction](../../docs/architecture.md#crate-dependency-direction) |
| Decide where values, control state, and theme live | [State and theme](../../docs/architecture.md#state-and-theme) |
| Change focus, input, overlays, drawing, or motion | [Desktop adapter boundaries](../../docs/architecture.md#desktop-adapter-boundaries) |
| Match GPUI and Base package identities | [Pinned GPUI and Base versions](../../docs/design-sources.md#gpui-and-base) |
| Modify public APIs or verify a change | [Coding conventions](../../docs/development.md#coding-conventions) and [testing conventions](../../docs/development.md#testing-conventions) |

### Integration contracts

- Verify constructors, callbacks, and feature gates against the public type before writing usage examples. The facade is the supported entry point for downstream code.
- Match the GPUI core and platform package identities in the integration guide. Initialize `MaterialRoot` and `init_components` before opening the first window through `MaterialRoot::open_window`.
- Keep controlled values in the application. Create retained component entities once in their owner, pass them by reference when rendering, and use stable IDs for repeated controls and items.

## Documentation

- Find project specifications in the [documentation index](../../docs/index.md) and supported variants in [Components](../../docs/component.md). Each component family links to its Material 3 specification.
- Look up tokens and theme defaults in [Pure theme](../../docs/theme/theme.md). Use [source precedence](../../docs/design-sources.md#source-precedence) when Material guidance and implementation references differ.
- Confirm import paths in the [public facade](../../crates/material/src/lib.rs) and find working compositions in the [Gallery stories](../../apps/gallery/src).

## Quick Reference

Use the [application recipe](references/application.md) for the dependency manifest and a stateful view.

```rust
use gpui::{AppContext, Context, IntoElement, QuitMode, Render, Window, WindowOptions};
use gpui_material::{Button, MaterialRoot, MaterialTheme};

struct Demo;

impl Render for Demo {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Button::new("save", "Save").on_activate(|_, _| println!("Save requested"))
    }
}

fn main() {
    gpui_platform::application()
        .with_quit_mode(QuitMode::LastWindowClosed)
        .run(|cx| {
            MaterialRoot::init(MaterialTheme::default(), cx);
            gpui_material::init_components(cx);
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| Demo))
                .expect("open Material window");
        });
}
```

- **Controlled value** — build `Switch::new("reminders", "Enable reminders", self.reminders_enabled)` in `render` and update the owner from `.on_change(...)`.
- **Retained editor** — create an `Entity<TextFieldState>` with `cx.new(|cx| TextFieldState::new(window, cx))` and render `TextField::new("name", "Name", &self.name)`.
- **Theme** — use `gpui_material::material_theme(cx)` for the application theme and `gpui_material::foundation::window_material_theme(window, cx)` for the effective window theme. See [theme overrides](../../docs/integration.md#api-and-migration).
- **Overlays** — retain a `DialogState` entity for `Dialog` or `Sheet`, and mount a `SnackbarHost` entity for queued messages. See the [overlay Gallery stories](../../apps/gallery/src/overlay_examples.rs).

## Component Catalog

Import these types from `gpui_material` unless a module is shown. See [Components](../../docs/component.md) for variant details and the matching Material specifications.

### Actions and selection

| Public types | What to wire |
| --- | --- |
| `Button`, `IconButton`, `Fab`, `ExtendedFab` | Commands via `.on_activate(...)`. |
| `ToggleButton`, `IconToggleButton`, `Checkbox`, `Switch` | Owner-provided values and `.on_change(...)`. `Checkbox` supports a mixed state. |
| `Radio`, `RadioGroup`, `Chip` | Owner-provided selection. Use `RadioGroup` for mutually exclusive choices and group keyboard navigation. |
| `Slider`, `SliderState`, `SliderEvent` | Retain the state entity for single-value, centered, or range input. |

### Content and feedback

| Public types | Use |
| --- | --- |
| `Text`, `Icon`, `Surface`, `Card` | Themed typography, symbols, surfaces, and cards. |
| `List`, `ListItem`, `Divider`, `Badge` | Group and label content. Use `List` for small collections. |
| `LinearProgress`, `CircularProgress`, `LoadingIndicator` | Progress and loading feedback. |
| `Snackbar`, `SnackbarHost` | Queue messages and optional actions in the window layout. |

### Input and overlays

| Public types | What to retain or compose |
| --- | --- |
| `TextField`, `TextFieldState`, `TextAreaState` | Use `TextFieldState` for single-line or secure input and `TextAreaState` with `TextField::multiline`. |
| `ExposedDropdown`, `ExposedDropdownState`, `DropdownOption` | Retain the entity and give options stable IDs for editable or selection-only lists. |
| `Menu`, `MenuItem`, `Tooltip` | Compose anchored or context menus and plain or rich tooltips. |
| `Dialog`, `DialogState`, `Sheet` | Compose a dialog or sheet around retained state. |

### Navigation and adaptive layout

| Public types | Use |
| --- | --- |
| `AppBar`, `Tabs`, `NavigationBar`, `NavigationRail` | Top app bars, tabs, and navigation destinations. |
| `AdaptiveNavigation`, `Scaffold` | Compose a window shell and adapt its navigation to the available width. |
| `ListDetail`, `SupportingPane`, `Feed` | Material canonical layouts. |

### Optional components

Enable these features on `gpui-material`.

| Feature | Public types |
| --- | --- |
| `expressive-groups` | `ButtonGroup`, `SplitButton` |
| `toolbars` | `FabMenu`, `Toolbar` |
| `search` | `SearchBar`, `SearchView`, `SearchState` |
| `date-time` | `date_time::{DatePicker, DatePickerState, TimePicker, TimePickerState}` |
| `carousel` | `Carousel`, `CarouselItem` |

## Testing

Use `cargo gallery text-field` to open the text field story, or pass the affected story name or ID. For automated interaction checks, follow existing `#[gpui::test]` cases that render production components and assert the interaction outcome, such as callback data, focus, or retained state. The Gallery suite runs with `cargo test --locked -p gpui-material-gallery --features test-support`. Select the relevant native checks separately when the change touches the OS bridge, IME, DPI, or assistive technology. [Local checks](../../docs/development.md#local-checks)

## GPUI and Base References

Use these pinned upstream references for runtime mechanics and unstyled primitives. Resolve Material API and design questions through this project's facade and specifications.

| Topic | Reference |
| --- | --- |
| `App`, `Window`, `Context<T>`, and retained entities | [Contexts][gpui-context] · [Entities][gpui-entity] |
| Actions, events, and focus restoration | [Actions][gpui-action] · [Events][gpui-event] · [Focus][gpui-focus] |
| Layout, stable IDs, and async updates | [Layout][gpui-layout] · [Element IDs][gpui-id] · [Async][gpui-async] |
| Unstyled controls and infrastructure | [GPUI Base][base] |

[gpui-context]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/skills/gpui-kit/references/gpui/context.md
[gpui-entity]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/skills/gpui-kit/references/gpui/entity.md
[gpui-action]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/skills/gpui-kit/references/gpui/action.md
[gpui-event]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/skills/gpui-kit/references/gpui/event.md
[gpui-focus]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/skills/gpui-kit/references/gpui/focus-handle.md
[gpui-layout]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/skills/gpui-kit/references/gpui/layout-style.md
[gpui-id]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/skills/gpui-kit/references/gpui/element-id.md
[gpui-async]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/skills/gpui-kit/references/gpui/async.md
[base]: https://github.com/longbridge/gpui-kit/blob/39ab849a93b5f15507b969164d49891db8d5b470/website/base/index.md

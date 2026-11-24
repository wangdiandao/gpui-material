---
name: gpui-material
description: Build and integrate Material 3 Expressive desktop interfaces with the gpui-material Rust facade. Use when setting up a Material window or theme, choosing or composing gpui-material controls, wiring controlled values and retained component state, optimizing rendering or frame pacing, or checking Material design and desktop behavior in this project.
---

# GPUI Material

Import controls and window setup from `gpui_material`, and use `gpui` and `gpui_platform` for the runtime and platform launcher. GPUI provides window and entity lifecycles, Base provides reusable interaction behavior, and Material supplies the tokens, themed controls, and composition.

## References in This Skill

| Reference | Open when |
| --- | --- |
| [Component conventions](references/conventions.md) | Choosing a constructor, state owner, callback, or stable ID contract. |
| [Application recipe](references/application.md) | Setting up `Cargo.toml` (including dev rendering optimization) and a complete stateful Material window. |
| [Development practices](references/practices.md) | Handling complex controls, focus, virtualization, custom shapes/elements, rendering performance, 120Hz+ frame pacing, and headless/native testing. |

## Read the Project Guides First

| Guide | Read when |
| --- | --- |
| [Design sources and version baseline](../../docs/design-sources.md) | Choosing a variant, changing tokens or interaction states, or evaluating a desktop adaptation. Follow the pinned Material 3 Expressive specification and its implementation references. |
| [Architecture and dependencies](../../docs/architecture.md) | Deciding crate boundaries, state ownership, Base integration, or public API shape. |
| [Cookbook](../../docs/cookbook.md) | Looking for end-to-end recipes for forms, theming, layouts, focus, custom controls, and slim builds. |
| [Development and verification](../../docs/development.md) | Changing the library or Gallery and selecting the applicable checks and performance tools. |
| [Downstream integration](../../docs/integration.md) | Adding the facade, starting a window, enabling features, or migrating an application. |

### Coding guide map

| Task | Reference |
| --- | --- |
| Choose a home for new code | [Crate dependency direction](../../docs/architecture.md#crate-dependency-direction) |
| Decide where values, control state, and theme live | [State and theme](../../docs/architecture.md#state-and-theme) |
| Change focus, input, overlays, drawing, or motion | [Desktop adapter boundaries](../../docs/architecture.md#desktop-adapter-boundaries) |
| Match GPUI and Base package identities | [Pinned GPUI and Base versions](../../docs/design-sources.md#gpui-and-base) |
| Modify public APIs or verify a change | [Coding conventions](../../docs/development.md#coding-conventions) and [testing conventions](../../docs/development.md#testing-conventions) |

### Core integration contracts

- Verify constructors, callbacks, and feature gates against the [public facade](../../crates/material/src/lib.rs) before writing usage examples.
- Match the pinned `gpui-pre`, `gpui-pre-platform`, and (if imported directly) `gpui-base` versions exactly. Initialize `MaterialRoot::init` and `gpui_material::init_components` before opening the first window through `MaterialRoot::open_window`.
- Keep controlled values in the application. Create retained component entities (`Entity<*State>`) once in their owner, pass them by reference when rendering, and use semantic stable IDs for repeated controls and items.
- Optimize rendering and layout dependencies in `[profile.dev.package]` and always measure frame rate or smoothness in `--release` builds. See [Development practices](references/practices.md#rendering-performance-and-frame-pacing).

## Documentation

- Find project specifications in the [documentation index](../../docs/index.md) and supported variants in [Components](../../docs/component.md). Each component family links to its Material 3 specification.
- Look up tokens and theme defaults in [Pure theme](../../docs/theme/theme.md). Use [source precedence](../../docs/design-sources.md#source-precedence) when Material guidance and implementation references differ.
- Find working compositions in the [Gallery stories](../../apps/gallery/src).

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
- **Retained editor** — create an `Entity<TextFieldState>` once with `cx.new(|cx| TextFieldState::new(window, cx))` and render `TextField::new("name", "Name", &self.name)`.
- **Theme** — use `gpui_material::material_theme(cx)` for the application theme, `gpui_material::foundation::window_material_theme(window, cx)` for the effective window theme, and `ThemeScope` for subtree overrides. See [theme overrides](../../docs/integration.md#api-and-migration).
- **Overlays** — retain a `DialogState` entity for `Dialog`, `Sheet`, or anchored `Menu`, and mount a `SnackbarHost` entity for queued messages. See the [overlay Gallery stories](../../apps/gallery/src/overlay_examples.rs).

## Component Catalog

Import these types from `gpui_material` unless a module is shown. See [Components](../../docs/component.md) for variant details and the matching Material specifications.

### Actions and selection

| Public types | What to wire |
| --- | --- |
| `Button`, `IconButton`, `Fab`, `ExtendedFab` | Commands via `.on_activate(...)`. Shorthand constructors exist for variants (`filled`, `tonal`, `elevated`, `outlined`, `text`). |
| `ToggleButton`, `IconToggleButton`, `Checkbox`, `Switch` | Owner-provided values and `.on_change(...)`. `Checkbox` supports a mixed state; toggle buttons support controlled `.loading(bool)`. |
| `Radio`, `RadioGroup`, `Chip`, `ChipGroup` | Owner-provided selection. Use `RadioGroup` for mutually exclusive choices and `ChipGroup` for group selection/defaults and roving focus. |
| `Slider`, `SliderState`, `SliderEvent` | Retain the state entity for single-value, centered, or range input; optionally attach caller-owned focus via `.track_focus(...)`. |

### Content and feedback

| Public types | Use |
| --- | --- |
| `Text`, `Icon`, `IconName`, `Surface`, `Card`, `Avatar`, `AvatarGroup` | Themed typography, embedded Material Symbols SVGs, surfaces, cards, and avatars. |
| `List`, `ListItem`, `Divider`, `Badge`, `Banner`, `EmptyState`, `Skeleton`, `DisplayScrollbar` | Group and label content, inline alerts, placeholders, and overlay scrollbars. Use `List` for small collections and virtualized GPUI `list` for large datasets. |
| `LinearProgress`, `CircularProgress`, `LoadingIndicator`, `PullToRefresh`, `SwipeToDismiss`, `DragHandle`, `DropTarget` | Progress, morphing loading feedback, gesture containers, and drag affordances. |
| `Snackbar`, `SnackbarHost` | Queue messages and optional actions in the window layout. |

### Input and overlays

| Public types | What to retain or compose |
| --- | --- |
| `TextField`, `TextFieldState`, `TextAreaState`, `TextFieldObfuscationMode`, `TextFieldMaskPattern` | Use `TextFieldState` for single-line or `TextField::secure` input (with Base `validate`/`pattern` and `MaskPattern`) and `TextAreaState` with `TextField::multiline`. |
| `ExposedDropdown`, `ExposedDropdownState`, `DropdownOption` | Retain the entity and give options stable IDs for editable or selection-only lists. |
| `Menu`, `MenuItem`, `Tooltip` | Compose anchored or context menus and plain or rich tooltips. |
| `Dialog`, `DialogState`, `Sheet` | Compose a dialog or sheet around retained state. |

### Navigation and adaptive layout

| Public types | Use |
| --- | --- |
| `AppBar`, `Tabs`, `NavigationBar`, `NavigationRail`, `NavigationDrawer`, `Breadcrumbs`, `Stepper` | Top app bars, tabs, navigation destinations, breadcrumbs, and multi-step flows. |
| `AdaptiveNavigation`, `Scaffold` | Compose a window shell and adapt its navigation to the available width. |
| `ListDetail`, `SupportingPane`, `Feed` | Material canonical adaptive layouts. |

### Optional features

Enable these features on `gpui-material` (or disable default `full` for slim builds).

| Feature | Public types |
| --- | --- |
| `expressive-groups` | `ButtonGroup`, `SplitButton` |
| `toolbars` | `FabMenu`, `Toolbar` |
| `search` | `SearchBar`, `SearchView`, `SearchState` |
| `date-time` | `date_time::{DatePicker, DatePickerState, TimePicker, TimePickerState}` |
| `carousel` | `Carousel`, `CarouselItem` |
| `window-chrome` | `window_chrome::{WindowChrome, ...}` (cross-platform custom title bar and caption controls) |
| `dynamic-color` | Dynamic color schemes, custom color harmonization (`ColorScheme::from_seed`, `harmonize_color`) |

## Testing and Profiling

- Open a single Gallery story with `cargo gallery text-field` (or `--release` for smooth rendering checks).
- Run headless UI tests with `cargo test --locked -p gpui-material-gallery --features test-support`.
- Run engineering checks with `cargo xtask check` and `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`.
- Profile frame times with `perf_bench` or `--frame-profile` in locked Release mode. See [Development practices](references/practices.md#testing-and-verification) and [Local checks](../../docs/development.md#local-checks).

## GPUI and Base References

Use these pinned upstream references (`v0.11.0` / `e97fe438`) for runtime mechanics and unstyled primitives. Resolve Material API and design questions through this project's facade and specifications.

| Topic | Reference |
| --- | --- |
| `App`, `Window`, `Context<T>`, and retained entities | [Contexts][gpui-context] · [Entities][gpui-entity] |
| Actions, events, and focus restoration | [Actions][gpui-action] · [Events][gpui-event] · [Focus][gpui-focus] |
| Layout, stable IDs, and async updates | [Layout][gpui-layout] · [Element IDs][gpui-id] · [Async][gpui-async] |
| Unstyled controls and infrastructure | [GPUI Base][base] |

[gpui-context]: https://github.com/longbridge/gpui-kit/blob/e97fe438129c5e4ccab504f2e6fafd9a4e396bba/skills/gpui-kit/references/gpui/context.md
[gpui-entity]: https://github.com/longbridge/gpui-kit/blob/e97fe438129c5e4ccab504f2e6fafd9a4e396bba/skills/gpui-kit/references/gpui/entity.md
[gpui-action]: https://github.com/longbridge/gpui-kit/blob/e97fe438129c5e4ccab504f2e6fafd9a4e396bba/skills/gpui-kit/references/gpui/action.md
[gpui-event]: https://github.com/longbridge/gpui-kit/blob/e97fe438129c5e4ccab504f2e6fafd9a4e396bba/skills/gpui-kit/references/gpui/event.md
[gpui-focus]: https://github.com/longbridge/gpui-kit/blob/e97fe438129c5e4ccab504f2e6fafd9a4e396bba/skills/gpui-kit/references/gpui/focus-handle.md
[gpui-layout]: https://github.com/longbridge/gpui-kit/blob/e97fe438129c5e4ccab504f2e6fafd9a4e396bba/skills/gpui-kit/references/gpui/layout-style.md
[gpui-id]: https://github.com/longbridge/gpui-kit/blob/e97fe438129c5e4ccab504f2e6fafd9a4e396bba/skills/gpui-kit/references/gpui/element-id.md
[gpui-async]: https://github.com/longbridge/gpui-kit/blob/e97fe438129c5e4ccab504f2e6fafd9a4e396bba/skills/gpui-kit/references/gpui/async.md
[base]: https://github.com/longbridge/gpui-kit/blob/e97fe438129c5e4ccab504f2e6fafd9a4e396bba/website/base/index.md

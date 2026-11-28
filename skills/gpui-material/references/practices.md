# Development practices

Practical patterns for wiring controls, writing custom elements, optimizing rendering and frame pacing, and testing `gpui-material` applications on GPUI 0.3.14 and GPUI Base 0.11.0.

## Handling controls and state

### 1. Respect the controlled vs. retained split

- **Controlled controls** (`Switch`, `Checkbox`, `Radio`, `ToggleButton`, `Chip`, `Tabs`, `NavigationBar`, `NavigationRail`): pass the current value into the builder on every `render`, update the owner in `.on_change(...)` / `.on_select(...)`, and call `cx.notify()` **only when the value actually changed**.
- **Retained controls** (`TextFieldState`, `TextAreaState`, `SliderState`, `ExposedDropdownState`, `DialogState`, `SnackbarHost`, `DatePickerState`, `TimePickerState`): create the `Entity<*State>` once in the owning view's constructor (`cx.new(...)`) and pass `&self.state` on each `render`. Never construct a state entity inside `render`, or focus, caret, undo history, and animations will reset on every frame.
- **Avoid reference cycles in callbacks**: capture `cx.entity().downgrade()` (a `WeakEntity<Self>`) in `.on_activate` / `.on_change` / `.on_select` closures instead of cloning a strong `Entity<Self>`.
- **Never mutate app state synchronously during `render`**: defer side effects or cross-entity mutations to event handlers or `window.defer(cx, ...)`.

### 2. Focus, keyboard navigation, and overlays

- **Caller-owned focus handles**: when you need to programmatically focus a control (e.g., validation error or shortcut), create a `FocusHandle` once in the view and attach it with `.track_focus(&handle)` (supported on `Radio`, `Slider`, `Button`, etc.).
- **Roving focus**: grouped controls (`NavigationDrawer`, `Stepper`, `List`, `Menu`, `RadioGroup`, `ButtonGroup`, `Toolbar`, `FabMenu`) use `move_roving_focus` (`Home`/`End`/arrow keys). Do not layer a competing arrow-key handler on top of them.
- **Overlay focus restoration**: open and close `Dialog`, `Sheet`, and anchored `Menu` through their `DialogState` entity so Base's `PopoverState` / `DialogState` records the previous focus handle and restores it on dismiss. Note that in GPUI Base 0.11.0, controlled popovers restore focus via `window.defer`, which runs at the end of the frame.
- **Outside click blur**: `TextField` and `ExposedDropdown` automatically blur when clicking outside or when disabled, while preserving focus on built-in trailing clear/visibility controls.

### 3. Secure and validated text fields

- Use `TextField::secure(id, label, &state)` with `.obfuscation_mode(TextFieldObfuscationMode::Hidden | Visible)` or `.visibility_toggle(...)`. While `Hidden`, Base masks with `•`, disables Copy/Cut, redacts IME `text_for_range` surrounding-text queries (`gpui-base 0.11.0+`), and suppresses accessible `value`, `SetValue`, and character count.
- For pre-commit input filtering, configure Base's `validate` / `pattern` or `TextFieldMaskPattern` (`MaskPattern`) on `TextFieldState` rather than mutating the buffer inside `render`. Use `TextFieldEvent::Change` / `Blur` for post-commit normalization.

### 4. Stable IDs and large collections

- Give repeated controls semantic composite IDs such as `("item-delete", item.id)` rather than volatile list indices. This preserves hover/press state, focus, and animations across insertions, filtering, and reordering.
- Use `List` for small, finite menus or settings groups. For large collections (100+ rows), render rows inside GPUI's virtualized `list` or `uniform_list` and pair with `DisplayScrollbar`.

### 5. Scoped theming and icons

- Read `window_material_theme(window, cx)` (or `current_material_theme(window, cx)` inside components) rather than `material_theme(cx)` so per-window and `ThemeScope` subtree overrides take effect.
- Wrap callbacks launched from a `ThemeScope` (such as opening a dialog or running an async action) with `ThemeScope::bind_action` so the scoped theme stays intact.
- Prefer `Icon::new(IconName::...)` for Material Symbols—they are embedded unchanged, inherit slot font size and foreground color, and need no runtime font loading. For multi-color custom SVGs (like a brand logo), wrap `Image::from_bytes(ImageFormat::Svg, bytes)` in a `static LazyLock<Arc<Image>>` and render with `gpui::img(...)` instead of re-allocating bytes in `render`.

## Custom elements and GPUI 0.3.14 patterns

- **Blanket `IntoElement`**: GPUI 0.3.14 provides a blanket `impl<E: Element> IntoElement for E`. Do not write a manual `impl IntoElement` for custom `Element` types.
- **Opaque curve `Path` and `Shape`**: `gpui::Path` is now an opaque curve-based primitive (`PathBuilder::fill()` / `PathBuilder::stroke(width)`), and `StrokeOptions` is configured via struct literal (`StrokeOptions { line_width: 2., ..StrokeOptions::default() }`). Use `gpui_material::foundation::Shape` when you need both smooth curve painting (`paint`, `paint_stroke`) and geometric queries or clipping (`contains`, `hit_test`, `intersects_bounds`, `clip_rect`, `clip_disc`, `clip_convex_polygon`, `clip_shape`, `exterior_mask`).
- **`WindowBounds::centered`**: takes an explicit `Option<DisplayId>` as its first argument (`WindowBounds::centered(None, size, cx)`).

## Rendering performance and frame pacing

### 1. Optimize rendering crates in dev builds, benchmark in Release

Unoptimized Rust builds make single-threaded UI draw and DirectX/Metal/WGPU scene submission exceed an 8.33 ms (120Hz) or 3.33 ms (300Hz) frame budget while showing low overall CPU usage (1 saturated core on a multi-core CPU) and low GPU usage.

- In `Cargo.toml`, optimize the rendering, layout, and text crates under `[profile.dev.package]`:

```toml
[profile.dev]
debug = 0

[profile.dev.package]
gpui-base = { opt-level = 3 }
gpui-pre = { opt-level = 3 }
gpui-pre-linux = { opt-level = 3 }
gpui-pre-macos = { opt-level = 3 }
gpui-pre-platform = { opt-level = 3 }
gpui-pre-sum-tree = { opt-level = 3 }
gpui-pre-wgpu = { opt-level = 3 }
gpui-pre-windows = { opt-level = 3 }
lyon_geom = { opt-level = 3 }
lyon_path = { opt-level = 3 }
resvg = { opt-level = 3 }
ropey = { opt-level = 3 }
rustybuzz = { opt-level = 3 }
smol = { opt-level = 3 }
taffy = { opt-level = 3 }
tiny-skia = { opt-level = 3 }
ttf-parser = { opt-level = 3 }
```

- Always evaluate 120Hz+ smoothness and run performance benchmarks in `--release` mode (`cargo run --locked --release`).

### 2. Avoid idle and off-screen animation loops

- Never call `window.request_animation_frame()` unconditionally inside `render` or `paint`.
- Use Foundation's unified motion helpers (`animate` / `animate_with_status`, `RetargetTransition`, `LoopClock`) and guard continuous loops with `is_bounds_visible(bounds, window)`, `window.is_window_active()`, and `!cx.reduce_motion()`.
- Off-screen, clipped, hidden, inactive, or settled animations must schedule **0** idle frames.

### 3. Shrink per-frame invalidation and rebuild scope

- **Isolate hot subtrees into separate `Entity<V>` views**: calling `cx.notify()` on the root shell rebuilds navigation chrome, sidebars, and status bars on every animation tick. Keep animated previews, editors, and frequently updated panels in their own child `Entity` views.
- **Skip no-op notifications**: compare new state against current state before calling `cx.notify()`.
- **Avoid per-frame allocations in `render`**: cache static images (`LazyLock<Arc<Image>>`), reuse `ShapePolygon` / `ShapeMorph` across frames, and avoid formatting unchanged strings or rebuilding large option lists on every draw.

### 4. Windows 11 high-refresh-rate notes (`gpui-pre-windows`)

- On Windows, GPUI renders via D3D11 + DirectComposition (`DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL`, `Present(0, 0)`) and paces frames from a `VSyncProvider` thread calling `DwmFlush()` → `RedrawWindow(RDW_INVALIDATE)` → `WM_PAINT`.
- If a Release build shows low CPU draw (< 2 ms) yet stays below 120 FPS on a high-refresh monitor:
  - Check Windows **Dynamic Refresh Rate (DRR)** and **Battery Saver** (`PowerMode::LowPower` activates GPUI's `low_power_frame_interval` throttle).
  - Check whether a fullscreen/borderless game or GPU driver profile is locking DWM composition for non-DirectFlip desktop windows to 60Hz.
  - Note that Win32 dispatches `WM_PAINT` at lower priority than queued input messages (`WM_MOUSEMOVE` / `WM_MOUSEWHEEL`), so 1000Hz mice can transiently delay paints during continuous motion.
- Measure actual CPU draw vs. frame submission interval using `--frame-profile` or `perf_bench`:

```powershell
cargo run --locked --release -p gpui-material-gallery --features frame-profile -- button --frame-profile
cargo test --locked --release -p gpui-material-gallery --features test-support --lib perf_bench -- --ignored --nocapture --test-threads=1
```

## Testing and verification

- **Layered test placement**: place tests at the lowest effective layer—pure Rust `#[test]` in `crates/`, single-binary public-facade UI integration tests in `crates/material/tests/integration/` (`cargo test --locked -p gpui-material --features test-support --test integration`), and Gallery headless `#[gpui::test]` suites in `apps/gallery/src/` (`cargo test --locked -p gpui-material-gallery --features test-support`). Cover keyboard, pointer, focus, disabled/error, cancellation, RTL, and reduced motion.
- **Use `gpui_material::test` (`test-support` feature)**: enable `test-support` under `[dev-dependencies]` and drive UI tests with `TestWindowExt` (`find`, `try_find`, `find_all`, `within`, `render_frame`, `click`, `click_at`, `click_with_options`, `hover`, `scroll`, `drag_to`, `press`, `input`), `TestQueryExt` (`elements`, `texts`, `find_by_label`, `find_all_by_role`), `TestEventExt` (stepwise pointer/key/modifier gestures), `TestInput` (UTF-16 composition bridge), and `TestAppContextExt::wait_for` (bounded test-clock polling outside `update_window`). Register `.test_support()` before `.track_focus(&handle)` so `focused()` diagnoses missed bindings, re-query immutable `ElementSnapshot` values after interactions, and never treat `disabled() == None` as proof a control is enabled.
- **GPUI 0.3.14 quad splitting**: `Window::insert_quad` splits rounded quads into an interior rectangle (`corner_radii = 0`) plus 4 border/corner strips. Use `gpui_material::test::painted_quads(window)` (re-exported in Gallery tests as `crate::painted_quads(window)`) to coalesce split quads when asserting `corner_radii` or quad counts.
- **Flushing `window.defer` in tests**: controlled popovers and deferred focus moves execute at the end of `cx.update_window(...)`. Split open → interact → assert into separate `cx.update_window` calls (or wait with `cx.wait_for(...)` outside `update_window`) rather than chaining them inside one closure.
- **Offscreen pixel rendering (`--test rendering`)**: `painted_quads()` cannot verify embedded Material Symbols SVGs, GPUI `Path` shapes/checkmarks/progress arcs, Expressive polygon `Surface` clipping, light/dark/high-contrast tonal surfaces, text visibility, focus-ring contrast, or RTL layout/icon mirroring. Run `cargo test --locked -p gpui-material-gallery --features desktop,test-support --test rendering` (`test = false`, `harness = false`), which runs real DirectX hidden-window readback (Windows, with WARP fallback), Metal (macOS), and WGPU/Mesa lavapipe (Linux) offscreen checks.
- **Live native verification**: when touching OS bridges, IME, DPI, window chrome, or accessibility trees, verify the built native binary directly (for example via Windows UI Automation / `agent-desktop`).

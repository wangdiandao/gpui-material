# Cookbook

English | [中文](cookbook.zh.md)

This cookbook collects practical patterns for building desktop applications with `gpui-material`. For dependency pinning, Cargo setup, Material Symbols, and upgrade notes, see [Downstream integration](integration.md). For the full list of supported controls and variants, see [Components](component.md).

<a id="应用初始化与状态所有权"></a>

## Application setup and state ownership

Call `gpui_material::init(theme, cx)` before opening windows (which installs `MaterialRoot` with your initial `MaterialTheme` and registers component key bindings), and open windows through `MaterialRoot::open_window` so GPUI Base owns overlay layers and focus traversal.

Split state by ownership:

| State kind | Owner | Examples |
| --- | --- | --- |
| Business values and selections | Application view or model | `bool` for `Switch` / `ToggleButton`, `CheckboxState` for `Checkbox`, `ElementId` / `SharedString` for `RadioGroup`, `ChipGroup`, and `NavigationItems` |
| Retained control entities | Created once in the owning view (`cx.new(...)`), passed by reference on every `render` | `TextFieldState`, `TextAreaState`, `ExposedDropdownState`, `SliderState`, `DialogState`, `SnackbarHost`, `CarouselState`, `SearchState` |

```rust
use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, QuitMode, Render, Styled,
    Subscription, Window, WindowOptions, div,
};
use gpui_material::{
    Button, MaterialRoot, MaterialTheme, Slider, SliderEvent, SliderState, Snackbar, SnackbarHost,
    Switch, TextField, TextFieldState,
};

struct SettingsApp {
    display_name: Entity<TextFieldState>,
    volume: Entity<SliderState>,
    snackbar: Entity<SnackbarHost>,
    reminders: bool,
    _subscriptions: [Subscription; 2],
}

impl SettingsApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let display_name = cx.new(|cx| TextFieldState::new(window, cx).placeholder("Display name"));
        let volume = cx.new(|_| SliderState::new().min(0.).max(100.).step(5.).default_value(60.));
        let snackbar = cx.new(|_| SnackbarHost::default());

        let name_sub = cx.observe(&display_name, |_, _, cx| cx.notify());
        let volume_sub = cx.subscribe(&volume, |_, _, event: &SliderEvent, cx| {
            if matches!(event, SliderEvent::Change(_) | SliderEvent::Release(_)) {
                cx.notify();
            }
        });

        Self {
            display_name,
            volume,
            snackbar,
            reminders: true,
            _subscriptions: [name_sub, volume_sub],
        }
    }
}

impl Render for SettingsApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity().downgrade();
        let save_owner = owner.clone();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .child(TextField::new("display-name", "Display name", &self.display_name))
            .child(
                Switch::new("reminders", "Enable reminders", self.reminders).on_change(
                    move |checked, _, cx| {
                        let _ = owner.update(cx, |this, cx| {
                            this.reminders = checked;
                            cx.notify();
                        });
                    },
                ),
            )
            .child(Slider::new("volume", "Volume", &self.volume).tick_intervals(10))
            .child(
                Button::filled("save", "Save").on_activate(move |window, cx| {
                    let _ = save_owner.update(cx, |this, cx| {
                        let name = this.display_name.read(cx).value();
                        let level = this.volume.read(cx).value().end();
                        this.snackbar.update(cx, |host, cx| {
                            host.push(
                                Snackbar::new(format!("Saved {name} (volume {level:.0}%)")),
                                window,
                                cx,
                            );
                        });
                    });
                }),
            )
            .child(self.snackbar.clone())
    }
}

fn main() {
    gpui_platform::application()
        .with_quit_mode(QuitMode::LastWindowClosed)
        .run(|cx| {
            gpui_material::init(MaterialTheme::default(), cx);
            MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| SettingsApp::new(window, cx))
            })
            .expect("open Material window");
        });
}
```

Use stable, semantic element IDs (`"display-name"`, `("row", id)`) for repeated controls and list items so focus, ripples, and animations follow the item across reorders.

<a id="表单与校验"></a>

## Forms and validation

Compose `TextField`, `ExposedDropdown`, `Checkbox`, `RadioGroup`, `Switch`, and `ChipGroup` with application-owned validation state:

- Show validation errors on `TextField` with `.error(message)` (and helper text with `.supporting_text(text)`). For pre-commit input filtering, attach `.validate(|text, _| ...)` on `TextFieldState`.
- Disable or lock a `TextField` on its retained `TextFieldState` (`state.set_disabled(true, cx)` / `state.set_readonly(true, cx)`). Other controls (`Button`, `Checkbox`, `Radio`, `Switch`, `Chip`, `ChipGroup`) accept `.disabled(bool)` directly on the element.
- Style or validate an `ExposedDropdown`'s underlying field via `.editor(|field| field.variant(TextFieldVariant::Outlined).error(...))`.
- Use `Button::filled(...).loading(self.submitting)` on submission to show an inline Material spinner, suppress duplicate activation, and preserve button width without layout jitter.

```rust
use gpui::{
    AppContext, Axis, Context, ElementId, Entity, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, px,
};
use gpui_material::{
    Button, Checkbox, CheckboxState, ChipGroup, ChipGroupItem, ChipGroupSelection, DropdownOption,
    ExposedDropdown, ExposedDropdownState, Radio, RadioGroup, SemanticTone, Switch, TextField,
    TextFieldState, TextFieldVariant,
};

struct SignupForm {
    email: Entity<TextFieldState>,
    region: Entity<ExposedDropdownState>,
    selected_region: Option<ElementId>,
    plan: ElementId,
    topics: Vec<SharedString>,
    accept_terms: CheckboxState,
    marketing: bool,
    submitted: bool,
    submitting: bool,
}

impl SignupForm {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let email = cx.new(|cx| TextFieldState::new(window, cx).placeholder("name@example.com"));
        let region = cx.new(|cx| ExposedDropdownState::new("", true, window, cx));
        cx.observe(&email, |_, _, cx| cx.notify()).detach();
        cx.observe(&region, |_, _, cx| cx.notify()).detach();
        Self {
            email,
            region,
            selected_region: None,
            plan: "pro".into(),
            topics: vec!["releases".into()],
            accept_terms: CheckboxState::Unchecked,
            marketing: false,
            submitted: false,
            submitting: false,
        }
    }

    fn email_error(&self, cx: &Context<Self>) -> Option<&'static str> {
        let value = self.email.read(cx).value();
        if self.submitted && (!value.contains('@') || value.ends_with('@')) {
            Some("Enter a valid email address")
        } else {
            None
        }
    }
}

impl Render for SignupForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity().downgrade();
        let region_owner = owner.clone();
        let plan_owner = owner.clone();
        let topics_owner = owner.clone();
        let terms_owner = owner.clone();
        let marketing_owner = owner.clone();
        let submit_owner = owner.clone();

        let email_error = self.email_error(cx);
        let region_missing = self.submitted && self.selected_region.is_none();
        let terms_missing = self.submitted && self.accept_terms != CheckboxState::Checked;

        let mut email_field = TextField::new("email", "Work email", &self.email)
            .variant(TextFieldVariant::Outlined)
            .supporting_text("Used for account recovery");
        if let Some(err) = email_error {
            email_field = email_field.error(err);
        }

        div()
            .flex()
            .flex_col()
            .gap_4()
            .max_w(px(440.))
            .child(email_field)
            .child(
                ExposedDropdown::new(
                    "region",
                    "Region",
                    &self.region,
                    [
                        DropdownOption::new("us", "United States"),
                        DropdownOption::new("eu", "Europe"),
                        DropdownOption::new("ap", "Asia-Pacific"),
                    ],
                )
                .editor(move |field| {
                    let field = field.variant(TextFieldVariant::Outlined);
                    if region_missing {
                        field.error("Select a region")
                    } else {
                        field
                    }
                })
                .on_select(move |option, _, cx| {
                    let _ = region_owner.update(cx, |this, cx| {
                        this.selected_region = Some(option.id.clone());
                        cx.notify();
                    });
                }),
            )
            .child(
                RadioGroup::new("plan", "Plan")
                    .axis(Axis::Horizontal)
                    .selected(self.plan.clone())
                    .options([
                        Radio::new("starter", "Starter"),
                        Radio::new("pro", "Pro"),
                        Radio::new("enterprise", "Enterprise").disabled(self.submitting),
                    ])
                    .on_select(move |id, _, cx| {
                        let _ = plan_owner.update(cx, |this, cx| {
                            this.plan = id.clone();
                            cx.notify();
                        });
                    }),
            )
            .child(
                ChipGroup::new(
                    "topics",
                    "Topics",
                    [
                        ChipGroupItem::new("releases", "Releases"),
                        ChipGroupItem::new("security", "Security").tone(SemanticTone::Warning),
                        ChipGroupItem::new("guides", "Guides"),
                    ],
                )
                .disabled(self.submitting)
                .selection(ChipGroupSelection::Multiple, self.topics.clone())
                .on_change(move |selected, _, cx| {
                    let _ = topics_owner.update(cx, |this, cx| {
                        this.topics = selected;
                        cx.notify();
                    });
                }),
            )
            .child(
                Switch::new("marketing", "Product announcements", self.marketing)
                    .disabled(self.submitting)
                    .on_change(move |checked, _, cx| {
                        let _ = marketing_owner.update(cx, |this, cx| {
                            this.marketing = checked;
                            cx.notify();
                        });
                    }),
            )
            .child(
                Checkbox::new("terms", "I accept the terms of service", self.accept_terms)
                    .error(terms_missing)
                    .disabled(self.submitting)
                    .on_change(move |next, _, cx| {
                        let _ = terms_owner.update(cx, |this, cx| {
                            this.accept_terms = next;
                            cx.notify();
                        });
                    }),
            )
            .child(
                Button::filled("submit", "Create account")
                    .loading(self.submitting)
                    .on_activate(move |_, cx| {
                        let _ = submit_owner.update(cx, |this, cx| {
                            this.submitted = true;
                            let valid = this.email_error(cx).is_none()
                                && this.selected_region.is_some()
                                && this.accept_terms == CheckboxState::Checked;
                            if valid {
                                this.submitting = true;
                                this.email.update(cx, |state, cx| state.set_disabled(true, cx));
                            }
                            cx.notify();
                        });
                    }),
            )
    }
}
```

<a id="主题深色模式动态配色与局部覆盖"></a>

## Theming, dark mode, dynamic color, and local overrides

`MaterialTheme` holds colors, typography, motion scheme, and layout policy (direction and density). See [Pure theme](theme/theme.md) for defaults and token details.

### Switching light, dark, and dynamic themes

Call `set_material_theme(theme, cx)` to update the application theme. Colors animate with the destination motion scheme's Slow Effects spring (or apply immediately under reduced motion) and project into `gpui_base::Theme::global`.

```rust
use gpui::App;
use gpui_material::{
    MaterialTheme, set_material_theme,
    theme::{Appearance, Density, MotionScheme},
};

fn switch_to_dark_baseline(cx: &mut App) {
    let theme = MaterialTheme::baseline(Appearance::Dark)
        .with_motion(MotionScheme::Expressive)
        .with_density(Density::Compact);
    set_material_theme(theme, cx);
}
```

With the `dynamic-color` Cargo feature enabled, derive full Material 3 palettes from a seed color, select a `ThemePreset`, or compute harmonized custom status groups:

```rust
use gpui::App;
use gpui_material::{
    MaterialTheme, set_material_theme,
    theme::{Appearance, Argb, DynamicVariant, SpecVersion, ThemePreset},
};

fn apply_dynamic_theme(appearance: Appearance, cx: &mut App) {
    // Tonal Spot Spec2021 from a brand seed
    let seeded = MaterialTheme::from_seed(Argb::from_u32(0xff00_6c9c), appearance, 0.0);

    // Expressive Spec2025 with higher contrast
    let expressive = MaterialTheme::from_seed_with(
        Argb::from_u32(0xff67_50a4),
        appearance,
        0.5,
        DynamicVariant::Expressive,
        SpecVersion::Spec2025,
    );

    // Or pick one of the 18 bundled presets
    let preset = ThemePreset::Ocean.theme(appearance);

    let _custom_brand = preset.colors.custom_color(Argb::from_u32(0xffd3_5400), true);
    let _ = (seeded, preset);
    set_material_theme(expressive, cx);
}
```

### Semantic tones

Use `SemanticTone` (`Default`, `Danger`, `Success`, `Warning`, `Info`) to apply Material 3 semantic color pairs across `Button`, `ToggleButton`, `IconButton`, `Chip`, `ChipGroupItem`, and `Snackbar` without hand-picking individual color roles:

```rust
use gpui_material::{Button, SemanticTone};

let delete_button = Button::filled("delete", "Delete workspace")
    .tone(SemanticTone::Danger)
    .on_activate(|_, _| {});
```

### Per-window and subtree overrides

Override one window without changing the rest of the application using `foundation::set_window_material_theme(Some(theme), window, cx)` (pass `None` to inherit the application theme again).

Override a subtree—including its theme, text direction (`Direction::Rtl`), and density—with `ThemeScope`:

```rust
use gpui::{IntoElement, ParentElement, div};
use gpui_material::{
    Button, MaterialTheme, ThemeScope,
    theme::{Appearance, Density, Direction},
};

fn rtl_preview_panel() -> impl IntoElement {
    ThemeScope::new(
        div()
            .flex()
            .gap_2()
            .child(Button::filled("apply", "Apply"))
            .child(Button::outlined("cancel", "Cancel")),
    )
    .theme(MaterialTheme::baseline(Appearance::Dark))
    .direction(Direction::Rtl)
    .density(Density::Compact)
}
```

`ThemeScope` does not mutate `gpui_base::Theme::global`. When a scoped trigger opens a deferred overlay or spawns a child native window, wrap deferred content with `ThemeScope::inherit(child, window, cx)` or wrap the callback with `ThemeScope::bind_action(action, window, cx)` so the active scope carries across.

<a id="布局自适应导航与浮层"></a>

## Layouts, adaptive navigation, and overlays

### Scaffold and adaptive navigation

Combine `Scaffold`, `NavigationItems`, and `AdaptiveNavigation` to switch automatically between `NavigationBar` (Compact) and `NavigationRail` (Medium and larger) based on available width. Mount `ListDetail` or `SupportingPane` inside the scaffold body for Material canonical multi-pane layouts, and wrap each scrollable pane in `foundation::isolated_scroll` so wheel input stays inside that pane.

```rust
use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Pixels, Render, ScrollHandle,
    SharedString, Styled, Window, div, px,
};
use gpui_material::{
    AdaptiveNavigation, AdaptiveNavigationState, AdaptiveNavigationType, AppBar, Fab, Icon,
    IconName, List, ListDetail, ListDetailPane, ListItem, NavigationItem, NavigationItems,
    Scaffold, Snackbar, SnackbarHost, SupportingPane, Text,
    foundation::isolated_scroll,
    theme::TypeRole,
};

struct WorkspaceShell {
    nav_state: Entity<AdaptiveNavigationState>,
    snackbar: Entity<SnackbarHost>,
    selected_tab: SharedString,
    selected_doc: usize,
    active_pane: ListDetailPane,
    show_support: bool,
    list_scroll: ScrollHandle,
    detail_scroll: ScrollHandle,
    support_scroll: ScrollHandle,
}

impl WorkspaceShell {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            nav_state: cx.new(|_| AdaptiveNavigationState::default()),
            snackbar: cx.new(|_| SnackbarHost::default()),
            selected_tab: "inbox".into(),
            selected_doc: 0,
            active_pane: ListDetailPane::List,
            show_support: false,
            list_scroll: ScrollHandle::new(),
            detail_scroll: ScrollHandle::new(),
            support_scroll: ScrollHandle::new(),
        }
    }

    fn render_shell(&self, width: Pixels, cx: &mut Context<Self>) -> impl IntoElement {
        let items = NavigationItems::new(
            "main-nav",
            "Primary navigation",
            [
                NavigationItem::new("inbox", "Inbox").icon(Icon::new(IconName::Inbox)),
                NavigationItem::new("activity", "Activity").icon(Icon::new(IconName::Notifications)),
            ],
        )
        .selected(self.selected_tab.clone())
        .on_select(cx.listener(|this, id: &SharedString, _, cx| {
            this.selected_tab = id.clone();
            cx.notify();
        }));

        let owner = cx.entity().downgrade();
        let body = if self.selected_tab.as_ref() == "inbox" {
            let list = List::new("docs").items((0..8).map(|ix| {
                let owner = owner.clone();
                ListItem::new(("doc", ix), format!("Document {}", ix + 1))
                    .selected(ix == self.selected_doc)
                    .on_activate(move |_, cx| {
                        let _ = owner.update(cx, |this, cx| {
                            this.selected_doc = ix;
                            this.active_pane = ListDetailPane::Detail;
                            cx.notify();
                        });
                    })
            }));
            ListDetail::new(
                "inbox-list-detail",
                width,
                isolated_scroll("list-scroll", &self.list_scroll).flex_1().p_4().child(list),
                isolated_scroll("detail-scroll", &self.detail_scroll)
                    .flex_1()
                    .p_4()
                    .child(Text::new(format!("Document {}", self.selected_doc + 1)).role(TypeRole::HeadlineSmall)),
            )
            .active_pane(self.active_pane)
            .into_any_element()
        } else {
            SupportingPane::new(
                "activity-supporting",
                width,
                isolated_scroll("main-scroll", &self.detail_scroll).flex_1().p_4().child("Recent activity"),
                isolated_scroll("support-scroll", &self.support_scroll).flex_1().p_4().child("Inspector"),
            )
            .show_support(self.show_support)
            .into_any_element()
        };

        let snackbar = self.snackbar.clone();
        let scaffold = Scaffold::new("workspace-scaffold", body)
            .app_bar(AppBar::new("workspace-bar", "Workspace"))
            .fab(
                Fab::new("new-doc", "Create document", Icon::new(IconName::Add)).on_activate(
                    move |window, cx| {
                        snackbar.update(cx, |host, cx| {
                            host.push(
                                Snackbar::new("Draft created").action("Undo", |_, _| {}),
                                window,
                                cx,
                            );
                        });
                    },
                ),
            )
            .snackbar(self.snackbar.clone());

        AdaptiveNavigation::new(scaffold, items, width)
            .navigation_type(AdaptiveNavigationType::for_width(width))
            .state(&self.nav_state)
    }
}
```

For fixed navigation chrome without automatic width switching, mount `NavigationBar::new(items)` or `NavigationRail` directly. Check `ListDetail::is_single_pane(width)` to show a back button in `AppBar` when the window narrows to one pane.

### Dialogs, sheets, menus, and snackbars

- **Dialog & Sheet**: Retain an `Entity<DialogState>` per modal. Open and close it with `state.update(cx, |s, cx| s.open(window, cx))` / `s.close(window, cx)` so focus trapping and focus restoration stay synchronized.
- **Menu**: Pass an anchor trigger and `MenuItem` list to `Menu::new(id, trigger, items)`. Use `.open(bool)` and `.on_open_change(...)` for controlled menus, `.submenu([...])` for nested menus, and `.context()` for right-click context menus.
- **Snackbar**: Mount a retained `Entity<SnackbarHost>` (directly or via `Scaffold::snackbar(...)`) and call `host.push(Snackbar::new(...), window, cx)`. Adding `.action(label, callback)` makes the snackbar persist until acted on or dismissed unless `.timeout(Some(...))` is called afterward.

```rust
use gpui::{AppContext, Context, Entity, IntoElement, ParentElement, Window, div, px};
use gpui_material::{
    Button, Dialog, DialogProperties, DialogState, Menu, MenuItem, Sheet, SheetEdge, SheetMode,
};

fn render_overlays(
    confirm_dialog: &Entity<DialogState>,
    details_sheet: &Entity<DialogState>,
    menu_open: bool,
    cx: &Context<impl 'static>,
) -> impl IntoElement {
    let dialog_trigger = confirm_dialog.clone();
    let sheet_trigger = details_sheet.clone();
    let _ = cx;

    div()
        .flex()
        .gap_2()
        .child(Button::outlined("open-dialog", "Archive…").on_activate(move |window, cx| {
            dialog_trigger.update(cx, |state, cx| state.open(window, cx));
        }))
        .child(Button::outlined("open-sheet", "Details…").on_activate(move |window, cx| {
            sheet_trigger.update(cx, |state, cx| state.open(window, cx));
        }))
        .child(
            Menu::new(
                "item-menu",
                Button::text("menu-btn", "More"),
                [
                    MenuItem::new("duplicate", "Duplicate").on_activate(|_, _| {}),
                    MenuItem::new("export", "Export").submenu([
                        MenuItem::new("csv", "CSV").on_activate(|_, _| {}),
                        MenuItem::new("json", "JSON").on_activate(|_, _| {}),
                    ]),
                ],
            )
            .open(menu_open),
        )
        .child(
            Dialog::new("archive-dialog", confirm_dialog, "Archive project?")
                .description("Archived projects move to read-only storage.")
                .properties(DialogProperties {
                    dismiss_on_outside_click: false,
                    ..DialogProperties::default()
                })
                .confirm("Archive", |_, _| true),
        )
        .child(
            Sheet::new("details-sheet", details_sheet, "Project details", "Visible to all members.")
                .edge(SheetEdge::End)
                .mode(SheetMode::Modal)
                .extent(px(360.)),
        )
}
```

<a id="焦点键盘导航交互观测与无障碍"></a>

## Focus, keyboard navigation, interaction observation, and accessibility

### Focus handles and roving focus

Attach caller-owned `FocusHandle` values with `.track_focus(&self.handle)` on interactive controls (`Button`, `Checkbox`, `Radio`, `Switch`, `Chip`, `Surface`, and `Slider::track_focus(RangeThumb::End, &focus)`) to restore focus after removing an item or closing a view.

- On removable chips, pass the restoration handle to `Chip::on_remove(label, Some(self.restore.clone()), handler)` so focus moves predictably when the chip is deleted via click, `Backspace`, or `Delete`.
- In custom virtualized lists, call `ListItem::track_focus(&row_focus).tab_stop(is_active_tab_stop)` to supply retained row focus and a single roving Tab stop while your collection owns full-model arrow navigation and scroll-into-view.

### Observing interaction state with `InteractionSource`

Retain an `Entity<InteractionSource>` and attach it to one or more controls (each with a distinct element ID in the window) using `.interaction_source(&self.interactions)`. Read the latest `InteractionState` (`hovered`, `pressed`, `focus_visible`, `dragged`, `disabled`) during `render` via `source.read(cx).state(window_id, &id.into())`, or subscribe to deferred `InteractionUpdate` events:

```rust
use gpui::{AppContext, Context, Entity, IntoElement, Render, Subscription, Window};
use gpui_material::{Button, InteractionSource, InteractionUpdate};

struct ObservedAction {
    interactions: Entity<InteractionSource>,
    status: &'static str,
    _sub: Subscription,
}

impl ObservedAction {
    fn new(cx: &mut Context<Self>) -> Self {
        let interactions = cx.new(|_| InteractionSource::new());
        let sub = cx.subscribe(&interactions, |this, _, update: &InteractionUpdate, cx| {
            this.status = match update.state {
                Some(state) if state.pressed => "Pressed",
                Some(state) if state.focus_visible => "Keyboard focus",
                Some(state) if state.hovered => "Hovered",
                _ => "Idle",
            };
            cx.notify();
        });
        Self {
            interactions,
            status: "Idle",
            _sub: sub,
        }
    }
}

impl Render for ObservedAction {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Button::filled("observed-save", self.status)
            .interaction_source(&self.interactions)
            .on_activate(|_, _| {})
    }
}
```

### RTL mirroring and reduced motion

- **RTL**: Set `Direction::Rtl` on `MaterialTheme::with_direction(Direction::Rtl)` or a local `ThemeScope::new(child).direction(Direction::Rtl)`. Material controls mirror layout, padding, and horizontal keyboard navigation automatically. Opt directional icons into mirroring with `Icon::new(IconName::ArrowForward).mirror_in_rtl(true)`, and mirror custom overlay anchors with `foundation::logical_anchor(anchor, theme.layout.direction)`.
- **Reduced motion**: Query the OS preference from GPUI with `cx.reduce_motion()`. Passing it to `theme.motion.spec(MotionKind::Spatial, MotionSpeed::Default, cx.reduce_motion())` returns `MotionSpec::Instant`, and `foundation::animate_with_status` / `LoopClock` settle immediately without scheduling motion frames.

<a id="构建自定义-material-控件"></a>

## Building custom Material controls

Use `gpui_material::foundation` when you need a bespoke control that still honors Material themes, state layers, ripples, focus rings, polygon shapes, and scoped overrides:

- `Surface::clickable(id, label, role)`, `Surface::selectable(id, label, role, selected)`, and `Surface::toggleable(id, label, role, checked)` provide themed background and foreground pairs, state layers, press ripples, keyboard focus rings, accessibility roles, and `.on_activate` / `.on_checked_change`.
- Apply rounded corners with `.shape(CornerRole::Medium.shape())` or any of the 35 Expressive polygons with `.polygon_shape(MaterialShape::Cookie7Sided)` (with shape-aware pointer hit testing enabled by default).
- Call `foundation::current_material_theme(window, cx)` to resolve the nearest `ThemeScope`, window override, or application theme, and `foundation::color(theme.colors.color(role))` to convert `Argb` to `gpui::Hsla`.
- Call `foundation::current_base_theme(window, cx)` when styling standalone `gpui_base` widgets inside a `ThemeScope` or window override so they match the local scope without mutating `gpui_base::Theme::global`.
- Use `foundation::Shape` (`Shape::rounded` / `Shape::polygon`) for custom canvas drawing and `foundation::Scrim::new(id).opacity(0.32).on_click(...)` for custom modal backdrops.

```rust
use gpui::{
    App, ElementId, FocusHandle, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, px,
};
use gpui_material::{
    Icon, IconName, MaterialShape, Scrim, Surface, SurfaceBorder, SurfaceFeedback, Text,
    foundation::{color, current_base_theme, current_material_theme},
    theme::{ColorRole, CornerRole, Elevation, TypeRole},
};

#[derive(IntoElement)]
struct FilterTile {
    id: ElementId,
    label: SharedString,
    checked: bool,
    focus: FocusHandle,
}

impl RenderOnce for FilterTile {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let _base_theme = current_base_theme(window, cx);
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let role = if self.checked {
            ColorRole::SecondaryContainer
        } else {
            ColorRole::SurfaceContainerLow
        };

        Surface::toggleable(self.id, self.label.clone(), role, self.checked)
            .shape(CornerRole::Medium.shape())
            .border(SurfaceBorder {
                width: px(1.),
                color: outline,
            })
            .tonal_elevation(Elevation::Level1)
            .feedback(SurfaceFeedback::default())
            .track_focus(&self.focus)
            .px_4()
            .py_3()
            .flex()
            .items_center()
            .gap_3()
            .child(
                Surface::new(ColorRole::PrimaryContainer)
                    .polygon_shape(MaterialShape::Cookie7Sided)
                    .size(px(32.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(IconName::Check).size(px(18.))),
            )
            .child(Text::new(self.label).role(TypeRole::LabelLarge))
    }
}

fn custom_backdrop(on_dismiss: impl Fn(&mut Window, &mut App) + 'static) -> impl IntoElement {
    Scrim::new("custom-scrim")
        .opacity(0.32)
        .accessibility_label("Close panel")
        .on_click(on_dismiss)
}
```

<a id="feature-开关与精简构建"></a>

## Feature flags and slim builds

By default, `gpui-material` enables `full`, which compiles all six core component groups (`actions`, `selection`, `inputs`, `surfaces`, `feedback`, and `navigation`). To reduce compile time and binary size in downstream applications, disable default features and enable only the groups and optional families you use:

```toml
[dependencies]
gpui = { package = "gpui-pre", version = "=0.3.14" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.14", features = ["font-kit", "x11", "wayland", "runtime_shaders"] }
gpui-material = { version = "0.4.1", default-features = false, features = [
    "actions",
    "inputs",
    "surfaces",
    "feedback",
    "dynamic-color",
    "window-chrome",
] }
```

Each optional feature automatically activates only the core component groups it depends on:

| Feature | Kind | What it enables |
| --- | --- | --- |
| `full` (default) | Core bundle | All six core component groups below |
| `actions` | Core group | `Button`, `ToggleButton`, `IconButton`, `IconToggleButton`, `Fab`, `ExtendedFab` |
| `selection` | Core group | `Checkbox`, `Radio`, `RadioGroup`, `Switch`, `Chip`, `ChipGroup`, `Slider`, `SliderState` |
| `inputs` | Core group | `TextField`, `TextFieldState`, `TextAreaState`, `ExposedDropdown`, `ExposedDropdownState` |
| `surfaces` | Core group | `Card`, `List`, `ListItem`, `VerticalDragHandle`, `DropTarget`, `SwipeToDismiss`, settings tiles (`CheckboxTile`, `RadioTile`, `SliderTile`, `SwitchTile`, `ChoiceTile`, `ListSection`) |
| `feedback` | Core group | `Banner`, `Dialog`, `Sheet`, `Menu`, `Snackbar`, `SnackbarHost`, `Tooltip`, `Label`, `LinearProgress`, `CircularProgress`, `LoadingIndicator`, `PullToRefresh` |
| `navigation` | Core group | `AppBar`, `BottomAppBar`, `Tabs`, `NavigationBar`, `NavigationRail`, `NavigationDrawer`, `Scaffold`, `AdaptiveNavigation`, `ListDetail`, `SupportingPane`, `Feed`, `Breadcrumbs`, `Stepper` |
| `window-chrome` | Optional | Cross-platform custom title bar under `gpui_material::window_chrome` |
| `date-time` | Optional | `DatePicker`, `DatePickerState`, `TimePicker`, `TimePickerState` under `gpui_material::date_time` |
| `search` | Optional | `SearchBar`, `SearchView`, `SearchState`, `SearchResult`, `SearchEvent` |
| `toolbars` | Optional | `Toolbar`, `FabMenu` and related toolbar types |
| `carousel` | Optional | `Carousel`, `CarouselItem`, `CarouselState` |
| `expressive-groups` | Optional | `ButtonGroup`, `SplitButton` |
| `dynamic-color` | Theme | `MaterialTheme::from_seed`, `from_seed_with`, `ThemePreset`, `CustomColorGroup`, `harmonize_color` |
| `serde` | Theme | `Serialize` / `Deserialize` on `MaterialTheme` and theme value types |

Display primitives (`Text`, `Icon`, `Badge`, `Divider`, `Avatar`, `EmptyView`, `Skeleton`, `DisplayScrollbar`, `SemanticTone`), `foundation`, and `theme` are always available regardless of component group features.

[Downstream integration](integration.md) · [Components](component.md) · [Pure theme](theme/theme.md) · [Architecture](architecture.md)

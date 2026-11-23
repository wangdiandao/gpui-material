# 实战手册

[English](cookbook.md) | 中文

本手册汇总使用 `gpui-material` 构建桌面应用的常见实战模式。依赖版本固定、Cargo 配置、Material Symbols 图标加载与版本迁移说明见[下游集成](integration.zh.md)；全部受支持控件及变体见[组件](component.zh.md)。

<a id="application-setup-and-state-ownership"></a>

## 应用初始化与状态所有权

在打开窗口前，使用初始 `MaterialTheme` 调用 `MaterialRoot::init`，再调用 `gpui_material::init_components(cx)` 注册组件快捷键，并通过 `MaterialRoot::open_window` 创建窗口，以便 GPUI Base 统一管理浮层与焦点遍历。

按所有权拆分状态：

| 状态类型 | 所有者 | 示例 |
| --- | --- | --- |
| 业务值与选中状态 | 应用视图或数据模型 | `Switch` / `ToggleButton` 的 `bool`、`Checkbox` 的 `CheckboxState`、`RadioGroup` / `ChipGroup` / `NavigationItems` 的 `ElementId` 或 `SharedString` |
| 持久控件实体 | 在所属视图中创建一次（`cx.new(...)`），每次 `render` 按引用传入 | `TextFieldState`、`TextAreaState`、`ExposedDropdownState`、`SliderState`、`DialogState`、`SnackbarHost`、`CarouselState`、`SearchState` |

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
            MaterialRoot::init(MaterialTheme::default(), cx);
            gpui_material::init_components(cx);
            MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| SettingsApp::new(window, cx))
            })
            .expect("open Material window");
        });
}
```

为重复渲染的控件和列表项提供稳定、有语义的元素 ID（如 `"display-name"`、`("row", id)`），确保重排时焦点、水波纹和动效可以准确跟随目标项。

<a id="forms-and-validation"></a>

## 表单与校验

将 `TextField`、`ExposedDropdown`、`Checkbox`、`RadioGroup`、`Switch` 和 `ChipGroup` 与应用自有的校验状态组合使用：

- 在 `TextField` 上使用 `.error(message)` 展示校验错误（`.supporting_text(text)` 展示辅助提示）；如需在输入提交前拦截非法字符，在 `TextFieldState` 上配置 `.validate(|text, _| ...)`。
- 禁用或只读 `TextField` 时应更新持久实体 `TextFieldState`（`state.set_disabled(true, cx)` / `state.set_readonly(true, cx)`）；其余控件（`Button`、`Checkbox`、`Radio`、`Switch`、`Chip`、`ChipGroup`）直接在元素上调用 `.disabled(bool)`。
- `ExposedDropdown` 可通过 `.editor(|field| field.variant(TextFieldVariant::Outlined).error(...))` 定制或校验底层输入框。
- 提交按钮使用 `Button::filled(...).loading(self.submitting)` 进入受控加载态：内联渲染 Material 旋转指示器、屏蔽重复触发，并保持按钮静止宽度不抖动。

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

<a id="theming-dark-mode-dynamic-color-and-local-overrides"></a>

## 主题、深色模式、动态配色与局部覆盖

`MaterialTheme` 持有色彩方案、排版、动效方案与布局策略（方向与密度）。默认值与 Token 详情见[纯主题](theme/theme.zh.md)。

### 切换浅色、深色与动态主题

调用 `set_material_theme(theme, cx)` 更新应用主题。色彩会按目标动效方案的 Slow Effects 弹簧平滑过渡（减少动态效果模式下立即切换），并自动投影到 `gpui_base::Theme::global`。

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

启用 `dynamic-color` Cargo feature 后，可以从种子色派生完整 Material 3 色板、选择内置 `ThemePreset`，或生成经色相协调的自定义状态色组：

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

### 语义色阶

使用 `SemanticTone`（`Default`、`Danger`、`Success`、`Warning`、`Info`，同时导出别名 `ButtonTone`）可在 `Button`、`ToggleButton`、`IconButton`、`Chip`、`ChipGroupItem` 和 `Snackbar` 上直接应用 Material 3 语义色对，无需手动拼装单个颜色角色：

```rust
use gpui_material::{Button, SemanticTone};

let delete_button = Button::filled("delete", "Delete workspace")
    .tone(SemanticTone::Danger)
    .on_activate(|_, _| {});
```

### 窗口级与子树级局部覆盖

使用 `foundation::set_window_material_theme(Some(theme), window, cx)` 为单个窗口覆盖主题而不影响应用其余部分（传入 `None` 恢复继承应用主题）。

使用 `ThemeScope` 为任意子树覆盖主题、文字方向（`Direction::Rtl`）与密度：

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

`ThemeScope` 不会篡改 `gpui_base::Theme::global`。当局部作用域中的触发器需要绘制延迟浮层或打开独立原生子窗口时，用 `ThemeScope::inherit(child, window, cx)` 包装延迟内容，或用 `ThemeScope::bind_action(action, window, cx)` 包装回调，以沿用当前作用域。

<a id="layouts-adaptive-navigation-and-overlays"></a>

## 布局、自适应导航与浮层

### Scaffold 与自适应导航

组合 `Scaffold`、`NavigationItems` 和 `AdaptiveNavigation`，根据可用宽度自动在 `NavigationBar`（Compact）与 `NavigationRail`（Medium 及以上）之间切换。在主体区域嵌入 `ListDetail` 或 `SupportingPane` 实现 Material 规范多栏布局，并用 `foundation::isolated_scroll` 包裹各滚动区域，避免滚轮事件穿透到外层容器。

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

若不需要随宽度切换导航形态，可直接挂载 `NavigationBar::new(items)` 或 `NavigationRail`。窄屏回退单栏时，通过 `ListDetail::is_single_pane(width)` 判断是否在 `AppBar` 中显示返回按钮。

### 对话框、面板、菜单与消息条

- **Dialog 与 Sheet**：为每个模态浮层保留一个 `Entity<DialogState>`，通过 `state.update(cx, |s, cx| s.open(window, cx))` 与 `s.close(window, cx)` 打开和关闭，确保焦点限制与关闭后的焦点恢复同步。
- **Menu**：将触发元素和 `MenuItem` 列表传入 `Menu::new(id, trigger, items)`。受控开关使用 `.open(bool)` 和 `.on_open_change(...)`，嵌套菜单使用 `.submenu([...])`，右键菜单调用 `.context()`。
- **Snackbar**：挂载持久 `Entity<SnackbarHost>`（直接挂载或通过 `Scaffold::snackbar(...)`），然后调用 `host.push(Snackbar::new(...), window, cx)` 入队。附带 `.action(label, callback)` 的消息默认常驻直到触发或关闭，如需自动消失可在其后显式调用 `.timeout(Some(...))`。

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

<a id="focus-keyboard-navigation-interaction-observation-and-accessibility"></a>

## 焦点、键盘导航、交互观测与无障碍

### 焦点句柄与漫游焦点

在交互控件（`Button`、`Checkbox`、`Radio`、`Switch`、`Chip`、`Surface` 以及 `Slider::track_focus(RangeThumb::End, &focus)`）上通过 `.track_focus(&self.handle)` 绑定调用方持有的 `FocusHandle`，用于删除条目或关闭视图后的焦点恢复。

- 可移除标签将恢复目标传给 `Chip::on_remove(label, Some(self.restore.clone()), handler)`，通过点击、`Backspace` 或 `Delete` 删除时焦点会按预期转移。
- 自定义虚拟列表可调用 `ListItem::track_focus(&row_focus, is_active_tab_stop)` 提供持久行焦点与单一 roving Tab stop，同时由集合负责完整模型的方向键导航和滚动显示。

### 使用 `InteractionSource` 观测交互状态

保留一个 `Entity<InteractionSource>`，并通过 `.interaction_source(&self.interactions)` 挂载到一个或多个控件（同一窗口内各控件须使用不同元素 ID）。既可在 `render` 中调用 `source.read(cx).state(window_id, &id.into())` 读取最新 `InteractionState`（`hovered`、`pressed`、`focus_visible`、`dragged`、`disabled`），也可订阅延迟派发的 `InteractionUpdate` 事件：

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

### RTL 镜像与减少动态效果

- **RTL**：通过 `MaterialTheme::with_direction(Direction::Rtl)` 或局部 `ThemeScope::new(child).direction(Direction::Rtl)` 启用从右到左布局。Material 控件会自动镜像排版、内边距和水平方向键导航。具有方向含义的图标需显式调用 `Icon::new(IconName::ArrowForward).mirror_in_rtl(true)`，自定义浮层锚点则通过 `foundation::logical_anchor(anchor, theme.layout.direction)` 镜像。
- **减少动态效果**：通过 GPUI 的 `cx.reduce_motion()` 读取系统偏好。将其传入 `theme.motion.spec(MotionKind::Spatial, MotionSpeed::Default, cx.reduce_motion())` 会直接解析为 `MotionSpec::Instant`，`foundation::animate_with_status` 与 `LoopClock` 也会立即收敛而不额外调度动画帧。

<a id="building-custom-material-controls"></a>

## 构建自定义 Material 控件

需要定制内置组件库之外的控件时，优先基于 `gpui_material::foundation` 构建，以自动复用 Material 主题、状态层、水波纹、焦点环、多边形形状和局部作用域覆盖：

- `Surface::clickable(id, label, role)`、`Surface::selectable(id, label, role, selected)` 和 `Surface::toggleable(id, label, role, checked)` 内置主题前背景色对、状态层、按压水波纹、键盘焦点环、无障碍角色以及 `.on_activate` / `.on_checked_change`。
- 使用 `.shape(CornerRole::Medium.shape())` 应用圆角，或用 `.polygon_shape(MaterialShape::Cookie7Sided)` 应用 35 种 Expressive 多边形之一（默认开启按形状轮廓的指针命中测试）。
- 调用 `foundation::current_material_theme(window, cx)` 解析最近的 `ThemeScope`、窗口覆盖或应用主题，并用 `foundation::color(theme.colors.color(role))` 将 `Argb` 转为 `gpui::Hsla`。
- 在 `ThemeScope` 或窗口覆盖内为独立 `gpui_base` 控件配色时，调用 `foundation::current_base_theme(window, cx)` 获取匹配的局部 Base Token，而无需修改 `gpui_base::Theme::global`。
- 自定义画布绘制可使用 `foundation::Shape`（`Shape::rounded` / `Shape::polygon`），自定义模态遮罩层可使用 `foundation::Scrim::new(id).opacity(0.32).on_click(...)`。

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

<a id="feature-flags-and-slim-builds"></a>

## Feature 开关与精简构建

`gpui-material` 默认启用 `full`，编译全部六个核心组件分组（`actions`、`selection`、`inputs`、`surfaces`、`feedback`、`navigation`）。在下游应用中关闭默认 feature 并按需选择分组和可选系列，可显著缩短编译时间并减小产物体积：

```toml
[dependencies]
gpui = { package = "gpui-pre", version = "=0.3.12" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.12", features = ["font-kit", "x11", "wayland", "runtime_shaders"] }
gpui-material = { version = "0.3.2", default-features = false, features = [
    "actions",
    "inputs",
    "surfaces",
    "feedback",
    "dynamic-color",
    "window-chrome",
] }
```

每个可选 feature 仅会自动激活其依赖的核心组件分组：

| Feature | 类别 | 启用的公开 API |
| --- | --- | --- |
| `full`（默认） | 核心合集 | 启用以下全部六个核心组件分组 |
| `actions` | 核心分组 | `Button`、`ToggleButton`、`IconButton`、`IconToggleButton`、`Fab`、`ExtendedFab` |
| `selection` | 核心分组 | `Checkbox`、`Radio`、`RadioGroup`、`Switch`、`Chip`、`ChipGroup`、`Slider`、`SliderState` |
| `inputs` | 核心分组 | `TextField`、`TextFieldState`、`TextAreaState`、`ExposedDropdown`、`ExposedDropdownState` |
| `surfaces` | 核心分组 | `Card`、`List`、`ListItem`、`VerticalDragHandle`、`DropTarget`、`SwipeToDismiss` 与设置项瓦片（`CheckboxTile`、`RadioTile`、`SliderTile`、`SwitchTile`、`ChoiceTile`、`ListSection`） |
| `feedback` | 核心分组 | `Banner`、`Dialog`、`Sheet`、`Menu`、`Snackbar`、`SnackbarHost`、`Tooltip`、`Label`、`LinearProgress`、`CircularProgress`、`LoadingIndicator`、`PullToRefresh` |
| `navigation` | 核心分组 | `AppBar`、`BottomAppBar`、`Tabs`、`NavigationBar`、`NavigationRail`、`NavigationDrawer`、`Scaffold`、`AdaptiveNavigation`、`ListDetail`、`SupportingPane`、`Feed`、`Breadcrumbs`、`Stepper` |
| `window-chrome` | 可选系列 | `gpui_material::window_chrome` 下的跨平台自绘标题栏 |
| `date-time` | 可选系列 | `gpui_material::date_time` 下的 `DatePicker`、`DatePickerState`、`TimePicker`、`TimePickerState` |
| `search` | 可选系列 | `SearchBar`、`SearchView`、`SearchState`、`SearchResult`、`SearchEvent` |
| `toolbars` | 可选系列 | `Toolbar`、`FabMenu` 及相关工具栏类型 |
| `carousel` | 可选系列 | `Carousel`、`CarouselItem`、`CarouselState` |
| `expressive-groups` | 可选系列 | `ButtonGroup`、`SplitButton` |
| `dynamic-color` | 主题特性 | `MaterialTheme::from_seed`、`from_seed_with`、`ThemePreset`、`CustomColorGroup`、`harmonize_color` |
| `serde` | 主题特性 | `MaterialTheme` 及主题值类型的 `Serialize` / `Deserialize` 支持 |

展示基础原语（`Text`、`Icon`、`Badge`、`Divider`、`Avatar`、`EmptyState`、`Skeleton`、`DisplayScrollbar`、`SemanticTone`）、`foundation` 与 `theme` 始终可用，不受核心组件分组开关影响。

[下游集成](integration.zh.md) · [组件](component.zh.md) · [纯主题](theme/theme.zh.md) · [架构与依赖](architecture.zh.md)

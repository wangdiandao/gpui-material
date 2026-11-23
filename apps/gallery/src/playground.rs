use std::{any::Any, collections::HashMap, sync::OnceLock};

use gpui::{
    AnyElement, AnyView, App, AppContext, ClipboardItem, Context, ElementId, Entity,
    InteractiveElement, IntoElement, ParentElement, Render, Role, StatefulInteractiveElement,
    Styled, Subscription, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    Button, ButtonVariant, Chip, ChipVariant, ControlSize, Icon, IconButton, IconName, Slider,
    SliderState, Surface, Switch, Text, TextField, TextFieldEvent, TextFieldState,
    TextFieldVariant, ThemeScope,
    foundation::{color, window_material_theme},
    theme::{Appearance, ColorRole, ColorScheme, Direction, ThemePreset, TypeRole},
};

use crate::{Story, styles_examples::preset_label};

const PREVIEW_PRESETS: [ThemePreset; 6] = [
    ThemePreset::Baseline,
    ThemePreset::Vibrant,
    ThemePreset::Expressive,
    ThemePreset::Ocean,
    ThemePreset::Forest,
    ThemePreset::Sunset,
];

fn cached_preview_scheme(preset: ThemePreset, appearance: Appearance) -> ColorScheme {
    static CACHE: OnceLock<[[ColorScheme; 2]; PREVIEW_PRESETS.len()]> = OnceLock::new();
    let table = CACHE.get_or_init(|| {
        std::array::from_fn(|ix| {
            let preset = PREVIEW_PRESETS[ix];
            [
                preset.colors(Appearance::Light),
                preset.colors(Appearance::Dark),
            ]
        })
    });
    let slot = PREVIEW_PRESETS
        .iter()
        .position(|p| *p == preset)
        .unwrap_or(0);
    let mode = usize::from(appearance == Appearance::Dark);
    table[slot][mode].clone()
}

/// One declarative configuration control rendered in the Configuration card.
#[derive(Clone, Copy, Debug)]
pub enum Control {
    Choice {
        id: &'static str,
        label: &'static str,
        options: &'static [&'static str],
        default: usize,
    },
    Switch {
        id: &'static str,
        label: &'static str,
        default: bool,
    },
    Slider {
        id: &'static str,
        label: &'static str,
        min: f32,
        max: f32,
        step: f32,
        default: f32,
    },
    Text {
        id: &'static str,
        label: &'static str,
        default: &'static str,
    },
}

impl Control {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Choice { id, .. }
            | Self::Switch { id, .. }
            | Self::Slider { id, .. }
            | Self::Text { id, .. } => id,
        }
    }
}

/// A titled group of configuration controls.
#[derive(Clone, Copy, Debug)]
pub struct Section {
    pub title: &'static str,
    pub controls: &'static [Control],
}

/// A value override applied by a [`Scenario`] preset.
#[derive(Clone, Copy, Debug)]
pub enum Set {
    Option(&'static str),
    Bool(bool),
    Number(f32),
    Text(&'static str),
}

/// One named scenario preset shown at the top of the Configuration card.
#[derive(Clone, Copy, Debug)]
pub struct Scenario {
    pub name: &'static str,
    pub values: &'static [(&'static str, Set)],
}

/// Runtime value of one configuration control.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Choice(usize),
    Bool(bool),
    Number(f32),
    Text(String),
}

/// Current playground configuration, passed to `preview` and `code`.
#[derive(Clone)]
pub struct Config {
    playground: &'static Playground,
    values: Vec<(&'static str, Value)>,
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.playground, other.playground) && self.values == other.values
    }
}

impl Config {
    pub fn defaults(playground: &'static Playground) -> Self {
        let mut values = Vec::new();
        for section in playground.sections {
            for control in section.controls {
                let default = match *control {
                    Control::Choice { default, .. } => Value::Choice(default),
                    Control::Switch { default, .. } => Value::Bool(default),
                    Control::Slider { default, .. } => Value::Number(default),
                    Control::Text { default, .. } => Value::Text(default.to_owned()),
                };
                values.push((control.id(), default));
            }
        }
        Self { playground, values }
    }

    pub fn scenario(playground: &'static Playground, scenario: &Scenario) -> Self {
        let mut config = Self::defaults(playground);
        for &(id, value) in scenario.values {
            config.apply_set(id, value);
        }
        config
    }

    fn apply_set(&mut self, id: &str, set: Set) {
        let Some(control) = self.playground.control(id) else {
            panic!("unknown playground control `{id}`");
        };
        let next = match (control, set) {
            (Control::Choice { options, .. }, Set::Option(option)) => {
                let ix = options
                    .iter()
                    .position(|candidate| *candidate == option)
                    .unwrap_or_else(|| panic!("unknown option `{option}` for `{id}`"));
                Value::Choice(ix)
            }
            (Control::Switch { .. }, Set::Bool(v)) => Value::Bool(v),
            (Control::Slider { min, max, .. }, Set::Number(v)) => Value::Number(v.clamp(min, max)),
            (Control::Text { .. }, Set::Text(v)) => Value::Text(v.to_owned()),
            _ => panic!("mismatched scenario value type for `{id}`"),
        };
        self.set_value(id, next);
    }

    fn get(&self, id: &str) -> &Value {
        self.values
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
            .unwrap_or_else(|| panic!("unknown playground control `{id}`"))
    }

    fn set_value(&mut self, id: &str, value: Value) {
        let slot = self
            .values
            .iter_mut()
            .find(|(key, _)| *key == id)
            .unwrap_or_else(|| panic!("unknown playground control `{id}`"));
        slot.1 = value;
    }

    pub fn option(&self, id: &str) -> &'static str {
        let Control::Choice { options, .. } = self
            .playground
            .control(id)
            .unwrap_or_else(|| panic!("unknown choice `{id}`"))
        else {
            panic!("control `{id}` is not Choice");
        };
        let Value::Choice(ix) = *self.get(id) else {
            unreachable!();
        };
        options[ix]
    }

    pub fn bool(&self, id: &str) -> bool {
        let Value::Bool(value) = *self.get(id) else {
            panic!("control `{id}` is not Switch");
        };
        value
    }

    pub fn number(&self, id: &str) -> f32 {
        let Value::Number(value) = *self.get(id) else {
            panic!("control `{id}` is not Slider");
        };
        value
    }

    pub fn text(&self, id: &str) -> &str {
        let Value::Text(ref value) = *self.get(id) else {
            panic!("control `{id}` is not Text");
        };
        value
    }
}

/// Declarative definition of one component playground.
pub struct Playground {
    pub scenarios: &'static [Scenario],
    pub sections: &'static [Section],
    pub preview:
        fn(&Config, &mut PreviewCtx, &mut Window, &mut Context<ComponentPageView>) -> AnyElement,
    pub code: fn(&Config) -> String,
}

impl Playground {
    pub fn control(&self, id: &str) -> Option<Control> {
        self.sections
            .iter()
            .flat_map(|section| section.controls.iter().copied())
            .find(|control| control.id() == id)
    }
}

pub type StatusCallback = std::rc::Rc<dyn Fn(String, &mut App)>;

/// Helper passed to a playground's `preview` function for retained preview-only state
/// and two-way syncing back to the Configuration card.
pub struct PreviewCtx<'a> {
    locals: &'a mut HashMap<&'static str, Box<dyn Any>>,
}

impl<'a> PreviewCtx<'a> {
    /// Returns or initializes preview-local state retained across redraws and navigation.
    pub fn local<T: 'static>(&mut self, key: &'static str, init: impl FnOnce() -> T) -> &mut T {
        self.locals
            .entry(key)
            .or_insert_with(|| Box::new(init()))
            .downcast_mut::<T>()
            .expect("playground local state type mismatch")
    }

    /// Returns a callback that writes a `bool` back to a `Control::Switch`.
    pub fn on_bool(
        &self,
        id: &'static str,
        cx: &Context<ComponentPageView>,
    ) -> impl Fn(bool, &mut Window, &mut App) + 'static {
        let owner = cx.weak_entity();
        move |next, window, cx| {
            let _ = owner.update(cx, |this, cx| {
                this.set_value(id, Value::Bool(next), window, cx);
            });
        }
    }

    /// Returns a callback that writes a choice index back to a `Control::Choice`.
    pub fn on_choice(
        &self,
        id: &'static str,
        cx: &Context<ComponentPageView>,
    ) -> impl Fn(usize, &mut Window, &mut App) + 'static {
        let owner = cx.weak_entity();
        move |next, window, cx| {
            let _ = owner.update(cx, |this, cx| {
                this.set_value(id, Value::Choice(next), window, cx);
            });
        }
    }

    /// Returns a callback that records an interactive status message shown below the preview.
    pub fn on_status(&self, cx: &Context<ComponentPageView>) -> StatusCallback {
        let owner = cx.weak_entity();
        std::rc::Rc::new(move |message, cx| {
            let _ = owner.update(cx, |this, cx| {
                this.last_event = Some(message);
                cx.notify();
            });
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageTab {
    Preview,
    Specs,
}

/// Redesigned component page with `Live preview | Specs` tabs, local `ThemeScope` preview canvas,
/// declarative `Configuration` card, generated Rust `Code` card, and lazy-loaded full `Specs` view.
pub struct ComponentPageView {
    story: Story,
    playground: &'static Playground,
    tab: PageTab,
    config: Config,
    active_scenario: Option<usize>,
    preview_preset: ThemePreset,
    preview_appearance: Option<Appearance>,
    sliders: HashMap<&'static str, Entity<SliderState>>,
    texts: HashMap<&'static str, Entity<TextFieldState>>,
    locals: HashMap<&'static str, Box<dyn Any>>,
    specs_view: Option<AnyView>,
    last_event: Option<String>,
    copied_code: bool,
    _subscriptions: Vec<Subscription>,
}

impl ComponentPageView {
    pub fn new(
        story: Story,
        playground: &'static Playground,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let config = Config::defaults(playground);
        let mut sliders = HashMap::new();
        let mut texts = HashMap::new();
        let mut subscriptions = Vec::new();

        for section in playground.sections {
            for control in section.controls {
                match *control {
                    Control::Slider {
                        id,
                        min,
                        max,
                        step,
                        default,
                        ..
                    } => {
                        let state = cx.new(|_| {
                            SliderState::new()
                                .min(min)
                                .max(max)
                                .step(step)
                                .default_value(default)
                        });
                        let sub = cx.observe(&state, move |this, state, cx| {
                            let next = state.read(cx).value().end();
                            let current = this.config.number(id);
                            if (current - next).abs() > 1e-4 {
                                this.config.set_value(id, Value::Number(next));
                                this.active_scenario = None;
                                this.copied_code = false;
                                cx.notify();
                            }
                        });
                        sliders.insert(id, state);
                        subscriptions.push(sub);
                    }
                    Control::Text { id, default, .. } => {
                        let state = cx.new(|cx| {
                            let mut field = TextFieldState::new(window, cx);
                            field.replace_all(default, window, cx);
                            field
                        });
                        let sub =
                            cx.subscribe_in(&state, window, move |this, state, event, _, cx| {
                                if matches!(event, TextFieldEvent::Change) {
                                    let next = state.read(cx).value().to_string();
                                    if this.config.text(id) != next {
                                        this.config.set_value(id, Value::Text(next));
                                        this.active_scenario = None;
                                        this.copied_code = false;
                                        cx.notify();
                                    }
                                }
                            });
                        texts.insert(id, state);
                        subscriptions.push(sub);
                    }
                    Control::Choice { .. } | Control::Switch { .. } => {}
                }
            }
        }

        Self {
            story,
            playground,
            tab: PageTab::Preview,
            config,
            active_scenario: Some(0),
            preview_preset: ThemePreset::Baseline,
            preview_appearance: None,
            sliders,
            texts,
            locals: HashMap::new(),
            specs_view: None,
            last_event: None,
            copied_code: false,
            _subscriptions: subscriptions,
        }
    }

    pub fn set_tab(&mut self, tab: PageTab, window: &mut Window, cx: &mut Context<Self>) {
        if self.tab == tab {
            return;
        }
        self.tab = tab;
        if tab == PageTab::Specs && self.specs_view.is_none() {
            self.specs_view = self.story.new_view(window, cx);
        }
        cx.notify();
    }

    fn sync_editors_to_config(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for section in self.playground.sections {
            for control in section.controls {
                match *control {
                    Control::Slider { id, .. } => {
                        let value = self.config.number(id);
                        if let Some(slider) = self.sliders.get(id) {
                            slider.update(cx, |state, cx| {
                                if (state.value().end() - value).abs() > 1e-4 {
                                    state.set_value(value, window, cx);
                                }
                            });
                        }
                    }
                    Control::Text { id, .. } => {
                        let value = self.config.text(id).to_owned();
                        if let Some(field) = self.texts.get(id) {
                            field.update(cx, |state, cx| {
                                if state.value() != value {
                                    state.replace_all(&value, window, cx);
                                }
                            });
                        }
                    }
                    Control::Choice { .. } | Control::Switch { .. } => {}
                }
            }
        }
    }

    fn set_value(
        &mut self,
        id: &'static str,
        value: Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.config.get(id) == &value {
            return;
        }
        self.config.set_value(id, value);
        self.active_scenario = None;
        self.copied_code = false;
        self.sync_editors_to_config(window, cx);
        cx.notify();
    }

    fn select_scenario(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.config = if index == 0 {
            Config::defaults(self.playground)
        } else {
            Config::scenario(self.playground, &self.playground.scenarios[index - 1])
        };
        self.active_scenario = Some(index);
        self.copied_code = false;
        self.sync_editors_to_config(window, cx);
        cx.notify();
    }

    fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_scenario(0, window, cx);
        self.last_event = None;
    }

    fn render_card_header(&self, window: &Window, cx: &Context<Self>) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let owner = cx.weak_entity();
        let effective_appearance = self
            .preview_appearance
            .unwrap_or_else(|| theme.colors.appearance());

        let tab_preview = {
            let owner = owner.clone();
            Button::new("page-tab-preview", "Live preview")
                .variant(if self.tab == PageTab::Preview {
                    ButtonVariant::FilledTonal
                } else {
                    ButtonVariant::Text
                })
                .size(ControlSize::ExtraSmall)
                .on_activate(move |window, cx| {
                    let _ = owner.update(cx, |this, cx| this.set_tab(PageTab::Preview, window, cx));
                })
        };
        let tab_specs = {
            let owner = owner.clone();
            Button::new("page-tab-specs", "Specs")
                .variant(if self.tab == PageTab::Specs {
                    ButtonVariant::FilledTonal
                } else {
                    ButtonVariant::Text
                })
                .size(ControlSize::ExtraSmall)
                .on_activate(move |window, cx| {
                    let _ = owner.update(cx, |this, cx| this.set_tab(PageTab::Specs, window, cx));
                })
        };

        let preset_cycle = {
            let owner = owner.clone();
            let current = self.preview_preset;
            Button::new(
                "preview-preset-cycle",
                format!("Theme: {}", preset_label(current)),
            )
            .variant(ButtonVariant::Outlined)
            .size(ControlSize::ExtraSmall)
            .on_activate(move |_, cx| {
                let _ = owner.update(cx, |this, cx| {
                    let ix = PREVIEW_PRESETS
                        .iter()
                        .position(|p| *p == this.preview_preset)
                        .unwrap_or(0);
                    this.preview_preset = PREVIEW_PRESETS[(ix + 1) % PREVIEW_PRESETS.len()];
                    cx.notify();
                });
            })
        };

        let mode_toggle = {
            let owner = owner;
            let next_mode = match effective_appearance {
                Appearance::Light => Appearance::Dark,
                Appearance::Dark => Appearance::Light,
            };
            let (icon, label) = match effective_appearance {
                Appearance::Light => (IconName::DarkMode, "Preview dark mode"),
                Appearance::Dark => (IconName::LightMode, "Preview light mode"),
            };
            IconButton::new("preview-mode-toggle", label, Icon::new(icon))
                .size(ControlSize::ExtraSmall)
                .on_activate(move |_, cx| {
                    let _ = owner.update(cx, |this, cx| {
                        this.preview_appearance = Some(next_mode);
                        cx.notify();
                    });
                })
        };

        div()
            .id("playground-card-header")
            .w_full()
            .px_4()
            .py_3()
            .border_b_1()
            .border_color(outline)
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap_2()
            .when(rtl, Styled::flex_row_reverse)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .when(rtl, Styled::flex_row_reverse)
                    .child(tab_preview)
                    .child(tab_specs),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .when(rtl, Styled::flex_row_reverse)
                    .child(preset_cycle)
                    .child(mode_toggle),
            )
    }

    fn render_code_card(
        &self,
        code: String,
        window: &Window,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let card_bg = color(theme.colors.color(ColorRole::SurfaceContainerLow));
        let code_bg = color(theme.colors.color(ColorRole::SurfaceContainerHighest));
        let on_surface = color(theme.colors.color(ColorRole::OnSurface));
        let copied = self.copied_code;
        let owner = cx.weak_entity();
        let copy_text = code.clone();

        div()
            .id("playground-code-card")
            .w_full()
            .rounded(px(16.))
            .bg(card_bg)
            .border_1()
            .border_color(outline)
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(
                div()
                    .px_4()
                    .py_2()
                    .border_b_1()
                    .border_color(outline)
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        Text::new("Rust code")
                            .id("code-title")
                            .role(TypeRole::TitleSmall),
                    )
                    .child(
                        Button::new(
                            "playground-copy-code",
                            if copied { "Copied" } else { "Copy" },
                        )
                        .variant(ButtonVariant::Text)
                        .size(ControlSize::ExtraSmall)
                        .leading_icon(Icon::new(if copied {
                            IconName::Check
                        } else {
                            IconName::Description
                        }))
                        .on_activate(move |_, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(copy_text.clone()));
                            let _ = owner.update(cx, |this, cx| {
                                this.copied_code = true;
                                cx.notify();
                            });
                        }),
                    ),
            )
            .child(
                div()
                    .p_4()
                    .bg(code_bg)
                    .text_xs()
                    .text_color(on_surface)
                    .child(code),
            )
    }

    fn render_config_card(
        &self,
        stacked: bool,
        window: &Window,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let card_bg = color(theme.colors.color(ColorRole::SurfaceContainerLow));
        let on_surface_variant = color(theme.colors.color(ColorRole::OnSurfaceVariant));
        let owner = cx.weak_entity();
        let is_default = self.config == Config::defaults(self.playground);

        let scenario_chips = std::iter::once("Default")
            .chain(self.playground.scenarios.iter().map(|s| s.name))
            .enumerate()
            .map(|(ix, name)| {
                let owner = owner.clone();
                Chip::new(("playground-scenario", ix), name)
                    .variant(ChipVariant::Filter)
                    .selected(self.active_scenario == Some(ix))
                    .on_change(move |_, window, cx| {
                        let _ = owner.update(cx, |this, cx| {
                            this.select_scenario(ix, window, cx);
                        });
                    })
            });

        let sections = self
            .playground
            .sections
            .iter()
            .enumerate()
            .map(|(s_ix, section)| {
                let controls = section.controls.iter().map(|control| {
                    let owner = owner.clone();
                    match *control {
                        Control::Choice {
                            id, label, options, ..
                        } => {
                            let selected = self.config.option(id);
                            div()
                                .id((ElementId::from("cfg-choice"), id))
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(on_surface_variant).child(label))
                                .child(div().flex().flex_wrap().gap_1().children(
                                    options.iter().enumerate().map(|(opt_ix, &opt)| {
                                        let owner = owner.clone();
                                        Chip::new(
                                            (ElementId::from("cfg-opt"), format!("{id}-{opt_ix}")),
                                            opt,
                                        )
                                        .variant(ChipVariant::Filter)
                                        .selected(opt == selected)
                                        .on_change(
                                            move |_, window, cx| {
                                                let _ = owner.update(cx, |this, cx| {
                                                    this.set_value(
                                                        id,
                                                        Value::Choice(opt_ix),
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            },
                                        )
                                    }),
                                ))
                                .into_any_element()
                        }
                        Control::Switch { id, label, .. } => {
                            let checked = self.config.bool(id);
                            div()
                                .id((ElementId::from("cfg-switch-row"), id))
                                .w_full()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap_2()
                                .child(div().flex_1().min_w_0().text_sm().child(label))
                                .child(
                                    Switch::new(
                                        (ElementId::from("cfg-switch"), id),
                                        label,
                                        checked,
                                    )
                                    .show_label(false)
                                    .on_change(
                                        move |next, window, cx| {
                                            let _ = owner.update(cx, |this, cx| {
                                                this.set_value(id, Value::Bool(next), window, cx);
                                            });
                                        },
                                    ),
                                )
                                .into_any_element()
                        }
                        Control::Slider { id, label, .. } => {
                            let value = self.config.number(id);
                            let state = &self.sliders[id];
                            div()
                                .id((ElementId::from("cfg-slider-row"), id))
                                .w_full()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(on_surface_variant)
                                                .child(label),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(on_surface_variant)
                                                .child(format!("{value:.1}")),
                                        ),
                                )
                                .child(
                                    Slider::new((ElementId::from("cfg-slider"), id), label, state)
                                        .size(ControlSize::ExtraSmall),
                                )
                                .into_any_element()
                        }
                        Control::Text { id, label, .. } => {
                            let state = &self.texts[id];
                            div()
                                .id((ElementId::from("cfg-text-row"), id))
                                .w_full()
                                .child(
                                    TextField::new((ElementId::from("cfg-text"), id), label, state)
                                        .variant(TextFieldVariant::Outlined),
                                )
                                .into_any_element()
                        }
                    }
                });

                div()
                    .id(("playground-section", s_ix))
                    .w_full()
                    .pt_3()
                    .border_t_1()
                    .border_color(outline)
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        Text::new(section.title)
                            .id("title")
                            .role(TypeRole::LabelLarge)
                            .color(ColorRole::OnSurfaceVariant),
                    )
                    .children(controls)
            });

        let reset_owner = owner.clone();
        div()
            .id("playground-config-card")
            .when(stacked, Styled::w_full)
            .when(!stacked, |card| card.w(px(332.)).flex_none())
            .rounded(px(16.))
            .bg(card_bg)
            .border_1()
            .border_color(outline)
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        Text::new("Configuration")
                            .id("config-title")
                            .role(TypeRole::TitleMedium)
                            .emphasized(true),
                    )
                    .child(
                        Button::new("playground-reset", "Reset")
                            .variant(ButtonVariant::Text)
                            .size(ControlSize::ExtraSmall)
                            .disabled(is_default)
                            .on_activate(move |window, cx| {
                                let _ = reset_owner.update(cx, |this, cx| this.reset(window, cx));
                            }),
                    ),
            )
            .when(!self.playground.scenarios.is_empty(), |card| {
                card.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(on_surface_variant)
                                .child("Scenarios"),
                        )
                        .child(div().flex().flex_wrap().gap_1().children(scenario_chips)),
                )
            })
            .children(sections)
    }
}

impl Render for ComponentPageView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = window_material_theme(window, cx).clone();
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let card_bg = color(theme.colors.color(ColorRole::SurfaceContainerLow));
        let on_surface_variant = color(theme.colors.color(ColorRole::OnSurfaceVariant));
        let stacked = f32::from(window.viewport_size().width) < 1180.;

        let effective_appearance = self
            .preview_appearance
            .unwrap_or_else(|| theme.colors.appearance());
        let mut scoped_theme = theme.clone();
        scoped_theme.colors = cached_preview_scheme(self.preview_preset, effective_appearance);

        if self.tab == PageTab::Specs {
            let specs = self
                .specs_view
                .get_or_insert_with(|| self.story.new_view(window, cx).expect("story view"))
                .clone();
            return div()
                .id("component-page-specs")
                .test_support()
                .w_full()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .w_full()
                        .rounded(px(16.))
                        .bg(card_bg)
                        .border_1()
                        .border_color(outline)
                        .child(self.render_card_header(window, cx)),
                )
                .child(ThemeScope::new(specs).theme(scoped_theme))
                .into_any_element();
        }

        let canvas_bg = color(scoped_theme.colors.color(ColorRole::Surface));

        let config_snapshot = self.config.clone();
        let preview_fn = self.playground.preview;
        let preview_element = {
            let mut ctx = PreviewCtx {
                locals: &mut self.locals,
            };
            preview_fn(&config_snapshot, &mut ctx, window, cx)
        };
        let generated_code = (self.playground.code)(&self.config);

        let preview_card = div()
            .id("playground-preview-card")
            .flex_1()
            .min_w_0()
            .rounded(px(16.))
            .bg(card_bg)
            .border_1()
            .border_color(outline)
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(self.render_card_header(window, cx))
            .child(
                ThemeScope::new(
                    Surface::new(ColorRole::Surface)
                        .w_full()
                        .min_h(px(280.))
                        .p_6()
                        .bg(canvas_bg)
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_3()
                        .child(preview_element)
                        .children(self.last_event.as_ref().map(|event| {
                            div()
                                .id("playground-event-status")
                                .role(Role::Status)
                                .aria_label(event.clone())
                                .pt_2()
                                .text_xs()
                                .text_color(on_surface_variant)
                                .child(event.clone())
                        })),
                )
                .theme(scoped_theme),
            );

        let left_column = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_4()
            .child(preview_card)
            .child(self.render_code_card(generated_code, window, cx));

        let config_card = self.render_config_card(stacked, window, cx);

        div()
            .id("component-page-preview")
            .test_support()
            .w_full()
            .flex()
            .when(stacked, Styled::flex_col)
            .items_start()
            .gap_4()
            .child(left_column)
            .child(config_card)
            .into_any_element()
    }
}

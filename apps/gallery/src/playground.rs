mod controls;
mod preview;
mod schema;

use std::{any::Any, cell::Cell, collections::HashMap, rc::Rc, sync::OnceLock};

use gpui::{
    AnyView, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Pixels,
    Render, Role, StatefulInteractiveElement, Styled, Subscription, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    SliderState, Surface, TextFieldEvent, TextFieldState, ThemeScope,
    foundation::{color, window_material_theme},
    theme::{Appearance, ColorRole, ColorScheme, ThemePreset},
};

use crate::Story;

pub use schema::{Config, Control, Playground, PreviewCtx, Scenario, Section, Set, Value};

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
    available_width: Rc<Cell<Pixels>>,
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
        available_width: Rc<Cell<Pixels>>,
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
            available_width,
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
}

impl Render for ComponentPageView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = window_material_theme(window, cx).clone();
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let card_bg = color(theme.colors.color(ColorRole::SurfaceContainerLow));
        let stacked = f32::from(self.available_width.get()) < 760.;

        let effective_appearance = self
            .preview_appearance
            .unwrap_or_else(|| theme.colors.appearance());
        let mut scoped_theme = theme.clone();
        scoped_theme.colors = cached_preview_scheme(self.preview_preset, effective_appearance);
        let scoped_on_surface_variant =
            color(scoped_theme.colors.color(ColorRole::OnSurfaceVariant));

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
            .test_support()
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
                                .text_color(scoped_on_surface_variant)
                                .child(event.clone())
                        })),
                )
                .theme(scoped_theme),
            );

        let left_column = div()
            .when(stacked, Styled::w_full)
            .when(!stacked, Styled::flex_1)
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

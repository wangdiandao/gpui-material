use gpui::StatefulInteractiveElement;
use std::{cell::RefCell, rc::Rc};

use gpui::{
    AccessibleAction, App, Axis, ElementId, Entity, FocusHandle, Hsla, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::{Radio as BaseRadio, RadioGroup as BaseRadioGroup, TestSupportExt};
use gpui_material_foundation::{
    InteractionSource, ThemeScope, animate, control_feedback::activate_feedback,
    current_material_theme, move_roving_focus,
};
use gpui_material_theme::{ColorRole, Direction, MotionKind, MotionSpeed};

use super::{
    base::{SelectionControlBase, bind_activate},
    focus_style, selection_target, selection_tone, tone,
};

type RadioChange = Rc<dyn Fn(&ElementId, &mut Window, &mut App)>;

/// Radio indicator pigments. Omitted values use the current window theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RadioColors {
    /// Ring and inner dot color when enabled and selected.
    pub selected: Option<Hsla>,
    /// Ring color when enabled and unselected.
    pub unselected: Option<Hsla>,
    /// Ring and inner dot color when disabled and selected.
    pub disabled_selected: Option<Hsla>,
    /// Ring color when disabled and unselected.
    pub disabled_unselected: Option<Hsla>,
}

impl RadioColors {
    fn resolve(self, selected: bool, disabled: bool, window: &Window, cx: &App) -> Hsla {
        match (selected, disabled) {
            (true, false) => self.selected,
            (false, false) => self.unselected,
            (true, true) => self.disabled_selected,
            (false, true) => self.disabled_unselected,
        }
        .unwrap_or_else(|| selection_tone(selected, disabled, window, cx))
    }
}

/// A radio option. A group owns the tab stop and the caller owns selection.
#[derive(IntoElement)]
pub struct Radio {
    base: SelectionControlBase,
    selected: bool,
    on_select: Option<crate::action::Activate>,
    colors: RadioColors,
}

impl Radio {
    /// Creates a radio option with an element ID and visible/accessible label.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            base: SelectionControlBase::new(id, label),
            selected: false,
            on_select: None,
            colors: RadioColors::default(),
        }
    }

    #[cfg(feature = "surfaces")]
    pub(crate) fn decorative(mut self, decorative: bool) -> Self {
        self.base.decorative(decorative);
        self
    }

    /// Controls whether the visible label is rendered. Defaults to `true`.
    /// When `false`, the constructor's label remains the accessible name and the trailing gap is omitted.
    pub fn show_label(mut self, show: bool) -> Self {
        self.base.show_label(show);
        self
    }

    /// Replaces the visible label while keeping the constructor's label as the accessible name.
    pub fn label_content(mut self, content: impl IntoElement) -> Self {
        self.base.label_content(content);
        self
    }

    /// Sets the controlled selected state of a standalone radio button.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Disables selection and keyboard focus for this option when `true`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.base.disabled(disabled);
        self
    }

    /// Invoked when this radio option is selected (standalone or inside a [`RadioGroup`]).
    pub fn on_select(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Overrides indicator colors for the four enabled and selected combinations.
    pub fn colors(mut self, colors: RadioColors) -> Self {
        self.colors = colors;
        self
    }

    /// Uses the caller's focus handle instead of an internal keyed handle, both standalone and
    /// inside a [`RadioGroup`].
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.base.track_focus(focus);
        self
    }

    /// Observes hover, press, keyboard focus, and disabled state.
    /// Group options are observed under their rendered, group-scoped IDs.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.base.interaction_source(source);
        self
    }

    fn render_indicator(&self, window: &mut Window, cx: &mut App) -> (Hsla, gpui::Div) {
        let motion = current_material_theme(window, cx).motion;
        let disabled = self.base.disabled;
        let reduced = disabled || cx.reduce_motion();
        let dot = animate(
            (self.base.id.clone(), "radio-dot"),
            f32::from(self.selected),
            motion.spec(MotionKind::Spatial, MotionSpeed::Fast, reduced),
            window,
            cx,
        )
        .clamp(0., 1.);
        let tint = animate(
            (self.base.id.clone(), "radio-color"),
            f32::from(self.selected),
            motion.spec(MotionKind::Effects, MotionSpeed::Fast, reduced),
            window,
            cx,
        )
        .clamp(0., 1.);
        let accent = crate::control_style::mix_color(
            self.colors.resolve(false, disabled, window, cx),
            self.colors.resolve(true, disabled, window, cx),
            tint,
        );
        let indicator = div()
            .size(px(20.))
            .rounded_full()
            .border_2()
            .border_color(accent)
            .flex()
            .items_center()
            .justify_center()
            .child(div().size(px(10. * dot)).rounded_full().bg(accent));
        (accent, indicator)
    }

    fn render_control(
        mut self,
        focus: Option<FocusHandle>,
        position: Option<(usize, usize)>,
        tab_stop: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> BaseRadio {
        let (accent, indicator) = self.render_indicator(window, cx);
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let density = theme.layout.density;
        let disabled = self.base.disabled;
        let interaction = self.base.prepare_interaction(focus, window, cx);
        let group = self.base.group_name("material-radio");
        let target = selection_target(
            group.clone(),
            indicator,
            accent,
            !disabled,
            interaction.ripple.clone(),
            direction,
            density,
            cx.reduce_motion(),
        );
        let click_ripple = interaction.ripple.clone();
        let label = self.base.label.clone();
        let radio = BaseRadio::new(self.base.id.clone())
            .checked(self.selected)
            .disabled(disabled)
            .tab_stop(tab_stop)
            .accessibility_label(label)
            .group(group)
            .border_2()
            .border_color(gpui::transparent_black())
            .rounded_md()
            .focus_visible(focus_style(tone(ColorRole::Secondary, window, cx)))
            .child(self.base.render_row(direction, target, window, cx))
            .track_focus(&interaction.focus)
            .when_some(position, |radio, (position, size)| {
                radio.set_position(position, size)
            });
        let radio = interaction.attach_feedback(radio);
        if disabled {
            return radio;
        }
        match self.on_select {
            Some(handler) => {
                let handler = bind_activate(handler, window, cx);
                let select = handler.clone();
                radio
                    .when(!self.selected, |radio| {
                        radio.on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                            select(window, cx)
                        })
                    })
                    .on_change(move |_, event, window, cx| {
                        cx.stop_propagation();
                        activate_feedback(event, &click_ripple, window, cx);
                        handler(window, cx);
                    })
            }
            None => radio.on_change(move |_, event, window, cx| {
                cx.stop_propagation();
                activate_feedback(event, &click_ripple, window, cx);
            }),
        }
    }
}

impl RenderOnce for Radio {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.base.decorative {
            let (_, indicator) = self.render_indicator(window, cx);
            return div()
                .id(self.base.id)
                .test_support()
                .flex_none()
                .size(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .child(indicator)
                .into_any_element();
        }
        self.render_control(None, None, true, window, cx)
            .into_any_element()
    }
}

/// A controlled single-selection group with one Tab stop and directional navigation.
#[derive(IntoElement)]
pub struct RadioGroup {
    id: ElementId,
    label: SharedString,
    selected: Option<ElementId>,
    options: Vec<Radio>,
    axis: Axis,
    on_select: Option<RadioChange>,
}

impl RadioGroup {
    /// Creates an empty vertical radio group with an element ID and accessible group label.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected: None,
            options: Vec::new(),
            axis: Axis::Vertical,
            on_select: None,
        }
    }

    /// Sets the ID of the currently selected option.
    pub fn selected(mut self, selected: impl Into<ElementId>) -> Self {
        self.selected = Some(selected.into());
        self
    }

    /// Appends [`Radio`] options to the group.
    pub fn options(mut self, options: impl IntoIterator<Item = Radio>) -> Self {
        self.options.extend(options);
        self
    }

    /// Sets the layout and arrow-key navigation axis (`Axis::Vertical` by default).
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    /// Invoked with the selected option's ID when selection changes via click or arrow keys.
    pub fn on_select(
        mut self,
        handler: impl Fn(&ElementId, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for RadioGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let direction = current_material_theme(window, cx).layout.direction;
        let current = self
            .options
            .iter()
            .position(|item| !item.base.disabled && Some(&item.base.id) == self.selected.as_ref());
        let tab_stop = current.or_else(|| self.options.iter().position(|item| !item.base.disabled));
        let size = self.options.len();
        let focus: Vec<_> = self
            .options
            .iter()
            .map(|item| {
                if let Some(focus) = &item.base.focus {
                    focus.clone()
                } else {
                    let id = ElementId::from((self.id.clone(), "radio-focus"));
                    window.with_id(id, |window| {
                        window
                            .use_keyed_state(item.base.id.clone(), cx, |_, cx| cx.focus_handle())
                            .read(cx)
                            .clone()
                    })
                }
            })
            .collect();
        let navigation: Vec<_> = self
            .options
            .iter()
            .zip(&focus)
            .map(|(item, focus)| {
                let child_select = item
                    .on_select
                    .clone()
                    .map(|handler| bind_activate(handler, window, cx));
                (
                    item.base.id.clone(),
                    focus.clone(),
                    !item.base.disabled,
                    child_select,
                )
            })
            .collect();
        let change = self.on_select.clone().map(|change| {
            let pending = Rc::new(RefCell::new(None::<ElementId>));
            let pending_arg = pending.clone();
            let bound = ThemeScope::bind_action(
                move |window, cx| {
                    if let Some(id) = pending_arg.borrow_mut().take() {
                        change(&id, window, cx);
                    }
                },
                window,
                cx,
            );
            move |id: &ElementId, window: &mut Window, cx: &mut App| {
                *pending.borrow_mut() = Some(id.clone());
                bound(window, cx);
            }
        });
        let selected = self.selected.clone();
        let axis = self.axis;
        let group = BaseRadioGroup::new(self.id.clone())
            .axis(axis)
            .aria_label(self.label)
            .flex()
            .gap_1()
            .when(axis == Axis::Vertical, Styled::flex_col)
            .when(axis == Axis::Horizontal, Styled::flex_wrap)
            .when(
                axis == Axis::Horizontal && direction == Direction::Rtl,
                Styled::flex_row_reverse,
            )
            .on_key_down(move |event, window, cx| {
                let items: Vec<_> = navigation
                    .iter()
                    .map(|(_, focus, enabled, _)| (focus, *enabled))
                    .collect();
                let Some(ix) =
                    move_roving_focus(&items, &event.keystroke.key, axis, direction, window, cx)
                else {
                    return;
                };
                cx.stop_propagation();
                if Some(&navigation[ix].0) != selected.as_ref() {
                    if let Some(child_select) = &navigation[ix].3 {
                        child_select(window, cx);
                    }
                    if let Some(change) = &change {
                        change(&navigation[ix].0, window, cx);
                    }
                }
            });

        group.children(self.options.into_iter().zip(focus).enumerate().map(
            |(ix, (mut radio, focus))| {
                let option_id = radio.base.id.clone();
                radio.base.id = ElementId::from((self.id.clone(), option_id.to_string()));
                radio.selected = Some(&option_id) == self.selected.as_ref();
                let child_handler = radio.on_select.take();
                let group_handler = self.on_select.clone();
                radio.on_select = match (child_handler, group_handler) {
                    (Some(child), Some(group)) => {
                        let callback: crate::action::Activate = Rc::new(move |window, cx| {
                            child(window, cx);
                            group(&option_id, window, cx);
                        });
                        Some(callback)
                    }
                    (Some(child), None) => Some(child),
                    (None, Some(group)) => {
                        let callback: crate::action::Activate =
                            Rc::new(move |window, cx| group(&option_id, window, cx));
                        Some(callback)
                    }
                    (None, None) => None,
                };
                radio.render_control(
                    Some(focus),
                    Some((ix + 1, size)),
                    Some(ix) == tab_stop,
                    window,
                    cx,
                )
            },
        ))
    }
}

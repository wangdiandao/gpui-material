use std::rc::Rc;

use gpui::{
    AccessibleAction, App, Axis, ElementId, Entity, FocusHandle, Hsla, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Radio as BaseRadio, RadioGroup as BaseRadioGroup};
use gpui_material_foundation::{
    InteractionSource, animate, current_material_theme, move_roving_focus,
};
use gpui_material_theme::{ColorRole, Direction, MotionKind, MotionSpeed};

use crate::action::{feedback_target, ripple_state, ripple_target};

use super::{focus_style, row, selection_target, selection_tone, tone};

type RadioChange = Rc<dyn Fn(&ElementId, &mut Window, &mut App)>;

/// Radio indicator pigments. Omitted values use the current window theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RadioColors {
    pub selected: Option<Hsla>,
    pub unselected: Option<Hsla>,
    pub disabled_selected: Option<Hsla>,
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
    id: ElementId,
    label: SharedString,
    selected: bool,
    disabled: bool,
    on_select: Option<crate::action::Activate>,
    colors: RadioColors,
    interaction_source: Option<Entity<InteractionSource>>,
}

impl Radio {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected: false,
            disabled: false,
            on_select: None,
            colors: RadioColors::default(),
            interaction_source: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Overrides indicator colors for the four enabled and selected combinations.
    pub fn colors(mut self, colors: RadioColors) -> Self {
        self.colors = colors;
        self
    }

    /// Observes hover, press, keyboard focus, and disabled state.
    /// Group options are observed under their rendered, group-scoped IDs.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.interaction_source = Some(source.clone());
        self
    }

    fn render_control(
        self,
        focus: Option<&FocusHandle>,
        position: Option<(usize, usize)>,
        tab_stop: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> BaseRadio {
        let motion = current_material_theme(window, cx).motion;
        let reduced = self.disabled || cx.reduce_motion();
        let dot = animate(
            (self.id.clone(), "radio-dot"),
            f32::from(self.selected),
            motion.spec(MotionKind::Spatial, MotionSpeed::Fast, reduced),
            window,
            cx,
        )
        .clamp(0., 1.);
        let tint = animate(
            (self.id.clone(), "radio-color"),
            f32::from(self.selected),
            motion.spec(MotionKind::Effects, MotionSpeed::Fast, reduced),
            window,
            cx,
        )
        .clamp(0., 1.);
        let accent = crate::motion::mix(
            self.colors.resolve(false, self.disabled, window, cx),
            self.colors.resolve(true, self.disabled, window, cx),
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
        let direction = current_material_theme(window, cx).layout.direction;
        let ripple = ripple_state(&self.id, !self.disabled, window, cx);
        let focus = focus.cloned().unwrap_or_else(|| {
            // Match Base's standalone focus ID so changing the source retains focus.
            window
                .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        let tracker = self
            .interaction_source
            .as_ref()
            .map(|source| InteractionSource::track(source, &self.id, window, cx));
        let observed = tracker.is_some();
        ripple
            .borrow_mut()
            .track(!self.disabled, &focus, tracker, window, cx);
        let group: SharedString = format!("material-radio/{}", self.id).into();
        let radio = BaseRadio::new(self.id)
            .when(!self.disabled, |control| {
                if observed {
                    feedback_target(control, ripple.clone(), true, Some(focus.clone()))
                } else {
                    ripple_target(control, ripple.clone())
                }
            })
            .checked(self.selected)
            .disabled(self.disabled)
            .tab_stop(tab_stop)
            .accessibility_label(self.label.clone())
            .group(group.clone())
            .border_2()
            .border_color(gpui::transparent_black())
            .rounded_md()
            .focus_visible(focus_style(tone(ColorRole::Secondary, window, cx)))
            .child(
                row(direction)
                    .child(selection_target(
                        group,
                        indicator,
                        accent,
                        !self.disabled,
                        ripple,
                        direction,
                        cx.reduce_motion(),
                    ))
                    .child(div().min_w_0().truncate().child(self.label)),
            )
            .track_focus(&focus)
            .when_some(position, |radio, (position, size)| {
                radio.set_position(position, size)
            });
        match self.on_select {
            Some(handler) if !self.disabled => {
                let select = handler.clone();
                radio
                    .when(!self.selected, |radio| {
                        radio.on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                            select(window, cx)
                        })
                    })
                    .on_change(move |_, _, window, cx| handler(window, cx))
            }
            _ => radio,
        }
    }
}

impl RenderOnce for Radio {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.render_control(None, None, true, window, cx)
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

    pub fn selected(mut self, selected: impl Into<ElementId>) -> Self {
        self.selected = Some(selected.into());
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = Radio>) -> Self {
        self.options.extend(options);
        self
    }

    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

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
            .position(|item| !item.disabled && Some(&item.id) == self.selected.as_ref());
        let tab_stop = current.or_else(|| self.options.iter().position(|item| !item.disabled));
        let size = self.options.len();
        let focus: Vec<_> = self
            .options
            .iter()
            .map(|item| {
                let id = ElementId::from((self.id.clone(), "radio-focus"));
                window.with_id(id, |window| {
                    window
                        .use_keyed_state(item.id.clone(), cx, |_, cx| cx.focus_handle())
                        .read(cx)
                        .clone()
                })
            })
            .collect();
        let navigation: Vec<_> = self
            .options
            .iter()
            .zip(&focus)
            .map(|(item, focus)| (item.id.clone(), focus.clone(), !item.disabled))
            .collect();
        let change = self.on_select.clone();
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
                    .map(|(_, focus, enabled)| (focus, *enabled))
                    .collect();
                let Some(ix) =
                    move_roving_focus(&items, &event.keystroke.key, axis, direction, window, cx)
                else {
                    return;
                };
                cx.stop_propagation();
                if let Some(change) = &change
                    && Some(&navigation[ix].0) != selected.as_ref()
                {
                    change(&navigation[ix].0, window, cx);
                }
            });

        group.children(self.options.into_iter().zip(&focus).enumerate().map(
            |(ix, (mut radio, focus))| {
                let option_id = radio.id.clone();
                radio.id = ElementId::from((self.id.clone(), option_id.to_string()));
                radio.selected = Some(&option_id) == self.selected.as_ref();
                let handler = self.on_select.clone();
                radio.on_select = handler.map(|handler| {
                    let callback: crate::action::Activate =
                        Rc::new(move |window, cx| handler(&option_id, window, cx));
                    callback
                });
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

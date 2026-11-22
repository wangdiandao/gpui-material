use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, BorderStyle, ClickEvent, Corners, Edges, ElementId, Entity,
    FocusHandle, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, Role,
    SharedString, StatefulInteractiveElement, Styled, Window, canvas, div, prelude::FluentBuilder,
    px, quad,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{
    InteractionFeedback, InteractionSource, InteractionState, ThemeScope, animate, color,
    current_material_theme, elevation_shadow_pair,
};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Direction, Elevation, MotionKind, MotionSpec, MotionSpeed,
};

use crate::{
    action::{Activate, activate_feedback, feedback_target, ripple_state},
    button_style::finite_length,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CardVariant {
    Elevated,
    #[default]
    Filled,
    Outlined,
}

/// The card's stroke. A zero-width border removes the stroke.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardBorder {
    pub width: Pixels,
    pub color: Hsla,
}

/// Per-instance colors. Omitted fields keep their variant and window defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CardColors {
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub disabled_container: Option<Hsla>,
    pub disabled_content: Option<Hsla>,
}

/// Shadow levels. Each omitted field keeps the variant default for its state.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CardElevations {
    pub resting: Option<Elevation>,
    pub pressed: Option<Elevation>,
    pub focused: Option<Elevation>,
    pub hovered: Option<Elevation>,
    pub dragged: Option<Elevation>,
    pub disabled: Option<Elevation>,
}

impl CardElevations {
    fn resolve(self, variant: CardVariant, state: InteractionState) -> Elevation {
        let (resting, hovered, dragged) = match variant {
            CardVariant::Elevated => (Elevation::Level1, Elevation::Level2, Elevation::Level4),
            CardVariant::Filled => (Elevation::Level0, Elevation::Level1, Elevation::Level3),
            CardVariant::Outlined => (Elevation::Level0, Elevation::Level0, Elevation::Level3),
        };
        if state.disabled {
            self.disabled.unwrap_or(resting)
        } else if state.dragged {
            self.dragged.unwrap_or(dragged)
        } else if state.pressed {
            self.pressed.unwrap_or(resting)
        } else if state.focus_visible {
            self.focused.unwrap_or(resting)
        } else if state.hovered {
            self.hovered.unwrap_or(hovered)
        } else {
            self.resting.unwrap_or(resting)
        }
    }
}

/// A static surface or a keyboard-operable action with independent child controls.
#[derive(IntoElement)]
pub struct Card {
    id: ElementId,
    variant: CardVariant,
    children: Vec<AnyElement>,
    activation: Option<(SharedString, Activate)>,
    accessible_label: Option<SharedString>,
    disabled: bool,
    dragged: bool,
    shape: CornerShape,
    border: Option<CardBorder>,
    colors: CardColors,
    elevations: CardElevations,
    content_padding: Edges<Pixels>,
    gap: Pixels,
    focus: Option<FocusHandle>,
    source: Option<Entity<InteractionSource>>,
}

impl Card {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: CardVariant::default(),
            children: Vec::new(),
            activation: None,
            accessible_label: None,
            disabled: false,
            dragged: false,
            shape: CornerRole::Medium.shape(),
            border: None,
            colors: CardColors::default(),
            elevations: CardElevations::default(),
            content_padding: Edges::all(px(16.)),
            gap: px(8.),
            focus: None,
            source: None,
        }
    }

    /// Creates an Elevated card.
    pub fn elevated(id: impl Into<ElementId>) -> Self {
        Self::new(id).variant(CardVariant::Elevated)
    }

    /// Creates a Filled card.
    pub fn filled(id: impl Into<ElementId>) -> Self {
        Self::new(id).variant(CardVariant::Filled)
    }

    /// Creates an Outlined card.
    pub fn outlined(id: impl Into<ElementId>) -> Self {
        Self::new(id).variant(CardVariant::Outlined)
    }

    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_activate(
        mut self,
        name: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.activation = Some((name.into(), Rc::new(handler)));
        self
    }

    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn border(mut self, border: CardBorder) -> Self {
        self.border = Some(border);
        self
    }

    pub fn colors(mut self, colors: CardColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn elevations(mut self, elevations: CardElevations) -> Self {
        self.elevations = elevations;
        self
    }

    /// Sets the card's inner padding in logical pixels. Defaults to 16 dp on all sides.
    pub fn content_padding(mut self, padding: Edges<Pixels>) -> Self {
        self.content_padding = Edges {
            top: finite_length(padding.top),
            right: finite_length(padding.right),
            bottom: finite_length(padding.bottom),
            left: finite_length(padding.left),
        };
        self
    }

    /// Sets the vertical gap between child elements. Defaults to 8 dp.
    pub fn gap(mut self, gap: Pixels) -> Self {
        self.gap = if f32::from(gap).is_finite() && gap >= px(0.) {
            gap
        } else {
            px(8.)
        };
        self
    }

    /// Names a static (`Role::Group`) card for assistive technology.
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    /// Displays a caller-owned drag. Clear the flag after drop or cancellation.
    pub fn dragged(mut self, dragged: bool) -> Self {
        self.dragged = dragged;
        self
    }

    /// Uses the caller's focus handle when the card is actionable.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.source = Some(source.clone());
        self
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Card {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let actionable = self.activation.is_some();
        let enabled = !self.disabled;
        let dragged = enabled && self.dragged;
        let feedback = ripple_state(&self.id, actionable && enabled, window, cx);
        let focus = self.focus.filter(|_| actionable).unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "card-focus"), cx, |_, cx| {
                    cx.focus_handle()
                })
                .read(cx)
                .clone()
        });
        let tracker = self
            .source
            .as_ref()
            .map(|source| InteractionSource::track(source, &self.id, window, cx));
        let interaction = {
            let mut feedback = feedback.borrow_mut();
            feedback.set_dragged(dragged);
            feedback.track(enabled, &focus, tracker, window, cx)
        };

        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let reduced = cx.reduce_motion();
        let tone = |role| color(theme.colors.color(role));
        let container = match self.variant {
            CardVariant::Elevated => ColorRole::SurfaceContainerLow,
            CardVariant::Filled => ColorRole::SurfaceContainerHighest,
            CardVariant::Outlined => ColorRole::Surface,
        };
        let background = if self.disabled {
            self.colors
                .disabled_container
                .unwrap_or_else(|| match self.variant {
                    CardVariant::Filled => {
                        tone(container).blend(tone(ColorRole::SurfaceVariant).opacity(0.38))
                    }
                    _ => tone(ColorRole::Surface),
                })
        } else {
            self.colors.container.unwrap_or_else(|| tone(container))
        };
        let foreground = if self.disabled {
            self.colors
                .disabled_content
                .unwrap_or_else(|| tone(ColorRole::OnSurface).opacity(0.38))
        } else {
            self.colors
                .content
                .unwrap_or_else(|| tone(ColorRole::OnSurface))
        };
        let border = self.border.unwrap_or_else(|| CardBorder {
            // Keep the default card content inset across variants.
            width: px(1.),
            color: if self.variant != CardVariant::Outlined {
                gpui::transparent_black()
            } else if self.disabled {
                tone(ColorRole::SurfaceContainerLow).blend(tone(ColorRole::Outline).opacity(0.12))
            } else {
                tone(ColorRole::OutlineVariant)
            },
        });
        let border_width = finite_length(border.width);
        let border_color = if border_width > px(0.) {
            border.color
        } else {
            gpui::transparent_black()
        };
        let ring = tone(ColorRole::Secondary);
        let shadow = tone(ColorRole::Shadow);
        let elevation_motion = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let layer_motion = theme
            .motion
            .spec(MotionKind::Effects, MotionSpeed::Fast, reduced);
        let elevation = animate(
            (self.id.clone(), "card-shadow"),
            self.elevations.resolve(self.variant, interaction).dp(),
            if enabled {
                elevation_motion
            } else {
                MotionSpec::Instant
            },
            window,
            cx,
        )
        .max(0.);
        let layer = animate(
            (self.id.clone(), "card-state-layer"),
            interaction
                .layer_with_ripple(reduced)
                .map_or(0., |layer| layer.opacity()),
            if enabled {
                layer_motion
            } else {
                MotionSpec::Instant
            },
            window,
            cx,
        )
        .clamp(0., 1.);
        let geometry = feedback.clone();
        let paint_feedback = feedback.clone();
        let shape = self.shape;
        let visual = canvas(
            move |bounds, window, _| {
                if enabled && (actionable || dragged) {
                    geometry
                        .borrow_mut()
                        .set_rounded_shape(bounds, shape, direction, window);
                } else {
                    geometry.borrow_mut().set_shape(None, window);
                }
                let [top_left, top_right, bottom_right, bottom_left] = shape
                    .radii(
                        f32::from(bounds.size.width),
                        f32::from(bounds.size.height),
                        direction,
                    )
                    .map(px);
                Corners {
                    top_left,
                    top_right,
                    bottom_right,
                    bottom_left,
                }
            },
            move |bounds, corners, window, cx| {
                if let Some(shadows) = elevation_shadow_pair(elevation, 1., shadow) {
                    window.paint_drop_shadows(bounds, corners, &shadows);
                }
                window.paint_quad(quad(
                    bounds,
                    corners,
                    background,
                    border_width,
                    border_color,
                    BorderStyle::Solid,
                ));
                let feedback = paint_feedback.borrow();
                if let Some(shape) = &feedback.shape {
                    InteractionFeedback::new(interaction, foreground, ring).paint_with_opacity(
                        shape,
                        feedback.ripple.as_ref(),
                        layer,
                        cx.background_executor().now(),
                        reduced,
                        window,
                    );
                }
            },
        )
        .absolute()
        .inset_0()
        .size_full();
        let content = apply_corner_shape(
            div()
                .relative()
                .w_full()
                .overflow_hidden()
                .text_color(foreground)
                .pt(self.content_padding.top + border_width)
                .pr(self.content_padding.right + border_width)
                .pb(self.content_padding.bottom + border_width)
                .pl(self.content_padding.left + border_width)
                .flex()
                .flex_col()
                .gap(self.gap),
            self.shape,
            direction,
        )
        .children(self.children);
        let surface = div()
            .id("card-surface")
            .relative()
            .w_full()
            .child(visual)
            .child(content);
        match self.activation {
            Some((name, activate)) => {
                let activate = Rc::new(ThemeScope::bind_action(
                    move |window, cx| activate(window, cx),
                    window,
                    cx,
                ));
                let invoke = activate.clone();
                let press = feedback.clone();
                gpui_base::Button::new(self.id)
                    .track_focus(&focus)
                    .tab_stop(true)
                    .disabled(self.disabled)
                    .w_full()
                    .border_2()
                    .border_color(gpui::transparent_black())
                    .accessibility_label(name)
                    .when(enabled && !dragged, |control| {
                        feedback_target(control, feedback, true, Some(focus))
                    })
                    .when(enabled, |control| {
                        control
                            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                if !dragged {
                                    invoke(window, cx);
                                }
                            })
                            .on_click(move |event, window, cx| {
                                cx.stop_propagation();
                                let native_drag =
                                    matches!(event, ClickEvent::Mouse(_)) && cx.has_active_drag();
                                if !native_drag && !press.borrow().blocks_activation(event) {
                                    activate_feedback(event, &press, window, cx);
                                    activate(window, cx);
                                }
                            })
                    })
                    .child(surface)
                    .into_any_element()
            }
            None => div()
                .id(self.id)
                .test_support()
                .role(Role::Group)
                .when_some(self.accessible_label, |card, label| card.aria_label(label))
                .child(surface)
                .into_any_element(),
        }
    }
}

fn apply_corner_shape<E: Styled>(element: E, shape: CornerShape, direction: Direction) -> E {
    if shape == CornerShape::Full {
        element.rounded_full()
    } else {
        let [tl, tr, br, bl] = shape.radii(f32::MAX, f32::MAX, direction);
        element
            .rounded_tl(px(tl))
            .rounded_tr(px(tr))
            .rounded_br(px(br))
            .rounded_bl(px(bl))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Elevation::*;

    #[test]
    fn elevation_overrides_resolve_each_state_and_precedence() {
        let levels = CardElevations {
            resting: Some(Level0),
            hovered: Some(Level1),
            focused: Some(Level2),
            pressed: Some(Level3),
            dragged: Some(Level4),
            disabled: Some(Level5),
        };
        for variant in [
            CardVariant::Elevated,
            CardVariant::Filled,
            CardVariant::Outlined,
        ] {
            let mut state = InteractionState::default();
            assert_eq!(levels.resolve(variant, state), Level0);
            state.hovered = true;
            assert_eq!(levels.resolve(variant, state), Level1);
            state.focus_visible = true;
            assert_eq!(levels.resolve(variant, state), Level2);
            state.pressed = true;
            assert_eq!(levels.resolve(variant, state), Level3);
            state.dragged = true;
            assert_eq!(levels.resolve(variant, state), Level4);
            state.disabled = true;
            assert_eq!(levels.resolve(variant, state), Level5);
        }
    }

    #[test]
    fn omitted_elevations_keep_the_pinned_variant_values() {
        let levels = CardElevations::default();
        for (variant, resting, hovered, dragged) in [
            (CardVariant::Elevated, Level1, Level2, Level4),
            (CardVariant::Filled, Level0, Level1, Level3),
            (CardVariant::Outlined, Level0, Level0, Level3),
        ] {
            let mut state = InteractionState::default();
            assert_eq!(levels.resolve(variant, state), resting);
            state.hovered = true;
            assert_eq!(levels.resolve(variant, state), hovered);
            state.focus_visible = true;
            assert_eq!(levels.resolve(variant, state), resting);
            state.pressed = true;
            assert_eq!(levels.resolve(variant, state), resting);
            state.dragged = true;
            assert_eq!(levels.resolve(variant, state), dragged);
            state.disabled = true;
            assert_eq!(levels.resolve(variant, state), resting);
        }
    }

    #[test]
    fn card_variant_shorthands_set_expected_variants() {
        assert_eq!(Card::elevated("c1").variant, CardVariant::Elevated);
        assert_eq!(Card::filled("c2").variant, CardVariant::Filled);
        assert_eq!(Card::outlined("c3").variant, CardVariant::Outlined);
    }
}

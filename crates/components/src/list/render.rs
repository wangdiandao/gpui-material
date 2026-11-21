use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    AccessibleAction, AnyElement, App, BorderStyle, ClickEvent, Corners, Div, ElementId,
    FocusHandle, Hsla, InteractiveElement, IntoElement, Keystroke, ParentElement, Pixels, Role,
    SharedString, Stateful, StatefulInteractiveElement, Styled, Window, accesskit::Toggled, canvas,
    div, prelude::FluentBuilder, px, quad,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{
    InteractionFeedback, InteractionSource, InteractionState, ThemeScope, animate, color,
    current_material_theme, elevation_shadow_pair,
};
use gpui_material_theme::{
    ColorRole, CornerShape, Direction, MotionKind, MotionSpec, MotionSpeed, TypeRole,
};

use crate::{
    Button, ButtonVariant, ControlSize, SwipeToDismiss, Text,
    action::{Activate, FeedbackState, activate_feedback, feedback_target, ripple_state},
    button_style::finite_length,
};

use super::long_press::LongPress;
use super::style::{ListItemAlignment, ListItemPadding, ListItemShapes, RowColors};
use super::{Expansion, ListItem, SecondaryAction, Slot, ValueChange};

impl Slot {
    fn render(self, id: &'static str, role: TypeRole, tint: Hsla, wrap: bool) -> AnyElement {
        let content = match self {
            Self::Text(value) => Text::new(value)
                .id(id)
                .role(role)
                .when(wrap, Text::wrap)
                .into_any_element(),
            Self::Custom(element) => element,
        };
        inherit_color((ElementId::from(id), "slot"), content, Some(tint))
    }
}

#[allow(clippy::too_many_arguments)]
fn render_visual_canvas(
    radii: [f32; 4],
    full: f32,
    direction: Direction,
    enabled: bool,
    elevation: f32,
    shadow: Hsla,
    container: Hsla,
    content_tint: Hsla,
    ring: Hsla,
    interaction: InteractionState,
    layer: f32,
    reduced: bool,
    feedback: &Rc<RefCell<FeedbackState>>,
) -> impl IntoElement {
    let geometry = feedback.clone();
    let ink = feedback.clone();
    canvas(
        move |bounds, window, _| {
            let extent = f32::from(bounds.size.width.min(bounds.size.height)) * 0.5;
            let shape = CornerShape::Radii(radii.map(|radius| radius + (extent - radius) * full));
            let [top_left, top_right, bottom_right, bottom_left] = shape
                .radii(
                    bounds.size.width.into(),
                    bounds.size.height.into(),
                    direction,
                )
                .map(px);
            if enabled {
                geometry
                    .borrow_mut()
                    .set_rounded_shape(bounds, shape, direction, window);
            } else {
                geometry.borrow_mut().set_shape(None, window);
            }
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
                container,
                px(0.),
                gpui::transparent_black(),
                BorderStyle::Solid,
            ));
            let feedback = ink.borrow();
            if let Some(shape) = &feedback.shape {
                // Base's outer ring keeps the existing list focus geometry.
                let mut visual_interaction = interaction;
                visual_interaction.focus_visible = false;
                InteractionFeedback::new(visual_interaction, content_tint, ring)
                    .paint_with_opacity(
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
}

#[allow(clippy::too_many_arguments)]
fn render_content_slot(
    label: SharedString,
    content: Option<AnyElement>,
    overline: Option<Slot>,
    supporting: Option<Slot>,
    colors: &RowColors,
    long_press: Option<&Rc<LongPress>>,
    enabled: bool,
    dragged: bool,
    text_height: Option<Rc<Cell<Pixels>>>,
) -> Div {
    div()
        .relative()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .overflow_hidden()
        .text_color(colors.content)
        .children(
            overline
                .map(|slot| slot.render("overline", TypeRole::LabelSmall, colors.overline, false)),
        )
        .child(
            div()
                .id("headline-slot")
                .child(content.unwrap_or_else(|| Text::new(label).id("label").into_any_element())),
        )
        .children(
            supporting.map(|slot| {
                slot.render("supporting", TypeRole::BodyMedium, colors.supporting, true)
            }),
        )
        .children(
            long_press
                .filter(|_| enabled && !dragged)
                .map(|press| press.target()),
        )
        .when_some(text_height, |slot, text_height| {
            slot.child(
                canvas(
                    move |bounds, window, _| {
                        if text_height.replace(bounds.size.height) != bounds.size.height {
                            window.refresh();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            )
        })
}

#[allow(clippy::too_many_arguments)]
fn render_surface(
    height: f32,
    three_lines: bool,
    alignment: ListItemAlignment,
    padding: Option<ListItemPadding>,
    layout: gpui_material_theme::LayoutPolicy,
    visual: impl IntoElement,
    leading: Option<AnyElement>,
    content: Div,
    trailing: Option<AnyElement>,
    long_press: Option<&SecondaryAction>,
    long_press_button: Option<ElementId>,
    colors: &RowColors,
    enabled: bool,
    dragged: bool,
) -> Stateful<Div> {
    let padding = padding.unwrap_or_else(|| {
        let vertical = px(if three_lines { 12. } else { 10. });
        ListItemPadding {
            top: vertical,
            bottom: vertical,
            ..ListItemPadding::default()
        }
    });
    let (left, right) = layout.horizontal(finite_length(padding.start), finite_length(padding.end));
    let press_button = long_press.zip(long_press_button).map(|(press, id)| {
        let invoke = press.handler.clone();
        Button::new(id, press.label.clone())
            .variant(ButtonVariant::Text)
            .size(ControlSize::ExtraSmall)
            .disabled(!enabled || dragged)
            .on_activate(move |window, cx| invoke(window, cx))
    });
    let alignment = if alignment == ListItemAlignment::Auto {
        if three_lines {
            ListItemAlignment::Start
        } else {
            ListItemAlignment::Center
        }
    } else {
        alignment
    };
    div()
        .id("list-surface")
        .relative()
        .min_h(px(height))
        .w_full()
        .flex()
        .map(|row| match alignment {
            ListItemAlignment::Start => row.items_start(),
            ListItemAlignment::End => row.items_end(),
            _ => row.items_center(),
        })
        .gap(px(12.))
        .pl(left)
        .pr(right)
        .pt(finite_length(padding.top))
        .pb(finite_length(padding.bottom))
        .when(layout.direction == Direction::Rtl, Styled::flex_row_reverse)
        .child(visual)
        .children(leading.map(|element| inherit_color("leading-slot", element, colors.leading)))
        .child(content)
        .children(trailing.map(|element| inherit_color("trailing-slot", element, colors.trailing)))
        .children(press_button.map(|button| div().id("long-press-slot").child(button)))
}

fn render_expanded_content(
    expansion: Option<Expansion>,
    interactive: bool,
    enabled: bool,
    focus: &FocusHandle,
    window: &mut Window,
    cx: &mut App,
) -> Option<Stateful<Div>> {
    expansion.and_then(|expansion| {
        let scope = window
            .use_keyed_state("disclosure-focus", cx, |_, cx| {
                cx.focus_handle().tab_stop(false)
            })
            .read(cx)
            .clone();
        if !expansion.expanded && interactive && enabled && scope.contains_focused(window, cx) {
            focus.focus(window, cx);
        }
        expansion.expanded.then(|| {
            div()
                .id("list-expanded-content")
                .track_focus(&scope)
                .child(expansion.content)
        })
    })
}

#[allow(clippy::too_many_arguments)]
fn wrap_interactive_row(
    id: &ElementId,
    label: SharedString,
    surface: Stateful<Div>,
    focus: FocusHandle,
    tab_stop: bool,
    selected: bool,
    enabled: bool,
    dragged: bool,
    checked: Option<Toggled>,
    checked_role: Role,
    expanded: Option<bool>,
    menu_a11y_toggle: Option<Activate>,
    ring: Hsla,
    feedback: Rc<RefCell<FeedbackState>>,
    primary: Option<Activate>,
    expanded_change: Option<ValueChange>,
    secondary_action: Option<SecondaryAction>,
    long_press: Option<Rc<LongPress>>,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let activate = feedback.clone();
    let mut control = gpui_base::Button::new((id.clone(), "activate"))
        .track_focus(&focus)
        .tab_stop(tab_stop)
        .w_full()
        .accessibility_label(label)
        .disabled(!enabled)
        .selected(selected)
        .when_some(checked, |control, checked| {
            control.role(checked_role).aria_toggled(checked)
        })
        .when_some(expanded, |control, expanded| {
            control.aria_expanded(expanded)
        })
        .border_2()
        .border_color(gpui::transparent_black())
        .rounded_md()
        .focus_visible(move |style| style.border_color(ring))
        .when(enabled && !dragged, |control| {
            feedback_target(control, feedback, true, Some(focus.clone()))
        });
    if let Some(toggle) = menu_a11y_toggle.filter(|_| enabled && !dragged) {
        let return_focus = focus.clone();
        control = control.on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
            return_focus.focus(window, cx);
            toggle(window, cx);
        });
    } else if let Some(primary) = primary.filter(|_| enabled && !dragged) {
        let invoke = primary.clone();
        let hold = long_press.clone();
        control = control
            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                invoke(window, cx)
            })
            .on_click(move |event, window, cx| {
                cx.stop_propagation();
                let native_drag = matches!(event, ClickEvent::Mouse(_)) && cx.has_active_drag();
                let allowed = !native_drag
                    && !activate.borrow().blocks_activation(event)
                    && !hold
                        .as_ref()
                        .is_some_and(|hold| hold.blocks_activation(event));
                if allowed || matches!(event, ClickEvent::Mouse(_)) {
                    activate_feedback(event, &activate, window, cx);
                }
                if allowed {
                    primary(window, cx);
                }
            });
    }
    if let Some((expanded, change)) = expanded
        .zip(expanded_change)
        .filter(|_| enabled && !dragged)
    {
        let change =
            ThemeScope::bind_action(move |window, cx| change(!expanded, window, cx), window, cx);
        control = control.on_a11y_action(
            if expanded {
                AccessibleAction::Collapse
            } else {
                AccessibleAction::Expand
            },
            move |_, window, cx| change(window, cx),
        );
    }
    if let Some((secondary, hold)) = secondary_action
        .zip(long_press)
        .filter(|_| enabled && !dragged)
    {
        let invoke = secondary.handler.clone();
        control = control
            .aria_description(secondary.label)
            .on_a11y_action(AccessibleAction::ShowContextMenu, move |_, window, cx| {
                invoke(window, cx)
            })
            .on_key_down(move |event, window, cx| {
                if !focus.is_focused(window) {
                    return;
                }
                if event.keystroke.key == "escape" {
                    hold.cancel();
                }
                if is_long_press_shortcut(&event.keystroke) {
                    cx.stop_propagation();
                    if !event.is_held {
                        (secondary.handler)(window, cx);
                    }
                }
            });
    }
    control.child(surface).into_any_element()
}

fn wrap_static_row(
    surface: Stateful<Div>,
    disabled: bool,
    checked: Option<Toggled>,
    checked_role: Role,
    expanded: Option<bool>,
) -> AnyElement {
    surface
        .when_some(checked, |row, checked| {
            row.role(checked_role).aria_toggled(checked)
        })
        .when_some(expanded, |row, expanded| {
            row.when(checked.is_none(), |row| row.role(Role::Group))
                .aria_expanded(expanded)
        })
        .when(
            disabled && (checked.is_some() || expanded.is_some()),
            |row| row.a11y_synthetic_children(|tree| tree.parent_node().set_disabled()),
        )
        .into_any_element()
}

impl ListItem {
    pub(super) fn render_row(
        self,
        resting: CornerShape,
        selected_shape: CornerShape,
        focus: Option<(FocusHandle, bool)>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let (focus, tab_stop) = focus.unwrap_or_else(|| {
            let focus = window
                .use_keyed_state("focus", cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone();
            (focus, true)
        });
        let primary = self.primary_action().map(|primary| {
            let bound: Activate = Rc::new(ThemeScope::bind_action(
                move |window, cx| primary(window, cx),
                window,
                cx,
            ));
            bound
        });
        let has_menu_trigger = self.has_menu_trigger();
        let secondary_action = self.long_press.map(|mut action| {
            let handler = action.handler;
            action.handler = Rc::new(ThemeScope::bind_action(
                move |window, cx| handler(window, cx),
                window,
                cx,
            ));
            action
        });
        let interactive = primary.is_some() || has_menu_trigger || secondary_action.is_some();
        let visible = self
            .swipe
            .as_ref()
            .is_none_or(|swipe| swipe.state.read(cx).shows_foreground());
        let enabled = !self.disabled && visible;
        let dragged = enabled
            && (self.dragged
                || self
                    .swipe
                    .as_ref()
                    .is_some_and(|swipe| swipe.state.read(cx).is_dragging()));
        let feedback = ripple_state(&self.id, interactive && enabled, window, cx);
        let tracker = self
            .source
            .as_ref()
            .filter(|_| visible)
            .map(|source| InteractionSource::track(source, &self.id, window, cx));
        let interaction = {
            let mut feedback = feedback.borrow_mut();
            feedback.set_dragged(dragged);
            feedback.track(enabled, &focus, tracker, window, cx)
        };

        let long_press = secondary_action.as_ref().map(|long_press| {
            LongPress::mount(
                (enabled && !dragged).then(|| long_press.handler.clone()),
                window,
                cx,
            )
        });
        let (layout, motion) = {
            let theme = current_material_theme(window, cx);
            (theme.layout, theme.motion)
        };
        let reduced = cx.reduce_motion();
        let selected =
            self.selected || (self.checked == Some(true) && self.checked_role != Role::Switch);
        let shape = self
            .shapes
            .resolve(resting, selected_shape, selected, interaction);
        let effects = motion.spec(MotionKind::Effects, MotionSpeed::Default, reduced);
        let spatial = motion.spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let selection =
            animate("selection", f32::from(selected), effects, window, cx).clamp(0., 1.);
        let (ring, shadow, colors) = {
            let scheme = &current_material_theme(window, cx).colors;
            let tone = |role| color(scheme.color(role));
            (
                tone(ColorRole::Secondary),
                tone(ColorRole::Shadow),
                self.colors.resolve(selected, selection, interaction, tone),
            )
        };
        let layer = animate(
            "opacity",
            interaction
                .layer_with_ripple(reduced)
                .map_or(0., |layer| layer.opacity()),
            if enabled {
                effects
            } else {
                MotionSpec::Instant
            },
            window,
            cx,
        )
        .clamp(0., 1.);
        let elevation = animate(
            "elevation",
            self.elevations.resolve(interaction).dp(),
            if enabled {
                spatial
            } else {
                MotionSpec::Instant
            },
            window,
            cx,
        )
        .max(0.);
        let static_single_line =
            self.overline.is_none() && self.supporting.is_none() && self.content.is_none();
        let static_three_lines = (self.overline.is_some() && self.supporting.is_some())
            || self.supporting.as_ref().is_some_and(Slot::has_line_break);
        let text_height = (!static_single_line && !static_three_lines).then(|| {
            window
                .use_keyed_state("text-height", cx, |_, _| Rc::new(Cell::new(px(0.))))
                .read(cx)
                .clone()
        });
        let three_lines = static_three_lines
            || text_height
                .as_ref()
                .is_some_and(|text_height| text_height.get() > px(44.));
        let height = match (self.overline.is_some(), self.supporting.is_some()) {
            _ if three_lines => 88.,
            (false, false) => 56.,
            _ => 72.,
        };
        let radii = match shape {
            CornerShape::Radii(radii) => radii,
            CornerShape::Full => [height * 0.5; 4],
        };
        let (full, radii) = if resting == selected_shape && self.shapes == ListItemShapes::default()
        {
            (f32::from(shape == CornerShape::Full), radii)
        } else {
            // Include the measured pill extent separately so the final shape follows
            // custom content that raises the row above its minimum height.
            let full = animate(
                "full",
                f32::from(shape == CornerShape::Full),
                spatial,
                window,
                cx,
            )
            .clamp(0., 1.);
            let radii = std::array::from_fn(|ix| {
                animate(["ts", "te", "be", "bs"][ix], radii[ix], spatial, window, cx).max(0.)
            });
            (full, radii)
        };
        let visual = render_visual_canvas(
            radii,
            full,
            layout.direction,
            enabled,
            elevation,
            shadow,
            colors.container,
            colors.content,
            ring,
            interaction,
            layer,
            reduced,
            &feedback,
        );
        let content = render_content_slot(
            self.label.clone(),
            self.content,
            self.overline,
            self.supporting,
            &colors,
            long_press.as_ref(),
            enabled,
            dragged,
            text_height,
        );
        let surface = render_surface(
            height,
            three_lines,
            self.alignment,
            self.padding,
            layout,
            visual,
            self.leading,
            content,
            self.trailing,
            secondary_action.as_ref(),
            self.long_press_button,
            &colors,
            enabled,
            dragged,
        );
        #[cfg(feature = "feedback")]
        let (menu_open, menu_a11y_toggle) = match self.menu_trigger {
            Some((open, toggle)) => (Some(open), Some(toggle)),
            None => (None, None),
        };
        #[cfg(not(feature = "feedback"))]
        let (menu_open, menu_a11y_toggle): (Option<bool>, Option<Activate>) = (None, None);
        let expanded =
            menu_open.or_else(|| self.expansion.as_ref().map(|expansion| expansion.expanded));
        let checked = self.checked.map(|checked| {
            if checked {
                Toggled::True
            } else {
                Toggled::False
            }
        });
        let expanded_content =
            render_expanded_content(self.expansion, interactive, enabled, &focus, window, cx);
        let row = if interactive {
            wrap_interactive_row(
                &self.id,
                self.label.clone(),
                surface,
                focus,
                tab_stop,
                self.selected,
                enabled,
                dragged,
                checked,
                self.checked_role,
                expanded,
                menu_a11y_toggle,
                ring,
                feedback,
                primary,
                self.expanded_change,
                secondary_action,
                long_press,
                window,
                cx,
            )
        } else {
            wrap_static_row(surface, self.disabled, checked, self.checked_role, expanded)
        };
        let id = self.id;
        let row = div()
            .id(id.clone())
            .role(Role::ListItem)
            .test_support()
            .aria_selected(self.selected)
            .when(!interactive, |row| row.aria_label(self.label))
            .flex()
            .flex_col()
            .w_full()
            .child(row)
            .children(expanded_content);
        match self.swipe {
            Some(swipe) => (swipe.configure)(SwipeToDismiss::new((id, "swipe"), &swipe.state, row))
                .when(self.disabled, |swipe| swipe.gestures_enabled(false))
                .into_any_element(),
            None => row.into_any_element(),
        }
    }
}

fn inherit_color(id: impl Into<ElementId>, element: AnyElement, color: Option<Hsla>) -> AnyElement {
    // Each slot owns a separate identity scope, including elements that use
    // implicit IDs such as Text and Icon.
    div()
        .id(id)
        .when_some(color, Styled::text_color)
        .child(element)
        .into_any_element()
}

fn is_long_press_shortcut(keystroke: &Keystroke) -> bool {
    let modifiers = keystroke.modifiers;
    keystroke.key == "f10"
        && modifiers.shift
        && !modifiers.control
        && !modifiers.alt
        && !modifiers.platform
}

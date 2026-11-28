use std::{cell::Cell, rc::Rc};

use gpui::{
    AccessibleAction, Anchor, App, Bounds, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, RenderOnce, Role, StatefulInteractiveElement, Styled, Window, canvas,
    deferred, div, point, prelude::FluentBuilder, px,
};
use gpui_base::{Placement, TestSupportExt};
use gpui_material_foundation::{OverlayMotion, ScaledOverlay, ThemeScope, current_material_theme};

use super::super::modal::ZLayer;
use super::interaction::{Dismiss, Interaction};
use super::surface::tooltip_surface;
use super::{DismissTooltip, FocusTooltipAction, FocusTooltipTrigger, Tooltip, TooltipPositioner};

#[derive(Clone, Default)]
struct Geometry {
    anchor: Rc<Cell<Option<Bounds<Pixels>>>>,
    placed: Rc<Cell<Bounds<Pixels>>>,
    side: Rc<Cell<Option<Placement>>>,
}

impl RenderOnce for Tooltip {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let label = self.label_visible.is_some();
        let state = window.use_keyed_state(
            (self.id.clone(), "tooltip-interaction"),
            cx,
            Interaction::new,
        );
        let geometry = window
            .use_keyed_state((self.id.clone(), "tooltip-geometry"), cx, |_, _| {
                Geometry::default()
            })
            .read(cx)
            .clone();
        let transient = !label && self.enable_user_input && self.persistent.is_none();
        state.update(cx, |state, cx| {
            state.configure(
                transient,
                (self.show_delay, self.hide_delay),
                self.trigger_focus.as_ref(),
                self.persistent.as_ref(),
                window,
                cx,
            )
        });
        let open = self.label_visible.unwrap_or_else(|| {
            self.persistent.as_ref().map_or_else(
                || state.read(cx).is_open(window, cx),
                |state| state.read(cx).is_visible(),
            )
        });
        let surface = state.read(cx).surface.clone();
        let first = self.focus_target.clone().or_else(|| {
            self.action
                .as_ref()
                .filter(|_| self.action_content.is_none())
                .map(|action| action.focus.clone())
        });
        let anchor = geometry.anchor.get();
        state.update(cx, |state, cx| {
            state.present(
                !label && open && anchor.is_some(),
                self.trigger_focus.as_ref(),
                self.focusable
                    .then(|| first.clone().unwrap_or_else(|| surface.clone())),
                window,
                cx,
            )
        });

        let dismiss = Dismiss {
            transient: state.clone(),
            persistent: self.persistent.clone(),
            request: self.on_dismiss_request.clone(),
        };
        let on_escape = dismiss.clone();
        let hover = state.clone();
        let measured = geometry.anchor.clone();
        // Avoid setting an outer `Role::Group` wrapper around the trigger so
        // Windows UIA does not emit a duplicate container node around controls.
        let trigger = div()
            .id(if label {
                "label-anchor"
            } else {
                "tooltip-trigger"
            })
            .aria_description(self.text.clone())
            .when(self.persistent.is_some(), |trigger| {
                trigger.aria_expanded(open)
            })
            .when(open && !label, |trigger| {
                trigger.when_some(first.clone(), |trigger, first| {
                    trigger.key_context("MaterialTooltipTrigger").on_action(
                        move |_: &FocusTooltipAction, window, cx| first.focus(window, cx),
                    )
                })
            })
            .when(transient, |trigger| {
                trigger.on_hover(move |hovering, window, cx| {
                    hover.update(cx, |state, cx| state.hover(false, *hovering, window, cx));
                })
            })
            .when_some(
                self.persistent
                    .clone()
                    .filter(|_| self.enable_user_input && !label),
                |trigger, visibility| {
                    let press = state.clone();
                    let release = state.clone();
                    let release_outside = state.clone();
                    let key = visibility.clone();
                    let accessible = visibility.clone();
                    trigger
                        .capture_any_mouse_down(move |event, _, cx| {
                            if event.button == MouseButton::Left {
                                press.update(cx, |state, _| state.pressed = true);
                            }
                        })
                        .capture_any_mouse_up(move |event, _, cx| {
                            if event.button == MouseButton::Left
                                && release.update(cx, |state, _| std::mem::take(&mut state.pressed))
                            {
                                visibility.update(cx, |state, cx| state.show(cx));
                            }
                        })
                        .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                            release_outside.update(cx, |state, _| state.pressed = false);
                        })
                        .capture_key_down(move |event, _, cx| {
                            if !event.is_held
                                && matches!(event.keystroke.key.as_str(), "enter" | "space")
                            {
                                key.update(cx, |state, cx| state.show(cx));
                            }
                        })
                        .on_a11y_action(AccessibleAction::Click, move |_, _, cx| {
                            accessible.update(cx, |state, cx| state.show(cx));
                        })
                },
            )
            .relative()
            .children(self.trigger.take())
            .child(
                canvas(
                    move |bounds, window, _| {
                        let anchor = (!label
                            || !bounds.intersect(&window.content_mask().bounds).is_empty())
                        .then_some(bounds);
                        if measured.replace(anchor) != anchor && open {
                            window.refresh();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            );
        let mut root =
            div()
                .id(self.id.clone())
                .relative()
                .child(trigger)
                .when(open && !label, |root| {
                    root.key_context("MaterialTooltip").on_action(
                        move |_: &DismissTooltip, window, cx| {
                            on_escape.request(window, cx);
                        },
                    )
                });
        let Some(anchor) = anchor else {
            return root;
        };
        let Some(motion) = OverlayMotion::sample(&self.id, open, window, cx) else {
            return root;
        };
        let last = geometry.placed.get();
        if !open && last.size.width <= px(0.) {
            return root;
        }

        let viewport = window.viewport_size();
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let style = self.appearance.resolve(self.rich, viewport, motion, &theme);
        let view = tooltip_surface(&mut self, open || label, style, motion, window, cx);
        let margin = self.appearance.margin(viewport);
        let view = if label {
            // The overlay owns the accessible text for its decorative children.
            div()
                .id("label-content")
                .role(Role::Group)
                .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
                .child(
                    view.max_h((viewport.height - margin * 2.) * motion.scale)
                        .overflow_hidden(),
                )
                .into_any_element()
        } else {
            view.into_any_element()
        };
        let caret = self.appearance.caret.take();
        let resolved_side = geometry.side.clone();
        let shift_tab = first.clone();
        let return_focus = self.trigger_focus;
        let anchor_bounds = geometry.anchor.clone();
        let tip = div()
            .id(match (label, open) {
                (true, true) => "label",
                (true, false) => "label-exit",
                (false, true) => "tooltip",
                (false, false) => "tooltip-exit",
            })
            .relative()
            .test_support()
            .opacity(motion.alpha)
            .when(open, |tip| {
                tip.role(Role::Tooltip)
                    .aria_label(self.text)
                    .when(!label, |tip| {
                        tip.track_focus(&surface)
                            .key_context("MaterialTooltipContent")
                            .on_action(move |_: &FocusTooltipTrigger, window, cx| {
                                if let Some(trigger) = &return_focus
                                    && shift_tab
                                        .as_ref()
                                        .is_some_and(|first| first.is_focused(window))
                                {
                                    trigger.focus(window, cx);
                                } else {
                                    cx.propagate();
                                }
                            })
                    })
                    .when(transient, |tip| {
                        tip.on_hover(move |hovering, window, cx| {
                            state.update(cx, |state, cx| state.hover(true, *hovering, window, cx));
                        })
                    })
                    .when(!label && dismiss.persistent.is_some(), |tip| {
                        tip.on_mouse_down_out(move |event, window, cx| {
                            if !anchor_bounds
                                .get()
                                .is_some_and(|anchor| anchor.contains(&event.position))
                            {
                                dismiss.request(window, cx);
                            }
                        })
                    })
            })
            .child(view)
            .children(caret.map(|caret| {
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        caret.paint(bounds, anchor, resolved_side.get(), style, window);
                    },
                )
                .absolute()
                .inset_0()
            }));
        let scaled = ScaledOverlay::new(
            motion
                .text(tip, window)
                .when(!open, |tip| tip.w(last.size.width * motion.scale)),
            motion,
            |_| point(0.5, 0.5),
        );
        let positioner = if open {
            let positioner = self.positioner.map_or_else(
                || {
                    TooltipPositioner::side(anchor)
                        .placement(self.placement)
                        .offset(self.offset)
                },
                |positioner| positioner(anchor, viewport, direction),
            );
            positioner
                .into_base(direction)
                .margin(margin)
                .when(!label, gpui_base::Positioner::occlude)
                .on_position(move |resolved| {
                    geometry.placed.set(resolved.bounds);
                    geometry.side.set(resolved.placement);
                })
                .child(scaled)
        } else {
            gpui_base::Positioner::corner(Anchor::TopLeft, last.origin)
                .margin(margin)
                .child(scaled.width(last.size.width))
        };
        root = root.child(
            deferred(ThemeScope::inherit(positioner, window, cx))
                .with_priority(ZLayer::TOOLTIP.priority()),
        );
        root
    }
}

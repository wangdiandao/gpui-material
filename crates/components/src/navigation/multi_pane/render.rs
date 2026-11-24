use std::time::Duration;

use gpui::{
    AnyElement, App, Bounds, ElementId, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Role, Size, StatefulInteractiveElement, Styled,
    WeakFocusHandle, Window, div, prelude::FluentBuilder, px, size,
};
use gpui_base::{FocusTrapElement, TestSupportExt};
use gpui_material_foundation::current_material_theme;

use crate::VerticalDragHandle;

use super::{
    AdaptivePane, AdaptivePaneLayout, AdaptivePaneState, PaneExpansionAnchor, PaneLayoutDirective,
    PanePlacement, PaneRole, PaneSeparator,
    geometry::{Boundary, GeometryInput, Measures, length},
    state::{Adaptation, Frames},
    types::pair_index,
};

struct LayoutFocus {
    panes: [FocusHandle; 3],
    separators: [FocusHandle; 9],
    remembered: [Option<FocusHandle>; 3],
    overlay: Option<PaneRole>,
    trap: Option<WeakFocusHandle>,
}

struct FocusPlan<'a> {
    active: Option<PaneRole>,
    overlay: Option<PaneRole>,
    modal: Option<&'a FocusHandle>,
    panes: &'a [Option<AdaptivePane>; 3],
    frames: Frames,
    separators: &'a [bool; 9],
}

impl LayoutFocus {
    fn new(cx: &mut App) -> Self {
        Self {
            panes: std::array::from_fn(|_| cx.focus_handle().tab_stop(false)),
            separators: std::array::from_fn(|_| cx.focus_handle().tab_stop(false)),
            remembered: Default::default(),
            overlay: None,
            trap: None,
        }
    }

    fn restore(&mut self, plan: FocusPlan<'_>, window: &mut Window, cx: &mut App) {
        let opened = self.overlay != plan.overlay;
        self.overlay = plan.overlay;
        // The pane's stable scope survives the projection. The separate modal
        // scope only exists while levitated, so Base cannot keep trapping Tab
        // when the same pane has reflowed or become a normal column.
        let prior_trap = self.trap.take();
        self.trap = plan.modal.map(FocusHandle::downgrade);
        let Some(focused) = window.focused(cx) else {
            return;
        };
        let old = self
            .panes
            .iter()
            .position(|pane| pane.contains(&focused, window));
        let boundary = self
            .separators
            .iter()
            .position(|scope| scope.contains(&focused, window));
        if let Some(ix) = old
            && focused != self.panes[ix]
        {
            self.remembered[ix] = Some(focused.clone());
        }
        let released_trap = prior_trap
            .and_then(|scope| scope.upgrade())
            .is_some_and(|scope| focused == scope && plan.modal != Some(&scope));
        let transfer_to_overlay = plan.overlay.filter(|_| opened || plan.modal.is_some());
        let needs_restore = released_trap
            || old.is_some_and(|ix| {
                plan.frames[ix].is_none()
                    || transfer_to_overlay.is_some_and(|active| ix != active.index())
            })
            || boundary.is_some_and(|ix| !plan.separators[ix] || transfer_to_overlay.is_some());
        if !needs_restore {
            return;
        }
        if let Some(next) = transfer_to_overlay.or(plan.active) {
            let ix = next.index();
            let scope = self.panes[ix].clone();
            let remembered = self.remembered[ix].clone();
            let entry = plan.panes[ix]
                .as_ref()
                .and_then(|pane| pane.entry_focus.clone());
            window.defer(cx, move |window, cx| {
                if window.focused(cx).is_none_or(|current| current == focused) {
                    remembered
                        .filter(|handle| scope.contains(handle, window))
                        .or_else(|| entry.filter(|handle| scope.contains(handle, window)))
                        .unwrap_or(scope)
                        .focus(window, cx);
                }
            });
        } else {
            window.blur(cx);
        }
    }
}

impl RenderOnce for AdaptivePaneLayout {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let viewport = size(length(self.viewport.width), length(self.viewport.height));
        let directive = self
            .directive
            .take()
            .unwrap_or_else(|| PaneLayoutDirective::for_size(viewport));
        let adaptation = Adaptation {
            horizontal: directive.horizontal_partitions,
            vertical: directive.vertical_partitions,
            strategies: std::array::from_fn(|ix| {
                self.panes[ix]
                    .as_ref()
                    .map(|pane| pane.strategy(viewport, &directive))
            }),
        };
        let input = GeometryInput {
            viewport,
            direction: current_material_theme(window, cx).layout.direction,
            directive: &directive,
            order: self.order,
            measures: std::array::from_fn(|ix| {
                self.panes[ix]
                    .as_ref()
                    .map_or_else(Measures::default, |pane| pane.measures)
            }),
        };
        let duration = if cx.reduce_motion() {
            Duration::ZERO
        } else {
            self.duration
        };
        let (active, placement, geometry, sample, transitioning) =
            self.state.update(cx, |state, cx| {
                let active = state.current_destination().role;
                let placement = state.project(adaptation);
                let geometry = input.layout(placement, |start, end| state.expansion(start, end));
                let sample = state.sample(
                    placement,
                    geometry.panes,
                    cx.background_executor().now(),
                    duration,
                    cx,
                );
                (
                    active,
                    placement,
                    geometry,
                    sample,
                    state.is_transitioning(),
                )
            });
        if sample.animate_next_frame {
            window.request_animation_frame();
        }

        let can_show = |role: PaneRole| sample.frames[role.index()].is_some();
        let overlay =
            self.order.into_iter().rev().find(|role| {
                can_show(*role) && matches!(placement[*role], PanePlacement::Levitate(_))
            });
        let modal = overlay.is_some_and(|role| {
            self.panes[role.index()]
                .as_ref()
                .is_some_and(|pane| pane.scrim.is_some())
        });
        let active = if can_show(active) {
            Some(active)
        } else {
            self.order.into_iter().find(|role| can_show(*role))
        };
        let trap = overlay.filter(|_| modal).map(|role| {
            window
                .use_keyed_state(
                    (
                        self.id.clone(),
                        ["primary-modal", "secondary-modal", "extra-modal"][role.index()],
                    ),
                    cx,
                    |_, cx| cx.focus_handle().tab_stop(false),
                )
                .read(cx)
                .clone()
        });
        let mut active_separators = [false; 9];
        for boundary in &geometry.boundaries {
            let ix = pair_index(boundary.start, boundary.end);
            // Do not create a movable target during a changing pane arrangement.
            active_separators[ix] = !transitioning
                && !modal
                && self.separators[ix].is_some()
                && boundary.min < boundary.max;
        }

        let focus_state = window.use_keyed_state((self.id.clone(), "pane-focus"), cx, |_, cx| {
            LayoutFocus::new(cx)
        });
        let (pane_focus, separator_focus) = focus_state.update(cx, |focus, cx| {
            focus.restore(
                FocusPlan {
                    active,
                    overlay,
                    modal: trap.as_ref(),
                    panes: &self.panes,
                    frames: sample.frames,
                    separators: &active_separators,
                },
                window,
                cx,
            );
            (focus.panes.clone(), focus.separators.clone())
        });

        let layout_id = self.id.clone();
        let mut root = div()
            .id(self.id)
            .test_support()
            .relative()
            .w(viewport.width)
            .h(viewport.height)
            .overflow_hidden();
        // Panes keep sibling scopes while changing between normal, reflowed and
        // levitated placements. Their child IDs never inherit a neighbor's path.
        for role in self.order {
            if let Some(pane) = self.panes[role.index()].take_if(|_| {
                can_show(role) && !matches!(placement[role], PanePlacement::Levitate(_))
            }) {
                root = root.child(render_pane(
                    pane,
                    sample.frames[role.index()].unwrap(),
                    &pane_focus[role.index()],
                    false,
                    None,
                    modal,
                ));
            }
        }
        for boundary in geometry.boundaries {
            let ix = pair_index(boundary.start, boundary.end);
            if active_separators[ix] {
                let separator = self.separators[ix].take().unwrap();
                root = root.child(render_separator(
                    &layout_id,
                    separator,
                    boundary,
                    viewport,
                    &self.state,
                    &separator_focus[ix],
                ));
            }
        }
        for role in self.order {
            if let Some(mut pane) = self.panes[role.index()].take_if(|_| can_show(role)) {
                let backdrop = pane.scrim.take();
                let is_modal = Some(role) == overlay && backdrop.is_some();
                if let Some(backdrop) = backdrop {
                    root = root.child(
                        div()
                            .id((pane.id.clone(), "backdrop"))
                            .absolute()
                            .inset_0()
                            .occlude()
                            .child(backdrop),
                    );
                }
                root = root.child(render_pane(
                    pane,
                    sample.frames[role.index()].unwrap(),
                    &pane_focus[role.index()],
                    true,
                    trap.as_ref().filter(|_| is_modal),
                    modal && Some(role) != overlay,
                ));
            }
        }
        root
    }
}

fn render_pane(
    pane: AdaptivePane,
    bounds: Bounds<Pixels>,
    focus: &FocusHandle,
    elevated: bool,
    modal: Option<&FocusHandle>,
    hidden_from_a11y: bool,
) -> AnyElement {
    let content_id: ElementId = (pane.id.clone(), "pane-content").into();
    let body = div()
        .id(content_id.clone())
        .size_full()
        .flex()
        .flex_col()
        .child(pane.content);
    let body = if let Some(trap) = modal {
        body.focus_trap(content_id, trap).into_any_element()
    } else {
        body.into_any_element()
    };
    div()
        .id(pane.id)
        .role(Role::Group)
        .test_support()
        .track_focus(focus)
        .absolute()
        .left(bounds.left())
        .top(bounds.top())
        .w(bounds.size.width)
        .h(bounds.size.height)
        .min_w_0()
        .min_h_0()
        .flex()
        .flex_col()
        .overflow_hidden()
        .when(elevated, |node| node.occlude())
        .when(hidden_from_a11y, |node| {
            node.a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
        })
        .child(body)
        .into_any_element()
}

fn render_separator(
    layout_id: &ElementId,
    separator: PaneSeparator,
    boundary: Boundary,
    viewport: Size<Pixels>,
    owner: &Entity<AdaptivePaneState>,
    focus: &FocusHandle,
) -> impl IntoElement {
    let pair = pair_index(boundary.start, boundary.end);
    let handle = VerticalDragHandle::new((layout_id.clone(), format!("pane-separator-{pair}")));
    let handle = match separator.build {
        Some(build) => build(handle),
        None => handle,
    };
    let resize = owner.clone();
    let snap = owner.clone();
    let keyboard_state = owner.clone();
    let keyboard_anchors = separator.anchors.clone();
    div()
        .id((layout_id.clone(), format!("pane-boundary-{pair}")))
        .track_focus(focus)
        .absolute()
        .left(boundary.center - px(24.))
        .top_0()
        .w(px(48.))
        .h(viewport.height)
        .flex()
        .items_center()
        .justify_center()
        .when(!keyboard_anchors.is_empty(), |container| {
            container.on_key_down(move |event, _, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let sign = match event.keystroke.key.as_str() {
                    "pageup" => 1.,
                    "pagedown" => -1.,
                    _ => return,
                };
                let anchor = boundary
                    .anchors(&keyboard_anchors)
                    .map(|(anchor, at)| (anchor, f32::from(at - boundary.position) * sign))
                    .filter(|(_, distance)| *distance > 0.01)
                    .min_by(|(_, a), (_, b)| a.total_cmp(b));
                if let Some((anchor, _)) = anchor {
                    keyboard_state.update(cx, |state, cx| {
                        state.set_expansion(boundary.start, boundary.end, anchor, cx)
                    });
                }
                cx.stop_propagation();
            })
        })
        .child(
            handle
                .resize_context(viewport)
                .keyboard_step(separator.keyboard_step)
                .on_resize(
                    separator.label,
                    boundary.position,
                    boundary.min..=boundary.max,
                    move |position, _, cx| {
                        resize.update(cx, |state, cx| {
                            state.set_expansion(
                                boundary.start,
                                boundary.end,
                                PaneExpansionAnchor::FromStart(position),
                                cx,
                            )
                        });
                    },
                )
                .on_resize_end(move |position, _, cx| {
                    let anchor = boundary
                        .anchors(&separator.anchors)
                        .map(|(anchor, at)| (anchor, f32::from((at - position).abs())))
                        .min_by(|(_, a), (_, b)| a.total_cmp(b));
                    if let Some((anchor, _)) = anchor {
                        snap.update(cx, |state, cx| {
                            state.set_expansion(boundary.start, boundary.end, anchor, cx)
                        });
                    }
                }),
        )
}

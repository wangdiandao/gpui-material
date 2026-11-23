use std::time::Duration;

use gpui::{
    AnyElement, App, Bounds, ElementId, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Role, SharedString, Size, StatefulInteractiveElement,
    Styled, WeakFocusHandle, Window, div, prelude::FluentBuilder, px, size,
};
use gpui_base::{FocusTrapElement, TestSupportExt};
use gpui_material_foundation::current_material_theme;

use crate::VerticalDragHandle;

mod geometry;
mod state;

use geometry::{Boundary, GeometryInput, Measures, length};
use state::{Adaptation, Frames};

pub use geometry::{PaneExpansionAnchor, PaneLayoutDirective, PaneMargins, PaneOverlayAlignment};
pub use state::{
    AdaptivePaneState, PaneAdaptStrategy, PaneBackBehavior, PaneDestination, PaneLayoutValue,
    PanePlacement, PaneRole,
};

type AdaptationRule = Box<dyn Fn(Size<Pixels>, &PaneLayoutDirective) -> PaneAdaptStrategy>;
type HandleBuilder = Box<dyn FnOnce(VerticalDragHandle) -> VerticalDragHandle>;

/// A pane slot with stable identity, logical geometry preferences and adaptation.
pub struct AdaptivePane {
    id: ElementId,
    content: AnyElement,
    measures: Measures,
    adapt: Option<PaneAdaptStrategy>,
    rule: Option<AdaptationRule>,
    scrim: Option<AnyElement>,
    entry_focus: Option<FocusHandle>,
}

impl AdaptivePane {
    /// Creates an adaptive pane with a stable element ID and body content.
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            content: content.into_any_element(),
            measures: Measures::default(),
            adapt: None,
            rule: None,
            scrim: None,
            entry_focus: None,
        }
    }

    /// Sets the pane's preferred width in logical pixels.
    pub fn preferred_width(mut self, width: Pixels) -> Self {
        self.measures.preferred_width = Some(width);
        self
    }

    /// Sets the pane's preferred height in logical pixels (used for levitated or stacked presentations).
    pub fn preferred_height(mut self, height: Pixels) -> Self {
        self.measures.preferred_height = Some(height);
        self
    }

    /// Sets the minimum width below which the pane will not shrink.
    pub fn minimum_width(mut self, width: Pixels) -> Self {
        self.measures.minimum_width = width;
        self
    }

    /// Sets logical outer margins around the pane.
    pub fn margins(mut self, margins: PaneMargins) -> Self {
        self.measures.margins = margins;
        self
    }

    /// Sets a fixed adaptation strategy when the pane cannot be shown as an inline column.
    pub fn adapt(mut self, strategy: PaneAdaptStrategy) -> Self {
        self.adapt = Some(strategy);
        self.rule = None;
        self
    }

    /// Calculates the adaptation strategy from the available size and directive.
    pub fn adapt_with(
        mut self,
        rule: impl Fn(Size<Pixels>, &PaneLayoutDirective) -> PaneAdaptStrategy + 'static,
    ) -> Self {
        self.rule = Some(Box::new(rule));
        self
    }

    /// Supplies a modal backdrop for the levitated presentation.
    pub fn levitate_scrim(mut self, scrim: impl IntoElement) -> Self {
        self.scrim = Some(scrim.into_any_element());
        self
    }

    /// Sets the focus target focused when this pane opens as an active or levitated overlay.
    pub fn entry_focus(mut self, focus: &FocusHandle) -> Self {
        self.entry_focus = Some(focus.clone());
        self
    }

    pub(super) fn has_adaptation(&self) -> bool {
        self.rule.is_some() || self.adapt.is_some()
    }

    fn strategy(&self, size: Size<Pixels>, directive: &PaneLayoutDirective) -> PaneAdaptStrategy {
        self.rule
            .as_ref()
            .map_or(self.adapt.unwrap_or_default(), |rule| rule(size, directive))
    }
}

/// An opt-in resize handle between two adjacent logical roles.
pub struct PaneSeparator {
    label: SharedString,
    anchors: Vec<PaneExpansionAnchor>,
    keyboard_step: Pixels,
    build: Option<HandleBuilder>,
}

impl PaneSeparator {
    /// Creates a resize separator with the given accessible label and an 8 px default keyboard step.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            anchors: Vec::new(),
            keyboard_step: px(8.),
            build: None,
        }
    }

    /// Sets snap anchors offered when dragging or stepping across the split boundary.
    pub fn anchors(mut self, anchors: impl IntoIterator<Item = PaneExpansionAnchor>) -> Self {
        self.anchors = anchors.into_iter().collect();
        self
    }

    /// Sets the logical pixel distance moved per arrow-key press (default 8 px).
    pub fn keyboard_step(mut self, step: Pixels) -> Self {
        self.keyboard_step = step;
        self
    }

    /// Configures the existing Material handle's appearance or interaction source.
    pub fn handle(
        mut self,
        build: impl FnOnce(VerticalDragHandle) -> VerticalDragHandle + 'static,
    ) -> Self {
        self.build = Some(Box::new(build));
        self
    }
}

fn pair_index(start: PaneRole, end: PaneRole) -> usize {
    start.index() * 3 + end.index()
}

/// A three-role scaffold with caller-supplied panes, an explicit available viewport,
/// and retained layout, expansion and navigation state.
#[derive(IntoElement)]
pub struct AdaptivePaneLayout {
    id: ElementId,
    viewport: Size<Pixels>,
    state: Entity<AdaptivePaneState>,
    directive: Option<PaneLayoutDirective>,
    panes: [Option<AdaptivePane>; 3],
    order: [PaneRole; 3],
    separators: [Option<PaneSeparator>; 9],
    duration: Duration,
}

impl AdaptivePaneLayout {
    /// Creates an adaptive multi-pane layout for the given viewport size and retained [`AdaptivePaneState`].
    pub fn new(
        id: impl Into<ElementId>,
        viewport: Size<Pixels>,
        state: &Entity<AdaptivePaneState>,
    ) -> Self {
        Self {
            id: id.into(),
            viewport,
            state: state.clone(),
            directive: None,
            panes: Default::default(),
            order: PaneRole::ALL,
            separators: Default::default(),
            duration: Duration::from_millis(240),
        }
    }

    /// Assigns an [`AdaptivePane`] to the given logical role slot.
    pub fn pane(mut self, role: PaneRole, pane: AdaptivePane) -> Self {
        self.panes[role.index()] = Some(pane);
        self
    }

    /// Overrides the [`PaneLayoutDirective`] derived from the viewport size.
    pub fn directive(mut self, directive: PaneLayoutDirective) -> Self {
        self.directive = Some(directive);
        self
    }

    /// Logical order of the three roles. Duplicate roles leave the previous order intact.
    pub fn order(mut self, order: [PaneRole; 3]) -> Self {
        if PaneRole::ALL.into_iter().all(|role| order.contains(&role)) {
            self.order = order;
        }
        self
    }

    /// Registers a resize separator between two adjacent logical pane roles.
    pub fn separator(mut self, start: PaneRole, end: PaneRole, separator: PaneSeparator) -> Self {
        self.separators[pair_index(start, end)] = Some(separator);
        self
    }

    /// Sets the animation duration used for pane layout transitions (default 240 ms).
    pub fn transition_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

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
    let body = div()
        .id("pane-content")
        .size_full()
        .flex()
        .flex_col()
        .child(pane.content);
    let body = if let Some(trap) = modal {
        body.focus_trap("pane-content", trap).into_any_element()
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
    separator: PaneSeparator,
    boundary: Boundary,
    viewport: Size<Pixels>,
    owner: &Entity<AdaptivePaneState>,
    focus: &FocusHandle,
) -> impl IntoElement {
    let pair = pair_index(boundary.start, boundary.end);
    let handle = VerticalDragHandle::new(("pane-separator", pair));
    let handle = match separator.build {
        Some(build) => build(handle),
        None => handle,
    };
    let resize = owner.clone();
    let snap = owner.clone();
    let keyboard_state = owner.clone();
    let keyboard_anchors = separator.anchors.clone();
    div()
        .id(("pane-boundary", pair))
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

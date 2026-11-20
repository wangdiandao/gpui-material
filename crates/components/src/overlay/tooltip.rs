mod appearance;
mod interaction;
mod label;

use std::{cell::Cell, rc::Rc};

use gpui::{
    AccessibleAction, Anchor, AnyElement, App, Bounds, ElementId, Entity, FocusHandle,
    InteractiveElement, IntoElement, KeyBinding, MouseButton, ParentElement, Pixels, RenderOnce,
    Role, SharedString, Size, StatefulInteractiveElement, Styled, Window, canvas, deferred, div,
    point, prelude::FluentBuilder, px,
};
use gpui_base::{Link, Placement, TestSupportExt};
use gpui_material_foundation::{ThemeScope, current_material_theme};
use gpui_material_theme::{CornerShape, Direction, Elevation, TypeRole};

use appearance::{Appearance, SurfaceStyle};
use interaction::{Dismiss, Interaction};

pub use appearance::{TooltipCaret, TooltipColors, TooltipPlacement};
pub use gpui_base::Positioner as TooltipPositioner;
pub use interaction::TooltipState;
pub use label::Label;

use crate::{
    Button, ButtonColors, ButtonVariant,
    button_style::finite_length,
    motion::{OverlayMotion, ScaledOverlay},
};

gpui::actions!(
    material_tooltip,
    [
        FocusTooltipAction,
        FocusTooltipTrigger,
        DismissTooltip,
        ActivateTooltipAction
    ]
);

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", FocusTooltipAction, Some("MaterialTooltipTrigger")),
        KeyBinding::new(
            "shift-tab",
            FocusTooltipTrigger,
            Some("MaterialTooltipContent"),
        ),
        KeyBinding::new("escape", DismissTooltip, Some("MaterialTooltip")),
        KeyBinding::new(
            "enter",
            ActivateTooltipAction,
            Some("MaterialTooltipAction"),
        ),
        KeyBinding::new(
            "space",
            ActivateTooltipAction,
            Some("MaterialTooltipAction"),
        ),
    ]);
}

type TooltipCallback = Rc<dyn Fn(&mut Window, &mut App)>;
type PositionProvider = Box<dyn Fn(Bounds<Pixels>, Size<Pixels>, Direction) -> TooltipPositioner>;

struct TooltipAction {
    label: SharedString,
    focus: FocusHandle,
    callback: TooltipCallback,
}

/// A transient plain or rich Material tooltip, or a persistent rich tooltip
/// with caller-owned visibility.
#[derive(IntoElement)]
pub struct Tooltip {
    id: ElementId,
    trigger: Option<AnyElement>,
    text: SharedString,
    title: Option<SharedString>,
    action: Option<TooltipAction>,
    title_content: Option<AnyElement>,
    body_content: Option<AnyElement>,
    action_content: Option<AnyElement>,
    link: Option<(SharedString, TooltipCallback)>,
    rich: bool,
    trigger_focus: Option<FocusHandle>,
    focus_target: Option<FocusHandle>,
    focusable: bool,
    enable_user_input: bool,
    persistent: Option<Entity<TooltipState>>,
    label_visible: Option<bool>,
    on_dismiss_request: Option<TooltipCallback>,
    placement: TooltipPlacement,
    offset: Pixels,
    positioner: Option<PositionProvider>,
    appearance: Appearance,
}

impl Tooltip {
    pub fn plain(
        id: impl Into<ElementId>,
        trigger: impl IntoElement,
        text: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            trigger: Some(trigger.into_any_element()),
            text: text.into(),
            title: None,
            action: None,
            title_content: None,
            body_content: None,
            action_content: None,
            link: None,
            rich: false,
            trigger_focus: None,
            focus_target: None,
            focusable: false,
            enable_user_input: true,
            persistent: None,
            label_visible: None,
            on_dismiss_request: None,
            placement: TooltipPlacement::Above,
            offset: px(4.),
            positioner: None,
            appearance: Appearance::default(),
        }
    }

    pub fn rich(
        id: impl Into<ElementId>,
        trigger: impl IntoElement,
        title: impl Into<SharedString>,
        text: impl Into<SharedString>,
    ) -> Self {
        Self {
            title: Some(title.into()),
            rich: true,
            ..Self::plain(id, trigger, text)
        }
    }

    /// Adds a text button to a rich tooltip.
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        focus: &FocusHandle,
        callback: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.rich = true;
        self.action = Some(TooltipAction {
            label: label.into(),
            focus: focus.clone(),
            callback: Rc::new(callback),
        });
        self
    }

    pub fn title_content(mut self, content: impl IntoElement) -> Self {
        self.rich = true;
        self.title_content = Some(content.into_any_element());
        self
    }

    pub fn body_content(mut self, content: impl IntoElement) -> Self {
        self.body_content = Some(content.into_any_element());
        self
    }

    /// Replaces the standard action with caller-authored interactive content.
    pub fn action_content(mut self, content: impl IntoElement) -> Self {
        self.rich = true;
        self.action_content = Some(content.into_any_element());
        self
    }

    /// Adds a semantic link. The callback owns the destination and navigation.
    pub fn link(
        mut self,
        label: impl Into<SharedString>,
        callback: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.rich = true;
        self.link = Some((label.into(), Rc::new(callback)));
        self
    }

    /// Uses the trigger's focus handle for keyboard activation and restoration.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.trigger_focus = Some(focus.clone());
        self
    }

    /// Uses the caller's first focus target inside the tooltip.
    pub fn focus_target(mut self, focus: &FocusHandle) -> Self {
        self.focus_target = Some(focus.clone());
        self
    }

    /// Moves focus to the tooltip when it opens. The default preserves focus.
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    /// Enables automatic trigger input for the chosen visibility mode.
    pub fn enable_user_input(mut self, enable: bool) -> Self {
        self.enable_user_input = enable;
        self
    }

    /// Uses caller-owned visibility for a persistent rich tooltip.
    pub fn persistent(mut self, state: &Entity<TooltipState>) -> Self {
        self.rich = true;
        self.persistent = Some(state.clone());
        self
    }

    /// Handles Escape or a persistent tooltip's outside press. A persistent
    /// tooltip with a handler closes when the owner accepts the request.
    pub fn on_dismiss_request(
        mut self,
        callback: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss_request = Some(Rc::new(callback));
        self
    }

    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Gap between the trigger and surface in logical pixels.
    pub fn offset(mut self, offset: Pixels) -> Self {
        self.offset = finite_length(offset);
        self
    }

    /// Replaces the standard placement with a positioner built from the trigger
    /// bounds, viewport size, and direction. The host applies viewport margins.
    pub fn positioner(
        mut self,
        positioner: impl Fn(Bounds<Pixels>, Size<Pixels>, Direction) -> TooltipPositioner + 'static,
    ) -> Self {
        self.positioner = Some(Box::new(positioner));
        self
    }

    pub fn caret(mut self, caret: TooltipCaret) -> Self {
        self.appearance.caret = Some(caret);
        self
    }

    pub fn max_width(mut self, width: Pixels) -> Self {
        self.appearance.max_width = Some(finite_length(width));
        self
    }

    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.appearance.shape = Some(shape);
        self
    }

    pub fn colors(mut self, colors: TooltipColors) -> Self {
        self.appearance.colors = colors;
        self
    }

    pub fn tonal_elevation(mut self, elevation: Elevation) -> Self {
        self.appearance.tonal_elevation = elevation;
        self
    }

    pub fn shadow_elevation(mut self, elevation: Elevation) -> Self {
        self.appearance.shadow_elevation = Some(elevation);
        self
    }
}

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
        let trigger = div()
            .id(if label {
                "label-anchor"
            } else {
                "tooltip-trigger"
            })
            .role(Role::Group)
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
        let (Some(motion), Some(anchor)) =
            (OverlayMotion::sample(&self.id, open, window, cx), anchor)
        else {
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
                        .placement(self.placement.physical(direction))
                        .offset(self.offset)
                },
                |positioner| positioner(anchor, viewport, direction),
            );
            positioner
                .margin(margin)
                .when(!label, TooltipPositioner::occlude)
                .on_position(move |resolved| {
                    geometry.placed.set(resolved.bounds);
                    geometry.side.set(resolved.placement);
                })
                .child(scaled)
        } else {
            TooltipPositioner::corner(Anchor::TopLeft, last.origin)
                .margin(margin)
                .child(scaled.width(last.size.width))
        };
        root = root.child(deferred(ThemeScope::inherit(positioner, window, cx)).with_priority(100));
        root
    }
}

fn tooltip_surface(
    tooltip: &mut Tooltip,
    open: bool,
    style: SurfaceStyle,
    motion: OverlayMotion,
    window: &Window,
    cx: &App,
) -> gpui::Div {
    let title = tooltip.title.take().map(IntoElement::into_any_element);
    let title = if open {
        tooltip.title_content.take().or(title)
    } else {
        title
    };
    let body = if open {
        tooltip.body_content.take()
    } else {
        None
    }
    .unwrap_or_else(|| tooltip.text.clone().into_any_element());
    let action = if open {
        tooltip.action_content.take()
    } else {
        None
    }
    .or_else(|| {
        tooltip
            .action
            .take()
            .map(|action| render_action(action, open, style, motion, window, cx))
    });
    let link = tooltip.link.take().map(|(label, callback)| {
        if open {
            let activate = callback.clone();
            Link::new("tooltip-link")
                .accessibility_label(label.clone())
                .text_color(style.action)
                .underline()
                .on_activate(move |_, window, cx| callback(window, cx))
                .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                    activate(window, cx)
                })
                .child(label)
                .into_any_element()
        } else {
            div()
                .text_color(style.action)
                .child(label)
                .into_any_element()
        }
    });
    let text_role = if tooltip.rich {
        TypeRole::BodyMedium
    } else {
        TypeRole::BodySmall
    };
    style
        .apply(div())
        .children(title.map(|title| {
            super::text_style(
                div().text_color(style.title).child(title),
                TypeRole::TitleSmall,
                motion.scale,
                window,
                cx,
            )
        }))
        .child(super::text_style(
            div().text_color(style.content).child(body),
            text_role,
            motion.scale,
            window,
            cx,
        ))
        .children(action.map(|action| div().text_color(style.action).child(action)))
        .children(link)
}

fn render_action(
    action: TooltipAction,
    open: bool,
    style: SurfaceStyle,
    motion: OverlayMotion,
    window: &Window,
    cx: &App,
) -> AnyElement {
    if open {
        let activate = action.callback.clone();
        return div()
            .id("tooltip-action-keys")
            .key_context("MaterialTooltipAction")
            .on_action(move |_: &ActivateTooltipAction, window, cx| activate(window, cx))
            .child(
                Button::new("tooltip-action", action.label)
                    .track_focus(&action.focus)
                    .variant(ButtonVariant::Text)
                    .scale(motion.scale)
                    .colors(ButtonColors {
                        content: style.action_override,
                        ..Default::default()
                    })
                    .on_activate(move |window, cx| (action.callback)(window, cx)),
            )
            .into_any_element();
    }
    let density = current_material_theme(window, cx).layout.density;
    div()
        .min_h(motion.px(density.interactive_size(40.)))
        .flex()
        .items_center()
        .justify_center()
        .child(
            super::text_style(
                div().px(motion.px(16.)).min_w_0().truncate(),
                TypeRole::LabelLarge,
                motion.scale,
                window,
                cx,
            )
            .text_color(style.action)
            .child(action.label),
        )
        .into_any_element()
}

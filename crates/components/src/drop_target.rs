use std::{marker::PhantomData, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, ClickEvent, ElementId, Entity, ExternalPaths, FocusHandle,
    Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Button as BaseButton, TestSupportExt};
use gpui_material_foundation::{InteractionSource, ThemeScope, color, current_material_theme};
use gpui_material_theme::{ColorRole, CornerRole, CornerShape, Direction, StateLayer, TypeRole};

use crate::{
    Icon, IconName, Text,
    action::{activate_feedback, feedback_target, ripple_state},
};

type DropPredicate<T> = Rc<dyn Fn(&T, &mut Window, &mut App) -> bool>;
type DropHandler<T> = Rc<dyn Fn(&T, &mut Window, &mut App)>;
type ActivateHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// Per-instance pigments for idle, valid drag-over, and rejected drag-over states of a [`DropTarget`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DropTargetColors {
    /// Background color while idle. Defaults to `SurfaceContainerLowest`.
    pub container: Option<Hsla>,
    /// Background color while an accepted payload is dragged over the target. Defaults to `PrimaryContainer`.
    pub active_container: Option<Hsla>,
    /// Background color while a rejected payload is dragged over the target. Defaults to `ErrorContainer`.
    pub invalid_container: Option<Hsla>,
    /// Dashed border color while idle. Defaults to `OutlineVariant`.
    pub border: Option<Hsla>,
    /// Dashed border color while an accepted payload is dragged over the target. Defaults to `Primary`.
    pub active_border: Option<Hsla>,
    /// Dashed border color while a rejected payload is dragged over the target. Defaults to `Error`.
    pub invalid_border: Option<Hsla>,
    /// Foreground color for text and icons while idle. Defaults to `OnSurfaceVariant`.
    pub content: Option<Hsla>,
    /// Foreground color while an accepted payload is dragged over the target. Defaults to `OnPrimaryContainer`.
    pub active_content: Option<Hsla>,
    /// Foreground color while a rejected payload is dragged over the target. Defaults to `OnErrorContainer`.
    pub invalid_content: Option<Hsla>,
}

#[derive(Clone, Copy)]
struct ResolvedPalette {
    container: Hsla,
    active_container: Hsla,
    invalid_container: Hsla,
    border: Hsla,
    active_border: Hsla,
    invalid_border: Hsla,
    content: Hsla,
    active_content: Hsla,
    invalid_content: Hsla,
    hover_tint: Hsla,
    ring: Hsla,
}

impl DropTargetColors {
    fn resolve(self, disabled: bool, window: &Window, cx: &App) -> ResolvedPalette {
        use ColorRole::*;
        let theme = current_material_theme(window, cx);
        let role = |r| color(theme.colors.color(r));
        let on_surface = role(OnSurface);
        let ring = role(Secondary);
        if disabled {
            return ResolvedPalette {
                container: self.container.unwrap_or_else(|| on_surface.opacity(0.06)),
                active_container: self
                    .active_container
                    .unwrap_or_else(|| on_surface.opacity(0.06)),
                invalid_container: self
                    .invalid_container
                    .unwrap_or_else(|| on_surface.opacity(0.06)),
                border: self.border.unwrap_or_else(|| on_surface.opacity(0.12)),
                active_border: self
                    .active_border
                    .unwrap_or_else(|| on_surface.opacity(0.12)),
                invalid_border: self
                    .invalid_border
                    .unwrap_or_else(|| on_surface.opacity(0.12)),
                content: self.content.unwrap_or_else(|| on_surface.opacity(0.38)),
                active_content: self
                    .active_content
                    .unwrap_or_else(|| on_surface.opacity(0.38)),
                invalid_content: self
                    .invalid_content
                    .unwrap_or_else(|| on_surface.opacity(0.38)),
                hover_tint: gpui::transparent_black(),
                ring,
            };
        }

        ResolvedPalette {
            container: self
                .container
                .unwrap_or_else(|| role(SurfaceContainerLowest)),
            active_container: self
                .active_container
                .unwrap_or_else(|| role(PrimaryContainer)),
            invalid_container: self
                .invalid_container
                .unwrap_or_else(|| role(ErrorContainer)),
            border: self.border.unwrap_or_else(|| role(OutlineVariant)),
            active_border: self.active_border.unwrap_or_else(|| role(Primary)),
            invalid_border: self.invalid_border.unwrap_or_else(|| role(Error)),
            content: self.content.unwrap_or_else(|| role(OnSurfaceVariant)),
            active_content: self
                .active_content
                .unwrap_or_else(|| role(OnPrimaryContainer)),
            invalid_content: self
                .invalid_content
                .unwrap_or_else(|| role(OnErrorContainer)),
            hover_tint: role(OnSurface).opacity(StateLayer::Hover.opacity()),
            ring,
        }
    }
}

/// Material-styled drag-and-drop container supporting external OS files (`ExternalPaths`)
/// and typed in-app GPUI drag payloads `T: 'static`.
#[derive(IntoElement)]
pub struct DropTarget<T: 'static = ExternalPaths> {
    id: ElementId,
    title: Option<SharedString>,
    supporting: Option<SharedString>,
    icon: Option<AnyElement>,
    content: Option<AnyElement>,
    shape: CornerShape,
    colors: DropTargetColors,
    disabled: bool,
    can_drop: Option<DropPredicate<T>>,
    on_drop: Option<DropHandler<T>>,
    on_activate: Option<ActivateHandler>,
    accessible_label: Option<SharedString>,
    focus: Option<FocusHandle>,
    source: Option<Entity<InteractionSource>>,
    _payload: PhantomData<fn(&T)>,
}

impl DropTarget<ExternalPaths> {
    /// Creates a drop target configured for external OS file drops (`gpui::ExternalPaths`).
    pub fn files(id: impl Into<ElementId>) -> Self {
        Self::new(id)
    }
}

impl<T: 'static> DropTarget<T> {
    /// Creates a drop target accepting GPUI drags of payload type `T`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            supporting: None,
            icon: None,
            content: None,
            shape: CornerRole::Large.shape(),
            colors: DropTargetColors::default(),
            disabled: false,
            can_drop: None,
            on_drop: None,
            on_activate: None,
            accessible_label: None,
            focus: None,
            source: None,
            _payload: PhantomData,
        }
    }

    /// Primary prompt text displayed in the default placeholder layout when `.content(...)` is not set.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Secondary description displayed below the title in the default placeholder layout.
    pub fn supporting(mut self, supporting: impl Into<SharedString>) -> Self {
        self.supporting = Some(supporting.into());
        self
    }

    /// Decorative icon displayed above the title in the default placeholder layout.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    /// Wraps caller-supplied child content with the drop target's border, feedback, and drop handlers.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    /// Sets the logical corner shape. Defaults to `CornerRole::Large`.
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = shape;
        self
    }

    /// Overrides idle, active drag-over, and invalid drag-over colors independently.
    pub fn colors(mut self, colors: DropTargetColors) -> Self {
        self.colors = colors;
        self
    }

    /// Disables drop registration, drag-over visual feedback, and optional click/keyboard activation.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Validates whether a dragged payload `T` can be dropped here.
    /// Rejected payloads show invalid/error visual feedback and skip `on_drop`.
    pub fn can_drop(
        mut self,
        predicate: impl Fn(&T, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.can_drop = Some(Rc::new(predicate));
        self
    }

    /// Invoked when an accepted payload `T` is dropped onto this target.
    pub fn on_drop(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_drop = Some(Rc::new(handler));
        self
    }

    /// Enables pointer click and keyboard Enter/Space activation (for example, to open a file picker).
    pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }

    /// Sets the accessible name exposed to assistive technology. Falls back to `.title(...)` or `"Drop target"`.
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    /// Uses a caller-owned focus handle when the drop target is actionable via `.on_activate(...)`.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    /// Observes hover, focus, press, and disabled states under this drop target's element ID.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.source = Some(source.clone());
        self
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

impl<T: 'static> RenderOnce for DropTarget<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let palette = self.colors.resolve(self.disabled, window, cx);
        let accessible_name = self
            .accessible_label
            .or_else(|| self.title.clone())
            .unwrap_or_else(|| "Drop target".into());
        let actionable = self.on_activate.is_some();
        let enabled = !self.disabled;
        let focus = self.focus.filter(|_| actionable).unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "drop-target-focus"), cx, |_, cx| {
                    cx.focus_handle()
                })
                .read(cx)
                .clone()
        });
        let feedback = ripple_state(&self.id, actionable && enabled, window, cx);
        let tracker = self
            .source
            .as_ref()
            .map(|source| InteractionSource::track(source, &self.id, window, cx));
        feedback
            .borrow_mut()
            .track(enabled, &focus, tracker, window, cx);

        let inner_content = match self.content {
            Some(content) => div().w_full().child(content).into_any_element(),
            None => {
                let icon = self
                    .icon
                    .unwrap_or_else(|| Icon::new(IconName::Add).size(px(28.)).into_any_element());
                let title = self.title.unwrap_or_else(|| "Drop files here".into());
                div()
                    .w_full()
                    .min_h(px(120.))
                    .p(px(24.))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(8.))
                    .child(div().text_size(px(28.)).child(icon))
                    .child(
                        Text::new(title)
                            .id("drop-target-title")
                            .role(TypeRole::TitleMedium)
                            .wrap(),
                    )
                    .children(self.supporting.map(|text| {
                        Text::new(text)
                            .id("drop-target-supporting")
                            .role(TypeRole::BodyMedium)
                            .wrap()
                    }))
                    .into_any_element()
            }
        };

        let body = if let Some(on_activate) = self.on_activate {
            let on_activate = Rc::new(ThemeScope::bind_action(
                move |window, cx| on_activate(window, cx),
                window,
                cx,
            ));
            let click_handler = on_activate.clone();
            let a11y_handler = on_activate;
            let press = feedback.clone();
            let hover_tint = palette.hover_tint;
            let ring = palette.ring;
            apply_corner_shape(
                BaseButton::new((self.id.clone(), "activate"))
                    .track_focus(&focus)
                    .tab_stop(enabled)
                    .disabled(self.disabled)
                    .w_full()
                    .accessibility_label(accessible_name)
                    .border_2()
                    .border_color(gpui::transparent_black()),
                self.shape,
                direction,
            )
            .when(enabled, |button| {
                feedback_target(button, feedback, true, Some(focus.clone()))
                    .hover(move |style| style.bg(hover_tint))
                    .focus_visible(move |style| style.border_color(ring))
                    .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                        a11y_handler(window, cx);
                    })
                    .on_click(move |event, window, cx| {
                        if matches!(event, ClickEvent::Mouse(_)) && cx.has_active_drag() {
                            return;
                        }
                        cx.stop_propagation();
                        activate_feedback(event, &press, window, cx);
                        click_handler(window, cx);
                    })
            })
            .child(inner_content)
            .into_any_element()
        } else {
            div()
                .id((self.id.clone(), "body"))
                .test_support()
                .w_full()
                .role(Role::Region)
                .aria_label(accessible_name)
                .child(inner_content)
                .into_any_element()
        };

        // Keep `drag_over` on the visual wrapper (without `can_drop_predicate`) so rejected
        // payloads still paint invalid feedback, while the drop receiver enforces `can_drop`.
        let visual_can_drop = self.can_drop.clone();
        let drop_can_drop = self.can_drop;
        let on_drop = self.on_drop;

        let receiver = div()
            .id((self.id.clone(), "receiver"))
            .test_support()
            .w_full()
            .when(!self.disabled, |receiver| {
                let check = drop_can_drop;
                receiver
                    .can_drop(move |any, window, cx| {
                        any.downcast_ref::<T>().is_some_and(|payload| {
                            check
                                .as_ref()
                                .is_none_or(|predicate| predicate(payload, window, cx))
                        })
                    })
                    .on_drop::<T>(move |payload, window, cx| {
                        if let Some(handler) = &on_drop {
                            handler(payload, window, cx);
                        }
                    })
            })
            .child(body);

        apply_corner_shape(
            div()
                .id(self.id)
                .test_support()
                .w_full()
                .overflow_hidden()
                .bg(palette.container)
                .border_2()
                .border_dashed()
                .border_color(palette.border)
                .text_color(palette.content),
            self.shape,
            direction,
        )
        .when(direction == Direction::Rtl, Styled::flex_row_reverse)
        .when(!self.disabled, move |surface| {
            surface.drag_over::<T>(move |style, payload, window, cx| {
                let accepted = visual_can_drop
                    .as_ref()
                    .is_none_or(|predicate| predicate(payload, window, cx));
                if accepted {
                    style
                        .bg(palette.active_container)
                        .border_color(palette.active_border)
                        .text_color(palette.active_content)
                } else {
                    style
                        .bg(palette.invalid_container)
                        .border_color(palette.invalid_border)
                        .text_color(palette.invalid_content)
                }
            })
        })
        .child(receiver)
    }
}

use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, ElementId, Entity, FocusHandle, Hsla, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Button as BaseButton, Collapsible as BaseCollapsible, TestSupportExt};
use gpui_material_components::{Divider, Text};
use gpui_material_foundation::{
    FocusRingPlacement, InteractionSource, Surface, SurfaceBorder, ThemeScope, color,
    control_feedback::{activate_feedback, feedback_target},
    current_material_theme, resolve_edges,
};
use gpui_material_theme::{ColorRole, CornerRole, CornerShape, Direction, LogicalEdges, TypeRole};

use super::{
    disclosure_chevron, disclosure_motion, feedback_overlay, keyed_feedback, keyed_focus,
    top_only_corners,
};

type OpenChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// Per-instance color overrides for [`Collapsible`]. Omitted fields use the window theme.
#[derive(Clone, Copy, Debug, Default)]
pub struct CollapsibleColors {
    /// Background color for the collapsible container surface.
    pub container: Option<Hsla>,
    /// Optional background color override for the header trigger row.
    pub header: Option<Hsla>,
    /// Foreground color for the header title.
    pub title: Option<Hsla>,
    /// Foreground color for the optional supporting text.
    pub supporting: Option<Hsla>,
    /// Foreground color for leading, trailing, and disclosure indicator icons.
    pub icon: Option<Hsla>,
    /// Border color when [`Collapsible::bordered`] is enabled.
    pub border: Option<Hsla>,
    /// Divider color between the header and expanded body.
    pub divider: Option<Hsla>,
    /// Title color when disabled.
    pub disabled_title: Option<Hsla>,
    /// Icon and indicator color when disabled.
    pub disabled_icon: Option<Hsla>,
}

/// Material 3 single expandable/collapsible container wrapping `gpui_base::Collapsible`.
///
/// The application owns the `open` state and applies requests reported through
/// [`Self::on_open_change`].
#[derive(IntoElement)]
pub struct Collapsible {
    id: ElementId,
    title: SharedString,
    open: bool,
    supporting: Option<SharedString>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    header_content: Option<AnyElement>,
    indicator: Option<AnyElement>,
    show_indicator: bool,
    content: Option<AnyElement>,
    keep_mounted: bool,
    disabled: bool,
    bordered: bool,
    divider: bool,
    shape: Option<CornerShape>,
    colors: CollapsibleColors,
    content_padding: LogicalEdges,
    focus: Option<FocusHandle>,
    interaction_source: Option<Entity<InteractionSource>>,
    on_open_change: Option<OpenChangeHandler>,
}

impl Collapsible {
    /// Creates a controlled Material 3 collapsible container with stable `id`, accessible
    /// `title`, and current `open` state.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>, open: bool) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            open,
            supporting: None,
            leading: None,
            trailing: None,
            header_content: None,
            indicator: None,
            show_indicator: true,
            content: None,
            keep_mounted: false,
            disabled: false,
            bordered: true,
            divider: true,
            shape: None,
            colors: CollapsibleColors::default(),
            content_padding: LogicalEdges::all(16.),
            focus: None,
            interaction_source: None,
            on_open_change: None,
        }
    }

    /// Sets optional supporting text displayed beneath the header title.
    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    /// Places a decorative leading icon or element at the logical start of the header.
    pub fn leading(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }

    /// Places a decorative trailing element before the disclosure chevron.
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    /// Replaces the visible header title/supporting block while retaining the constructor
    /// `title` as the accessible button name.
    pub fn header_content(mut self, element: impl IntoElement) -> Self {
        self.header_content = Some(element.into_any_element());
        self
    }

    /// Replaces the default rotating disclosure chevron with custom decorative content.
    pub fn indicator(mut self, element: impl IntoElement) -> Self {
        self.indicator = Some(element.into_any_element());
        self
    }

    /// Shows or hides the trailing disclosure indicator (defaults to `true`).
    pub fn show_indicator(mut self, show: bool) -> Self {
        self.show_indicator = show;
        self
    }

    /// Sets the collapsible body content displayed when expanded.
    pub fn content(mut self, element: impl IntoElement) -> Self {
        self.content = Some(element.into_any_element());
        self
    }

    /// Keeps the collapsible body mounted at zero height while collapsed.
    pub fn keep_mounted(mut self, keep_mounted: bool) -> Self {
        self.keep_mounted = keep_mounted;
        self
    }

    /// Disables user toggling, skips keyboard focus, and applies disabled styling.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Draws a 1 px `OutlineVariant` perimeter border (defaults to `true`).
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Draws a horizontal divider between the header and expanded body (defaults to `true`).
    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = divider;
        self
    }

    /// Overrides the container corner shape (defaults to `CornerRole::Medium`).
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Overrides individual container, text, icon, border, and divider colors.
    pub fn colors(mut self, colors: CollapsibleColors) -> Self {
        self.colors = colors;
        self
    }

    /// Sets logical inner padding for the expanded content region (defaults to 16 dp on all sides).
    pub fn content_padding(mut self, padding: LogicalEdges) -> Self {
        self.content_padding = padding;
        self
    }

    /// Attaches a caller-owned [`FocusHandle`] to the collapsible header trigger.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    /// Observes trigger hover, press, keyboard focus, and disabled state in `source`.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.interaction_source = Some(source.clone());
        self
    }

    /// Handles a user request to toggle the controlled `open` state.
    pub fn on_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Collapsible {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let rtl = direction == Direction::Rtl;
        let density = theme.layout.density;
        let reduced_motion = cx.reduce_motion();

        let on_surface = color(theme.colors.color(ColorRole::OnSurface));
        let on_surface_variant = color(theme.colors.color(ColorRole::OnSurfaceVariant));
        let outline_variant = color(theme.colors.color(ColorRole::OutlineVariant));
        let focus_ring = color(theme.colors.color(ColorRole::Secondary));

        let disabled_fg = on_surface.opacity(0.38);
        let title_color = if self.disabled {
            self.colors.disabled_title.unwrap_or(disabled_fg)
        } else {
            self.colors.title.unwrap_or(on_surface)
        };
        let supporting_color = if self.disabled {
            self.colors.disabled_title.unwrap_or(disabled_fg)
        } else {
            self.colors.supporting.unwrap_or(on_surface_variant)
        };
        let icon_color = if self.disabled {
            self.colors.disabled_icon.unwrap_or(disabled_fg)
        } else {
            self.colors.icon.unwrap_or(on_surface_variant)
        };
        let border_color = self.colors.border.unwrap_or(outline_variant);
        let divider_color = self.colors.divider.unwrap_or(outline_variant);
        let shape = self.shape.unwrap_or_else(|| CornerRole::Medium.shape());

        let motion = disclosure_motion(
            ElementId::from((self.id.clone(), "motion")),
            self.open,
            window,
            cx,
        );

        let trigger_id = ElementId::from((self.id.clone(), "trigger"));
        let content_id = ElementId::from((self.id.clone(), "content"));
        let reveal_id = ElementId::from((self.id.clone(), "reveal"));

        let focus = keyed_focus(&self.id, "trigger", self.focus, window, cx);
        let feedback = keyed_feedback(&self.id, "trigger", window, cx);
        let tracker = self
            .interaction_source
            .as_ref()
            .map(|source| InteractionSource::track(source, &trigger_id, window, cx));
        let enabled = !self.disabled;
        let interaction = {
            let mut state = feedback.borrow_mut();
            state.cancel_unavailable(enabled, window);
            state.track(enabled, &focus, tracker, window, cx)
        };

        let next_open = !self.open;
        let toggle = (enabled)
            .then_some(self.on_open_change)
            .flatten()
            .map(|handler| {
                Rc::new(ThemeScope::bind_action(
                    move |window, cx| handler(next_open, window, cx),
                    window,
                    cx,
                ))
            });

        let header_corners = if self.open || motion.value > 0.01 {
            top_only_corners(shape)
        } else {
            shape
        };

        let header_min_h = px(density.interactive_size(if self.supporting.is_some() {
            56.0
        } else {
            48.0
        }));

        let header_label_block = match self.header_content {
            Some(custom) => div().flex_1().min_w_0().child(custom),
            None => div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .when(rtl, Styled::items_end)
                .child(
                    Text::new(self.title.clone())
                        .role(TypeRole::TitleMedium)
                        .color_override(title_color)
                        .wrap(),
                )
                .children(self.supporting.map(|supporting| {
                    Text::new(supporting)
                        .role(TypeRole::BodySmall)
                        .color_override(supporting_color)
                        .wrap()
                })),
        };

        let indicator = self.show_indicator.then(|| {
            let glyph = self
                .indicator
                .unwrap_or_else(|| disclosure_chevron(motion.value).into_any_element());
            div()
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .text_color(icon_color)
                .child(glyph)
        });

        let mut trigger = BaseButton::new(trigger_id)
            .accessibility_label(self.title)
            .disabled(!enabled)
            .track_focus(&focus)
            .relative()
            .w_full()
            .min_h(header_min_h)
            .px(px(16.))
            .py(px(10.))
            .flex()
            .flex_row()
            .items_center()
            .justify_start()
            .gap(px(12.))
            .when(rtl, Styled::flex_row_reverse)
            .when_some(self.colors.header, |this, bg| this.bg(bg))
            .aria_expanded(self.open)
            .when(enabled, Styled::cursor_pointer)
            .child(feedback_overlay(
                feedback.clone(),
                header_corners,
                direction,
                interaction,
                on_surface,
                focus_ring,
                FocusRingPlacement::Inset,
                reduced_motion,
            ))
            .children(self.leading.map(|leading| {
                div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(icon_color)
                    .child(leading)
            }))
            .child(header_label_block)
            .children(self.trailing.map(|trailing| {
                div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(icon_color)
                    .child(trailing)
            }))
            .children(indicator);

        if enabled {
            trigger = feedback_target(trigger, feedback.clone(), true, Some(focus.clone()));
            if let Some(toggle) = toggle {
                let click_toggle = toggle.clone();
                let click_feedback = feedback.clone();
                let a11y_toggle = toggle;
                trigger = trigger
                    .on_click(move |event, window, cx| {
                        if click_feedback.borrow().blocks_activation(event) {
                            return;
                        }
                        activate_feedback(event, &click_feedback, window, cx);
                        click_toggle(window, cx);
                        cx.stop_propagation();
                    })
                    .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                        a11y_toggle(window, cx);
                    });
            }
        }

        let padding = resolve_edges(self.content_padding, direction);
        let body = self.content.map(|content| {
            div()
                .id(content_id)
                .test_support()
                .role(Role::Region)
                .w_full()
                .flex()
                .flex_col()
                .when(self.divider, |this| {
                    this.child(Divider::horizontal().color_override(divider_color))
                })
                .child(
                    div()
                        .w_full()
                        .pt(padding.top)
                        .pr(padding.right)
                        .pb(padding.bottom)
                        .pl(padding.left)
                        .child(content),
                )
        });

        let mut base = BaseCollapsible::new().open(self.open).child(trigger);
        if let Some(body) = body {
            base = base.content(body);
        }
        if motion.is_running() || (self.keep_mounted && !self.open) {
            base = base.reveal(reveal_id, motion.value);
        }

        let mut surface = Surface::new(ColorRole::SurfaceContainerLow)
            .w_full()
            .flex()
            .flex_col()
            .shape(shape);
        if let Some(container) = self.colors.container {
            surface = surface.container_color(container);
        }
        if self.bordered {
            surface = surface.border(SurfaceBorder {
                width: px(1.),
                color: border_color,
            });
        }

        div()
            .id(self.id)
            .test_support()
            .w_full()
            .child(surface.child(base))
    }
}

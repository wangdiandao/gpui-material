use std::{f32::consts::PI, rc::Rc};

use gpui::{
    AccessibleAction, App, Axis, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px, radians,
};
use gpui_base::{
    Button as BaseButton, Pagination as BasePagination, PaginationItem as BasePaginationItem,
    PaginationState as BasePaginationState, TestSupportExt,
};
use gpui_material_components::{ControlSize, Icon, IconButton, IconName, Text};
use gpui_material_foundation::{
    FocusRingPlacement, ThemeScope, color,
    control_feedback::{activate_feedback, feedback_target},
    current_material_theme, move_roving_focus,
};
use gpui_material_theme::{ColorRole, CornerRole, CornerShape, Direction, TypeRole};

use super::{feedback_overlay, keyed_feedback, keyed_focus};

type PageChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Per-instance color overrides for [`Pagination`]. Omitted fields use the window theme.
#[derive(Clone, Copy, Debug, Default)]
pub struct PaginationColors {
    /// Background color for the active page pill (defaults to `PrimaryContainer`).
    pub active_container: Option<Hsla>,
    /// Foreground color for the active page number (defaults to `OnPrimaryContainer`).
    pub active_content: Option<Hsla>,
    /// Optional background color for inactive page pills (defaults to transparent).
    pub inactive_container: Option<Hsla>,
    /// Foreground color for inactive page numbers (defaults to `OnSurface`).
    pub inactive_content: Option<Hsla>,
    /// Foreground color for ellipsis indicators (defaults to `OnSurfaceVariant`).
    pub ellipsis: Option<Hsla>,
    /// Foreground color for disabled controls (defaults to `OnSurface` at 38% opacity).
    pub disabled_content: Option<Hsla>,
}

/// Material 3 desktop pagination bar wrapping `gpui_base::Pagination`.
///
/// Renders RTL-mirrored previous and next icon buttons, active `PrimaryContainer` page
/// pills, and ellipsis truncation for long page ranges. The application owns `current_page`
/// and applies requested updates from [`Self::on_change`].
#[derive(IntoElement)]
pub struct Pagination {
    id: ElementId,
    current_page: usize,
    total_pages: usize,
    visible_pages: usize,
    disabled: bool,
    accessible_label: Option<SharedString>,
    previous_label: Option<SharedString>,
    next_label: Option<SharedString>,
    shape: Option<CornerShape>,
    colors: PaginationColors,
    on_change: Option<PageChangeHandler>,
}

impl Pagination {
    /// Creates a controlled Material 3 pagination bar for 1-based `current_page` out of
    /// `total_pages`. Values below `1` are clamped into `1..=total_pages.max(1)`.
    pub fn new(id: impl Into<ElementId>, current_page: usize, total_pages: usize) -> Self {
        Self {
            id: id.into(),
            current_page,
            total_pages,
            visible_pages: 5,
            disabled: false,
            accessible_label: None,
            previous_label: None,
            next_label: None,
            shape: None,
            colors: PaginationColors::default(),
            on_change: None,
        }
    }

    /// Sets the visible page budget used before inserting ellipses (clamped to at least `5`).
    pub fn visible_pages(mut self, visible_pages: usize) -> Self {
        self.visible_pages = visible_pages.max(5);
        self
    }

    /// Disables previous/next buttons and all page pills.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Overrides the `Role::Navigation` landmark name (defaults to `"Pagination"`).
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    /// Overrides the accessible name of the previous-page button (defaults to `"Previous page"`).
    pub fn previous_label(mut self, label: impl Into<SharedString>) -> Self {
        self.previous_label = Some(label.into());
        self
    }

    /// Overrides the accessible name of the next-page button (defaults to `"Next page"`).
    pub fn next_label(mut self, label: impl Into<SharedString>) -> Self {
        self.next_label = Some(label.into());
        self
    }

    /// Overrides the corner shape of page number pills (defaults to `CornerRole::Full`).
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Overrides active, inactive, ellipsis, and disabled colors.
    pub fn colors(mut self, colors: PaginationColors) -> Self {
        self.colors = colors;
        self
    }

    /// Handles a requested 1-based page change.
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

fn resolved_items(state: &BasePaginationState) -> Vec<BasePaginationItem> {
    let items = state.items();
    if items.is_empty() {
        vec![BasePaginationItem::Page(1)]
    } else {
        items
    }
}

impl RenderOnce for Pagination {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let rtl = direction == Direction::Rtl;
        let density = theme.layout.density;
        let reduced_motion = cx.reduce_motion();

        let on_surface = color(theme.colors.color(ColorRole::OnSurface));
        let on_surface_variant = color(theme.colors.color(ColorRole::OnSurfaceVariant));
        let primary_container = color(theme.colors.color(ColorRole::PrimaryContainer));
        let on_primary_container = color(theme.colors.color(ColorRole::OnPrimaryContainer));
        let focus_ring = color(theme.colors.color(ColorRole::Secondary));
        let disabled_fg = self
            .colors
            .disabled_content
            .unwrap_or_else(|| on_surface.opacity(0.38));

        let active_bg = self.colors.active_container.unwrap_or(primary_container);
        let active_fg = self.colors.active_content.unwrap_or(on_primary_container);
        let inactive_fg = self.colors.inactive_content.unwrap_or(on_surface);
        let ellipsis_fg = self.colors.ellipsis.unwrap_or(on_surface_variant);
        let pill_shape = self.shape.unwrap_or_else(|| CornerRole::Full.shape());
        let pill_size = px(density.visual_size(36.0).max(32.0));

        let mut base_state = BasePaginationState::new(self.current_page, self.total_pages)
            .visible_pages(self.visible_pages)
            .disabled(self.disabled);
        if let Some(on_change) = self.on_change {
            base_state = base_state.on_change(move |page, window, cx| {
                on_change(page, window, cx);
            });
        }

        let current_page = base_state.current_page();
        let prev_target = base_state.previous_page();
        let next_target = base_state.next_page();
        let items = resolved_items(&base_state);

        let nav_label = self
            .accessible_label
            .unwrap_or_else(|| SharedString::from("Pagination"));
        let prev_label = self
            .previous_label
            .unwrap_or_else(|| SharedString::from("Previous page"));
        let next_label = self
            .next_label
            .unwrap_or_else(|| SharedString::from("Next page"));

        let prev_id = ElementId::from((self.id.clone(), "previous"));
        let next_id = ElementId::from((self.id.clone(), "next"));
        let prev_focus = keyed_focus(&self.id, "previous", None, window, cx);
        let next_focus = keyed_focus(&self.id, "next", None, window, cx);

        let mut roving: Vec<(FocusHandle, bool)> = Vec::with_capacity(items.len() + 2);
        roving.push((prev_focus.clone(), prev_target.is_some()));

        let mut page_focuses: Vec<Option<FocusHandle>> = Vec::with_capacity(items.len());
        for item in &items {
            match item {
                BasePaginationItem::Page(page) => {
                    let handle = keyed_focus(&self.id, format!("page-{page}"), None, window, cx);
                    roving.push((handle.clone(), !self.disabled));
                    page_focuses.push(Some(handle));
                }
                BasePaginationItem::Ellipsis(_) => page_focuses.push(None),
            }
        }
        roving.push((next_focus.clone(), next_target.is_some()));

        let prev_state = base_state.clone();
        let prev_button = IconButton::standard(
            prev_id,
            prev_label,
            Icon::new(IconName::ChevronRight)
                .rotate(radians(PI))
                .mirror_in_rtl(true),
        )
        .size(ControlSize::ExtraSmall)
        .track_focus(&prev_focus)
        .disabled(prev_target.is_none())
        .on_activate(move |window, cx| {
            if let Some(target) = prev_state.previous_page() {
                prev_state.request_page(target, window, cx);
            }
        });

        let next_state = base_state.clone();
        let next_button = IconButton::standard(
            next_id,
            next_label,
            Icon::new(IconName::ChevronRight).mirror_in_rtl(true),
        )
        .size(ControlSize::ExtraSmall)
        .track_focus(&next_focus)
        .disabled(next_target.is_none())
        .on_activate(move |window, cx| {
            if let Some(target) = next_state.next_page() {
                next_state.request_page(target, window, cx);
            }
        });

        let mut nav = BasePagination::new(
            ElementId::from((self.id.clone(), "nav")),
            base_state.clone(),
        )
        .accessibility_label(nav_label)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .when(rtl, Styled::flex_row_reverse)
        .on_key_down(move |event, window, cx| {
            if event.keystroke.modifiers.modified() {
                return;
            }
            if !roving
                .iter()
                .any(|(focus, enabled)| *enabled && focus.is_focused(window))
            {
                return;
            }
            let refs: Vec<(&FocusHandle, bool)> = roving
                .iter()
                .map(|(focus, enabled)| (focus, *enabled))
                .collect();
            if move_roving_focus(
                &refs,
                event.keystroke.key.as_str(),
                Axis::Horizontal,
                direction,
                window,
                cx,
            )
            .is_some()
            {
                cx.stop_propagation();
            }
        })
        .child(prev_button);

        let pill_radius = pill_shape.radii(36., 36., direction)[0];
        for (item, page_focus) in items.into_iter().zip(page_focuses) {
            match item {
                BasePaginationItem::Page(page) => {
                    let is_active = page == current_page;
                    let enabled = !self.disabled;
                    let pill_id = ElementId::from((self.id.clone(), format!("page-{page}")));
                    let focus = page_focus.expect("page item always allocates focus handle");
                    let feedback = keyed_feedback(&self.id, format!("page-{page}"), window, cx);
                    let interaction = {
                        let mut state = feedback.borrow_mut();
                        state.cancel_unavailable(enabled, window);
                        state.interaction(enabled, &focus, window)
                    };

                    let bg = if is_active {
                        if enabled {
                            Some(active_bg)
                        } else {
                            Some(on_surface.opacity(0.12))
                        }
                    } else {
                        self.colors.inactive_container
                    };
                    let fg = if !enabled {
                        disabled_fg
                    } else if is_active {
                        active_fg
                    } else {
                        inactive_fg
                    };
                    let aria_name: SharedString = if is_active {
                        format!("Page {page}, current page").into()
                    } else {
                        format!("Page {page}").into()
                    };

                    let state_for_click = base_state.clone();
                    let state_for_a11y = base_state.clone();
                    let request_click = Rc::new(ThemeScope::bind_action(
                        move |window, cx| state_for_click.request_page(page, window, cx),
                        window,
                        cx,
                    ));
                    let request_a11y = Rc::new(ThemeScope::bind_action(
                        move |window, cx| state_for_a11y.request_page(page, window, cx),
                        window,
                        cx,
                    ));

                    let mut pill = BaseButton::new(pill_id)
                        .accessibility_label(aria_name)
                        .disabled(!enabled)
                        .track_focus(&focus)
                        .relative()
                        .min_w(pill_size)
                        .h(pill_size)
                        .px(px(10.))
                        .rounded(px(pill_radius))
                        .when_some(bg, Styled::bg)
                        .aria_selected(is_active)
                        .when(enabled && !is_active, Styled::cursor_pointer)
                        .child(feedback_overlay(
                            feedback.clone(),
                            pill_shape,
                            direction,
                            interaction,
                            fg,
                            focus_ring,
                            FocusRingPlacement::Outer,
                            reduced_motion,
                        ))
                        .child(
                            Text::new(page.to_string())
                                .role(TypeRole::LabelLarge)
                                .emphasized(is_active)
                                .color_override(fg),
                        );

                    if enabled {
                        pill = feedback_target(pill, feedback.clone(), true, Some(focus.clone()));
                        let click_feedback = feedback.clone();
                        pill = pill
                            .on_click(move |event, window, cx| {
                                if click_feedback.borrow().blocks_activation(event) {
                                    return;
                                }
                                activate_feedback(event, &click_feedback, window, cx);
                                request_click(window, cx);
                                cx.stop_propagation();
                            })
                            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                request_a11y(window, cx);
                            });
                    }

                    nav = nav.child(pill);
                }
                BasePaginationItem::Ellipsis(range) => {
                    let ellipsis_id =
                        ElementId::from((self.id.clone(), format!("ellipsis-{}", range.start)));
                    nav = nav.child(
                        div()
                            .id(ellipsis_id)
                            .test_support()
                            .min_w(pill_size)
                            .h(pill_size)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(Text::new("…").role(TypeRole::LabelLarge).color_override(
                                if self.disabled {
                                    disabled_fg
                                } else {
                                    ellipsis_fg
                                },
                            )),
                    );
                }
            }
        }

        nav = nav.child(next_button);

        div().id(self.id).test_support().child(nav)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_page_still_yields_active_page_one() {
        let state = BasePaginationState::new(1, 1);
        assert_eq!(resolved_items(&state), vec![BasePaginationItem::Page(1)]);
    }

    #[test]
    fn long_page_range_inserts_ellipses() {
        let state = BasePaginationState::new(6, 12).visible_pages(5);
        let items = resolved_items(&state);
        assert!(items.contains(&BasePaginationItem::Page(1)));
        assert!(items.contains(&BasePaginationItem::Page(6)));
        assert!(items.contains(&BasePaginationItem::Page(12)));
        assert!(
            items
                .iter()
                .filter(|item| matches!(item, BasePaginationItem::Ellipsis(_)))
                .count()
                == 2
        );
    }
}

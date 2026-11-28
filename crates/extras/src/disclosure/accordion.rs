use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, Axis, ElementId, Entity, FocusHandle, Hsla,
    InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{
    Accordion as BaseAccordion, AccordionHeader as BaseAccordionHeader,
    AccordionItem as BaseAccordionItem, AccordionPanel as BaseAccordionPanel,
    AccordionTrigger as BaseAccordionTrigger, MotionReveal,
};
use gpui_material_components::{Divider, Text};
use gpui_material_foundation::{
    FocusRingPlacement, InteractionSource, Surface, SurfaceBorder, ThemeScope, color,
    control_feedback::{activate_feedback, feedback_target},
    current_material_theme, move_roving_focus, resolve_edges,
};
use gpui_material_theme::{ColorRole, CornerRole, CornerShape, Direction, LogicalEdges, TypeRole};

use super::{
    bottom_only_corners, disclosure_chevron, disclosure_motion, feedback_overlay, keyed_feedback,
    keyed_focus, top_only_corners,
};

type ExpandedChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;
type ToggleCallback = Rc<dyn Fn(&mut Window, &mut App)>;

/// Expansion policy for [`Accordion`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AccordionMode {
    /// At most one item can be expanded at a time; activating the open item collapses it.
    #[default]
    Single,
    /// Exactly one item remains expanded; activating the currently open item is ignored.
    SingleRequired,
    /// Any number of items can be expanded independently.
    Multiple,
}

/// Visual grouping treatment for [`Accordion`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AccordionVariant {
    /// Contiguous surface container with horizontal dividers between items.
    #[default]
    Standard,
    /// Material 3 Expressive segmented surfaces with grouped outer/inner corners and a 2 dp gap.
    Segmented,
}

/// Per-instance color overrides for [`Accordion`]. Omitted fields use the window theme.
#[derive(Clone, Copy, Debug, Default)]
pub struct AccordionColors {
    /// Surface container color for collapsed items (and the standard outer container).
    pub container: Option<Hsla>,
    /// Optional container color override for expanded items.
    pub expanded_container: Option<Hsla>,
    /// Foreground color for item titles.
    pub title: Option<Hsla>,
    /// Foreground color for optional item supporting text.
    pub supporting: Option<Hsla>,
    /// Foreground color for leading, trailing, and chevron icons.
    pub icon: Option<Hsla>,
    /// Perimeter border color when [`Accordion::bordered`] is enabled.
    pub border: Option<Hsla>,
    /// Divider color between items and between headers and panels.
    pub divider: Option<Hsla>,
    /// Title color when disabled.
    pub disabled_title: Option<Hsla>,
    /// Icon color when disabled.
    pub disabled_icon: Option<Hsla>,
}

/// One controlled section inside an [`Accordion`].
pub struct AccordionItem {
    id: SharedString,
    title: SharedString,
    supporting: Option<SharedString>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    header_content: Option<AnyElement>,
    content: Option<AnyElement>,
    disabled: bool,
    keep_mounted: bool,
    heading_level: usize,
    focus: Option<FocusHandle>,
    interaction_source: Option<Entity<InteractionSource>>,
}

impl AccordionItem {
    /// Creates an accordion section with a stable `id` and accessible `title`.
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            supporting: None,
            leading: None,
            trailing: None,
            header_content: None,
            content: None,
            disabled: false,
            keep_mounted: false,
            heading_level: 3,
            focus: None,
            interaction_source: None,
        }
    }

    /// Sets optional supporting text displayed beneath the item title.
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

    /// Replaces the visible header title block while preserving `title` as the accessible name.
    pub fn header_content(mut self, element: impl IntoElement) -> Self {
        self.header_content = Some(element.into_any_element());
        self
    }

    /// Sets the panel body displayed while this item is expanded.
    pub fn content(mut self, element: impl IntoElement) -> Self {
        self.content = Some(element.into_any_element());
        self
    }

    /// Disables this item's trigger and skips it during roving keyboard navigation.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Keeps the panel mounted in the element tree while collapsed.
    pub fn keep_mounted(mut self, keep_mounted: bool) -> Self {
        self.keep_mounted = keep_mounted;
        self
    }

    /// Sets the heading accessibility level (defaults to `3`).
    pub fn heading_level(mut self, level: usize) -> Self {
        self.heading_level = level.max(1);
        self
    }

    /// Attaches a caller-owned [`FocusHandle`] to this item's trigger.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    /// Observes this trigger's interaction state under `ElementId::from((accordion_id, item_id))`.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.interaction_source = Some(source.clone());
        self
    }
}

/// Material 3 controlled single- or multi-expand accordion wrapping `gpui_base::Accordion`.
#[derive(IntoElement)]
pub struct Accordion {
    id: ElementId,
    items: Vec<AccordionItem>,
    accessible_label: Option<SharedString>,
    mode: AccordionMode,
    variant: AccordionVariant,
    expanded: Vec<SharedString>,
    on_change: Option<ExpandedChangeHandler>,
    disabled: bool,
    bordered: bool,
    spacing: Option<Pixels>,
    content_padding: LogicalEdges,
    colors: AccordionColors,
}

impl Accordion {
    /// Creates an accordion container from an ordered collection of [`AccordionItem`] entries.
    pub fn new(id: impl Into<ElementId>, items: impl IntoIterator<Item = AccordionItem>) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().collect(),
            accessible_label: None,
            mode: AccordionMode::Single,
            variant: AccordionVariant::Standard,
            expanded: Vec::new(),
            on_change: None,
            disabled: false,
            bordered: true,
            spacing: None,
            content_padding: LogicalEdges::all(16.),
            colors: AccordionColors::default(),
        }
    }

    /// Sets an optional accessible label on the `Role::Group` accordion root.
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    /// Sets the expansion mode (`Single`, `SingleRequired`, or `Multiple`).
    pub fn mode(mut self, mode: AccordionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the visual variant (`Standard` or `Segmented`).
    pub fn variant(mut self, variant: AccordionVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the controlled list of currently expanded item IDs.
    pub fn expanded<I, S>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.expanded = ids.into_iter().map(Into::into).collect();
        self
    }

    /// Sets a single controlled expanded item ID (convenience for `Single`/`SingleRequired` modes).
    pub fn expanded_id(mut self, id: Option<impl Into<SharedString>>) -> Self {
        self.expanded = id.into_iter().map(Into::into).collect();
        self
    }

    /// Handles requested updates to the controlled expanded item IDs.
    pub fn on_change(
        mut self,
        handler: impl Fn(Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Convenience callback for single-expand modes reporting the requested active item ID.
    pub fn on_single_change(
        mut self,
        handler: impl Fn(Option<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        if self.mode == AccordionMode::Multiple {
            self.mode = AccordionMode::Single;
        }
        self.on_change = Some(Rc::new(move |ids, window, cx| {
            handler(ids.into_iter().next(), window, cx);
        }));
        self
    }

    /// Disables all items in the accordion (combined with per-item [`AccordionItem::disabled`]).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Draws a 1 px `OutlineVariant` border on the accordion surface(s) (defaults to `true`).
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Overrides vertical spacing between items in [`AccordionVariant::Segmented`] (defaults to 2 dp).
    pub fn spacing(mut self, spacing: Pixels) -> Self {
        if f32::from(spacing).is_finite() {
            self.spacing = Some(spacing.max(px(0.)));
        }
        self
    }

    /// Sets logical inner padding for expanded item panels (defaults to 16 dp on all sides).
    pub fn content_padding(mut self, padding: LogicalEdges) -> Self {
        self.content_padding = padding;
        self
    }

    /// Overrides container, title, supporting text, icon, border, and divider colors.
    pub fn colors(mut self, colors: AccordionColors) -> Self {
        self.colors = colors;
        self
    }
}

fn next_expanded_ids(
    mode: AccordionMode,
    current: &[SharedString],
    item_order: &[SharedString],
    toggled: &SharedString,
) -> Option<Vec<SharedString>> {
    let is_open = current.iter().any(|id| id == toggled);
    match mode {
        AccordionMode::Single => Some(if is_open {
            Vec::new()
        } else {
            vec![toggled.clone()]
        }),
        AccordionMode::SingleRequired => {
            if is_open {
                None
            } else {
                Some(vec![toggled.clone()])
            }
        }
        AccordionMode::Multiple => {
            let mut next = Vec::with_capacity(current.len() + 1);
            for id in item_order {
                let currently_open = current.iter().any(|open| open == id);
                let should_be_open = if id == toggled {
                    !currently_open
                } else {
                    currently_open
                };
                if should_be_open {
                    next.push(id.clone());
                }
            }
            Some(next)
        }
    }
}

fn segmented_corners(index: usize, count: usize) -> CornerShape {
    let outer = 16.0;
    let inner = 4.0;
    if count <= 1 {
        CornerShape::uniform(outer)
    } else if index == 0 {
        CornerShape::Radii([outer, outer, inner, inner])
    } else if index + 1 == count {
        CornerShape::Radii([inner, inner, outer, outer])
    } else {
        CornerShape::uniform(inner)
    }
}

impl RenderOnce for Accordion {
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
        let border_color = self.colors.border.unwrap_or(outline_variant);
        let divider_color = self.colors.divider.unwrap_or(outline_variant);

        let item_count = self.items.len();
        let item_order: Vec<SharedString> = self.items.iter().map(|item| item.id.clone()).collect();
        let padding = resolve_edges(self.content_padding, direction);

        let mut focus_entries: Vec<(FocusHandle, bool, bool)> = Vec::with_capacity(item_count);
        for item in &self.items {
            let enabled = !self.disabled && !item.disabled;
            let open = self.expanded.iter().any(|id| id == &item.id);
            let handle = keyed_focus(&self.id, item.id.clone(), item.focus.clone(), window, cx);
            focus_entries.push((handle, enabled, open));
        }

        let active_tab_stop = focus_entries
            .iter()
            .position(|(focus, enabled, _)| *enabled && focus.is_focused(window))
            .or_else(|| {
                focus_entries
                    .iter()
                    .position(|(_, enabled, open)| *enabled && *open)
            })
            .or_else(|| focus_entries.iter().position(|(_, enabled, _)| *enabled));

        let roving_handles: Vec<(FocusHandle, bool)> = focus_entries
            .iter()
            .map(|(focus, enabled, _)| (focus.clone(), *enabled))
            .collect();

        let mut base = BaseAccordion::new(self.id.clone())
            .w_full()
            .flex()
            .flex_col()
            .when_some(self.accessible_label.clone(), |this, label| {
                this.aria_label(label)
            })
            .when(self.variant == AccordionVariant::Segmented, |this| {
                this.gap(self.spacing.unwrap_or(px(2.)))
            });

        if !roving_handles.is_empty() {
            let roving = roving_handles.clone();
            base = base.on_key_down(move |event, window, cx| {
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
                    Axis::Vertical,
                    direction,
                    window,
                    cx,
                )
                .is_some()
                {
                    cx.stop_propagation();
                }
            });
        }

        let standard_shape = CornerRole::Medium.shape();
        let mode = self.mode;
        let expanded_ids = Rc::new(self.expanded);
        let order_rc = Rc::new(item_order);
        let on_change = self.on_change;

        for (ix, item) in self.items.into_iter().enumerate() {
            let (focus, enabled, open) = focus_entries[ix].clone();
            let is_tab_stop = active_tab_stop == Some(ix);
            let trigger_id = ElementId::from((self.id.clone(), item.id.clone()));
            let header_id = ElementId::from((self.id.clone(), format!("{}-header", item.id)));
            let panel_id = ElementId::from((self.id.clone(), format!("{}-panel", item.id)));
            let motion_id = ElementId::from((self.id.clone(), format!("{}-motion", item.id)));
            let reveal_id = ElementId::from((self.id.clone(), format!("{}-reveal", item.id)));

            let motion = disclosure_motion(motion_id, open, window, cx);
            let feedback = keyed_feedback(&self.id, item.id.clone(), window, cx);
            let tracker = item
                .interaction_source
                .as_ref()
                .map(|source| InteractionSource::track(source, &trigger_id, window, cx));
            let interaction = {
                let mut state = feedback.borrow_mut();
                state.cancel_unavailable(enabled, window);
                state.track(enabled, &focus, tracker, window, cx)
            };

            let title_color = if !enabled {
                self.colors.disabled_title.unwrap_or(disabled_fg)
            } else {
                self.colors.title.unwrap_or(on_surface)
            };
            let supporting_color = if !enabled {
                self.colors.disabled_title.unwrap_or(disabled_fg)
            } else {
                self.colors.supporting.unwrap_or(on_surface_variant)
            };
            let icon_color = if !enabled {
                self.colors.disabled_icon.unwrap_or(disabled_fg)
            } else {
                self.colors.icon.unwrap_or(on_surface_variant)
            };

            let item_shape = match self.variant {
                AccordionVariant::Standard => {
                    if item_count <= 1 {
                        standard_shape
                    } else if ix == 0 {
                        top_only_corners(standard_shape)
                    } else if ix + 1 == item_count {
                        bottom_only_corners(standard_shape)
                    } else {
                        CornerShape::uniform(0.0)
                    }
                }
                AccordionVariant::Segmented => segmented_corners(ix, item_count),
            };
            let trigger_corners = if open || motion.value > 0.01 {
                top_only_corners(item_shape)
            } else {
                item_shape
            };

            let request_toggle: Option<ToggleCallback> = (enabled)
                .then_some(on_change.as_ref())
                .flatten()
                .map(|handler| {
                    let handler = handler.clone();
                    let current = expanded_ids.clone();
                    let order = order_rc.clone();
                    let toggled_id = item.id.clone();
                    let action: ToggleCallback = Rc::new(ThemeScope::bind_action(
                        move |window, cx| {
                            if let Some(next) =
                                next_expanded_ids(mode, &current, &order, &toggled_id)
                            {
                                handler(next, window, cx);
                            }
                        },
                        window,
                        cx,
                    ));
                    action
                });

            let header_min_h = px(density.interactive_size(if item.supporting.is_some() {
                56.0
            } else {
                48.0
            }));

            let header_label_block = match item.header_content {
                Some(custom) => div().flex_1().min_w_0().child(custom),
                None => div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .when(rtl, Styled::items_end)
                    .child(
                        Text::new(item.title.clone())
                            .role(TypeRole::TitleMedium)
                            .color_override(title_color)
                            .wrap(),
                    )
                    .children(item.supporting.map(|supporting| {
                        Text::new(supporting)
                            .role(TypeRole::BodySmall)
                            .color_override(supporting_color)
                            .wrap()
                    })),
            };

            let mut trigger = BaseAccordionTrigger::new(trigger_id)
                .open(open)
                .disabled(!enabled)
                .relative()
                .w_full()
                .min_h(header_min_h)
                .px(px(16.))
                .py(px(10.))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(12.))
                .when(rtl, Styled::flex_row_reverse)
                .aria_label(item.title.clone())
                .when(enabled, |this| {
                    this.cursor_pointer()
                        .track_focus(&focus)
                        .tab_stop(is_tab_stop)
                })
                .when(!enabled, |this| {
                    this.a11y_synthetic_children(|tree| tree.parent_node().set_disabled())
                })
                .child(feedback_overlay(
                    feedback.clone(),
                    trigger_corners,
                    direction,
                    interaction,
                    on_surface,
                    focus_ring,
                    FocusRingPlacement::Inset,
                    reduced_motion,
                ))
                .children(item.leading.map(|leading| {
                    div()
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(icon_color)
                        .child(leading)
                }))
                .child(header_label_block)
                .children(item.trailing.map(|trailing| {
                    div()
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(icon_color)
                        .child(trailing)
                }))
                .child(
                    div()
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(icon_color)
                        .child(disclosure_chevron(motion.value)),
                );

            if enabled {
                trigger = feedback_target(trigger, feedback.clone(), true, Some(focus.clone()));
                if let Some(toggle) = request_toggle {
                    let click_toggle = toggle.clone();
                    let click_feedback = feedback.clone();
                    let a11y_toggle = toggle;
                    trigger = trigger
                        .on_change(move |_, event, window, cx| {
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

            let header = BaseAccordionHeader::new(trigger)
                .id(header_id)
                .level(item.heading_level)
                .w_full();

            let panel = item.content.map(|content| {
                let inner = div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .child(Divider::horizontal().color_override(divider_color))
                    .child(
                        div()
                            .w_full()
                            .pt(padding.top)
                            .pr(padding.right)
                            .pb(padding.bottom)
                            .pl(padding.left)
                            .child(content),
                    );
                let panel_child: AnyElement = if motion.is_running() || (item.keep_mounted && !open)
                {
                    MotionReveal::new(reveal_id, motion.value, inner.into_any_element())
                        .into_any_element()
                } else {
                    inner.into_any_element()
                };
                BaseAccordionPanel::new()
                    .id(panel_id)
                    .open(open)
                    .keep_mounted(item.keep_mounted || motion.is_running())
                    .w_full()
                    .child(panel_child)
            });

            let mut base_item = BaseAccordionItem::new()
                .open(open)
                .disabled(!enabled)
                .w_full()
                .flex()
                .flex_col()
                .header(header);
            if let Some(panel) = panel {
                base_item = base_item.panel(panel);
            }

            match self.variant {
                AccordionVariant::Standard => {
                    if ix > 0 {
                        base = base.child(Divider::horizontal().color_override(divider_color));
                    }
                    let expanded_bg = if open {
                        self.colors.expanded_container
                    } else {
                        None
                    };
                    base = base.child(
                        div()
                            .w_full()
                            .when_some(expanded_bg, Styled::bg)
                            .child(base_item),
                    );
                }
                AccordionVariant::Segmented => {
                    let mut card = Surface::new(ColorRole::SurfaceContainerLow)
                        .w_full()
                        .flex()
                        .flex_col()
                        .shape(item_shape);
                    let custom_bg = if open {
                        self.colors.expanded_container.or(self.colors.container)
                    } else {
                        self.colors.container
                    };
                    if let Some(bg) = custom_bg {
                        card = card.container_color(bg);
                    }
                    if self.bordered {
                        card = card.border(SurfaceBorder {
                            width: px(1.),
                            color: border_color,
                        });
                    }
                    base = base.child(card.child(base_item));
                }
            }
        }

        match self.variant {
            AccordionVariant::Standard => {
                let mut container = Surface::new(ColorRole::SurfaceContainerLow)
                    .w_full()
                    .flex()
                    .flex_col()
                    .shape(standard_shape);
                if let Some(bg) = self.colors.container {
                    container = container.container_color(bg);
                }
                if self.bordered {
                    container = container.border(SurfaceBorder {
                        width: px(1.),
                        color: border_color,
                    });
                }
                container.child(base).into_any_element()
            }
            AccordionVariant::Segmented => base.into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(list: &[&str]) -> Vec<SharedString> {
        list.iter().map(|s| SharedString::from(*s)).collect()
    }

    #[test]
    fn single_mode_switches_and_collapses_active_item() {
        let order = ids(&["a", "b", "c"]);
        assert_eq!(
            next_expanded_ids(AccordionMode::Single, &[], &order, &"b".into()),
            Some(ids(&["b"]))
        );
        assert_eq!(
            next_expanded_ids(AccordionMode::Single, &ids(&["b"]), &order, &"c".into()),
            Some(ids(&["c"]))
        );
        assert_eq!(
            next_expanded_ids(AccordionMode::Single, &ids(&["c"]), &order, &"c".into()),
            Some(Vec::new())
        );
    }

    #[test]
    fn single_required_mode_keeps_active_item_open() {
        let order = ids(&["a", "b"]);
        assert_eq!(
            next_expanded_ids(
                AccordionMode::SingleRequired,
                &ids(&["a"]),
                &order,
                &"a".into()
            ),
            None
        );
        assert_eq!(
            next_expanded_ids(
                AccordionMode::SingleRequired,
                &ids(&["a"]),
                &order,
                &"b".into()
            ),
            Some(ids(&["b"]))
        );
    }

    #[test]
    fn multiple_mode_toggles_items_preserving_accordion_order() {
        let order = ids(&["a", "b", "c"]);
        assert_eq!(
            next_expanded_ids(AccordionMode::Multiple, &ids(&["c"]), &order, &"a".into()),
            Some(ids(&["a", "c"]))
        );
        assert_eq!(
            next_expanded_ids(
                AccordionMode::Multiple,
                &ids(&["a", "c"]),
                &order,
                &"c".into()
            ),
            Some(ids(&["a"]))
        );
    }
}

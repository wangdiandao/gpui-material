use std::{cell::Cell, rc::Rc};

use gpui::{
    App, Axis, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, Role,
    ScrollHandle, SharedString, Size, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_components::ControlElevations;
use gpui_material_foundation::{
    animate, color, current_material_strings, current_material_theme, move_roving_focus,
};
use gpui_material_theme::{CornerShape, Direction, Elevation, MotionKind, MotionSpec, MotionSpeed};

use super::{
    FabContent, Toolbar, ToolbarFabPosition, ToolbarOrientation, ToolbarPlacement,
    actions::{
        partition_actions, render_fab, render_overflow_menu, render_visible_actions,
        sync_focused_scroll,
    },
    focus_handle,
    layout::{StripGeometry, ToolbarSlot, apply_padding, arrange_horizontal},
    palette::{
        ScrollPresentation, ToolbarPalette, resolve_scroll_presentation, resolve_target_elevation,
    },
};

impl RenderOnce for Toolbar {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if let Some(navigation) = self.navigation {
            return navigation;
        }
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let density = theme.layout.density;
        let horizontal = self.orientation == ToolbarOrientation::Horizontal;
        let rtl = direction == Direction::Rtl;
        let docked = self.placement == ToolbarPlacement::Docked;
        let palette = ToolbarPalette::resolve(&self, |role| color(theme.colors.color(role)));
        let scheme = theme.motion;
        let reduced = cx.reduce_motion();
        let transition = if reduced {
            MotionSpec::Instant
        } else {
            self.transition_motion
                .unwrap_or_else(|| scheme.spec(MotionKind::Spatial, MotionSpeed::Fast, false))
        };
        let viewport = window.viewport_size();
        let scope = focus_handle(&self.id, "strip-focus", window, cx);
        let exposed = scope.contains_focused(window, cx);
        let footprint = window
            .use_keyed_state((self.id.clone(), "strip-footprint"), cx, |_, _| {
                Rc::new(Cell::new(Size::<Pixels>::default()))
            })
            .read(cx)
            .clone();
        let behavior = self.scroll_behavior.unwrap_or_default();
        let ScrollPresentation {
            slide,
            hidden_strip,
        } = resolve_scroll_presentation(
            &mut self, &footprint, behavior, scheme, reduced, exposed, rtl, window, cx,
        );
        let target_elevation =
            resolve_target_elevation(self.placement, self.expanded, self.elevations);
        let uses_elevation = !docked
            || target_elevation != Elevation::Level0
            || self.elevations != ControlElevations::default();
        let elevation = if uses_elevation {
            animate(
                (self.id.clone(), "toolbar-elevation"),
                target_elevation.dp(),
                transition,
                window,
                cx,
            )
            .max(0.)
        } else {
            0.
        };
        let custom_surface = self.shape.is_some()
            || uses_elevation
            || behavior.exit_direction.is_some()
            || self.content.is_some()
            || self.leading.is_some()
            || self.trailing.is_some()
            || self.arrangement.is_some() && docked && horizontal;
        let arrangement = self.arrangement.filter(|_| docked && horizontal);
        let spacing = self.spacing.min(if horizontal {
            viewport.width
        } else {
            viewport.height
        });
        let strip_min = px(density.interactive_size(64.));
        let action_size = px(density.interactive_size(48.));
        let toolbar_scroll = window
            .use_keyed_state((self.id.clone(), "action-scroll"), cx, |_, _| {
                ScrollHandle::new()
            })
            .read(cx)
            .clone();

        let actions = std::mem::take(&mut self.actions);
        let (visible, hidden) = partition_actions(&self.id, actions, self.expanded, window, cx);
        let has_more = !hidden.is_empty();
        let overflow_focus = focus_handle(&self.id, "overflow-focus", window, cx);
        let fab_focus = focus_handle(&self.id, "fab-focus", window, cx);
        let fab = self.fab.take().filter(|_| !docked);
        let fab_first = fab
            .as_ref()
            .is_some_and(|fab| fab.position == ToolbarFabPosition::Start);
        let mut bar_order = visible
            .iter()
            .map(|(action, handle)| (handle.clone(), action.enabled))
            .collect::<Vec<_>>();
        if has_more {
            bar_order.push((overflow_focus.clone(), true));
        }
        let first_tab = bar_order
            .iter()
            .position(|(handle, enabled)| *enabled && handle.is_focused(window))
            .or_else(|| bar_order.iter().position(|(_, enabled)| *enabled));
        let mirrored = horizontal && rtl;
        let standard_fab = fab
            .as_ref()
            .is_some_and(|fab| matches!(fab.content, FabContent::Standard { .. }));
        let fab_element =
            fab.map(|fab| render_fab(&self, fab, &fab_focus, palette.default_fab_color));

        let mut action_strip = div()
            .id((self.id.clone(), "actions"))
            .test_support()
            .track_focus(&scope)
            .tab_stop(false)
            .flex()
            .items_center()
            .min_w_0()
            .min_h_0()
            .flex_shrink(1.)
            .when(horizontal, |strip| strip.max_w_full().overflow_x_scroll())
            .when(!horizontal, |strip| {
                strip.flex_col().max_h_full().overflow_y_scroll()
            })
            .track_scroll(&toolbar_scroll)
            .text_color(palette.ink)
            .when(!custom_surface, |strip| {
                strip
                    .bg(palette.surface)
                    .when(!docked, Styled::rounded_full)
            })
            .when(docked && horizontal, Styled::w_full)
            .when(docked && !horizontal, Styled::h_full)
            .when_else(
                horizontal,
                |strip| strip.min_h(strip_min),
                |strip| strip.min_w(strip_min),
            )
            .when_else(
                horizontal,
                |strip| strip.px(px(if docked { 16. } else { 8. })).py(px(8.)),
                |strip| strip.py(px(if docked { 16. } else { 8. })).px(px(8.)),
            )
            .when_some(
                self.expanded_height.filter(|_| horizontal && docked),
                |strip, height| {
                    strip
                        .h(height.min(viewport.height))
                        .min_h_0()
                        .overflow_y_hidden()
                },
            )
            .gap(if arrangement.is_some() {
                px(0.)
            } else {
                spacing
            });
        if let Some(padding) = self.padding {
            action_strip = apply_padding(padding, action_strip, direction, viewport);
        }

        let mut commands =
            render_visible_actions(&self.id, visible, first_tab, action_size, palette);
        if has_more {
            let count = bar_order.len();
            let overflow_label = self.overflow_label.unwrap_or_else(|| {
                SharedString::from(current_material_strings(window, cx).more_actions.clone())
            });
            commands.push(render_overflow_menu(
                &self.id,
                overflow_label,
                hidden,
                overflow_focus,
                first_tab == Some(count - 1),
                horizontal,
                direction,
                action_size,
                palette,
            ));
        }

        let mut strip_children = Vec::new();
        if hidden_strip {
            bar_order.clear();
        } else {
            let mut side = |part, element, expanded: Option<bool>| {
                let slot = ToolbarSlot::new(&self.id, part, element, window, cx);
                slot.expandable(
                    expanded.unwrap_or(self.expanded),
                    horizontal,
                    transition,
                    window,
                    cx,
                )
            };
            let leading = self
                .leading
                .take()
                .and_then(|element| side("leading", element, self.leading_expanded));
            let trailing = self
                .trailing
                .take()
                .and_then(|element| side("trailing", element, self.trailing_expanded));
            strip_children.extend(leading);
            strip_children.extend(
                self.content.take().map(|element| {
                    ToolbarSlot::new(&self.id, "body", element, window, cx).content()
                }),
            );
            let first_command = strip_children.len();
            strip_children.extend(commands);
            strip_children.extend(trailing);

            sync_focused_scroll(
                &self.id,
                &strip_children,
                first_command,
                first_tab,
                arrangement.is_some(),
                mirrored,
                &toolbar_scroll,
                window,
                cx,
            );
        }

        let mut children = strip_children
            .into_iter()
            .map(|(element, _)| element)
            .collect::<Vec<_>>();
        if let Some(arrangement) = arrangement {
            children = arrange_horizontal(children, arrangement, spacing);
        }
        if mirrored {
            children.reverse();
        }
        action_strip = action_strip.children(children);
        let action_strip = if custom_surface {
            StripGeometry {
                horizontal,
                docked,
                direction,
                footprint,
                track_footprint: behavior.exit_direction.is_some(),
                shape: self.shape.unwrap_or(if docked {
                    CornerShape::Radii([0.; 4])
                } else {
                    CornerShape::Full
                }),
                surface: palette.surface,
                shadow: palette.shadow,
                elevation,
                slide,
                hidden: hidden_strip,
            }
            .frame(action_strip)
        } else {
            action_strip.into_any_element()
        };
        if standard_fab {
            let index = if fab_first { 0 } else { bar_order.len() };
            bar_order.insert(index, (fab_focus, true));
        }
        let root = div()
            .id(self.id)
            .test_support()
            .role(Role::Toolbar)
            .aria_label(self.label)
            .aria_orientation(if horizontal {
                gpui::accesskit::Orientation::Horizontal
            } else {
                gpui::accesskit::Orientation::Vertical
            })
            .flex()
            .items_center()
            .min_w_0()
            .min_h_0()
            .max_w_full()
            .max_h_full()
            .when(horizontal && rtl, Styled::flex_row_reverse)
            .when(!horizontal, Styled::flex_col)
            .when(docked && horizontal, Styled::w_full)
            .when(docked && !horizontal, Styled::h_full)
            .gap(px(4.))
            .on_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified()
                    || !bar_order
                        .iter()
                        .any(|(handle, _)| handle.is_focused(window))
                {
                    return;
                }
                let items = bar_order
                    .iter()
                    .map(|(handle, enabled)| (handle, *enabled))
                    .collect::<Vec<_>>();
                if move_roving_focus(
                    &items,
                    &event.keystroke.key,
                    if horizontal {
                        Axis::Horizontal
                    } else {
                        Axis::Vertical
                    },
                    direction,
                    window,
                    cx,
                )
                .is_some()
                {
                    window.refresh();
                    cx.stop_propagation();
                }
            });
        if fab_first {
            root.children(fab_element)
                .child(action_strip)
                .into_any_element()
        } else {
            root.child(action_strip)
                .children(fab_element)
                .into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui_material_components::ControlElevations;
    use gpui_material_theme::{Elevation, LogicalEdges};

    use super::*;
    use crate::ContentArrangement;

    #[test]
    fn floating_toolbar_defaults_to_level3_when_expanded_and_level0_when_collapsed() {
        assert_eq!(
            resolve_target_elevation(
                ToolbarPlacement::Floating,
                true,
                ControlElevations::default()
            ),
            Elevation::Level3
        );
        assert_eq!(
            resolve_target_elevation(
                ToolbarPlacement::Floating,
                false,
                ControlElevations::default()
            ),
            Elevation::Level0
        );
        assert_eq!(
            resolve_target_elevation(ToolbarPlacement::Docked, true, ControlElevations::default()),
            Elevation::Level0
        );
        assert_eq!(
            resolve_target_elevation(
                ToolbarPlacement::Docked,
                false,
                ControlElevations::default()
            ),
            Elevation::Level0
        );
    }

    #[test]
    fn control_elevations_override_expanded_and_collapsed_toolbar_levels() {
        let overrides = ControlElevations {
            resting: Some(Elevation::Level1),
            disabled: Some(Elevation::Level2),
            ..Default::default()
        };
        assert_eq!(
            resolve_target_elevation(ToolbarPlacement::Floating, true, overrides),
            Elevation::Level1
        );
        assert_eq!(
            resolve_target_elevation(ToolbarPlacement::Floating, false, overrides),
            Elevation::Level2
        );
    }

    #[test]
    fn toolbar_builder_accepts_unified_primitives() {
        let toolbar = Toolbar::new("bar", "Actions", [])
            .arrangement(ContentArrangement::SpaceEvenly)
            .padding(LogicalEdges::new(4., 8., 6., 10.))
            .elevations(ControlElevations {
                resting: Some(Elevation::Level2),
                disabled: Some(Elevation::Level0),
                ..Default::default()
            });
        assert_eq!(toolbar.arrangement, Some(ContentArrangement::SpaceEvenly));
        assert_eq!(toolbar.padding, Some(LogicalEdges::new(4., 8., 6., 10.)));
        assert_eq!(toolbar.elevations.resting, Some(Elevation::Level2));
        assert_eq!(toolbar.elevations.disabled, Some(Elevation::Level0));
    }
}

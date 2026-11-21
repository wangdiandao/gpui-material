use std::{cell::Cell, rc::Rc};

use gpui::{
    Anchor, AnyElement, App, Axis, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, Point, RenderOnce, Role, ScrollHandle, SharedString, Size,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Button, TestSupportExt};
use gpui_material_components::{Fab, FabColor, FabSize, Icon, IconName, Menu, MenuItem};
use gpui_material_foundation::{animate, color, current_material_theme, move_roving_focus};
use gpui_material_theme::{
    ColorRole, CornerShape, Direction, Elevation, MotionKind, MotionScheme, MotionSpec, MotionSpeed,
};

use super::{
    FabContent, Invoke, Toolbar, ToolbarAction, ToolbarColor, ToolbarElevations,
    ToolbarExitDirection, ToolbarFab, ToolbarFabPosition, ToolbarOrientation, ToolbarPlacement,
    ToolbarScrollBehavior, ToolbarScrollState, activate_key, focus_handle,
    layout::{StripGeometry, ToolbarSlot, arrange_horizontal},
    style_button,
};

#[derive(Clone, Copy)]
struct ToolbarPalette {
    surface: Hsla,
    ink: Hsla,
    selected_fill: Hsla,
    selected_ink: Hsla,
    ring: Hsla,
    shadow: Hsla,
    default_fab_color: FabColor,
}

impl ToolbarPalette {
    fn resolve(toolbar: &Toolbar, tone: impl Fn(ColorRole) -> Hsla) -> Self {
        let (surface_role, content_role, selected_role, selected_ink_role, default_fab_color) =
            match toolbar.color {
                ToolbarColor::Standard => (
                    ColorRole::SurfaceContainer,
                    ColorRole::OnSurfaceVariant,
                    ColorRole::SecondaryContainer,
                    ColorRole::OnSecondaryContainer,
                    FabColor::Primary,
                ),
                ToolbarColor::Vibrant => (
                    ColorRole::PrimaryContainer,
                    ColorRole::OnPrimaryContainer,
                    ColorRole::SurfaceContainer,
                    ColorRole::OnSurface,
                    FabColor::Tertiary,
                ),
            };
        Self {
            surface: toolbar
                .colors
                .container
                .unwrap_or_else(|| tone(surface_role)),
            ink: toolbar.colors.content.unwrap_or_else(|| tone(content_role)),
            selected_fill: toolbar
                .colors
                .selected_container
                .unwrap_or_else(|| tone(selected_role)),
            selected_ink: toolbar
                .colors
                .selected_content
                .unwrap_or_else(|| tone(selected_ink_role)),
            ring: tone(ColorRole::OnSurface),
            shadow: tone(ColorRole::Shadow),
            default_fab_color,
        }
    }
}

struct ScrollPresentation {
    slide: Point<Pixels>,
    hidden_strip: bool,
}

#[allow(clippy::too_many_arguments)]
fn resolve_scroll_presentation(
    toolbar: &mut Toolbar,
    footprint: &Rc<Cell<Size<Pixels>>>,
    behavior: ToolbarScrollBehavior,
    scheme: MotionScheme,
    reduced: bool,
    exposed: bool,
    rtl: bool,
    window: &mut Window,
    cx: &mut App,
) -> ScrollPresentation {
    let limit = behavior.exit_direction.map_or(0., |edge| {
        let size = footprint.get();
        f32::from(match edge {
            ToolbarExitDirection::Start | ToolbarExitDirection::End => size.width,
            ToolbarExitDirection::Top | ToolbarExitDirection::Bottom => size.height,
        })
    });
    let scroll_state = toolbar.scroll_state.take().or_else(|| {
        (toolbar.scroll.is_some() || toolbar.scroll_behavior.is_some()).then(|| {
            window.use_keyed_state((toolbar.id.clone(), "scroll-expansion"), cx, |_, _| {
                ToolbarScrollState::new()
            })
        })
    });
    let Some(state) = scroll_state else {
        return ScrollPresentation {
            slide: Point::default(),
            hidden_strip: false,
        };
    };
    let linked = toolbar.scroll.as_ref().map(|scroll| -scroll.offset().y);
    let position = state.update(cx, |state, _| state.prepare(linked, limit, behavior));
    let fraction = animate(
        (toolbar.id.clone(), "toolbar-scroll"),
        position.target,
        position.motion.spec(behavior, scheme, reduced),
        window,
        cx,
    )
    .clamp(0., 1.);
    toolbar.expanded &= position.expanded;
    state.update(cx, |state, cx| {
        if state.display(position, fraction, limit, exposed) {
            cx.notify();
        }
    });
    let distance = px(if exposed { 0. } else { fraction * limit });
    let hidden_strip = limit > 0. && distance >= px(limit);
    let slide = match behavior.exit_direction {
        Some(ToolbarExitDirection::Top) => gpui::point(px(0.), -distance),
        Some(ToolbarExitDirection::Bottom) => gpui::point(px(0.), distance),
        Some(ToolbarExitDirection::Start) if rtl => gpui::point(distance, px(0.)),
        Some(ToolbarExitDirection::End) if !rtl => gpui::point(distance, px(0.)),
        Some(ToolbarExitDirection::Start | ToolbarExitDirection::End) => {
            gpui::point(-distance, px(0.))
        }
        None => Point::default(),
    };
    ScrollPresentation {
        slide,
        hidden_strip,
    }
}

fn partition_actions(
    id: &ElementId,
    actions: Vec<ToolbarAction>,
    expanded: bool,
    window: &mut Window,
    cx: &mut App,
) -> (Vec<(ToolbarAction, FocusHandle)>, Vec<ToolbarAction>) {
    let mut visible = Vec::new();
    let mut hidden = Vec::new();
    for action in actions {
        let action_id = ElementId::from((id.clone(), action.id.clone()));
        let handle = focus_handle(&action_id, "focus", window, cx);
        // Keep the active keyboard target mounted during scroll collapse.
        if expanded || !action.secondary || handle.is_focused(window) {
            visible.push((action, handle));
        } else {
            hidden.push(action);
        }
    }
    (visible, hidden)
}

fn render_fab(
    toolbar: &Toolbar,
    fab: ToolbarFab,
    fab_focus: &FocusHandle,
    default_color: FabColor,
) -> AnyElement {
    let content = match fab.content {
        FabContent::Standard {
            icon,
            label,
            invoke,
        } => Fab::new((toolbar.id.clone(), "fab"), label, Icon::new(icon))
            .track_focus(fab_focus)
            .color(toolbar.fab_color.unwrap_or(default_color))
            .colors(toolbar.fab_colors)
            .size(if toolbar.expanded {
                FabSize::Small
            } else {
                FabSize::Medium
            })
            .on_activate(move |window, cx| invoke(window, cx))
            .into_any_element(),
        FabContent::Custom(content) => content,
    };
    div().flex_none().child(content).into_any_element()
}

fn render_visible_actions(
    id: &ElementId,
    visible: Vec<(ToolbarAction, FocusHandle)>,
    first_tab: Option<usize>,
    palette: ToolbarPalette,
) -> Vec<(AnyElement, FocusHandle)> {
    visible
        .into_iter()
        .enumerate()
        .map(|(index, (action, handle))| {
            let callback = action.invoke;
            let selected = action.selected.unwrap_or(false);
            let fill = if selected {
                palette.selected_fill
            } else {
                palette.surface
            };
            let foreground = if !action.enabled {
                palette.ink.opacity(0.38)
            } else if selected {
                palette.selected_ink
            } else {
                palette.ink
            };
            let button = Button::new((id.clone(), action.id))
                .track_focus(&handle)
                .tab_stop(first_tab == Some(index))
                .accessibility_label(action.label)
                .disabled(!action.enabled)
                .size(px(48.))
                .flex_none()
                .rounded_full()
                .child(Icon::new(action.icon).size(px(24.)))
                .when_some(action.selected, |button, selected| {
                    button.aria_toggled(if selected {
                        gpui::accesskit::Toggled::True
                    } else {
                        gpui::accesskit::Toggled::False
                    })
                })
                .when_some(callback.filter(|_| action.enabled), |button, callback| {
                    let key = callback.clone();
                    let a11y = callback.clone();
                    button
                        .on_click(move |_, window, cx| callback(window, cx))
                        .on_a11y_action(gpui::AccessibleAction::Click, move |_, window, cx| {
                            a11y(window, cx)
                        })
                        .capture_key_down(move |event, window, cx| {
                            activate_key(event, window, cx, &*key)
                        })
                });
            (
                style_button(button, fill, foreground, palette.ring, action.enabled)
                    .into_any_element(),
                handle,
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn render_overflow_menu(
    id: &ElementId,
    overflow_label: SharedString,
    hidden: Vec<ToolbarAction>,
    overflow_focus: FocusHandle,
    is_tab_stop: bool,
    horizontal: bool,
    palette: ToolbarPalette,
) -> (AnyElement, FocusHandle) {
    let entries = hidden.into_iter().map(|action| {
        let item = MenuItem::new(action.id, action.label).disabled(!action.enabled);
        let item = if let Some(selected) = action.selected {
            item.checked(selected)
        } else {
            item
        };
        if let Some(invoke) = action.invoke {
            item.on_activate(move |window, cx| invoke(window, cx))
        } else {
            item
        }
    });
    let trigger_id = id.clone();
    let scroll_focus = overflow_focus.clone();
    let trigger = move |open, toggle: Invoke| {
        let focus = overflow_focus.clone();
        let accessibility_toggle = toggle.clone();
        let button = Button::new((trigger_id, "overflow"))
            .accessibility_label(overflow_label)
            .aria_expanded(open)
            .track_focus(&overflow_focus)
            .tab_stop(is_tab_stop)
            .size(px(48.))
            .rounded_full()
            .on_a11y_action(gpui::AccessibleAction::Click, move |_, window, cx| {
                focus.focus(window, cx);
                toggle(window, cx);
            })
            .on_a11y_action(
                if open {
                    gpui::AccessibleAction::Collapse
                } else {
                    gpui::AccessibleAction::Expand
                },
                {
                    let focus = overflow_focus.clone();
                    move |_, window, cx| {
                        focus.focus(window, cx);
                        accessibility_toggle(window, cx);
                    }
                },
            )
            .child(Icon::new(IconName::ArrowDropDown).size(px(24.)));
        style_button(button, palette.surface, palette.ink, palette.ring, true).into_any_element()
    };
    (
        div()
            .flex_none()
            .child(
                Menu::composed((id.clone(), "overflow-menu"), entries, trigger).anchor(
                    if horizontal {
                        Anchor::BottomRight
                    } else {
                        Anchor::LeftCenter
                    },
                ),
            )
            .into_any_element(),
        scroll_focus,
    )
}

#[allow(clippy::too_many_arguments)]
fn sync_focused_scroll(
    id: &ElementId,
    strip_children: &[(AnyElement, FocusHandle)],
    first_command: usize,
    first_tab: Option<usize>,
    arranged: bool,
    mirrored: bool,
    toolbar_scroll: &ScrollHandle,
    window: &mut Window,
    cx: &mut App,
) {
    // Direct child indices include slot and spacer elements. The scroll handle
    // uses their physical order, including the reversal of a horizontal RTL strip.
    let mut children_count = strip_children.len();
    if arranged && children_count > 0 {
        children_count = children_count * 2 + 1;
    }
    let reveal_index = strip_children
        .iter()
        .position(|(_, handle)| handle.contains_focused(window, cx))
        .or_else(|| first_tab.map(|index| first_command + index))
        .map(|index| {
            let index = if arranged { index * 2 + 1 } else { index };
            if mirrored {
                children_count - index - 1
            } else {
                index
            }
        });
    let reveal = window.use_keyed_state((id.clone(), "focused-action"), cx, |_, _| {
        None::<(usize, Size<Pixels>)>
    });
    let target = reveal_index.map(|index| (index, toolbar_scroll.bounds().size));
    reveal.update(cx, |previous, _| {
        if *previous != target {
            if let Some((index, _)) = target {
                toolbar_scroll.scroll_to_item(index);
            }
            *previous = target;
        }
    });
}

impl RenderOnce for Toolbar {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if let Some(navigation) = self.navigation {
            return navigation;
        }
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
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
        let custom_surface = self.shape.is_some()
            || self.elevations != ToolbarElevations::default()
            || behavior.exit_direction.is_some()
            || self.content.is_some()
            || self.leading.is_some()
            || self.trailing.is_some()
            || self.arrangement.is_some() && docked && horizontal;
        let elevation = if self.elevations != ToolbarElevations::default() {
            let elevation = if self.expanded {
                self.elevations.expanded
            } else {
                self.elevations.collapsed
            };
            animate(
                (self.id.clone(), "toolbar-elevation"),
                elevation.unwrap_or(Elevation::Level0).dp(),
                transition,
                window,
                cx,
            )
            .max(0.)
        } else {
            0.
        };
        let arrangement = self.arrangement.filter(|_| docked && horizontal);
        let spacing = self.spacing.min(if horizontal {
            viewport.width
        } else {
            viewport.height
        });
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
                |strip| strip.min_h(px(64.)),
                |strip| strip.min_w(px(64.)),
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
            action_strip = padding.apply(action_strip, direction, viewport);
        }

        let mut commands = render_visible_actions(&self.id, visible, first_tab, palette);
        if has_more {
            let count = bar_order.len();
            commands.push(render_overflow_menu(
                &self.id,
                self.overflow_label,
                hidden,
                overflow_focus,
                first_tab == Some(count - 1),
                horizontal,
                palette,
            ));
        }

        let mut strip_children = Vec::new();
        if hidden_strip {
            bar_order.clear();
        } else {
            let mut side = |part, element, expanded: Option<bool>| {
                let slot = ToolbarSlot::new(&self.id, part, element, window, cx);
                // Intrinsic measurement and prepaint must see the same GPUI
                // ID path for stateful controls like an editor.
                window.with_id(self.id.clone(), |window| {
                    window.with_id((self.id.clone(), "actions"), |window| {
                        slot.expandable(
                            expanded.unwrap_or(self.expanded),
                            horizontal,
                            transition,
                            palette.ink,
                            window,
                            cx,
                        )
                    })
                })
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

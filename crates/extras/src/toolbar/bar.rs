use std::{cell::Cell, rc::Rc};

use super::{Invoke, activate_key, focus_handle, style_button};
use gpui::{
    Anchor, AnyElement, App, Axis, ElementId, Entity, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Role, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Button, TestSupportExt};
use gpui_material_components::{
    ButtonColors, Fab, FabColor, FabSize, Icon, IconName, Menu, MenuItem,
};
use gpui_material_foundation::{animate, color, current_material_theme, move_roving_focus};
use gpui_material_theme::{
    ColorRole, CornerShape, Direction, Elevation, MotionKind, MotionSpec, MotionSpeed,
};

mod layout;
mod scroll;

use layout::{StripGeometry, ToolbarSlot, arrange_horizontal};
pub use layout::{ToolbarArrangement, ToolbarElevations, ToolbarPadding};
use scroll::length;
pub use scroll::{ToolbarExitDirection, ToolbarFling, ToolbarScrollBehavior, ToolbarScrollState};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarPlacement {
    #[default]
    Floating,
    Docked,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarColor {
    #[default]
    Standard,
    Vibrant,
}

/// Optional toolbar pigments. The adjacent FAB uses its own palette.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ToolbarColors {
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub selected_container: Option<Hsla>,
    pub selected_content: Option<Hsla>,
}

/// Places the FAB in logical order. Horizontal positions mirror in RTL.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToolbarFabPosition {
    Start,
    #[default]
    End,
}

pub struct ToolbarAction {
    id: SharedString,
    label: SharedString,
    icon: IconName,
    enabled: bool,
    selected: Option<bool>,
    secondary: bool,
    invoke: Option<Invoke>,
}

impl ToolbarAction {
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: IconName,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon,
            enabled: true,
            selected: None,
            secondary: false,
            invoke: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.enabled = !disabled;
        self
    }

    /// Marks a controlled toggle and specifies its current state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// Places this action in the overflow menu when the toolbar is collapsed.
    pub fn secondary(mut self, secondary: bool) -> Self {
        self.secondary = secondary;
        self
    }

    pub fn on_activate(mut self, invoke: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.invoke = Some(std::rc::Rc::new(invoke));
        self
    }
}

struct ToolbarFab {
    position: ToolbarFabPosition,
    content: FabContent,
}

enum FabContent {
    Standard {
        icon: IconName,
        label: SharedString,
        invoke: Invoke,
    },
    Custom(AnyElement),
}

/// A floating or docked group of labeled actions with optional adjacent FAB.
/// Supply a bounded main-axis extent for scrollable overflow. The toolbar
/// reveals the focused action even when its scroll container is narrower.
#[derive(IntoElement)]
pub struct Toolbar {
    id: ElementId,
    label: SharedString,
    actions: Vec<ToolbarAction>,
    placement: ToolbarPlacement,
    orientation: ToolbarOrientation,
    color: ToolbarColor,
    fab: Option<ToolbarFab>,
    expanded: bool,
    scroll: Option<ScrollHandle>,
    navigation: Option<AnyElement>,
    overflow_label: SharedString,
    content: Option<AnyElement>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    leading_expanded: Option<bool>,
    trailing_expanded: Option<bool>,
    arrangement: Option<ToolbarArrangement>,
    spacing: Pixels,
    expanded_height: Option<Pixels>,
    padding: Option<ToolbarPadding>,
    shape: Option<CornerShape>,
    elevations: ToolbarElevations,
    colors: ToolbarColors,
    fab_color: Option<FabColor>,
    fab_colors: ButtonColors,
    scroll_state: Option<Entity<ToolbarScrollState>>,
    scroll_behavior: Option<ToolbarScrollBehavior>,
    transition_motion: Option<MotionSpec>,
}

impl Toolbar {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        actions: impl IntoIterator<Item = ToolbarAction>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            actions: actions.into_iter().collect(),
            placement: ToolbarPlacement::Floating,
            orientation: ToolbarOrientation::Horizontal,
            color: ToolbarColor::Standard,
            fab: None,
            expanded: true,
            scroll: None,
            navigation: None,
            overflow_label: "More actions".into(),
            content: None,
            leading: None,
            trailing: None,
            leading_expanded: None,
            trailing_expanded: None,
            arrangement: None,
            spacing: px(4.),
            expanded_height: None,
            padding: None,
            shape: None,
            elevations: ToolbarElevations::default(),
            colors: ToolbarColors::default(),
            fab_color: None,
            fab_colors: ButtonColors::default(),
            scroll_state: None,
            scroll_behavior: None,
            transition_motion: None,
        }
    }

    pub fn placement(mut self, placement: ToolbarPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn orientation(mut self, orientation: ToolbarOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn color(mut self, color: ToolbarColor) -> Self {
        self.color = color;
        self
    }

    /// Adds an adjacent FAB to a floating toolbar. Add the primary action
    /// through the action list when it belongs inside a docked toolbar.
    pub fn fab(
        mut self,
        position: ToolbarFabPosition,
        label: impl Into<SharedString>,
        icon: IconName,
        invoke: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.fab = Some(ToolbarFab {
            position,
            content: FabContent::Standard {
                icon,
                label: label.into(),
                invoke: Rc::new(invoke),
            },
        });
        self
    }

    /// Uses a caller-authored adjacent control in a floating toolbar.
    pub fn fab_content(mut self, position: ToolbarFabPosition, element: impl IntoElement) -> Self {
        self.fab = Some(ToolbarFab {
            position,
            content: FabContent::Custom(element.into_any_element()),
        });
        self
    }

    pub fn fab_color(mut self, color: FabColor) -> Self {
        self.fab_color = Some(color);
        self
    }

    pub fn fab_colors(mut self, colors: ButtonColors) -> Self {
        self.fab_colors = colors;
        self
    }

    /// Inserts arbitrary content before the generated actions.
    pub fn content(mut self, element: impl IntoElement) -> Self {
        self.content = Some(element.into_any_element());
        self
    }

    pub fn leading_content(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }

    pub fn trailing_content(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    pub fn leading_expanded(mut self, expanded: bool) -> Self {
        self.leading_expanded = Some(expanded);
        self
    }

    pub fn trailing_expanded(mut self, expanded: bool) -> Self {
        self.trailing_expanded = Some(expanded);
        self
    }

    /// Distributes free space in a horizontal docked toolbar.
    pub fn arrangement(mut self, arrangement: ToolbarArrangement) -> Self {
        self.arrangement = Some(arrangement);
        self
    }

    pub fn spacing(mut self, spacing: Pixels) -> Self {
        self.spacing = px(length(spacing).unwrap_or(4.));
        self
    }

    /// Sets the height of a horizontal docked toolbar.
    pub fn expanded_height(mut self, height: Pixels) -> Self {
        self.expanded_height = length(height).filter(|height| *height > 0.).map(px);
        self
    }

    pub fn padding(mut self, padding: ToolbarPadding) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    pub fn elevations(mut self, elevations: ToolbarElevations) -> Self {
        self.elevations = elevations;
        self
    }

    pub fn colors(mut self, colors: ToolbarColors) -> Self {
        self.colors = colors;
        self
    }

    /// Controls whether secondary actions appear directly in the toolbar.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Uses the linked content's vertical scroll offset to collapse and expand
    /// the toolbar after a 40 dp change in the scroll direction.
    pub fn track_scroll(mut self, scroll: &ScrollHandle) -> Self {
        self.scroll = Some(scroll.clone());
        self
    }

    pub fn scroll_state(mut self, state: &Entity<ToolbarScrollState>) -> Self {
        self.scroll_state = Some(state.clone());
        self
    }

    pub fn scroll_behavior(mut self, behavior: ToolbarScrollBehavior) -> Self {
        self.scroll_behavior = Some(behavior);
        self
    }

    pub fn transition_motion(mut self, motion: MotionSpec) -> Self {
        self.transition_motion = Some(motion);
        self
    }

    /// Renders the supplied navigation instead of the toolbar. Use this for
    /// a responsive region shared between navigation and contextual actions.
    pub fn navigation(mut self, navigation: impl IntoElement) -> Self {
        self.navigation = Some(navigation.into_any_element());
        self
    }

    pub fn overflow_label(mut self, label: impl Into<SharedString>) -> Self {
        self.overflow_label = label.into();
        self
    }
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
        let (surface_role, content_role, selected_role, selected_ink, fab_color) = match self.color
        {
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
        let tone = |role| color(theme.colors.color(role));
        let surface = self.colors.container.unwrap_or_else(|| tone(surface_role));
        let ink = self.colors.content.unwrap_or_else(|| tone(content_role));
        let selected_fill = self
            .colors
            .selected_container
            .unwrap_or_else(|| tone(selected_role));
        let selected_ink = self
            .colors
            .selected_content
            .unwrap_or_else(|| tone(selected_ink));
        let ring = tone(ColorRole::OnSurface);
        let shadow = tone(ColorRole::Shadow);
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
                Rc::new(Cell::new(gpui::Size::<Pixels>::default()))
            })
            .read(cx)
            .clone();
        let behavior = self.scroll_behavior.unwrap_or_default();
        let limit = behavior.exit_direction.map_or(0., |edge| {
            let size = footprint.get();
            f32::from(match edge {
                ToolbarExitDirection::Start | ToolbarExitDirection::End => size.width,
                ToolbarExitDirection::Top | ToolbarExitDirection::Bottom => size.height,
            })
        });
        let scroll_state = self.scroll_state.take().or_else(|| {
            (self.scroll.is_some() || self.scroll_behavior.is_some()).then(|| {
                window.use_keyed_state((self.id.clone(), "scroll-expansion"), cx, |_, _| {
                    ToolbarScrollState::new()
                })
            })
        });
        let mut slide = gpui::Point::default();
        let mut hidden_strip = false;
        if let Some(state) = scroll_state {
            let linked = self.scroll.as_ref().map(|scroll| -scroll.offset().y);
            let position = state.update(cx, |state, _| state.prepare(linked, limit, behavior));
            let fraction = animate(
                (self.id.clone(), "toolbar-scroll"),
                position.target,
                position.motion.spec(behavior, scheme, reduced),
                window,
                cx,
            )
            .clamp(0., 1.);
            self.expanded &= position.expanded;
            state.update(cx, |state, cx| {
                if state.display(position, fraction, limit, exposed) {
                    cx.notify();
                }
            });
            let distance = px(if exposed { 0. } else { fraction * limit });
            hidden_strip = limit > 0. && distance >= px(limit);
            slide = match behavior.exit_direction {
                Some(ToolbarExitDirection::Top) => gpui::point(px(0.), -distance),
                Some(ToolbarExitDirection::Bottom) => gpui::point(px(0.), distance),
                Some(ToolbarExitDirection::Start) if rtl => gpui::point(distance, px(0.)),
                Some(ToolbarExitDirection::End) if !rtl => gpui::point(distance, px(0.)),
                Some(ToolbarExitDirection::Start | ToolbarExitDirection::End) => {
                    gpui::point(-distance, px(0.))
                }
                None => gpui::Point::default(),
            };
        }
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
        let mut visible = Vec::new();
        let mut hidden = Vec::new();
        for action in self.actions {
            let action_id = ElementId::from((self.id.clone(), action.id.clone()));
            let handle = focus_handle(&action_id, "focus", window, cx);
            // Keep the active keyboard target mounted during scroll collapse.
            if self.expanded || !action.secondary || handle.is_focused(window) {
                visible.push((action, handle));
            } else {
                hidden.push(action);
            }
        }
        let has_more = !hidden.is_empty();
        let overflow_focus = focus_handle(&self.id, "overflow-focus", window, cx);
        let fab_focus = focus_handle(&self.id, "fab-focus", window, cx);
        let fab = self.fab.filter(|_| !docked);
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
        let fab_element = fab.map(|fab| {
            let content = match fab.content {
                FabContent::Standard {
                    icon,
                    label,
                    invoke,
                } => Fab::new((self.id.clone(), "fab"), label, Icon::new(icon))
                    .track_focus(&fab_focus)
                    .color(self.fab_color.unwrap_or(fab_color))
                    .colors(self.fab_colors)
                    .size(if self.expanded {
                        FabSize::Small
                    } else {
                        FabSize::Medium
                    })
                    .on_activate(move |window, cx| invoke(window, cx))
                    .into_any_element(),
                FabContent::Custom(content) => content,
            };
            div().flex_none().child(content)
        });
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
            .text_color(ink)
            .when(!custom_surface, |strip| {
                strip.bg(surface).when(!docked, Styled::rounded_full)
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
        let mut commands = visible
            .into_iter()
            .enumerate()
            .map(|(index, (action, handle))| {
                let callback = action.invoke;
                let selected = action.selected.unwrap_or(false);
                let fill = if selected { selected_fill } else { surface };
                let foreground = if !action.enabled {
                    ink.opacity(0.38)
                } else if selected {
                    selected_ink
                } else {
                    ink
                };
                let button = Button::new((self.id.clone(), action.id))
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
                    style_button(button, fill, foreground, ring, action.enabled).into_any_element(),
                    handle,
                )
            })
            .collect::<Vec<_>>();
        if has_more {
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
            let id = self.id.clone();
            let count = bar_order.len();
            let scroll_focus = overflow_focus.clone();
            let trigger = move |open, toggle: Invoke| {
                let focus = overflow_focus.clone();
                let accessibility_toggle = toggle.clone();
                let button = Button::new((id, "overflow"))
                    .accessibility_label(self.overflow_label)
                    .aria_expanded(open)
                    .track_focus(&overflow_focus)
                    .tab_stop(first_tab == Some(count - 1))
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
                style_button(button, surface, ink, ring, true).into_any_element()
            };
            commands.push((
                div()
                    .flex_none()
                    .child(
                        Menu::composed((self.id.clone(), "overflow-menu"), entries, trigger)
                            .anchor(if horizontal {
                                Anchor::BottomRight
                            } else {
                                Anchor::LeftCenter
                            }),
                    )
                    .into_any_element(),
                scroll_focus,
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
                            ink,
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

            // Direct child indices include slot and spacer elements. The scroll handle
            // uses their physical order, including the reversal of a horizontal RTL strip.
            let mut children_count = strip_children.len();
            if arrangement.is_some() && children_count > 0 {
                children_count = children_count * 2 + 1;
            }
            let reveal_index = strip_children
                .iter()
                .position(|(_, handle)| handle.contains_focused(window, cx))
                .or_else(|| first_tab.map(|index| first_command + index))
                .map(|index| {
                    let index = if arrangement.is_some() {
                        index * 2 + 1
                    } else {
                        index
                    };
                    if mirrored {
                        children_count - index - 1
                    } else {
                        index
                    }
                });
            let reveal = window.use_keyed_state((self.id.clone(), "focused-action"), cx, |_, _| {
                None::<(usize, gpui::Size<Pixels>)>
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
                shape: self.shape.unwrap_or(if docked {
                    CornerShape::Radii([0.; 4])
                } else {
                    CornerShape::Full
                }),
                surface,
                shadow,
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

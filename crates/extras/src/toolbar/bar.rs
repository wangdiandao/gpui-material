use super::{Invoke, activate_key, focus_handle, style_button};
use gpui::{
    Anchor, AnyElement, App, Axis, ElementId, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, Role, ScrollHandle, SharedString, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Button, TestSupportExt};
use gpui_material_components::{Fab, FabColor, FabSize, Icon, IconName, Menu, MenuItem};
use gpui_material_foundation::{color, move_roving_focus, window_material_theme};
use gpui_material_theme::{ColorRole, Direction};

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
    icon: IconName,
    label: SharedString,
    position: ToolbarFabPosition,
    invoke: Invoke,
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
            icon,
            label: label.into(),
            invoke: std::rc::Rc::new(invoke),
        });
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

#[derive(Default)]
struct ScrollExpansion {
    // Track offsets since the last direction change, including short wheel steps.
    last: Option<Pixels>,
    origin: Pixels,
    direction: i8,
    collapsed: bool,
}

impl ScrollExpansion {
    fn update(&mut self, offset: Pixels) -> bool {
        let last = self.last.replace(offset).unwrap_or(offset);
        let direction = if offset > last {
            1
        } else if offset < last {
            -1
        } else {
            self.direction
        };
        if direction != self.direction {
            self.direction = direction;
            self.origin = last;
        }
        if offset <= px(0.) {
            self.collapsed = false;
        } else if offset - self.origin >= px(40.) {
            self.collapsed = true;
        } else if self.origin - offset >= px(40.) {
            self.collapsed = false;
        }
        !self.collapsed
    }
}

impl RenderOnce for Toolbar {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if let Some(navigation) = self.navigation {
            return navigation;
        }
        let theme = window_material_theme(window, cx);
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
        let surface = tone(surface_role);
        let ink = tone(content_role);
        let selected_fill = tone(selected_role);
        let selected_ink = tone(selected_ink);
        let ring = tone(ColorRole::OnSurface);
        if let Some(scroll) = self.scroll.as_ref() {
            let expansion =
                window.use_keyed_state((self.id.clone(), "scroll-expansion"), cx, |_, _| {
                    ScrollExpansion::default()
                });
            self.expanded &=
                expansion.update(cx, |expansion, _| expansion.update(-scroll.offset().y));
        }
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
        let focused = bar_order
            .iter()
            .position(|(handle, _)| handle.is_focused(window));
        let count = bar_order.len();
        let mirrored = horizontal && rtl;
        // GPUI clamps scroll offsets from the physical left edge. Reverse the
        // children before laying out an RTL strip so overflow grows rightward.
        let physical_index = |index| if mirrored { count - index - 1 } else { index };
        let reveal =
            window.use_keyed_state(
                (self.id.clone(), "focused-action"),
                cx,
                |_, _| None::<usize>,
            );
        let reveal_index = focused.or(first_tab).map(physical_index);
        reveal.update(cx, |previous, _| {
            if *previous != reveal_index {
                if let Some(index) = reveal_index {
                    toolbar_scroll.scroll_to_item(index);
                }
                *previous = reveal_index;
            }
        });

        let fab_element = fab.as_ref().map(|fab| {
            let invoke = fab.invoke.clone();
            div().flex_none().child(
                Fab::new(
                    (self.id.clone(), "fab"),
                    fab.label.clone(),
                    Icon::new(fab.icon),
                )
                .track_focus(&fab_focus)
                .color(fab_color)
                .size(if self.expanded {
                    FabSize::Small
                } else {
                    FabSize::Medium
                })
                .on_activate(move |window, cx| invoke(window, cx)),
            )
        });
        let mut action_strip = div()
            .id((self.id.clone(), "actions"))
            .test_support()
            .flex()
            .items_center()
            .min_w_0()
            .min_h_0()
            .when(horizontal, |strip| strip.max_w_full().overflow_x_scroll())
            .when(!horizontal, |strip| {
                strip.flex_col().max_h_full().overflow_y_scroll()
            })
            .track_scroll(&toolbar_scroll)
            .bg(surface)
            .when(!docked, Styled::rounded_full)
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
            .gap(px(4.));
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
                style_button(button, fill, foreground, ring, action.enabled).into_any_element()
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
            let trigger = move |open, toggle: Invoke| {
                let focus = overflow_focus.clone();
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
                    .child(Icon::new(IconName::ArrowDropDown).size(px(24.)));
                style_button(button, surface, ink, ring, true).into_any_element()
            };
            commands.push(
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
            );
        }
        if mirrored {
            commands.reverse();
        }
        action_strip = action_strip.children(commands);
        if fab_element.is_some() {
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
    use super::ScrollExpansion;
    use gpui::px;

    #[test]
    fn scroll_threshold_collapses_and_expands_in_both_directions() {
        let mut expansion = ScrollExpansion::default();
        assert!(expansion.update(px(0.)));
        assert!(expansion.update(px(39.)));
        assert!(!expansion.update(px(40.)));
        assert!(!expansion.update(px(110.)));
        assert!(!expansion.update(px(72.)));
        assert!(expansion.update(px(70.)));
        assert!(expansion.update(px(0.)));
    }
}

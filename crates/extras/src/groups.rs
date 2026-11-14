use std::{cell::Cell, collections::HashMap, rc::Rc};

use gpui::{
    App, Axis, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_material_components::{
    ButtonVariant, ControlSize, Icon, IconName, Menu, MenuItem,
    button_composition::{ButtonPart, Segment, label_width, split_trailing_size},
};
use gpui_material_foundation::{animate, move_roving_focus, window_material_theme};
use gpui_material_theme::{Direction, MotionKind, MotionSpeed};

type Command = Rc<dyn Fn(&mut Window, &mut App)>;
type SelectionChange = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonGroupVariant {
    #[default]
    Standard,
    Connected,
}

/// Selection is controlled by the caller. Required selection preserves the
/// last active choice once the caller has supplied or accepted a choice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonGroupSelection {
    #[default]
    Actions,
    Single,
    SingleRequired,
    Multiple,
    MultipleRequired,
}

impl ButtonGroupSelection {
    fn next(self, selection: &[SharedString], id: &SharedString) -> Option<Vec<SharedString>> {
        let contains = selection.contains(id);
        let required = matches!(self, Self::SingleRequired | Self::MultipleRequired);
        match self {
            Self::Actions => None,
            _ if contains && required && selection.len() == 1 => None,
            _ if contains => Some(
                selection
                    .iter()
                    .filter(|value| *value != id)
                    .cloned()
                    .collect(),
            ),
            Self::Single | Self::SingleRequired => Some(vec![id.clone()]),
            Self::Multiple | Self::MultipleRequired => {
                let mut next = selection.to_vec();
                next.push(id.clone());
                Some(next)
            }
        }
    }
}

/// One entry with a stable, caller-provided identifier.
pub struct ButtonGroupItem {
    id: SharedString,
    label: SharedString,
    disabled: bool,
    command: Option<Command>,
}

impl ButtonGroupItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            command: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Runs in action groups. Selection groups use the group's on_change callback.
    pub fn on_activate(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.command = Some(Rc::new(callback));
        self
    }
}

/// An expressive row with controlled selection and a Material menu for
/// items that do not fit. Arrow keys move focus without changing selection.
#[derive(IntoElement)]
pub struct ButtonGroup {
    id: ElementId,
    label: SharedString,
    items: Vec<ButtonGroupItem>,
    variant: ButtonGroupVariant,
    appearance: ButtonVariant,
    size: ControlSize,
    selection: ButtonGroupSelection,
    selected: Vec<SharedString>,
    on_change: Option<SelectionChange>,
    overflow_label: SharedString,
}

impl ButtonGroup {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = ButtonGroupItem>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            items: items.into_iter().collect(),
            variant: ButtonGroupVariant::default(),
            appearance: ButtonVariant::default(),
            size: ControlSize::default(),
            selection: ButtonGroupSelection::default(),
            selected: Vec::new(),
            on_change: None,
            overflow_label: "More actions".into(),
        }
    }

    pub fn variant(mut self, variant: ButtonGroupVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn appearance(mut self, appearance: ButtonVariant) -> Self {
        self.appearance = appearance;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn selection(
        mut self,
        mode: ButtonGroupSelection,
        selected: impl IntoIterator<Item = SharedString>,
    ) -> Self {
        self.selection = mode;
        self.selected = selected.into_iter().collect();
        self
    }

    pub fn on_change(
        mut self,
        callback: impl Fn(Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(callback));
        self
    }

    pub fn overflow_label(mut self, label: impl Into<SharedString>) -> Self {
        self.overflow_label = label.into();
        self
    }
}

struct GroupState {
    width: Rc<Cell<Pixels>>,
    press: Rc<Cell<Option<usize>>>,
    focus: HashMap<SharedString, FocusHandle>,
    overflow_focus: FocusHandle,
}

impl RenderOnce for ButtonGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let direction = theme.layout.direction;
        let spec = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Fast, cx.reduce_motion());
        let connected = self.variant == ButtonGroupVariant::Connected;
        let gap = if connected { px(2.) } else { px(12.) };
        let width = window.viewport_size().width;
        let state =
            window.use_keyed_state((self.id.clone(), "group-state"), cx, |_, cx| GroupState {
                width: Rc::new(Cell::new(width)),
                press: Rc::new(Cell::new(None)),
                focus: HashMap::new(),
                overflow_focus: cx.focus_handle(),
            });
        let (width, press, handles, overflow_focus) = state.update(cx, |state, cx| {
            let mut prior = std::mem::take(&mut state.focus);
            state.focus = self
                .items
                .iter()
                .map(|item| {
                    let handle = prior.remove(&item.id).unwrap_or_else(|| cx.focus_handle());
                    (item.id.clone(), handle)
                })
                .collect();
            (
                state.width.clone(),
                state.press.clone(),
                state.focus.clone(),
                state.overflow_focus.clone(),
            )
        });
        let widths: Vec<_> = self
            .items
            .iter()
            .map(|item| label_width(self.size, item.label.clone(), window, cx))
            .collect();
        let overflow_width = px(split_trailing_size(self.size).1);
        let shown = visible_count(&widths, gap, overflow_width, width.get());
        let overflow = shown < self.items.len();
        let count = shown + usize::from(overflow);
        let selected: Vec<_> = self
            .items
            .iter()
            .filter(|item| self.selected.contains(&item.id))
            .map(|item| item.id.clone())
            .take(
                if matches!(
                    self.selection,
                    ButtonGroupSelection::Single | ButtonGroupSelection::SingleRequired
                ) {
                    1
                } else {
                    usize::MAX
                },
            )
            .collect();
        let focus: Vec<_> = self.items[..shown]
            .iter()
            .map(|item| (handles[&item.id].clone(), !item.disabled))
            .chain(overflow.then(|| (overflow_focus.clone(), true)))
            .collect();
        let tab_stop = focus
            .iter()
            .position(|(handle, enabled)| *enabled && handle.is_focused(window))
            .or_else(|| {
                self.items[..shown]
                    .iter()
                    .position(|item| !item.disabled && selected.contains(&item.id))
            })
            .or_else(|| focus.iter().position(|(_, enabled)| *enabled));
        if overflow
            && self.items[shown..]
                .iter()
                .any(|item| handles[&item.id].is_focused(window))
        {
            let overflow_focus = overflow_focus.clone();
            window.on_next_frame(move |window, cx| overflow_focus.focus(window, cx));
        }
        let mut row = div()
            .id(self.id.clone())
            .role(Role::Group)
            .aria_label(self.label)
            .relative()
            .on_mouse_up(gpui::MouseButton::Left, {
                let press = press.clone();
                move |_, window, _| {
                    if press.take().is_some() {
                        window.refresh();
                    }
                }
            })
            .on_mouse_up_out(gpui::MouseButton::Left, {
                let press = press.clone();
                move |_, window, _| {
                    if press.take().is_some() {
                        window.refresh();
                    }
                }
            })
            .flex()
            .w_full()
            .min_w_0()
            .gap(gap)
            .when(direction == Direction::Rtl, Styled::flex_row_reverse)
            .on_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let handles: Vec<_> = focus
                    .iter()
                    .map(|(handle, enabled)| (handle, *enabled))
                    .collect();
                if move_roving_focus(
                    &handles,
                    &event.keystroke.key,
                    Axis::Horizontal,
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
        let shown_widths = &widths[..shown];
        let targets = pressed_widths(shown_widths, press.get());
        let segment = |index| {
            connected.then_some(match (index == 0, index + 1 == count) {
                (true, true) => Segment::Only,
                (true, false) => Segment::Leading,
                (false, true) => Segment::Trailing,
                (false, false) => Segment::Middle,
            })
        };
        let mut hidden = Vec::new();
        for (index, item) in self.items.into_iter().enumerate() {
            let is_selected = selected.contains(&item.id);
            let action = if self.selection == ButtonGroupSelection::Actions {
                item.command
            } else {
                self.selection
                    .next(&selected, &item.id)
                    .zip(self.on_change.clone())
                    .map(|(next, callback)| {
                        Rc::new(move |window: &mut Window, cx: &mut App| {
                            callback(next.clone(), window, cx)
                        }) as Command
                    })
            };
            if index < shown {
                let width = px(animate(
                    (
                        ElementId::from((self.id.clone(), item.id.clone())),
                        "group-width",
                    ),
                    f32::from(targets[index]),
                    spec,
                    window,
                    cx,
                )
                .max(1.));
                let button = ButtonPart::new((self.id.clone(), item.id.clone()), item.label, width)
                    .size(self.size)
                    .variant(self.appearance)
                    .segment(segment(index))
                    .disabled(item.disabled)
                    .track_focus(&handles[&item.id])
                    .tab_stop(tab_stop == Some(index))
                    .selected(
                        (self.selection != ButtonGroupSelection::Actions).then_some(is_selected),
                    )
                    .when_some(action, |button, action| {
                        button.on_activate(move |window, cx| action(window, cx))
                    });
                let press = press.clone();
                row = row.child(
                    div()
                        .id((ElementId::from("group-item"), item.id))
                        .flex_none()
                        .on_mouse_down(gpui::MouseButton::Left, move |_, window, _| {
                            if !item.disabled {
                                press.set(Some(index));
                                window.refresh();
                            }
                        })
                        .child(button),
                );
            } else {
                let mut entry = MenuItem::new(item.id, item.label).disabled(item.disabled);
                if self.selection != ButtonGroupSelection::Actions {
                    entry = entry.checked(is_selected);
                }
                if let Some(action) = action {
                    entry = entry.on_activate(move |window, cx| action(window, cx));
                }
                hidden.push(entry);
            }
        }
        if overflow {
            let trigger_id = ElementId::from((self.id.clone(), "overflow"));
            let label = self.overflow_label;
            let size = self.size;
            let appearance = self.appearance;
            let shape = segment(shown);
            row = row.child(Menu::composed(
                (self.id.clone(), "overflow-menu"),
                hidden,
                move |_, toggle| {
                    let return_focus = overflow_focus.clone();
                    ButtonPart::new(trigger_id, label, overflow_width)
                        .size(size)
                        .variant(appearance)
                        .segment(shape)
                        .track_focus(&overflow_focus)
                        .tab_stop(tab_stop == Some(shown))
                        .icon(Icon::new(IconName::ArrowDropDown))
                        .on_accessible_activate(move |window, cx| {
                            return_focus.focus(window, cx);
                            toggle(window, cx);
                        })
                        .into_any_element()
                },
            ));
        }
        let view = window.current_view();
        row.child(
            gpui::canvas(
                move |bounds, window, _| {
                    if width.replace(bounds.size.width) != bounds.size.width {
                        window.on_next_frame(move |_, cx| cx.notify(view));
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        )
    }
}

fn visible_count(widths: &[Pixels], gap: Pixels, overflow: Pixels, available: Pixels) -> usize {
    let required =
        widths.iter().copied().sum::<Pixels>() + gap * widths.len().saturating_sub(1) as f32;
    if required <= available {
        return widths.len();
    }
    let mut remaining = available - overflow;
    widths
        .iter()
        .take_while(|&&width| {
            remaining -= width + gap;
            remaining >= px(0.)
        })
        .count()
}

fn pressed_widths(widths: &[Pixels], active: Option<usize>) -> Vec<Pixels> {
    let mut resolved = widths.to_vec();
    let Some(active) = active.filter(|&index| index < widths.len()) else {
        return resolved;
    };
    let left = active.checked_sub(1);
    let right = (active + 1 < widths.len()).then_some(active + 1);
    let neighbors = left.into_iter().chain(right);
    let count = neighbors.clone().count().max(1) as f32;
    let requested = widths[active] * (0.15 / count);
    for index in neighbors {
        let contribution = requested.min((widths[index] - px(24.)).max(px(0.)));
        resolved[index] -= contribution;
        resolved[active] += contribution;
    }
    resolved
}

/// A primary action and a separate menu trigger with distinct accessible
/// names, keyboard stops and disabled states.
#[derive(IntoElement)]
pub struct SplitButton {
    id: ElementId,
    label: SharedString,
    menu_label: SharedString,
    menu: Vec<MenuItem>,
    size: ControlSize,
    variant: ButtonVariant,
    disabled: bool,
    menu_disabled: bool,
    on_activate: Option<Command>,
    focus: Option<(FocusHandle, FocusHandle)>,
}

impl SplitButton {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        menu_label: impl Into<SharedString>,
        menu: impl IntoIterator<Item = MenuItem>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            menu_label: menu_label.into(),
            menu: menu.into_iter().collect(),
            size: ControlSize::default(),
            variant: ButtonVariant::default(),
            disabled: false,
            menu_disabled: false,
            on_activate: None,
            focus: None,
        }
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn menu_disabled(mut self, disabled: bool) -> Self {
        self.menu_disabled = disabled;
        self
    }

    pub fn on_activate(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(callback));
        self
    }

    /// Supplies independent focus handles for the command and menu trigger.
    pub fn track_focus(mut self, primary: &FocusHandle, menu: &FocusHandle) -> Self {
        self.focus = Some((primary.clone(), menu.clone()));
        self
    }
}

impl RenderOnce for SplitButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let direction = window_material_theme(window, cx).layout.direction;
        let primary_width = label_width(self.size, self.label.clone(), window, cx);
        let menu_width = px(split_trailing_size(self.size).1);
        let (primary_focus, menu_focus) = self.focus.unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "split-focus"), cx, |_, cx| {
                    (cx.focus_handle(), cx.focus_handle())
                })
                .read(cx)
                .clone()
        });
        let primary = ButtonPart::new((self.id.clone(), "primary"), self.label, primary_width)
            .size(self.size)
            .variant(self.variant)
            .segment(Some(Segment::SplitLeading))
            .track_focus(&primary_focus)
            .disabled(self.disabled)
            .when_some(self.on_activate, |button, action| {
                button.on_activate(move |window, cx| action(window, cx))
            });
        let restore = menu_focus.clone();
        let menu_disabled = self.menu_disabled || self.menu.is_empty();
        let secondary_id = ElementId::from((self.id.clone(), "secondary"));
        let trigger = move |open| {
            ButtonPart::new(secondary_id, self.menu_label, menu_width)
                .size(self.size)
                .variant(self.variant)
                .segment(Some(Segment::SplitTrailing))
                .track_focus(&menu_focus)
                .disabled(menu_disabled)
                .icon(
                    Icon::new(IconName::ArrowDropDown).rotate(gpui::Radians(if open {
                        std::f32::consts::PI
                    } else {
                        0.
                    })),
                )
        };
        let menu = if menu_disabled {
            trigger(false).into_any_element()
        } else {
            Menu::composed((self.id.clone(), "menu"), self.menu, move |open, toggle| {
                trigger(open)
                    .on_accessible_activate(move |window, cx| {
                        restore.focus(window, cx);
                        toggle(window, cx);
                    })
                    .into_any_element()
            })
            .into_any_element()
        };
        div()
            .id(self.id)
            .role(Role::Group)
            .flex()
            .min_w_0()
            .max_w_full()
            .gap(px(2.))
            .when(direction == Direction::Rtl, Styled::flex_row_reverse)
            .child(primary)
            .child(menu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_selection_preserves_last_choice_and_single_mode_replaces_it() {
        let a = SharedString::from("a");
        let b = SharedString::from("b");
        for mode in [
            ButtonGroupSelection::SingleRequired,
            ButtonGroupSelection::MultipleRequired,
        ] {
            assert_eq!(mode.next(std::slice::from_ref(&a), &a), None);
        }
        assert_eq!(
            ButtonGroupSelection::SingleRequired.next(std::slice::from_ref(&a), &b),
            Some(vec![b.clone()])
        );
        assert_eq!(
            ButtonGroupSelection::MultipleRequired.next(std::slice::from_ref(&a), &b),
            Some(vec![a, b])
        );
    }

    #[test]
    fn overflow_reserves_the_trigger_and_press_preserves_total_width() {
        let widths = [px(60.), px(90.), px(80.)];
        assert_eq!(visible_count(&widths, px(2.), px(48.), px(234.)), 3);
        assert_eq!(visible_count(&widths, px(2.), px(48.), px(201.)), 1);
        assert_eq!(visible_count(&widths, px(2.), px(48.), px(30.)), 0);
        let adjusted = pressed_widths(&widths, Some(1));
        assert_eq!(
            adjusted.iter().copied().sum::<Pixels>(),
            widths.iter().copied().sum::<Pixels>()
        );
        assert!(adjusted[1] > widths[1]);
        assert!(adjusted[0] < widths[0] && adjusted[2] < widths[2]);
    }
}

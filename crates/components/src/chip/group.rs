use std::rc::Rc;

use gpui::{
    AnyElement, App, Axis, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, Role, ScrollHandle, SharedString, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{current_material_theme, move_roving_focus};
use gpui_material_theme::Direction;

use crate::SemanticTone;
use crate::action::Activate;
use crate::control_style::finite_length;

use super::{Chip, ChipVariant};

type SelectionChange = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;

/// Controlled selection policy for [`ChipGroup`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ChipGroupSelection {
    /// Zero or one chip may be selected; activating the active chip clears selection.
    Single,
    /// Exactly one chip stays selected once selected; activating the sole selected chip is a no-op.
    SingleRequired,
    /// Zero or more chips may be selected independently.
    #[default]
    Multiple,
}

impl ChipGroupSelection {
    pub(crate) fn next(
        self,
        selection: &[SharedString],
        id: &SharedString,
    ) -> Option<Vec<SharedString>> {
        let contains = selection.iter().any(|value| value == id);
        match self {
            Self::SingleRequired if contains && selection.len() == 1 => None,
            Self::Single if contains => Some(Vec::new()),
            Self::Single | Self::SingleRequired => Some(vec![id.clone()]),
            Self::Multiple if contains => Some(
                selection
                    .iter()
                    .filter(|value| *value != id)
                    .cloned()
                    .collect(),
            ),
            Self::Multiple => {
                let mut next = selection.to_vec();
                next.push(id.clone());
                Some(next)
            }
        }
    }
}

/// One chip entry in a [`ChipGroup`] with a stable caller-owned identifier.
pub struct ChipGroupItem {
    id: SharedString,
    label: SharedString,
    leading: Option<AnyElement>,
    avatar: Option<AnyElement>,
    trailing: Option<AnyElement>,
    disabled: bool,
    tonal: Option<bool>,
    elevated: Option<bool>,
    tone: SemanticTone,
    remove: Option<(SharedString, Activate)>,
}

impl ChipGroupItem {
    /// Creates a chip entry with a stable group-unique identifier and visible label.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            leading: None,
            avatar: None,
            trailing: None,
            disabled: false,
            tonal: None,
            elevated: None,
            tone: SemanticTone::Default,
            remove: None,
        }
    }

    /// Sets an optional decorative leading icon on this chip.
    pub fn leading_icon(mut self, icon: impl IntoElement) -> Self {
        self.leading = Some(icon.into_any_element());
        self
    }

    /// Sets an optional circular avatar (for Input chips) taking precedence over the leading icon.
    pub fn avatar(mut self, avatar: impl IntoElement) -> Self {
        self.avatar = Some(avatar.into_any_element());
        self
    }

    /// Sets optional trailing visual content on this chip.
    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    /// Disables this individual chip item.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Overrides the group's tonal setting for this individual chip.
    pub fn tonal(mut self, tonal: bool) -> Self {
        self.tonal = Some(tonal);
        self
    }

    /// Overrides the group's elevated setting for this individual chip.
    pub fn elevated(mut self, elevated: bool) -> Self {
        self.elevated = Some(elevated);
        self
    }

    /// Applies a Material 3 semantic tone to this individual chip.
    pub fn tone(mut self, tone: SemanticTone) -> Self {
        self.tone = tone;
        self
    }

    /// Adds an independent removal action to a Filter or Input chip in the group.
    pub fn on_remove(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.remove = Some((label.into(), Rc::new(handler)));
        self
    }
}

/// A controlled collection of Material chips with M3 8 dp spacing, roving
/// keyboard focus, RTL mirroring, and `Role::Group` semantics.
#[derive(IntoElement)]
pub struct ChipGroup {
    id: ElementId,
    name: SharedString,
    items: Vec<ChipGroupItem>,
    selection_mode: ChipGroupSelection,
    selected: Vec<SharedString>,
    on_change: Option<SelectionChange>,
    variant: ChipVariant,
    tonal: bool,
    elevated: bool,
    disabled: bool,
    scrollable: bool,
    scroll_handle: Option<ScrollHandle>,
    spacing: Pixels,
    row_spacing: Pixels,
}

impl ChipGroup {
    /// Creates a chip group with an element ID, accessible listbox label, and member items.
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        items: impl IntoIterator<Item = ChipGroupItem>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            items: items.into_iter().collect(),
            selection_mode: ChipGroupSelection::default(),
            selected: Vec::new(),
            on_change: None,
            variant: ChipVariant::Filter,
            tonal: false,
            elevated: false,
            disabled: false,
            scrollable: false,
            scroll_handle: None,
            spacing: px(8.),
            row_spacing: px(8.),
        }
    }

    /// Sets the controlled selection mode and active chip IDs.
    pub fn selection(
        mut self,
        mode: ChipGroupSelection,
        selected: impl IntoIterator<Item = SharedString>,
    ) -> Self {
        self.selection_mode = mode;
        self.selected = selected.into_iter().collect();
        self
    }

    /// Receives the requested next selection list whenever an enabled chip is activated.
    pub fn on_change(
        mut self,
        handler: impl Fn(Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Shorthand for [`ChipGroupSelection::SingleRequired`] with a single active ID.
    pub fn single_select(
        mut self,
        selected: impl Into<SharedString>,
        on_select: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.selection_mode = ChipGroupSelection::SingleRequired;
        self.selected = vec![selected.into()];
        self.on_change = Some(Rc::new(move |next, window, cx| {
            if let Some(id) = next.into_iter().next() {
                on_select(id, window, cx);
            }
        }));
        self
    }

    /// Selects the chip variant rendered for members. Defaults to [`ChipVariant::Filter`].
    pub fn variant(mut self, variant: ChipVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the default expressive Tonal appearance for members unless overridden per item.
    pub fn tonal(mut self, tonal: bool) -> Self {
        self.tonal = tonal;
        self
    }

    /// Sets the default elevated appearance for members unless overridden per item.
    pub fn elevated(mut self, elevated: bool) -> Self {
        self.elevated = elevated;
        self
    }

    /// Disables all members in the group when `true` (OR-ed with per-item disabled state).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Switches between wrapped rows (`false`, default) and a single horizontal
    /// scrolling row (`true`).
    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }

    /// Attaches a caller-owned horizontal [`ScrollHandle`] when `.scrollable(true)` is enabled.
    pub fn scroll_handle(mut self, handle: &ScrollHandle) -> Self {
        self.scroll_handle = Some(handle.clone());
        self
    }

    /// Sets horizontal spacing between chips in logical pixels. Defaults to `8 dp`.
    pub fn spacing(mut self, spacing: Pixels) -> Self {
        self.spacing = finite_length(spacing);
        self
    }

    /// Sets vertical spacing between wrapped rows in logical pixels. Defaults to `8 dp`.
    pub fn row_spacing(mut self, row_spacing: Pixels) -> Self {
        self.row_spacing = finite_length(row_spacing);
        self
    }
}

impl RenderOnce for ChipGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let direction = current_material_theme(window, cx).layout.direction;
        let focus_handles: Vec<FocusHandle> = self
            .items
            .iter()
            .map(|item| {
                let scope_id = ElementId::from((self.id.clone(), "chip-focus"));
                window.with_id(scope_id, |window| {
                    window
                        .use_keyed_state(item.id.clone(), cx, |_, cx| cx.focus_handle())
                        .read(cx)
                        .clone()
                })
            })
            .collect();

        let group_disabled = self.disabled;
        let group_tonal = self.tonal;
        let group_elevated = self.elevated;

        let focused_index = self
            .items
            .iter()
            .zip(&focus_handles)
            .position(|(item, focus)| {
                !(group_disabled || item.disabled) && focus.is_focused(window)
            });
        let selected_index = self.items.iter().position(|item| {
            !(group_disabled || item.disabled) && self.selected.iter().any(|id| id == &item.id)
        });
        let first_enabled = self
            .items
            .iter()
            .position(|item| !(group_disabled || item.disabled));
        let tab_stop = focused_index.or(selected_index).or(first_enabled);

        let nav_entries: Vec<(FocusHandle, bool)> = self
            .items
            .iter()
            .zip(&focus_handles)
            .map(|(item, focus)| (focus.clone(), !(group_disabled || item.disabled)))
            .collect();

        let scroll_handle = self.scrollable.then(|| {
            self.scroll_handle.clone().unwrap_or_else(|| {
                window
                    .use_keyed_state((self.id.clone(), "chip-group-scroll"), cx, |_, _| {
                        ScrollHandle::new()
                    })
                    .read(cx)
                    .clone()
            })
        });

        let selection: Rc<[SharedString]> = Rc::from(self.selected.as_slice());
        let mode = self.selection_mode;
        let variant = self.variant;
        let group_id = self.id.clone();

        let chips: Vec<AnyElement> = self
            .items
            .into_iter()
            .enumerate()
            .map(|(ix, item)| {
                let focus = focus_handles[ix].clone();
                let is_selected = selection.iter().any(|id| id == &item.id);
                let item_id = item.id.clone();
                let chip_id = ElementId::from((group_id.clone(), item_id.clone()));

                let restore = item.remove.as_ref().and_then(|_| {
                    ((ix + 1)..nav_entries.len())
                        .find(|&next| nav_entries[next].1)
                        .or_else(|| (0..ix).rev().find(|&prev| nav_entries[prev].1))
                        .map(|target| nav_entries[target].0.clone())
                });

                let mut chip = Chip::new(chip_id, item.label)
                    .variant(variant)
                    .selected(is_selected)
                    .disabled(group_disabled || item.disabled)
                    .tonal(item.tonal.unwrap_or(group_tonal))
                    .elevated(item.elevated.unwrap_or(group_elevated))
                    .tone(item.tone)
                    .track_focus(&focus)
                    .tab_stop(Some(ix) == tab_stop);

                if let Some(leading) = item.leading {
                    chip = chip.leading_icon(leading);
                }
                if let Some(avatar) = item.avatar {
                    chip = chip.avatar(avatar);
                }
                if let Some(trailing) = item.trailing {
                    chip = chip.trailing(trailing);
                }
                if let Some((remove_label, remove_handler)) = item.remove {
                    chip = chip.on_remove(remove_label, restore, move |window, cx| {
                        remove_handler(window, cx);
                    });
                }
                if let Some(on_change) = self.on_change.clone() {
                    let selection = selection.clone();
                    let target_id = item_id.clone();
                    if matches!(variant, ChipVariant::Filter | ChipVariant::Input) {
                        chip = chip.on_change(move |_, window, cx| {
                            if let Some(next) = mode.next(&selection, &target_id) {
                                on_change(next, window, cx);
                            }
                        });
                    } else {
                        chip = chip.on_activate(move |window, cx| {
                            if let Some(next) = mode.next(&selection, &target_id) {
                                on_change(next, window, cx);
                            }
                        });
                    }
                }
                chip.into_any_element()
            })
            .collect();

        let mut container = div()
            .id(self.id)
            .test_support()
            .role(Role::Group)
            .aria_label(self.name)
            .flex()
            .flex_row()
            .items_center()
            .when(direction == Direction::Rtl, Styled::flex_row_reverse)
            .on_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let key = event.keystroke.key.as_str();
                match key {
                    "home" => {
                        if let Some((focus, _)) = nav_entries.iter().find(|(_, enabled)| *enabled) {
                            focus.focus(window, cx);
                            cx.stop_propagation();
                        }
                    }
                    "end" => {
                        if let Some((focus, _)) =
                            nav_entries.iter().rev().find(|(_, enabled)| *enabled)
                        {
                            focus.focus(window, cx);
                            cx.stop_propagation();
                        }
                    }
                    "left" | "right" => {
                        let refs: Vec<(&FocusHandle, bool)> = nav_entries
                            .iter()
                            .map(|(focus, enabled)| (focus, *enabled))
                            .collect();
                        if move_roving_focus(&refs, key, Axis::Horizontal, direction, window, cx)
                            .is_some()
                        {
                            cx.stop_propagation();
                        }
                    }
                    _ => {}
                }
            });

        if let Some(handle) = scroll_handle {
            container = container
                .w_full()
                .overflow_x_scroll()
                .track_scroll(&handle)
                .gap(self.spacing)
                .children(chips);
        } else {
            container = container
                .flex_wrap()
                .gap_x(self.spacing)
                .gap_y(self.row_spacing)
                .children(chips);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_group_selection_modes_compute_expected_transitions() {
        let a: SharedString = "a".into();
        let b: SharedString = "b".into();

        // Single: toggle on, switch, and clear.
        assert_eq!(
            ChipGroupSelection::Single.next(&[], &a),
            Some(vec![a.clone()])
        );
        assert_eq!(
            ChipGroupSelection::Single.next(std::slice::from_ref(&a), &b),
            Some(vec![b.clone()])
        );
        assert_eq!(
            ChipGroupSelection::Single.next(std::slice::from_ref(&a), &a),
            Some(vec![])
        );

        // SingleRequired: switch allowed, clearing sole active item is ignored.
        assert_eq!(
            ChipGroupSelection::SingleRequired.next(&[], &a),
            Some(vec![a.clone()])
        );
        assert_eq!(
            ChipGroupSelection::SingleRequired.next(std::slice::from_ref(&a), &a),
            None
        );
        assert_eq!(
            ChipGroupSelection::SingleRequired.next(std::slice::from_ref(&a), &b),
            Some(vec![b.clone()])
        );

        // Multiple: independent add and remove.
        assert_eq!(
            ChipGroupSelection::Multiple.next(&[], &a),
            Some(vec![a.clone()])
        );
        assert_eq!(
            ChipGroupSelection::Multiple.next(std::slice::from_ref(&a), &b),
            Some(vec![a.clone(), b.clone()])
        );
        assert_eq!(
            ChipGroupSelection::Multiple.next(&[a.clone(), b.clone()], &a),
            Some(vec![b])
        );
    }
}

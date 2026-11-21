use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use gpui::{
    App, AppContext, Axis, ElementId, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_material_components::{
    ButtonPadding, ButtonShapes, ControlSize, Icon, IconName, Menu, MenuItem,
    button_composition::{
        ButtonPart, Segment, group_icon_width, group_label_width, split_trailing_size,
    },
};
use gpui_material_foundation::{
    InteractionSource, animate, current_material_theme, font_cache_revision, move_roving_focus,
};
use gpui_material_theme::{Direction, MotionKind, MotionSpeed, Typography};

use super::{
    ButtonGroup, ButtonGroupAlignment, ButtonGroupItem, ButtonGroupSelection, ButtonGroupVariant,
    ButtonGroupVerticalAlignment, Command, ItemBehavior, ItemContent, SelectionChange,
    ToggleChange,
    layout::{nonnegative, pressed_widths, visible_count, weighted_widths},
};

struct LabelWidthEntry {
    label: SharedString,
    padding: Option<ButtonPadding>,
    leading_icon: bool,
    width: Pixels,
}

struct LabelWidthCache {
    size: ControlSize,
    rem_size: Pixels,
    font_revision: u64,
    typography: Typography,
    entries: Vec<LabelWidthEntry>,
}

struct GroupState {
    width: Rc<Cell<Pixels>>,
    focus: HashMap<SharedString, FocusHandle>,
    scope: FocusHandle,
    overflow_focus: FocusHandle,
    overflow_open: Rc<Cell<bool>>,
    shown: Option<usize>,
    source: Entity<InteractionSource>,
    label_widths: RefCell<Option<LabelWidthCache>>,
}

struct SyncedGroupState {
    handles: Vec<FocusHandle>,
    scope: FocusHandle,
    overflow_focus: FocusHandle,
    menu_open: Rc<Cell<bool>>,
    previous_visible: Option<usize>,
    source: Entity<InteractionSource>,
}

impl ButtonGroupItem {
    fn preferred_width(
        &self,
        index: usize,
        size: ControlSize,
        minimum: Pixels,
        cache: &mut LabelWidthCache,
        window: &Window,
        cx: &App,
    ) -> Pixels {
        if self.weight > 0. {
            return minimum;
        }
        let width = match self.content {
            ItemContent::Label | ItemContent::LeadingIcon(_) => {
                let leading_icon = matches!(self.content, ItemContent::LeadingIcon(_));
                let cached = cache
                    .entries
                    .get(index)
                    .filter(|entry| {
                        entry.label == self.label
                            && entry.padding == self.style.padding
                            && entry.leading_icon == leading_icon
                    })
                    .or_else(|| {
                        cache.entries.iter().find(|entry| {
                            entry.label == self.label
                                && entry.padding == self.style.padding
                                && entry.leading_icon == leading_icon
                        })
                    })
                    .map(|entry| entry.width);
                if let Some(width) = cached {
                    width
                } else {
                    let width = group_label_width(
                        size,
                        self.label.clone(),
                        self.style.padding,
                        leading_icon,
                        window,
                        cx,
                    );
                    cache.entries.push(LabelWidthEntry {
                        label: self.label.clone(),
                        padding: self.style.padding,
                        leading_icon,
                        width,
                    });
                    width
                }
            }
            ItemContent::Icon(_) => group_icon_width(size, self.icon_width, self.style.padding),
            ItemContent::Custom(width, _) => nonnegative(width).unwrap_or(minimum),
        };
        width.max(minimum)
    }

    fn selected(&self, mode: ButtonGroupSelection, selected: &[SharedString]) -> Option<bool> {
        match self.behavior {
            ItemBehavior::Grouped(_) if mode != ButtonGroupSelection::Actions => {
                Some(selected.contains(&self.id))
            }
            ItemBehavior::Toggle(value, _) => Some(value),
            _ => None,
        }
    }

    fn activation(
        &self,
        mode: ButtonGroupSelection,
        selected: &[SharedString],
        on_change: Option<SelectionChange>,
    ) -> Option<Command> {
        match &self.behavior {
            ItemBehavior::Grouped(command) if mode == ButtonGroupSelection::Actions => {
                command.clone()
            }
            ItemBehavior::Grouped(_) => {
                mode.next(selected, &self.id)
                    .zip(on_change)
                    .map(|(next, callback)| {
                        Rc::new(move |window: &mut Window, cx: &mut App| {
                            callback(next.clone(), window, cx)
                        }) as Command
                    })
            }
            ItemBehavior::Action(command) => Some(command.clone()),
            ItemBehavior::Toggle(value, callback) => {
                let (next, callback) = (!value, callback.clone());
                Some(Rc::new(move |window, cx| callback(next, window, cx)))
            }
        }
    }

    fn render_member(&self, button: ButtonPart, shapes: ButtonShapes) -> ButtonPart {
        let button = match &self.content {
            ItemContent::Label => button,
            ItemContent::LeadingIcon(icon) => button.leading_icon(icon()),
            ItemContent::Icon(icon) => button.icon_width(self.icon_width).icon(icon()),
            ItemContent::Custom(_, content) => button.content(content()),
        };
        self.style.apply(button, shapes)
    }

    fn render_overflow(self, selected: Option<bool>, command: Option<Command>) -> MenuItem {
        let mut entry = MenuItem::new(self.id, self.label.clone()).disabled(self.disabled);
        if let Some(checked) = selected {
            entry = entry.checked(checked);
        }
        if let ItemContent::LeadingIcon(icon) | ItemContent::Icon(icon) = self.content {
            entry = entry.leading(move || icon());
        }
        if let Some(content) = self.overflow_content {
            entry = entry.content(move || content());
        }
        if let Some(command) = command {
            entry = entry.on_activate(move |window, cx| command(window, cx));
        }
        entry
    }
}

fn segment_at(connected: bool, index: usize, count: usize) -> Option<Segment> {
    connected.then_some(match (index == 0, index + 1 == count) {
        (true, true) => Segment::Only,
        (true, false) => Segment::Leading,
        (false, true) => Segment::Trailing,
        (false, false) => Segment::Middle,
    })
}

fn sync_group_state(
    state: &Entity<GroupState>,
    items: &[ButtonGroupItem],
    shown: usize,
    cx: &mut App,
) -> SyncedGroupState {
    let needs_update = {
        let state = state.read(cx);
        let keys_match = items.len() == state.focus.len()
            && items.iter().all(|item| state.focus.contains_key(&item.id));
        !keys_match || state.shown != Some(shown)
    };
    if !needs_update {
        let state = state.read(cx);
        let handles = items
            .iter()
            .map(|item| state.focus[&item.id].clone())
            .collect();
        return SyncedGroupState {
            handles,
            scope: state.scope.clone(),
            overflow_focus: state.overflow_focus.clone(),
            menu_open: state.overflow_open.clone(),
            previous_visible: state.shown,
            source: state.source.clone(),
        };
    }
    state.update(cx, |state, cx| {
        let keys_match = items.len() == state.focus.len()
            && items.iter().all(|item| state.focus.contains_key(&item.id));
        if !keys_match {
            let mut prior = std::mem::take(&mut state.focus);
            state.focus = items
                .iter()
                .map(|item| {
                    let handle = prior.remove(&item.id).unwrap_or_else(|| cx.focus_handle());
                    (item.id.clone(), handle)
                })
                .collect();
        }
        let previous_visible = state.shown.replace(shown);
        let handles = items
            .iter()
            .map(|item| state.focus[&item.id].clone())
            .collect();
        SyncedGroupState {
            handles,
            scope: state.scope.clone(),
            overflow_focus: state.overflow_focus.clone(),
            menu_open: state.overflow_open.clone(),
            previous_visible,
            source: state.source.clone(),
        }
    })
}

fn resolve_selected(
    items: &[ButtonGroupItem],
    selection: ButtonGroupSelection,
    selected: &[SharedString],
) -> Vec<SharedString> {
    items
        .iter()
        .filter(|item| {
            matches!(item.behavior, ItemBehavior::Grouped(_)) && selected.contains(&item.id)
        })
        .map(|item| item.id.clone())
        .take(
            if matches!(
                selection,
                ButtonGroupSelection::Single | ButtonGroupSelection::SingleRequired
            ) {
                1
            } else {
                usize::MAX
            },
        )
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn reconcile_overflow_focus(
    shown: usize,
    overflow: bool,
    synced: &SyncedGroupState,
    focus: &[(FocusHandle, bool)],
    on_overflow_open_change: &Option<ToggleChange>,
    window: &mut Window,
    cx: &mut App,
) {
    if overflow {
        if let Some(displaced) = synced.handles[shown..]
            .iter()
            .find(|handle| handle.is_focused(window))
        {
            move_focus_after_layout(displaced.clone(), synced.overflow_focus.clone(), window);
        }
    } else if let Some(previous) = synced.previous_visible {
        let was_open = synced.menu_open.replace(false);
        if (synced.overflow_focus.is_focused(window)
            || was_open && synced.scope.contains_focused(window, cx))
            && let Some(replacement) = focus
                .iter()
                .skip(previous)
                .chain(focus)
                .find(|(_, enabled)| *enabled)
            && let Some(displaced) = window.focused(cx)
        {
            move_focus_after_layout(displaced, replacement.0.clone(), window);
        }
        if was_open && let Some(on_change) = on_overflow_open_change.clone() {
            window.defer(cx, move |window, cx| on_change(false, window, cx));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn build_group_row(
    id: ElementId,
    label: SharedString,
    scope: &FocusHandle,
    gap: Pixels,
    direction: Direction,
    alignment: ButtonGroupAlignment,
    vertical_alignment: ButtonGroupVerticalAlignment,
    focus: Vec<(FocusHandle, bool)>,
) -> gpui::Stateful<gpui::Div> {
    let mut row = div()
        .id(id)
        .role(Role::Group)
        .aria_label(label)
        .track_focus(scope)
        .relative()
        .flex()
        .w_full()
        .min_w_0()
        .gap(gap)
        .when(direction == Direction::Rtl, Styled::flex_row_reverse)
        .on_key_down(move |event, window, cx| {
            if event.keystroke.modifiers.modified() {
                return;
            }
            let handles = focus
                .iter()
                .map(|(handle, enabled)| (handle, *enabled))
                .collect::<Vec<_>>();
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
    row = match alignment {
        ButtonGroupAlignment::Start => row.justify_start(),
        ButtonGroupAlignment::Center => row.justify_center(),
        ButtonGroupAlignment::End => row.justify_end(),
        ButtonGroupAlignment::SpaceBetween => row.justify_between(),
        ButtonGroupAlignment::SpaceAround => row.justify_around(),
        ButtonGroupAlignment::SpaceEvenly => row.justify_evenly(),
    };
    match vertical_alignment {
        ButtonGroupVerticalAlignment::Top => row.items_start(),
        ButtonGroupVerticalAlignment::Center => row.items_center(),
        ButtonGroupVerticalAlignment::Bottom => row.items_end(),
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let minimum = px(theme.layout.density.interactive_size(1.));
        let spec = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Fast, cx.reduce_motion());
        let connected = self.variant == ButtonGroupVariant::Connected;
        let gap = self
            .spacing
            .unwrap_or(if connected { px(2.) } else { px(12.) });
        let width = window.viewport_size().width;
        let state =
            window.use_keyed_state((self.id.clone(), "group-state"), cx, |_, cx| GroupState {
                width: Rc::new(Cell::new(width)),
                focus: HashMap::new(),
                scope: cx.focus_handle(),
                overflow_focus: cx.focus_handle(),
                overflow_open: Rc::new(Cell::new(false)),
                shown: None,
                source: cx.new(|_| InteractionSource::new()),
                label_widths: RefCell::new(None),
            });
        let preferred = {
            let group_state = state.read(cx);
            let rem_size = window.rem_size();
            let font_revision = font_cache_revision(cx);
            let mut slot = group_state.label_widths.borrow_mut();
            let cache_valid = slot.as_ref().is_some_and(|cached| {
                cached.size == self.size
                    && cached.rem_size == rem_size
                    && cached.font_revision == font_revision
                    && cached.typography == theme.typography
            });
            if !cache_valid {
                *slot = Some(LabelWidthCache {
                    size: self.size,
                    rem_size,
                    font_revision,
                    typography: theme.typography.clone(),
                    entries: Vec::with_capacity(self.items.len()),
                });
            }
            let cache = slot.as_mut().expect("label width cache initialized");
            if cache.entries.len() > self.items.len().saturating_mul(2).max(16) {
                cache.entries.clear();
            }
            self.items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    item.preferred_width(index, self.size, minimum, cache, window, cx)
                })
                .collect::<Vec<_>>()
        };
        let preferred_trigger = self
            .overflow_trigger
            .as_ref()
            .and_then(|(width, _)| nonnegative(*width))
            .filter(|width| *width > px(0.))
            .unwrap_or_else(|| px(split_trailing_size(self.size).1));
        let width = state.read(cx).width.clone();
        let available = width.get();
        let overflow_width = preferred_trigger.min(available.max(px(1.)));
        let shown = visible_count(&preferred, gap, overflow_width, available);
        let overflow = shown < self.items.len();
        let count = shown + usize::from(overflow);
        let synced = sync_group_state(&state, &self.items, shown, cx);
        let selected = resolve_selected(&self.items, self.selection, &self.selected);
        let focus: Vec<_> = self.items[..shown]
            .iter()
            .zip(&synced.handles[..shown])
            .map(|(item, handle)| (handle.clone(), !item.disabled))
            .chain(overflow.then(|| (synced.overflow_focus.clone(), true)))
            .collect();
        let tab_stop = focus
            .iter()
            .position(|(handle, enabled)| *enabled && handle.is_focused(window))
            .or_else(|| {
                self.items[..shown].iter().position(|item| {
                    !item.disabled && item.selected(self.selection, &selected) == Some(true)
                })
            })
            .or_else(|| focus.iter().position(|(_, enabled)| *enabled));
        reconcile_overflow_focus(
            shown,
            overflow,
            &synced,
            &focus,
            &self.on_overflow_open_change,
            window,
            cx,
        );

        let mut row = build_group_row(
            self.id.clone(),
            self.label,
            &synced.scope,
            gap,
            direction,
            self.alignment,
            self.vertical_alignment,
            focus,
        );

        let window_id = window.window_handle().window_id();
        let active = self.items[..shown].iter().position(|item| {
            !item.disabled
                && item
                    .source
                    .as_ref()
                    .unwrap_or(&synced.source)
                    .read(cx)
                    .state(
                        window_id,
                        &ElementId::from((self.id.clone(), item.id.clone())),
                    )
                    .is_some_and(|state| state.pressed)
        });
        let weights = self.items[..shown]
            .iter()
            .map(|item| item.weight)
            .collect::<Vec<_>>();
        let limits = self.items[..shown]
            .iter()
            .map(|item| item.compression_limit)
            .collect::<Vec<_>>();
        let occupied =
            (if overflow { overflow_width } else { px(0.) }) + gap * count.saturating_sub(1) as f32;
        let widths = weighted_widths(&preferred[..shown], &weights, available - occupied);
        let targets = pressed_widths(&widths, &limits, active, self.expanded_ratio);
        let mut hidden = Vec::new();
        for (index, item) in self.items.into_iter().enumerate() {
            let selected_state = item.selected(self.selection, &selected);
            let action = item.activation(self.selection, &selected, self.on_change.clone());
            if index >= shown {
                hidden.push(item.render_overflow(selected_state, action));
                continue;
            }
            let id = ElementId::from((self.id.clone(), item.id.clone()));
            let width = px(animate(
                (id.clone(), "group-width"),
                f32::from(targets[index]),
                spec,
                window,
                cx,
            )
            .max(1.));
            let shape = segment_at(connected, index, count);
            let button = ButtonPart::new(id, item.label.clone(), width)
                .size(self.size)
                .shape(self.shape)
                .variant(item.appearance.unwrap_or(self.appearance))
                .segment(shape)
                .disabled(item.disabled)
                .track_focus(&synced.handles[index])
                .tab_stop(tab_stop == Some(index))
                .selected(selected_state)
                .interaction_source(item.source.as_ref().unwrap_or(&synced.source))
                .when_some(action, |button, action| {
                    button.on_activate(move |window, cx| action(window, cx))
                });
            let part = div()
                .id((ElementId::from("group-item"), item.id.clone()))
                .flex_none()
                .child(item.render_member(button, self.shapes.for_segment(shape)));
            row = row.child(match item.vertical_alignment {
                Some(ButtonGroupVerticalAlignment::Top) => part.self_start(),
                Some(ButtonGroupVerticalAlignment::Center) => part.self_center(),
                Some(ButtonGroupVerticalAlignment::Bottom) => part.self_end(),
                None => part,
            });
        }
        if overflow {
            let trigger_id = ElementId::from((self.id.clone(), "overflow"));
            let shape = segment_at(connected, shown, count);
            let shapes = self.shapes.for_segment(shape);
            let content = self.overflow_trigger.map(|(_, content)| content);
            let menu_open = synced.menu_open;
            let overflow_focus = synced.overflow_focus;
            let menu = Menu::composed(
                (self.id.clone(), "overflow-menu"),
                hidden,
                move |open, toggle| {
                    menu_open.set(open);
                    let return_focus = overflow_focus.clone();
                    let button = ButtonPart::new(trigger_id, self.overflow_label, overflow_width)
                        .size(self.size)
                        .shape(self.shape)
                        .variant(self.appearance)
                        .segment(shape)
                        .shapes(shapes)
                        .track_focus(&overflow_focus)
                        .tab_stop(tab_stop == Some(shown))
                        .on_accessible_activate(move |window, cx| {
                            return_focus.focus(window, cx);
                            toggle(window, cx);
                        });
                    match content {
                        Some(content) => button.content(content(open)),
                        None => button.icon(Icon::new(IconName::ArrowDropDown)),
                    }
                    .into_any_element()
                },
            )
            .when_some(self.overflow_open, Menu::open)
            .when_some(self.on_overflow_open_change, |menu, change| {
                menu.on_open_change(move |open, window, cx| change(open, window, cx))
            })
            .when_some(self.overflow_content, |menu, content| {
                menu.content(move || content())
            });
            row = row.child(menu);
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

fn move_focus_after_layout(from: FocusHandle, to: FocusHandle, window: &Window) {
    window.on_next_frame(move |window, cx| {
        if window.focused(cx).is_none() || from.is_focused(window) {
            to.focus(window, cx);
        }
    });
}

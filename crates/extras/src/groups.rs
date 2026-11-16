use std::{cell::Cell, collections::HashMap, rc::Rc};

use gpui::{
    AnyElement, App, AppContext, Axis, ElementId, Entity, FocusHandle, InteractiveElement,
    IntoElement, ParentElement, Pixels, RenderOnce, Role, SharedString, StatefulInteractiveElement,
    Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_material_components::{
    ButtonBorder, ButtonColors, ButtonElevations, ButtonPadding, ButtonShape, ButtonShapes,
    ButtonVariant, ControlSize, Icon, IconButtonWidth, IconName, Menu, MenuItem,
    button_composition::{
        ButtonPart, Segment, group_icon_width, group_label_width, split_trailing_size,
    },
};
use gpui_material_foundation::{
    InteractionSource, animate, move_roving_focus, window_material_theme,
};
use gpui_material_theme::{Direction, MotionKind, MotionSpeed};

type Command = Rc<dyn Fn(&mut Window, &mut App)>;
type SelectionChange = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;
type ToggleChange = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type Decoration = Rc<dyn Fn() -> AnyElement>;
type TriggerContent = Box<dyn FnOnce(bool) -> AnyElement>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonGroupVariant {
    #[default]
    Standard,
    Connected,
}

/// Logical horizontal arrangement of visible members and the overflow trigger.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonGroupAlignment {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonGroupVerticalAlignment {
    #[default]
    Top,
    Center,
    Bottom,
}

/// Overrides connected shapes by a segment's logical position in the visible row.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ButtonGroupConnectedShapes {
    pub leading: ButtonShapes,
    pub middle: ButtonShapes,
    pub trailing: ButtonShapes,
    pub only: ButtonShapes,
}

impl ButtonGroupConnectedShapes {
    fn for_segment(self, segment: Option<Segment>) -> ButtonShapes {
        match segment {
            Some(Segment::Leading) => self.leading,
            Some(Segment::Middle) => self.middle,
            Some(Segment::Trailing) => self.trailing,
            Some(Segment::Only) => self.only,
            _ => ButtonShapes::default(),
        }
    }
}

/// Per-member appearance overrides. Omitted fields keep group or Material defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ButtonGroupItemStyle {
    pub shapes: ButtonShapes,
    pub colors: ButtonColors,
    pub border: Option<ButtonBorder>,
    pub elevations: ButtonElevations,
    pub padding: Option<ButtonPadding>,
}

impl ButtonGroupItemStyle {
    fn apply(self, button: ButtonPart, shapes: ButtonShapes) -> ButtonPart {
        button
            .shapes(ButtonShapes {
                resting: self.shapes.resting.or(shapes.resting),
                pressed: self.shapes.pressed.or(shapes.pressed),
                checked: self.shapes.checked.or(shapes.checked),
            })
            .colors(self.colors)
            .elevations(self.elevations)
            .when_some(self.border, ButtonPart::border)
            .when_some(self.padding, ButtonPart::content_padding)
    }
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
    behavior: ItemBehavior,
    content: ItemContent,
    overflow_content: Option<Decoration>,
    appearance: Option<ButtonVariant>,
    icon_width: IconButtonWidth,
    style: ButtonGroupItemStyle,
    weight: f32,
    compression_limit: Option<Pixels>,
    vertical_alignment: Option<ButtonGroupVerticalAlignment>,
    source: Option<Entity<InteractionSource>>,
}

enum ItemBehavior {
    Grouped(Option<Command>),
    Action(Command),
    Toggle(bool, ToggleChange),
}

enum ItemContent {
    Label,
    LeadingIcon(Decoration),
    Icon(Decoration),
    Custom(Pixels, Decoration),
}

impl ButtonGroupItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            behavior: ItemBehavior::Grouped(None),
            content: ItemContent::Label,
            overflow_content: None,
            appearance: None,
            icon_width: IconButtonWidth::Uniform,
            style: ButtonGroupItemStyle::default(),
            weight: 0.,
            compression_limit: None,
            vertical_alignment: None,
            source: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Runs in action groups. Selection groups use the group's on_change callback.
    pub fn on_activate(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.behavior = ItemBehavior::Grouped(Some(Rc::new(callback)));
        self
    }

    /// Adds a command that remains independent of the group's selection mode.
    pub fn action(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.behavior = ItemBehavior::Action(Rc::new(callback));
        self
    }

    /// Adds a toggle whose checked value is controlled independently of the group.
    pub fn toggle(
        mut self,
        selected: bool,
        callback: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.behavior = ItemBehavior::Toggle(selected, Rc::new(callback));
        self
    }

    pub fn leading_icon<E: IntoElement>(mut self, icon: impl Fn() -> E + 'static) -> Self {
        self.content = ItemContent::LeadingIcon(Rc::new(move || icon().into_any_element()));
        self
    }

    pub fn icon_only<E: IntoElement>(mut self, icon: impl Fn() -> E + 'static) -> Self {
        self.content = ItemContent::Icon(Rc::new(move || icon().into_any_element()));
        self
    }

    pub fn icon_width(mut self, width: IconButtonWidth) -> Self {
        self.icon_width = width;
        self
    }

    /// Replaces row content with decorative artwork of the supplied preferred width.
    pub fn content<E: IntoElement>(
        mut self,
        width: Pixels,
        content: impl Fn() -> E + 'static,
    ) -> Self {
        self.content = ItemContent::Custom(width, Rc::new(move || content().into_any_element()));
        self
    }

    /// Replaces the menu row's visible label. The member owns activation and its accessible name.
    pub fn overflow_item_content<E: IntoElement>(
        mut self,
        content: impl Fn() -> E + 'static,
    ) -> Self {
        self.overflow_content = Some(Rc::new(move || content().into_any_element()));
        self
    }

    pub fn appearance(mut self, appearance: ButtonVariant) -> Self {
        self.appearance = Some(appearance);
        self
    }

    pub fn style(mut self, style: ButtonGroupItemStyle) -> Self {
        self.style = style;
        self
    }

    /// Shares available space with other weighted members. Zero weight keeps intrinsic sizing.
    pub fn weight(mut self, weight: f32) -> Self {
        self.weight = if weight.is_finite() {
            weight.max(0.)
        } else {
            0.
        };
        self
    }

    /// Limits how many logical pixels a neighbor can borrow from this member.
    pub fn compression_limit(mut self, limit: Pixels) -> Self {
        self.compression_limit = nonnegative(limit);
        self
    }

    pub fn vertical_alignment(mut self, alignment: ButtonGroupVerticalAlignment) -> Self {
        self.vertical_alignment = Some(alignment);
        self
    }

    /// Shares the member's interaction state, including press-driven width changes.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.source = Some(source.clone());
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
    shape: ButtonShape,
    shapes: ButtonGroupConnectedShapes,
    alignment: ButtonGroupAlignment,
    vertical_alignment: ButtonGroupVerticalAlignment,
    spacing: Option<Pixels>,
    expanded_ratio: f32,
    overflow_trigger: Option<(Pixels, TriggerContent)>,
    overflow_content: Option<Decoration>,
    overflow_open: Option<bool>,
    on_overflow_open_change: Option<ToggleChange>,
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
            shape: ButtonShape::Round,
            shapes: ButtonGroupConnectedShapes::default(),
            alignment: ButtonGroupAlignment::default(),
            vertical_alignment: ButtonGroupVerticalAlignment::default(),
            spacing: None,
            expanded_ratio: 0.15,
            overflow_trigger: None,
            overflow_content: None,
            overflow_open: None,
            on_overflow_open_change: None,
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

    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn connected_shapes(mut self, shapes: ButtonGroupConnectedShapes) -> Self {
        self.shapes = shapes;
        self
    }

    pub fn alignment(mut self, alignment: ButtonGroupAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn vertical_alignment(mut self, alignment: ButtonGroupVerticalAlignment) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    pub fn spacing(mut self, spacing: Pixels) -> Self {
        self.spacing = nonnegative(spacing);
        self
    }

    pub fn expanded_ratio(mut self, ratio: f32) -> Self {
        self.expanded_ratio = if ratio.is_finite() {
            ratio.max(0.)
        } else {
            0.15
        };
        self
    }

    /// Supplies decorative content for the named overflow trigger.
    pub fn overflow_trigger<E: IntoElement>(
        mut self,
        width: Pixels,
        content: impl FnOnce(bool) -> E + 'static,
    ) -> Self {
        self.overflow_trigger = Some((
            width,
            Box::new(move |open| content(open).into_any_element()),
        ));
        self
    }

    /// Adds repeatable decorative content before the overflow rows.
    pub fn overflow_content<E: IntoElement>(mut self, content: impl Fn() -> E + 'static) -> Self {
        self.overflow_content = Some(Rc::new(move || content().into_any_element()));
        self
    }

    /// Supplies the caller-owned open value for the overflow menu.
    pub fn overflow_open(mut self, open: bool) -> Self {
        self.overflow_open = Some(open);
        self
    }

    pub fn on_overflow_open_change(
        mut self,
        on_change: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_overflow_open_change = Some(Rc::new(on_change));
        self
    }
}

struct GroupState {
    width: Rc<Cell<Pixels>>,
    focus: HashMap<SharedString, FocusHandle>,
    scope: FocusHandle,
    overflow_focus: FocusHandle,
    overflow_open: Rc<Cell<bool>>,
    shown: Option<usize>,
    source: Entity<InteractionSource>,
}

impl ButtonGroupItem {
    fn preferred_width(
        &self,
        size: ControlSize,
        minimum: Pixels,
        window: &Window,
        cx: &App,
    ) -> Pixels {
        let width = match self.content {
            ItemContent::Label | ItemContent::LeadingIcon(_) => group_label_width(
                size,
                self.label.clone(),
                self.style.padding,
                matches!(self.content, ItemContent::LeadingIcon(_)),
                window,
                cx,
            ),
            ItemContent::Icon(_) => group_icon_width(size, self.icon_width, self.style.padding),
            ItemContent::Custom(width, _) => nonnegative(width).unwrap_or(minimum),
        };
        if self.weight > 0. {
            minimum
        } else {
            width.max(minimum)
        }
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

impl RenderOnce for ButtonGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
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
            });
        let preferred = self
            .items
            .iter()
            .map(|item| item.preferred_width(self.size, minimum, window, cx))
            .collect::<Vec<_>>();
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
        let (handles, scope, overflow_focus, menu_open, previous_visible, source) =
            state.update(cx, |state, cx| {
                let mut prior = std::mem::take(&mut state.focus);
                state.focus = self
                    .items
                    .iter()
                    .map(|item| {
                        let handle = prior.remove(&item.id).unwrap_or_else(|| cx.focus_handle());
                        (item.id.clone(), handle)
                    })
                    .collect();
                let previous_visible = state.shown.replace(shown);
                (
                    state.focus.clone(),
                    state.scope.clone(),
                    state.overflow_focus.clone(),
                    state.overflow_open.clone(),
                    previous_visible,
                    state.source.clone(),
                )
            });
        let selected: Vec<_> = self
            .items
            .iter()
            .filter(|item| {
                matches!(item.behavior, ItemBehavior::Grouped(_))
                    && self.selected.contains(&item.id)
            })
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
                self.items[..shown].iter().position(|item| {
                    !item.disabled && item.selected(self.selection, &selected) == Some(true)
                })
            })
            .or_else(|| focus.iter().position(|(_, enabled)| *enabled));
        if overflow {
            if let Some(displaced) = self.items[shown..]
                .iter()
                .map(|item| &handles[&item.id])
                .find(|handle| handle.is_focused(window))
            {
                move_focus_after_layout(displaced.clone(), overflow_focus.clone(), window);
            }
        } else if let Some(previous) = previous_visible {
            let was_open = menu_open.replace(false);
            if (overflow_focus.is_focused(window) || was_open && scope.contains_focused(window, cx))
                && let Some(replacement) = focus
                    .iter()
                    .skip(previous)
                    .chain(&focus)
                    .find(|(_, enabled)| *enabled)
                && let Some(displaced) = window.focused(cx)
            {
                move_focus_after_layout(displaced, replacement.0.clone(), window);
            }
            if was_open && let Some(on_change) = self.on_overflow_open_change.clone() {
                window.defer(cx, move |window, cx| on_change(false, window, cx));
            }
        }

        let mut row = div()
            .id(self.id.clone())
            .role(Role::Group)
            .aria_label(self.label)
            .track_focus(&scope)
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
        row = match self.alignment {
            ButtonGroupAlignment::Start => row.justify_start(),
            ButtonGroupAlignment::Center => row.justify_center(),
            ButtonGroupAlignment::End => row.justify_end(),
            ButtonGroupAlignment::SpaceBetween => row.justify_between(),
            ButtonGroupAlignment::SpaceAround => row.justify_around(),
            ButtonGroupAlignment::SpaceEvenly => row.justify_evenly(),
        };
        row = match self.vertical_alignment {
            ButtonGroupVerticalAlignment::Top => row.items_start(),
            ButtonGroupVerticalAlignment::Center => row.items_center(),
            ButtonGroupVerticalAlignment::Bottom => row.items_end(),
        };

        let segment = |index| {
            connected.then_some(match (index == 0, index + 1 == count) {
                (true, true) => Segment::Only,
                (true, false) => Segment::Leading,
                (false, true) => Segment::Trailing,
                (false, false) => Segment::Middle,
            })
        };
        let window_id = window.window_handle().window_id();
        let active = self.items[..shown].iter().position(|item| {
            !item.disabled
                && item
                    .source
                    .as_ref()
                    .unwrap_or(&source)
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
            let shape = segment(index);
            let button = ButtonPart::new(id, item.label.clone(), width)
                .size(self.size)
                .shape(self.shape)
                .variant(item.appearance.unwrap_or(self.appearance))
                .segment(shape)
                .disabled(item.disabled)
                .track_focus(&handles[&item.id])
                .tab_stop(tab_stop == Some(index))
                .selected(selected_state)
                .interaction_source(item.source.as_ref().unwrap_or(&source))
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
            let shape = segment(shown);
            let shapes = self.shapes.for_segment(shape);
            let content = self.overflow_trigger.map(|(_, content)| content);
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

fn nonnegative(value: Pixels) -> Option<Pixels> {
    (f32::from(value).is_finite() && value >= px(0.)).then_some(value)
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

fn weighted_widths(base: &[Pixels], weights: &[f32], available: Pixels) -> Vec<Pixels> {
    let total_weight = weights.iter().map(|&weight| f64::from(weight)).sum::<f64>();
    if total_weight == 0. {
        return base.to_vec();
    }
    let spare = (available - base.iter().copied().sum::<Pixels>()).max(px(0.));
    base.iter()
        .zip(weights)
        .map(|(&width, &weight)| width + spare * (f64::from(weight) / total_weight) as f32)
        .collect()
}

fn pressed_widths(
    widths: &[Pixels],
    limits: &[Option<Pixels>],
    active: Option<usize>,
    ratio: f32,
) -> Vec<Pixels> {
    let mut resolved = widths.to_vec();
    let Some(active) = active.filter(|&index| index < widths.len()) else {
        return resolved;
    };
    let neighbors = active
        .checked_sub(1)
        .into_iter()
        .chain((active + 1 < widths.len()).then_some(active + 1));
    let count = neighbors.clone().count().max(1) as f32;
    let requested = widths[active] * (ratio / count);
    for index in neighbors {
        let available = (widths[index] - px(24.)).max(px(0.));
        let contribution =
            requested.min(limits[index].map_or(available, |limit| limit.min(available)));
        resolved[index] -= contribution;
        resolved[active] += contribution;
    }
    resolved
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
        let adjusted = pressed_widths(&widths, &[None; 3], Some(1), 0.15);
        assert_eq!(
            adjusted.iter().copied().sum::<Pixels>(),
            widths.iter().copied().sum::<Pixels>()
        );
        assert!(adjusted[1] > widths[1]);
        assert!(adjusted[0] < widths[0] && adjusted[2] < widths[2]);
    }

    #[test]
    fn weighted_space_and_neighbor_compression_preserve_the_available_width() {
        let widths = weighted_widths(&[px(48.), px(48.), px(60.)], &[1., 3., 0.], px(236.));
        assert_eq!(widths, [px(68.), px(108.), px(60.)]);
        assert_eq!(
            pressed_widths(&widths, &[Some(px(4.)), None, Some(px(0.))], Some(1), 0.5),
            [px(64.), px(112.), px(60.)]
        );
    }
}

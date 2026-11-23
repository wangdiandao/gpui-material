use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, IntoElement, Pixels, SharedString, Window,
    prelude::FluentBuilder,
};
use gpui_material_components::{
    ButtonBorder, ButtonColors, ButtonElevations, ButtonPadding, ButtonShape, ButtonShapes,
    ButtonVariant, ControlSize, IconButtonWidth, SemanticTone,
    button_composition::{ButtonPart, Segment},
};
use gpui_material_foundation::InteractionSource;

mod layout;
mod render;

use layout::nonnegative;

type Command = Rc<dyn Fn(&mut Window, &mut App)>;
type SelectionChange = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;
type ToggleChange = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type Decoration = Rc<dyn Fn() -> AnyElement>;
type TriggerContent = Box<dyn FnOnce(bool) -> AnyElement>;

/// Visual layout style for a [`ButtonGroup`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonGroupVariant {
    /// Separate pill/rounded buttons with dynamic neighbor expansion on press.
    #[default]
    Standard,
    /// Adjoining segmented buttons with shared inner corners.
    Connected,
}

/// Logical horizontal arrangement of visible members and the overflow trigger.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonGroupAlignment {
    /// Packs members toward the leading edge (respects RTL).
    #[default]
    Start,
    /// Centers members within the available horizontal space.
    Center,
    /// Packs members toward the trailing edge (respects RTL).
    End,
    /// Distributes remaining space evenly between members.
    SpaceBetween,
    /// Distributes remaining space around each member, with half-size outer margins.
    SpaceAround,
    /// Distributes remaining space equally between members and both outer edges.
    SpaceEvenly,
}

/// Vertical cross-axis alignment for members inside a [`ButtonGroup`] row.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonGroupVerticalAlignment {
    /// Aligns the member to the top edge of the row.
    #[default]
    Top,
    /// Centers the member vertically within the row.
    Center,
    /// Aligns the member to the bottom edge of the row.
    Bottom,
}

/// Overrides connected shapes by a segment's logical position in the visible row.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ButtonGroupConnectedShapes {
    /// Corner shapes for the first visible segment in a connected group.
    pub leading: ButtonShapes,
    /// Corner shapes for interior segments in a connected group.
    pub middle: ButtonShapes,
    /// Corner shapes for the last visible segment in a connected group.
    pub trailing: ButtonShapes,
    /// Corner shapes when only a single segment is visible.
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
    /// Corner shape overrides for resting, pressed, and checked states.
    pub shapes: ButtonShapes,
    /// Container and content color overrides.
    pub colors: ButtonColors,
    /// Optional border override.
    pub border: Option<ButtonBorder>,
    /// Elevation overrides across interaction states.
    pub elevations: ButtonElevations,
    /// Optional content padding override.
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
    /// No group-managed selection; members act as standalone command buttons.
    #[default]
    Actions,
    /// At most one member may be selected; clicking the active member clears it.
    Single,
    /// Exactly one member remains selected once a selection exists.
    SingleRequired,
    /// Any number of members (including zero) may be selected simultaneously.
    Multiple,
    /// One or more members may be selected; the last active member cannot be cleared.
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
    tone: Option<SemanticTone>,
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
    /// Creates a button group member with a stable `id` and visible/accessible `label`.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            behavior: ItemBehavior::Grouped(None),
            content: ItemContent::Label,
            overflow_content: None,
            appearance: None,
            tone: None,
            icon_width: IconButtonWidth::Uniform,
            style: ButtonGroupItemStyle::default(),
            weight: 0.,
            compression_limit: None,
            vertical_alignment: None,
            source: None,
        }
    }

    /// Disables activation and selection toggling for this member when `true`.
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

    /// Displays a leading icon alongside the member's label.
    pub fn leading_icon<E: IntoElement>(mut self, icon: impl Fn() -> E + 'static) -> Self {
        self.content = ItemContent::LeadingIcon(Rc::new(move || icon().into_any_element()));
        self
    }

    /// Renders this member as an icon-only button while retaining `label` for accessibility and overflow.
    pub fn icon_only<E: IntoElement>(mut self, icon: impl Fn() -> E + 'static) -> Self {
        self.content = ItemContent::Icon(Rc::new(move || icon().into_any_element()));
        self
    }

    /// Sets the width preset used when this member is rendered as an icon-only button.
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

    /// Overrides the group's default [`ButtonVariant`] for this member.
    pub fn appearance(mut self, appearance: ButtonVariant) -> Self {
        self.appearance = Some(appearance);
        self
    }

    /// Overrides the semantic tone for this member (both inline and in the overflow menu).
    pub fn tone(mut self, tone: SemanticTone) -> Self {
        self.tone = Some(tone);
        self
    }

    /// Applies per-member visual style overrides.
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

    /// Overrides the group's vertical cross-axis alignment for this member.
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
    tone: SemanticTone,
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
    /// Creates a standard button group with an element `id`, accessible `label`, and `items`.
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
            tone: SemanticTone::Default,
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

    /// Creates a connected Material 3 Expressive button group.
    pub fn connected(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = ButtonGroupItem>,
    ) -> Self {
        Self::new(id, label, items).variant(ButtonGroupVariant::Connected)
    }

    /// Sets whether members are rendered as separate (`Standard`) or adjoining (`Connected`) buttons.
    pub fn variant(mut self, variant: ButtonGroupVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the default [`ButtonVariant`] applied to group members.
    pub fn appearance(mut self, appearance: ButtonVariant) -> Self {
        self.appearance = appearance;
        self
    }

    /// Applies a default semantic tone across members and overflow menu items unless overridden per item.
    pub fn tone(mut self, tone: SemanticTone) -> Self {
        self.tone = tone;
        self
    }

    /// Sets the [`ControlSize`] shared by all members and the overflow trigger.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Configures caller-controlled selection `mode` and the currently `selected` item IDs.
    pub fn selection(
        mut self,
        mode: ButtonGroupSelection,
        selected: impl IntoIterator<Item = SharedString>,
    ) -> Self {
        self.selection = mode;
        self.selected = selected.into_iter().collect();
        self
    }

    /// Registers a callback invoked with the next list of selected item IDs when selection changes.
    pub fn on_change(
        mut self,
        callback: impl Fn(Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Configures required single selection (`ButtonGroupSelection::SingleRequired`) and
    /// delivers the newly selected item ID directly.
    pub fn single_select(
        mut self,
        selected: impl Into<SharedString>,
        on_select: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.selection = ButtonGroupSelection::SingleRequired;
        self.selected = vec![selected.into()];
        self.on_change = Some(Rc::new(move |next, window, cx| {
            if let Some(id) = next.into_iter().next() {
                on_select(id, window, cx);
            }
        }));
        self
    }

    /// Configures deselectable single selection (`ButtonGroupSelection::Single`) and
    /// delivers the newly selected item ID, or `None` when the active option is cleared.
    pub fn single_optional_select(
        mut self,
        selected: Option<SharedString>,
        on_select: impl Fn(Option<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.selection = ButtonGroupSelection::Single;
        self.selected = selected.into_iter().collect();
        self.on_change = Some(Rc::new(move |next, window, cx| {
            on_select(next.into_iter().next(), window, cx);
        }));
        self
    }

    /// Sets the accessible name for the overflow menu trigger button. Defaults to `"More actions"`.
    pub fn overflow_label(mut self, label: impl Into<SharedString>) -> Self {
        self.overflow_label = label.into();
        self
    }

    /// Sets the base [`ButtonShape`] for standard button groups.
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }

    /// Overrides segment corner shapes for [`ButtonGroupVariant::Connected`].
    pub fn connected_shapes(mut self, shapes: ButtonGroupConnectedShapes) -> Self {
        self.shapes = shapes;
        self
    }

    /// Sets the horizontal alignment of visible members within the group row.
    pub fn alignment(mut self, alignment: ButtonGroupAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets the default vertical cross-axis alignment of members within the group row.
    pub fn vertical_alignment(mut self, alignment: ButtonGroupVerticalAlignment) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    /// Overrides the horizontal gap between members in logical pixels.
    pub fn spacing(mut self, spacing: Pixels) -> Self {
        self.spacing = nonnegative(spacing);
        self
    }

    /// Sets the press-expansion width ratio for [`ButtonGroupVariant::Standard`] members. Defaults to `0.15`.
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

    /// Registers a callback invoked when the overflow menu requests to open or close.
    pub fn on_overflow_open_change(
        mut self,
        on_change: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_overflow_open_change = Some(Rc::new(on_change));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connected_and_single_select_shortcuts_configure_expected_fields() {
        let required = ButtonGroup::connected(
            "period",
            "Period",
            [
                ButtonGroupItem::new("day", "Day"),
                ButtonGroupItem::new("week", "Week"),
            ],
        )
        .single_select("week", |_, _, _| {});
        assert_eq!(required.variant, ButtonGroupVariant::Connected);
        assert_eq!(required.selection, ButtonGroupSelection::SingleRequired);
        assert_eq!(required.selected, vec![SharedString::from("week")]);
        assert!(required.on_change.is_some());

        let optional_selected = ButtonGroup::connected(
            "filter",
            "Filter",
            [
                ButtonGroupItem::new("open", "Open"),
                ButtonGroupItem::new("closed", "Closed"),
            ],
        )
        .single_optional_select(Some("open".into()), |_, _, _| {});
        assert_eq!(optional_selected.variant, ButtonGroupVariant::Connected);
        assert_eq!(optional_selected.selection, ButtonGroupSelection::Single);
        assert_eq!(optional_selected.selected, vec![SharedString::from("open")]);
        assert!(optional_selected.on_change.is_some());

        let optional_empty = ButtonGroup::connected("filter-empty", "Filter", [])
            .single_optional_select(None, |_, _, _| {});
        assert_eq!(optional_empty.selection, ButtonGroupSelection::Single);
        assert!(optional_empty.selected.is_empty());
        assert!(optional_empty.on_change.is_some());
    }
}

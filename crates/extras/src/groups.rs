use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, IntoElement, Pixels, SharedString, Window,
    prelude::FluentBuilder,
};
use gpui_material_components::{
    ButtonBorder, ButtonColors, ButtonElevations, ButtonPadding, ButtonShape, ButtonShapes,
    ButtonVariant, ControlSize, IconButtonWidth,
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

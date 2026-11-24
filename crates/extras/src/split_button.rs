use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_material_components::{
    ButtonColors, ButtonVariant, ControlBorder, ControlElevations, ControlShapes, ControlSize,
    Icon, IconName, LogicalEdges, Menu, MenuItem, SemanticTone,
    button_composition::{ButtonPart, Segment, label_width, split_trailing_size},
};
use gpui_material_foundation::{InteractionSource, current_material_theme};
use gpui_material_theme::Direction;

type Command = Rc<dyn Fn(&mut Window, &mut App)>;
type Change = Rc<dyn Fn(bool, &mut Window, &mut App)>;

enum Leading {
    Label,
    IconAndLabel(AnyElement),
    Icon(AnyElement),
    Content(AnyElement),
}

enum Trailing {
    Arrow,
    Icon(AnyElement),
    Content(AnyElement),
}

enum TrailingAction {
    Menu(Vec<MenuItem>),
    Invoke(Command),
    Toggle { selected: bool, change: Change },
}

/// Optional per-segment appearance. Omitted values keep the selected size
/// and variant's defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SplitButtonSegmentStyle {
    /// Corner shape overrides for resting, pressed, and checked states.
    pub shapes: ControlShapes,
    /// Container and content color overrides.
    pub colors: ButtonColors,
    /// Optional border override.
    pub border: Option<ControlBorder>,
    /// Elevation overrides across interaction states.
    pub elevations: ControlElevations,
    /// Optional content padding override.
    pub padding: Option<LogicalEdges>,
}

#[derive(Default)]
struct Part {
    style: SplitButtonSegmentStyle,
    source: Option<Entity<InteractionSource>>,
}

impl Part {
    fn apply(self, button: ButtonPart) -> ButtonPart {
        button
            .shapes(self.style.shapes)
            .colors(self.style.colors)
            .elevations(self.style.elevations)
            .when_some(self.style.border, ButtonPart::border)
            .when_some(self.style.padding, ButtonPart::content_padding)
            .when_some(self.source.as_ref(), ButtonPart::interaction_source)
    }
}

/// Two adjoining actions with independent focus, content, and appearance.
#[derive(IntoElement)]
pub struct SplitButton {
    id: ElementId,
    leading_name: SharedString,
    trailing_name: SharedString,
    leading_content: Leading,
    trailing_content: Trailing,
    leading_style: Part,
    trailing_style: Part,
    trailing_action: TrailingAction,
    size: ControlSize,
    variant: ButtonVariant,
    leading_tone: SemanticTone,
    trailing_tone: SemanticTone,
    spacing: Pixels,
    disabled: bool,
    trailing_disabled: bool,
    on_activate: Option<Command>,
    focus: Option<(FocusHandle, FocusHandle)>,
}

impl SplitButton {
    /// Creates a split button with a primary `leading_name`, a `trailing_name` for the secondary
    /// control, and default dropdown `menu` items.
    pub fn new(
        id: impl Into<ElementId>,
        leading_name: impl Into<SharedString>,
        trailing_name: impl Into<SharedString>,
        menu: impl IntoIterator<Item = MenuItem>,
    ) -> Self {
        Self {
            id: id.into(),
            leading_name: leading_name.into(),
            trailing_name: trailing_name.into(),
            leading_content: Leading::Label,
            trailing_content: Trailing::Arrow,
            leading_style: Part::default(),
            trailing_style: Part::default(),
            trailing_action: TrailingAction::Menu(menu.into_iter().collect()),
            size: ControlSize::default(),
            variant: ButtonVariant::default(),
            leading_tone: SemanticTone::Default,
            trailing_tone: SemanticTone::Default,
            spacing: px(2.),
            disabled: false,
            trailing_disabled: false,
            on_activate: None,
            focus: None,
        }
    }

    /// Sets the [`ControlSize`] shared by both segments.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Sets the [`ButtonVariant`] shared by both segments.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Applies a Material 3 semantic tone to both segments.
    pub fn tone(mut self, tone: SemanticTone) -> Self {
        self.leading_tone = tone;
        self.trailing_tone = tone;
        self
    }

    /// Applies a Material 3 semantic tone to the leading segment.
    pub fn leading_tone(mut self, tone: SemanticTone) -> Self {
        self.leading_tone = tone;
        self
    }

    /// Applies a Material 3 semantic tone to the trailing segment.
    pub fn trailing_tone(mut self, tone: SemanticTone) -> Self {
        self.trailing_tone = tone;
        self
    }

    /// Sets the space between segments. Invalid lengths use the default 2 dp.
    pub fn spacing(mut self, spacing: Pixels) -> Self {
        self.spacing = if f32::from(spacing).is_finite() {
            spacing.max(px(0.))
        } else {
            px(2.)
        };
        self
    }

    /// Sets availability for the leading action.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Alias for [`Self::trailing_disabled`] when the trailing segment opens a menu.
    pub fn menu_disabled(self, disabled: bool) -> Self {
        self.trailing_disabled(disabled)
    }

    /// Disables the trailing segment (menu trigger, command, or toggle) when `true`.
    pub fn trailing_disabled(mut self, disabled: bool) -> Self {
        self.trailing_disabled = disabled;
        self
    }

    /// Registers a callback invoked when the leading primary segment is activated.
    pub fn on_activate(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(callback));
        self
    }

    /// Replaces the trailing menu or toggle with a command.
    pub fn on_trailing_activate(
        mut self,
        callback: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.trailing_action = TrailingAction::Invoke(Rc::new(callback));
        self
    }

    /// Replaces the trailing menu or command with a caller-controlled toggle.
    pub fn trailing_toggle(
        mut self,
        selected: bool,
        callback: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.trailing_action = TrailingAction::Toggle {
            selected,
            change: Rc::new(callback),
        };
        self
    }

    /// Displays an icon before the leading segment's label.
    pub fn leading_icon(mut self, icon: impl IntoElement) -> Self {
        self.leading_content = Leading::IconAndLabel(icon.into_any_element());
        self
    }

    /// Renders the leading segment with an icon only, keeping `leading_name` as its accessible name.
    pub fn leading_icon_only(mut self, icon: impl IntoElement) -> Self {
        self.leading_content = Leading::Icon(icon.into_any_element());
        self
    }

    /// Replaces the leading label's presentation and retains its accessible name.
    pub fn leading_content(mut self, content: impl IntoElement) -> Self {
        self.leading_content = Leading::Content(content.into_any_element());
        self
    }

    /// Replaces the default dropdown arrow icon on the trailing segment.
    pub fn trailing_icon(mut self, icon: impl IntoElement) -> Self {
        self.trailing_content = Trailing::Icon(icon.into_any_element());
        self
    }

    /// Replaces the trailing icon's presentation and retains its accessible name.
    pub fn trailing_content(mut self, content: impl IntoElement) -> Self {
        self.trailing_content = Trailing::Content(content.into_any_element());
        self
    }

    /// Applies appearance overrides to the leading segment.
    pub fn leading_style(mut self, style: SplitButtonSegmentStyle) -> Self {
        self.leading_style.style = style;
        self
    }

    /// Applies appearance overrides to the trailing segment.
    pub fn trailing_style(mut self, style: SplitButtonSegmentStyle) -> Self {
        self.trailing_style.style = style;
        self
    }

    /// Connects an [`InteractionSource`] to observe interaction states on the leading segment.
    pub fn leading_interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.leading_style.source = Some(source.clone());
        self
    }

    /// Connects an [`InteractionSource`] to observe interaction states on the trailing segment.
    pub fn trailing_interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.trailing_style.source = Some(source.clone());
        self
    }

    /// Supplies independent focus handles for the leading and trailing segments.
    pub fn track_focus(mut self, primary: &FocusHandle, trailing: &FocusHandle) -> Self {
        self.focus = Some((primary.clone(), trailing.clone()));
        self
    }
}

impl RenderOnce for SplitButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let direction = current_material_theme(window, cx).layout.direction;
        let (primary_focus, trailing_focus) = self.focus.unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "split-focus"), cx, |_, cx| {
                    (cx.focus_handle(), cx.focus_handle())
                })
                .read(cx)
                .clone()
        });
        let primary_id = ElementId::from((self.id.clone(), "primary"));
        let primary = if matches!(self.leading_content, Leading::Label)
            && self.leading_style.style.padding.is_none()
        {
            let width = label_width(self.size, self.leading_name.clone(), window, cx);
            ButtonPart::new(primary_id, self.leading_name, width)
        } else {
            ButtonPart::intrinsic(primary_id, self.leading_name)
        };
        let primary = match self.leading_content {
            Leading::Label => primary,
            Leading::IconAndLabel(icon) => primary.leading_icon(icon),
            Leading::Icon(icon) => primary.icon(icon),
            Leading::Content(content) => primary.content(content),
        };
        let primary = self
            .leading_style
            .apply(primary)
            .size(self.size)
            .variant(self.variant)
            .tone(self.leading_tone)
            .segment(Some(Segment::SplitLeading))
            .track_focus(&primary_focus)
            .disabled(self.disabled)
            .when_some(self.on_activate, |button, action| {
                button.on_activate(move |window, cx| action(window, cx))
            });

        let trailing_id = ElementId::from((self.id.clone(), "secondary"));
        let restore = trailing_focus.clone();
        let trailing = move |active| {
            let part = if matches!(self.trailing_content, Trailing::Content(_))
                || self.trailing_style.style.padding.is_some()
            {
                ButtonPart::intrinsic(trailing_id, self.trailing_name)
            } else {
                ButtonPart::new(
                    trailing_id,
                    self.trailing_name,
                    px(split_trailing_size(self.size).1),
                )
            };
            let part = match self.trailing_content {
                Trailing::Arrow => part.icon(Icon::new(IconName::ArrowDropDown).rotate(
                    gpui::Radians(if active { std::f32::consts::PI } else { 0. }),
                )),
                Trailing::Icon(icon) => part.icon(icon),
                Trailing::Content(content) => part.content(content),
            };
            self.trailing_style
                .apply(part)
                .size(self.size)
                .variant(self.variant)
                .tone(self.trailing_tone)
                .segment(Some(Segment::SplitTrailing))
                .track_focus(&trailing_focus)
                .disabled(self.trailing_disabled)
        };
        let trailing = match self.trailing_action {
            TrailingAction::Menu(items) if self.trailing_disabled || items.is_empty() => {
                trailing(false).disabled(true).into_any_element()
            }
            TrailingAction::Menu(items) => {
                Menu::composed((self.id.clone(), "menu"), items, move |open, toggle| {
                    trailing(open)
                        .presentation_selected(open)
                        .on_accessible_activate(move |window, cx| {
                            restore.focus(window, cx);
                            toggle(window, cx);
                        })
                        .into_any_element()
                })
                .into_any_element()
            }
            TrailingAction::Invoke(command) => trailing(false)
                .on_activate(move |window, cx| command(window, cx))
                .into_any_element(),
            TrailingAction::Toggle { selected, change } => trailing(selected)
                .selected(Some(selected))
                .on_change(move |next, window, cx| change(next, window, cx))
                .into_any_element(),
        };
        div()
            .id(self.id)
            .role(Role::Group)
            .flex()
            .items_center()
            .min_w_0()
            .max_w_full()
            .gap(self.spacing)
            .when(direction == Direction::Rtl, Styled::flex_row_reverse)
            .child(primary)
            .child(trailing)
    }
}

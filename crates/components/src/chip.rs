mod group;
mod render;

pub use group::{ChipGroup, ChipGroupItem, ChipGroupSelection};

use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, FocusHandle, Hsla, IntoElement, Pixels, SharedString,
    Window, px,
};
use gpui_material_foundation::InteractionSource;
use gpui_material_theme::Elevation;

use crate::SemanticTone;
use crate::action::{Activate, Change};
use crate::button_style::{
    ButtonBorder, ButtonElevations, ButtonPadding, ButtonShapes, finite_length,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ChipVariant {
    #[default]
    Assist,
    Filter,
    Input,
    Suggestion,
}

/// Per-slot pigments for one chip state. Omitted fields retain the defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ChipStateColors {
    pub container: Option<Hsla>,
    pub label: Option<Hsla>,
    pub leading: Option<Hsla>,
    pub trailing: Option<Hsla>,
    pub border: Option<Hsla>,
}

/// Enabled, selected, and disabled chip pigments. Disabled-selected fields fall
/// back to disabled overrides. Selection applies to Filter and Input chips.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ChipColors {
    pub enabled: ChipStateColors,
    pub selected: ChipStateColors,
    pub disabled: ChipStateColors,
    pub disabled_selected: ChipStateColors,
}

impl ChipColors {
    fn for_state(self, disabled: bool, selected: bool) -> ChipStateColors {
        match (disabled, selected) {
            (false, false) => self.enabled,
            (false, true) => self.selected,
            (true, false) => self.disabled,
            (true, true) => ChipStateColors {
                container: self.disabled_selected.container.or(self.disabled.container),
                label: self.disabled_selected.label.or(self.disabled.label),
                leading: self.disabled_selected.leading.or(self.disabled.leading),
                trailing: self.disabled_selected.trailing.or(self.disabled.trailing),
                border: self.disabled_selected.border.or(self.disabled.border),
            },
        }
    }
}

/// Optional shadows for each state. Disabled and dragged take precedence over
/// press, keyboard focus, hover, and rest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChipElevations {
    pub resting: Option<Elevation>,
    pub pressed: Option<Elevation>,
    pub focused: Option<Elevation>,
    pub hovered: Option<Elevation>,
    pub dragged: Option<Elevation>,
    pub disabled: Option<Elevation>,
}

impl ChipElevations {
    fn button(self) -> ButtonElevations {
        ButtonElevations {
            resting: self.resting,
            pressed: self.pressed,
            focused: self.focused,
            hovered: self.hovered,
            disabled: self.disabled,
        }
    }
}

/// A Material chip. Filter and Input chips request controlled selection changes.
/// Filter and Input chips can expose a separate removal action with its own focus.
#[derive(IntoElement)]
pub struct Chip {
    id: ElementId,
    label: SharedString,
    variant: ChipVariant,
    selected: bool,
    elevated: bool,
    disabled: bool,
    tonal: bool,
    tone: SemanticTone,
    dragged: bool,
    content: Option<AnyElement>,
    leading: Option<AnyElement>,
    avatar: Option<AnyElement>,
    trailing: Option<AnyElement>,
    shapes: ButtonShapes,
    colors: ChipColors,
    border: Option<ButtonBorder>,
    elevations: ChipElevations,
    spacing: Pixels,
    padding: Option<ButtonPadding>,
    focus: Option<FocusHandle>,
    tab_stop: bool,
    interaction_source: Option<Entity<InteractionSource>>,
    on_activate: Option<Activate>,
    on_change: Option<Change>,
    remove: Option<(SharedString, Activate)>,
}

impl Chip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: ChipVariant::default(),
            selected: false,
            elevated: false,
            disabled: false,
            tonal: false,
            tone: SemanticTone::Default,
            dragged: false,
            content: None,
            leading: None,
            avatar: None,
            trailing: None,
            shapes: ButtonShapes::default(),
            colors: ChipColors::default(),
            border: None,
            elevations: ChipElevations::default(),
            spacing: px(8.),
            padding: None,
            focus: None,
            tab_stop: true,
            interaction_source: None,
            on_activate: None,
            on_change: None,
            remove: None,
        }
    }

    /// Creates an Assist chip.
    pub fn assist(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).variant(ChipVariant::Assist)
    }

    /// Creates a Filter chip.
    pub fn filter(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).variant(ChipVariant::Filter)
    }

    /// Creates an Input chip.
    pub fn input(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).variant(ChipVariant::Input)
    }

    /// Creates a Suggestion chip.
    pub fn suggestion(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).variant(ChipVariant::Suggestion)
    }

    pub fn variant(mut self, variant: ChipVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Applies a Material 3 semantic tone shortcut for enabled and selected states.
    /// Explicit `.colors` / `.border` overrides and disabled states still take precedence.
    pub fn tone(mut self, tone: SemanticTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn elevated(mut self, elevated: bool) -> Self {
        self.elevated = elevated;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Selects the expressive Tonal defaults for Filter, elevated Filter, and Input.
    pub fn tonal(mut self, tonal: bool) -> Self {
        self.tonal = tonal;
        self
    }

    /// Supplies visual label content. The constructor's label names the action.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn leading_icon(mut self, icon: impl IntoElement) -> Self {
        self.leading = Some(icon.into_any_element());
        self
    }

    /// Sets the Input chip's circular avatar, taking precedence over its leading icon.
    pub fn avatar(mut self, avatar: impl IntoElement) -> Self {
        self.avatar = Some(avatar.into_any_element());
        self
    }

    /// Sets visual trailing content for Assist, Filter, or Input. With an
    /// independent removal action this is that button's visual content.
    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    pub fn shapes(mut self, shapes: ButtonShapes) -> Self {
        self.shapes = shapes;
        self
    }

    pub fn colors(mut self, colors: ChipColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn border(mut self, border: ButtonBorder) -> Self {
        self.border = Some(border);
        self
    }

    pub fn elevations(mut self, elevations: ChipElevations) -> Self {
        self.elevations = elevations;
        self
    }

    /// Reflects caller-owned drag state and cancels pending pointer activation.
    pub fn dragged(mut self, dragged: bool) -> Self {
        self.dragged = dragged;
        self
    }

    pub fn spacing(mut self, spacing: Pixels) -> Self {
        self.spacing = finite_length(spacing);
        self
    }

    pub fn content_padding(mut self, padding: ButtonPadding) -> Self {
        self.padding = Some(ButtonPadding {
            top: finite_length(padding.top),
            end: finite_length(padding.end),
            bottom: finite_length(padding.bottom),
            start: finite_length(padding.start),
        });
        self
    }

    /// Attaches a caller-owned focus handle to the primary chip action.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    pub(crate) fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    /// Observes the primary chip action under the constructor's chip ID.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.interaction_source = Some(source.clone());
        self
    }

    pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }

    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Adds an independent removal action to a Filter or Input chip.
    /// The optional restoration target is focused before calling the owner.
    /// The owner can pass the next chip or a stable control outside the group.
    pub fn on_remove(
        mut self,
        name: impl Into<SharedString>,
        restore: Option<FocusHandle>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.remove = Some((
            name.into(),
            Rc::new(move |window, cx| {
                if let Some(restore) = &restore {
                    restore.focus(window, cx);
                }
                handler(window, cx);
            }),
        ));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_variant_shorthands_and_tone_configure_expected_fields() {
        assert_eq!(Chip::assist("c1", "Assist").variant, ChipVariant::Assist);
        assert_eq!(Chip::filter("c2", "Filter").variant, ChipVariant::Filter);
        assert_eq!(Chip::input("c3", "Input").variant, ChipVariant::Input);
        assert_eq!(
            Chip::suggestion("c4", "Suggest").variant,
            ChipVariant::Suggestion
        );
        let toned = Chip::filter("c5", "Danger").tone(SemanticTone::Danger);
        assert_eq!(toned.tone, SemanticTone::Danger);
    }
}

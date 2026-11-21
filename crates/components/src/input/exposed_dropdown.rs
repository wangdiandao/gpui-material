use std::rc::Rc;

use crate::MenuSurfaceStyle;
use gpui::{
    AnyElement, App, ElementId, Entity, Hsla, IntoElement, Pixels, SharedString, Window, px,
};

use super::TextField;

mod render;
mod state;

pub use state::ExposedDropdownState;
use state::Visibility;

/// A selectable dropdown value. IDs remain stable when options are reordered.
#[derive(Clone, PartialEq, Eq)]
pub struct DropdownOption {
    pub id: ElementId,
    pub label: SharedString,
}

impl DropdownOption {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// Focus and activation behavior for the primary dropdown anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropdownAnchor {
    /// Allows typing and keeps focus in the field when the menu opens.
    PrimaryEditable,
    /// Moves focus to the list when the read-only field opens the menu.
    PrimaryNotEditable,
}

type OpenChange = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type Dismiss = Rc<dyn Fn(&mut Window, &mut App)>;
type Select = Rc<dyn Fn(&DropdownOption, &mut Window, &mut App)>;
type Editor = Box<dyn FnOnce(TextField) -> AnyElement>;
type AnchorContent = Box<dyn FnOnce(bool) -> AnyElement>;
type OptionContent = Rc<dyn Fn(&DropdownOption, bool, &mut Window, &mut App) -> AnyElement>;

/// A Material field and anchored list in editable and selection-only modes.
/// Typing filters the list, while only selecting an option calls the owner.
#[derive(IntoElement)]
pub struct ExposedDropdown {
    id: ElementId,
    label: SharedString,
    state: Entity<ExposedDropdownState>,
    options: Rc<[DropdownOption]>,
    primary: Option<DropdownAnchor>,
    primary_enabled: bool,
    secondary_enabled: bool,
    secondary: Option<(SharedString, AnchorContent)>,
    editor: Option<Editor>,
    option_content: Option<OptionContent>,
    visibility: Visibility,
    match_anchor_width: bool,
    popup_width: Pixels,
    appearance: MenuSurfaceStyle,
    container_color: Option<Hsla>,
    on_select: Option<Select>,
}

impl ExposedDropdown {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<ExposedDropdownState>,
        options: impl IntoIterator<Item = DropdownOption>,
    ) -> Self {
        Self::with_shared_options(id, label, state, options.into_iter().collect::<Vec<_>>())
    }

    /// Uses a caller-retained option slice to avoid per-render collection and
    /// linear slice equality comparisons.
    pub fn with_shared_options(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<ExposedDropdownState>,
        options: impl Into<Rc<[DropdownOption]>>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: state.clone(),
            options: options.into(),
            primary: None,
            primary_enabled: true,
            secondary_enabled: true,
            secondary: None,
            editor: None,
            option_content: None,
            visibility: Visibility::default(),
            match_anchor_width: true,
            popup_width: px(240.),
            appearance: MenuSurfaceStyle::default(),
            container_color: None,
            on_select: None,
        }
    }

    /// Controls primary activation and focus. Pair the anchor with the state's editing mode.
    pub fn primary_anchor(mut self, anchor: DropdownAnchor) -> Self {
        self.primary = Some(anchor);
        self
    }

    pub fn primary_anchor_enabled(mut self, enabled: bool) -> Self {
        self.primary_enabled = enabled;
        self
    }

    /// Supplies the name and decorative content of the secondary anchor.
    pub fn secondary_anchor<E: IntoElement>(
        mut self,
        label: impl Into<SharedString>,
        content: impl FnOnce(bool) -> E + 'static,
    ) -> Self {
        self.secondary = Some((
            label.into(),
            Box::new(move |open| content(open).into_any_element()),
        ));
        self
    }

    pub fn secondary_anchor_enabled(mut self, enabled: bool) -> Self {
        self.secondary_enabled = enabled;
        self
    }

    /// Styles or wraps the field carrying the dropdown's retained editor and semantics.
    pub fn editor<E: IntoElement>(mut self, render: impl FnOnce(TextField) -> E + 'static) -> Self {
        self.editor = Some(Box::new(move |field| render(field).into_any_element()));
        self
    }

    /// Supplies repeatable decorative content for each visible option.
    pub fn option_content<E: IntoElement>(
        mut self,
        render: impl Fn(&DropdownOption, bool, &mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.option_content = Some(Rc::new(move |option, highlighted, window, cx| {
            render(option, highlighted, window, cx).into_any_element()
        }));
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.visibility.controlled = Some(open);
        self
    }

    pub fn on_open_change(
        mut self,
        callback: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.visibility.change = Some(Rc::new(callback));
        self
    }

    pub fn on_dismiss(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.visibility.dismiss = Some(Rc::new(callback));
        self
    }

    pub fn match_anchor_width(mut self, matches: bool) -> Self {
        self.match_anchor_width = matches;
        self
    }

    /// Sets the independent width used with match_anchor_width(false).
    pub fn popup_width(mut self, width: Pixels) -> Self {
        if f32::from(width).is_finite() && width > px(0.) {
            self.popup_width = width;
        }
        self
    }

    pub fn appearance(mut self, appearance: MenuSurfaceStyle) -> Self {
        self.appearance = appearance;
        self
    }

    pub fn container_color(mut self, color: Hsla) -> Self {
        self.container_color = Some(color);
        self
    }

    pub fn on_select(
        mut self,
        on_select: impl Fn(&DropdownOption, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(on_select));
        self
    }
}

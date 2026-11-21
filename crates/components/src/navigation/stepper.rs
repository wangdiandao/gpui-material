use std::rc::Rc;

use gpui::{AnyElement, App, ElementId, Hsla, IntoElement, SharedString, Window};

mod render;

/// Layout axis for a [`Stepper`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StepperOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// Visual and semantic status of one step in a [`Stepper`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StepStatus {
    #[default]
    Upcoming,
    Active,
    Completed,
    Error,
    Disabled,
}

impl StepStatus {
    pub(super) fn accessible_label(self) -> &'static str {
        match self {
            Self::Upcoming => "upcoming",
            Self::Active => "current",
            Self::Completed => "completed",
            Self::Error => "error",
            Self::Disabled => "disabled",
        }
    }
}

/// One step entry in a [`Stepper`].
pub struct StepItem {
    pub(super) id: SharedString,
    pub(super) label: SharedString,
    pub(super) supporting: Option<SharedString>,
    pub(super) status: Option<StepStatus>,
    pub(super) icon: Option<AnyElement>,
    pub(super) content: Option<AnyElement>,
    pub(super) editable: bool,
}

impl StepItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            supporting: None,
            status: None,
            icon: None,
            content: None,
            editable: false,
        }
    }

    /// Optional supporting subtitle shown below the step label.
    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    /// Explicitly sets the step status, overriding any status derived from `Stepper::active_step`.
    pub fn status(mut self, status: StepStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Replaces the default numbered or status glyph inside the step indicator badge.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    /// Body content displayed beneath this step when active in vertical orientation.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    /// Marks this step as selectable in linear mode even if it is upcoming.
    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    /// Convenience helper to disable a step.
    pub fn disabled(mut self, disabled: bool) -> Self {
        if disabled {
            self.status = Some(StepStatus::Disabled);
        } else if self.status == Some(StepStatus::Disabled) {
            self.status = None;
        }
        self
    }
}

/// Per-instance pigment overrides for [`Stepper`]. Omitted fields use the window theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StepperColors {
    pub active_container: Option<Hsla>,
    pub active_content: Option<Hsla>,
    pub completed_container: Option<Hsla>,
    pub completed_content: Option<Hsla>,
    pub upcoming_container: Option<Hsla>,
    pub upcoming_content: Option<Hsla>,
    pub error_container: Option<Hsla>,
    pub error_content: Option<Hsla>,
    pub connector: Option<Hsla>,
    pub active_connector: Option<Hsla>,
    pub label: Option<Hsla>,
    pub supporting: Option<Hsla>,
}

pub(super) type StepSelect = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Horizontal or vertical Material 3 multi-step indicator and navigator.
#[derive(IntoElement)]
pub struct Stepper {
    pub(super) id: ElementId,
    pub(super) steps: Vec<StepItem>,
    pub(super) orientation: StepperOrientation,
    pub(super) active_step: Option<usize>,
    pub(super) non_linear: bool,
    pub(super) colors: StepperColors,
    pub(super) on_select: Option<StepSelect>,
    pub(super) accessible_label: SharedString,
}

impl Stepper {
    pub fn new(id: impl Into<ElementId>, steps: impl IntoIterator<Item = StepItem>) -> Self {
        Self {
            id: id.into(),
            steps: steps.into_iter().collect(),
            orientation: StepperOrientation::default(),
            active_step: None,
            non_linear: false,
            colors: StepperColors::default(),
            on_select: None,
            accessible_label: "Steps".into(),
        }
    }

    /// Chooses horizontal (default) or vertical layout.
    pub fn orientation(mut self, orientation: StepperOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Marks steps `< index` as `Completed`, `== index` as `Active`, and `> index` as `Upcoming`
    /// unless a step explicitly sets its own `StepItem::status(...)`.
    pub fn active_step(mut self, index: usize) -> Self {
        self.active_step = Some(index);
        self
    }

    /// Allows activating any non-disabled step (including upcoming steps) when `on_select` is set.
    pub fn non_linear(mut self, non_linear: bool) -> Self {
        self.non_linear = non_linear;
        self
    }

    /// Overrides indicator, connector, and label colors independently.
    pub fn colors(mut self, colors: StepperColors) -> Self {
        self.colors = colors;
        self
    }

    /// Invoked with the zero-based step index when a selectable step is activated.
    pub fn on_select(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Sets the accessible name for the stepper container. Defaults to `"Steps"`.
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = label.into();
        self
    }
}

pub(super) fn resolve_step_status(
    explicit: Option<StepStatus>,
    index: usize,
    active_step: Option<usize>,
) -> StepStatus {
    if let Some(status) = explicit {
        return status;
    }
    match active_step {
        Some(active) if index < active => StepStatus::Completed,
        Some(active) if index == active => StepStatus::Active,
        Some(_) => StepStatus::Upcoming,
        None => StepStatus::Upcoming,
    }
}

pub(super) fn step_is_interactive(
    status: StepStatus,
    editable: bool,
    non_linear: bool,
    has_on_select: bool,
) -> bool {
    if !has_on_select || status == StepStatus::Disabled {
        return false;
    }
    non_linear
        || editable
        || matches!(
            status,
            StepStatus::Completed | StepStatus::Active | StepStatus::Error
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_step_derives_statuses_unless_explicitly_overridden() {
        assert_eq!(resolve_step_status(None, 0, Some(1)), StepStatus::Completed);
        assert_eq!(resolve_step_status(None, 1, Some(1)), StepStatus::Active);
        assert_eq!(resolve_step_status(None, 2, Some(1)), StepStatus::Upcoming);
        assert_eq!(
            resolve_step_status(Some(StepStatus::Error), 1, Some(1)),
            StepStatus::Error
        );
        assert_eq!(
            resolve_step_status(Some(StepStatus::Disabled), 0, Some(1)),
            StepStatus::Disabled
        );
        assert_eq!(resolve_step_status(None, 0, None), StepStatus::Upcoming);
    }

    #[test]
    fn linear_and_non_linear_interactivity_respect_disabled_and_editable_rules() {
        assert!(!step_is_interactive(StepStatus::Active, false, true, false));
        assert!(!step_is_interactive(StepStatus::Disabled, true, true, true));
        assert!(!step_is_interactive(
            StepStatus::Upcoming,
            false,
            false,
            true
        ));
        assert!(step_is_interactive(StepStatus::Upcoming, true, false, true));
        assert!(step_is_interactive(StepStatus::Upcoming, false, true, true));
        assert!(step_is_interactive(
            StepStatus::Completed,
            false,
            false,
            true
        ));
        assert!(step_is_interactive(StepStatus::Error, false, false, true));
    }
}

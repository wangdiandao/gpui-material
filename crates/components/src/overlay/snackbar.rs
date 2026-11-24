mod host;
mod render;

use std::{rc::Rc, time::Duration};

use gpui::{AnyElement, App, EntityId, Hsla, IntoElement, SharedString, Window};
use gpui_material_theme::CornerShape;

use crate::SemanticTone;

pub use host::SnackbarHost;

pub(super) type Action = Rc<dyn Fn(&mut Window, &mut App)>;
pub(super) type Content = Box<dyn Fn(&mut Window, &mut App) -> AnyElement>;
pub(super) type Completion = Rc<dyn Fn(SnackbarResult, &mut Window, &mut App)>;
pub(super) type HostRenderer =
    Box<dyn Fn(&Snackbar, SnackbarParts, &mut Window, &mut App) -> AnyElement>;

/// How a current snackbar left its host's queue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnackbarResult {
    /// The snackbar's action button was activated.
    ActionPerformed,
    /// The snackbar timed out, was dismissed by the user, or was dismissed programmatically.
    Dismissed,
}

/// An entry in one snackbar host's queue.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SnackbarId {
    pub(super) host: EntityId,
    pub(super) sequence: usize,
}

/// Optional per-region colors. Omitted fields use the window's inverse palette.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SnackbarColors {
    /// Background color override for the snackbar container.
    pub container: Option<Hsla>,
    /// Foreground color override for the message text.
    pub content: Option<Hsla>,
    /// Foreground color override for the action button.
    pub action: Option<Hsla>,
    /// Foreground color override for the dismiss button.
    pub dismiss: Option<Hsla>,
}

/// Host-owned status and activation elements for a custom layout.
#[non_exhaustive]
pub struct SnackbarParts {
    /// The rendered message element.
    pub message: AnyElement,
    /// The optional rendered action control.
    pub action: Option<AnyElement>,
    /// The optional rendered dismiss control.
    pub dismiss: Option<AnyElement>,
    /// Scale to apply to dimensions in the custom container.
    pub scale: f32,
}

/// A message for a window's snackbar queue. The default timeout is four seconds.
pub struct Snackbar {
    pub(super) message: SharedString,
    pub(super) timeout: Option<Duration>,
    pub(super) action: Option<(SharedString, Action)>,
    pub(super) message_content: Option<Content>,
    pub(super) action_content: Option<Content>,
    pub(super) dismiss_label: Option<SharedString>,
    pub(super) dismiss_content: Option<(SharedString, Content)>,
    pub(super) show_dismiss: bool,
    pub(super) action_on_new_line: bool,
    pub(super) shape: CornerShape,
    pub(super) tone: SemanticTone,
    pub(super) colors: SnackbarColors,
    pub(super) on_dismiss: Option<Action>,
    pub(super) on_result: Option<Completion>,
}

impl Snackbar {
    /// Creates a snackbar with the given message text and a default four-second timeout.
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            timeout: Some(Duration::from_secs(4)),
            action: None,
            message_content: None,
            action_content: None,
            dismiss_label: None,
            dismiss_content: None,
            show_dismiss: true,
            action_on_new_line: false,
            shape: CornerShape::Radii([4.; 4]),
            tone: SemanticTone::Default,
            colors: SnackbarColors::default(),
            on_dismiss: None,
            on_result: None,
        }
    }

    /// Sets how long the message is visible. `None` waits for explicit dismissal.
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Adds a keyboard-accessible action. By default, messages with an action
    /// persist. Call `timeout` after this method to set a duration explicitly.
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        action: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), Rc::new(action)));
        self.timeout = None;
        self
    }

    /// Returns the plain message used as the status announcement.
    pub fn message(&self) -> &SharedString {
        &self.message
    }

    /// Returns the accessible name of the optional action.
    pub fn action_label(&self) -> Option<&SharedString> {
        self.action.as_ref().map(|(label, _)| label)
    }

    /// Replaces the visible message with repeatable decorative content.
    pub fn message_content<E: IntoElement>(
        mut self,
        content: impl Fn(&mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.message_content = Some(Box::new(move |window, cx| {
            content(window, cx).into_any_element()
        }));
        self
    }

    /// Replaces the visible content of the named action supplied by [`Self::action`].
    pub fn action_content<E: IntoElement>(
        mut self,
        content: impl Fn(&mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.action_content = Some(Box::new(move |window, cx| {
            content(window, cx).into_any_element()
        }));
        self
    }

    /// Overrides the accessible name of the default dismiss button (default `"Dismiss message"`).
    pub fn dismiss_label(mut self, label: impl Into<SharedString>) -> Self {
        self.dismiss_label = Some(label.into());
        self
    }

    /// Supplies the accessible name and decorative content of the close affordance.
    pub fn dismiss_content<E: IntoElement>(
        mut self,
        label: impl Into<SharedString>,
        content: impl Fn(&mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.dismiss_content = Some((
            label.into(),
            Box::new(move |window, cx| content(window, cx).into_any_element()),
        ));
        self
    }

    /// Shows or hides the explicit dismiss button. It is shown by default.
    pub fn show_dismiss(mut self, show: bool) -> Self {
        self.show_dismiss = show;
        self
    }

    /// Places the action below the message for a long action label.
    pub fn action_on_new_line(mut self, enabled: bool) -> Self {
        self.action_on_new_line = enabled;
        self
    }

    /// Overrides the corner shape of the snackbar container.
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = shape;
        self
    }

    /// Applies a Material 3 semantic tone shortcut.
    ///
    /// [`SemanticTone::Default`] keeps the M3 inverse palette; semantic tones
    /// use `(Container, OnContainer, Role)` while explicit [`Self::colors`]
    /// overrides still take precedence.
    pub fn tone(mut self, tone: SemanticTone) -> Self {
        self.tone = tone;
        self
    }

    /// Overrides the snackbar's container, content, action, and dismiss colors.
    pub fn colors(mut self, colors: SnackbarColors) -> Self {
        self.colors = colors;
        self
    }

    /// Runs on timeout, user dismissal, or explicit dismissal by the caller.
    pub fn on_dismiss(mut self, dismissed: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(dismissed));
        self
    }

    /// Receives one completion after any action or dismissal callback has run.
    pub fn on_result(
        mut self,
        complete: impl Fn(SnackbarResult, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_result = Some(Rc::new(complete));
        self
    }
}

use std::rc::Rc;

use gpui::{
    AnyElement, AnyWindowHandle, App, AppContext, Context, ElementId, Entity, Hsla, IntoElement,
    Pixels, SharedString, Window, WindowKind, WindowOptions, px,
};
use gpui_material_foundation::{MaterialRoot, current_material_theme, set_window_material_theme};
use gpui_material_theme::{CornerShape, Elevation};

use crate::SemanticTone;

mod render;
mod state;

pub use state::DialogState;
pub(super) use state::ModalSession;
use state::{NativeDialog, NativeDialogView, restore_parent};

/// An accepted transition from an open to a closed Material dialog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DialogCloseReason {
    /// Closed by activating the confirmation action.
    Confirm,
    /// Closed by activating the cancel or close action.
    Cancel,
    /// Closed by pressing the Escape key.
    Escape,
    /// Closed by clicking the modal scrim outside the surface.
    OutsideClick,
    /// Closed by the user closing a native dialog window.
    WindowClose,
    /// Closed programmatically via [`DialogState::close`].
    Programmatic,
}

/// Window-relative presentation for a Material dialog.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DialogVariant {
    /// Standard centered modal dialog.
    #[default]
    Basic,
    /// Uses a full-screen task surface in compact windows and a basic dialog
    /// at wider breakpoints, as required by Material dialog guidance.
    FullScreen,
}

/// User dismissal routes for a Material dialog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DialogProperties {
    /// Whether pressing Escape requests dismissal (default `true`).
    pub dismiss_on_escape: bool,
    /// Whether clicking the backdrop scrim requests dismissal (default `true`).
    pub dismiss_on_outside_click: bool,
}

impl Default for DialogProperties {
    fn default() -> Self {
        Self {
            dismiss_on_escape: true,
            dismiss_on_outside_click: true,
        }
    }
}

/// Local overrides for a dialog surface and its content slots.
#[derive(Clone, Copy, Debug, Default)]
pub struct DialogColors {
    /// Background color override for the dialog container.
    pub container: Option<Hsla>,
    /// Foreground color override for the optional header icon.
    pub icon: Option<Hsla>,
    /// Foreground color override for the dialog title.
    pub title: Option<Hsla>,
    /// Foreground color override for the supporting description text.
    pub text: Option<Hsla>,
}

type Confirm = Rc<dyn Fn(&mut Window, &mut App) -> bool>;
pub(super) type ModalDismissRequest = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App) -> bool>;
pub(super) type ModalClosed = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App)>;
type ActionSlot = Box<dyn FnOnce(DialogAction) -> AnyElement>;
type BodySlot = Box<dyn FnOnce(DialogAction, DialogAction) -> AnyElement>;

/// Activation for a dialog slot, valid while that open dialog owns input.
#[derive(Clone)]
pub struct DialogAction {
    session: ModalSession,
    reason: DialogCloseReason,
    validate: Option<Confirm>,
}

impl DialogAction {
    /// Routes user activation through the dialog's policy and close callback.
    pub fn activate(&self, window: &mut Window, cx: &mut App) {
        if self.session.current(window, cx)
            && self
                .validate
                .as_ref()
                .is_none_or(|validate| validate(window, cx))
        {
            self.session.finish(self.reason, window, cx);
        }
    }
}

/// A named Material modal with optional structured slots or caller-defined content.
#[derive(IntoElement)]
pub struct Dialog {
    id: ElementId,
    state: Entity<DialogState>,
    title: SharedString,
    show_title: bool,
    title_content: Option<AnyElement>,
    icon: Option<AnyElement>,
    description: Option<SharedString>,
    body: Option<BodySlot>,
    variant: DialogVariant,
    unstructured: bool,
    confirm: Option<(SharedString, Confirm)>,
    confirm_tone: SemanticTone,
    confirm_disabled: bool,
    confirm_loading: bool,
    confirm_content: Option<ActionSlot>,
    dismiss_content: Option<ActionSlot>,
    cancel: Option<SharedString>,
    show_cancel: bool,
    layer: usize,
    properties: DialogProperties,
    shape: Option<CornerShape>,
    colors: DialogColors,
    tonal_elevation: Option<Elevation>,
    shadow_elevation: Option<Elevation>,
    scrim_color: Option<Hsla>,
    scrim_opacity: f32,
    width: Pixels,
    dismiss_request: Option<ModalDismissRequest>,
    on_closed: Option<ModalClosed>,
}

impl Dialog {
    /// Creates a basic Material dialog bound to `state` with the given accessible title.
    pub fn new(
        id: impl Into<ElementId>,
        state: &Entity<DialogState>,
        title: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            title: title.into(),
            show_title: true,
            title_content: None,
            icon: None,
            description: None,
            body: None,
            variant: DialogVariant::Basic,
            unstructured: false,
            confirm: None,
            confirm_tone: SemanticTone::Default,
            confirm_disabled: false,
            confirm_loading: false,
            confirm_content: None,
            dismiss_content: None,
            cancel: None,
            show_cancel: true,
            layer: 0,
            properties: DialogProperties::default(),
            shape: None,
            colors: DialogColors::default(),
            tonal_elevation: None,
            shadow_elevation: None,
            scrim_color: None,
            scrim_opacity: 0.32,
            width: px(560.),
            dismiss_request: None,
            on_closed: None,
        }
    }

    /// Creates a basic alert dialog with a description and no default cancel button.
    pub fn alert(
        id: impl Into<ElementId>,
        state: &Entity<DialogState>,
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
    ) -> Self {
        Self::new(id, state, title)
            .description(description)
            .show_cancel(false)
    }

    /// Creates a modal shell whose body owns the structure and scrolling.
    pub fn unstructured(
        id: impl Into<ElementId>,
        state: &Entity<DialogState>,
        accessibility_label: impl Into<SharedString>,
    ) -> Self {
        Self {
            unstructured: true,
            ..Self::new(id, state, accessibility_label)
        }
    }

    /// Opens a separate GPUI modal window, with the active window as its native parent.
    /// Supply the invoking window and build a fresh dialog on each render. An already
    /// open state returns its native window handle, or None for an in-window dialog.
    pub fn open_window(
        state: &Entity<DialogState>,
        parent: &mut Window,
        mut options: WindowOptions,
        cx: &mut App,
        build: impl Fn(&Entity<DialogState>, &mut Window, &mut App) -> Self + 'static,
    ) -> gpui::Result<Option<AnyWindowHandle>> {
        let current = state.read(cx);
        if current.is_open() {
            return Ok(current.native.as_ref().map(|native| native.child));
        }
        let parent_handle = parent.window_handle();
        if cx
            .active_window()
            .is_none_or(|active| active.window_id() != parent_handle.window_id())
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the dialog's parent window must be active",
            )
            .into());
        }
        options.kind = WindowKind::Dialog;
        options.focus = true;
        options.show = true;
        let previous = parent.focused(cx);
        let theme = current_material_theme(parent, cx).clone();
        let state = state.clone();
        let window = MaterialRoot::open_window(options, cx, move |window, cx| {
            set_window_material_theme(Some(theme), window, cx);
            let child = window.window_handle();
            let weak = state.downgrade();
            let closed = cx.on_window_closed(move |cx, id| {
                if id == child.window_id()
                    && let Some(state) = weak.upgrade()
                {
                    state.update(cx, |state, cx| {
                        if let Some(native) = state.native.take() {
                            state.open = false;
                            state.last_close_reason = Some(DialogCloseReason::Programmatic);
                            restore_parent(native.parent, state.previous.take(), cx);
                            cx.notify();
                        }
                    });
                }
            });
            state.update(cx, |state, cx| {
                state.native = Some(NativeDialog {
                    parent: parent_handle,
                    child,
                    _closed: closed,
                });
                state.begin(previous, window, cx);
            });
            cx.new(|cx: &mut Context<NativeDialogView>| {
                cx.observe(&state, |_, _, cx| cx.notify()).detach();
                NativeDialogView {
                    state,
                    build: Box::new(build),
                }
            })
        })?;
        Ok(Some(window.into()))
    }

    /// Sets the presentation variant (default [`DialogVariant::Basic`]).
    pub fn variant(mut self, variant: DialogVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Adds a decorative icon above the dialog title.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    /// Replaces the visual heading, keeping the constructor's accessible name.
    pub fn title_content(mut self, title: impl IntoElement) -> Self {
        self.title_content = Some(title.into_any_element());
        self
    }

    /// Shows or hides the visible title while preserving the accessible dialog name.
    pub fn show_title(mut self, show: bool) -> Self {
        self.show_title = show;
        self
    }

    /// Sets the supporting text displayed beneath the dialog title.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets custom body content rendered inside the dialog's scrollable body region.
    pub fn body(mut self, body: impl IntoElement) -> Self {
        let body = body.into_any_element();
        self.body = Some(Box::new(|_, _| body));
        self
    }

    /// Builds the body with the confirmation and cancellation activation handles.
    pub fn body_content<E: IntoElement>(
        mut self,
        body: impl FnOnce(DialogAction, DialogAction) -> E + 'static,
    ) -> Self {
        self.body = Some(Box::new(move |confirm, dismiss| {
            body(confirm, dismiss).into_any_element()
        }));
        self
    }

    /// Sets the default action and validation. Returning false keeps the dialog open.
    pub fn confirm(
        mut self,
        label: impl Into<SharedString>,
        action: impl Fn(&mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.confirm = Some((label.into(), Rc::new(action)));
        self
    }

    /// Sets a non-validating confirmation action that always closes the dialog after running.
    pub fn on_confirm(
        self,
        label: impl Into<SharedString>,
        action: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.confirm(label, move |window, cx| {
            action(window, cx);
            true
        })
    }

    /// Applies a Material 3 semantic tone to the default confirmation button.
    pub fn confirm_tone(mut self, tone: SemanticTone) -> Self {
        self.confirm_tone = tone;
        self
    }

    /// Disables the default confirmation button and suppresses Enter confirmation.
    pub fn confirm_disabled(mut self, disabled: bool) -> Self {
        self.confirm_disabled = disabled;
        self
    }

    /// Shows an inline busy spinner on the default confirmation button and suppresses Enter confirmation.
    pub fn confirm_loading(mut self, loading: bool) -> Self {
        self.confirm_loading = loading;
        self
    }

    /// Builds a custom confirmation control. Wire its activation to the supplied action.
    pub fn confirm_content<E: IntoElement>(
        mut self,
        content: impl FnOnce(DialogAction) -> E + 'static,
    ) -> Self {
        self.confirm_content = Some(Box::new(move |confirm| content(confirm).into_any_element()));
        self
    }

    /// Builds a custom cancellation control. Wire its activation to the supplied action.
    pub fn dismiss_content<E: IntoElement>(
        mut self,
        content: impl FnOnce(DialogAction) -> E + 'static,
    ) -> Self {
        self.dismiss_content = Some(Box::new(move |dismiss| content(dismiss).into_any_element()));
        self
    }

    /// Overrides the label of the default cancellation button (defaults to
    /// [`gpui_material_theme::MaterialStrings::cancel`], `"Cancel"`).
    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel = Some(label.into());
        self
    }

    /// Sets whether to include a default cancellation control when there is no custom one.
    pub fn show_cancel(mut self, show: bool) -> Self {
        self.show_cancel = show;
        self
    }

    /// Set a higher layer for a nested modal. Only the top modal can receive input.
    pub fn layer(mut self, layer: usize) -> Self {
        self.layer = layer;
        self
    }

    /// Configures Escape and outside-click dismissal behavior.
    pub fn properties(mut self, properties: DialogProperties) -> Self {
        self.properties = properties;
        self
    }

    /// Handles Escape, outside-click, and system window-close requests. Return false to veto.
    pub fn on_dismiss_request(
        mut self,
        request: impl Fn(DialogCloseReason, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.dismiss_request = Some(Rc::new(request));
        self
    }

    /// Reports one accepted user close after the visibility state has changed.
    pub fn on_closed(
        mut self,
        close: impl Fn(DialogCloseReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_closed = Some(Rc::new(close));
        self
    }

    /// Overrides the corner shape of the dialog surface.
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Overrides the dialog's container, icon, title, and text colors.
    pub fn colors(mut self, colors: DialogColors) -> Self {
        self.colors = colors;
        self
    }

    /// Selects a tonal Material surface when the container color uses the theme.
    pub fn tonal_elevation(mut self, elevation: Elevation) -> Self {
        self.tonal_elevation = Some(elevation);
        self
    }

    /// Overrides the shadow elevation cast by the dialog surface.
    pub fn shadow_elevation(mut self, elevation: Elevation) -> Self {
        self.shadow_elevation = Some(elevation);
        self
    }

    /// Overrides the backdrop scrim color.
    pub fn scrim_color(mut self, color: Hsla) -> Self {
        self.scrim_color = Some(color);
        self
    }

    /// Sets in-window backdrop opacity, clamped to 0–1. NaN uses the default.
    pub fn scrim_opacity(mut self, opacity: f32) -> Self {
        self.scrim_opacity = if opacity.is_nan() {
            0.32
        } else {
            opacity.clamp(0., 1.)
        };
        self
    }

    /// Sets the maximum width of the centered dialog in logical pixels.
    pub fn width(mut self, width: Pixels) -> Self {
        if f32::from(width).is_finite() {
            self.width = width.max(px(0.));
        }
        self
    }
}

use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, AnyWindowHandle, App, AppContext, Context, ElementId, Entity, Hsla,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Role, SharedString,
    StatefulInteractiveElement, Styled, Subscription, WeakFocusHandle, Window, WindowKind,
    WindowOptions, div, px,
};
use gpui_material_foundation::{
    MaterialRoot, Scrim, current_material_theme, set_window_material_theme,
};
use gpui_material_theme::{CornerShape, Elevation};

mod render;
mod state;

pub use state::DialogState;
pub(super) use state::ModalSession;
use state::{NativeDialog, NativeDialogView, restore_parent};

pub(super) fn exiting_panel<E: StatefulInteractiveElement + ParentElement>(
    panel: E,
    id: &ElementId,
    state: &Entity<DialogState>,
    window: &mut Window,
    cx: &mut App,
) -> E {
    let focus = state.read(cx).focus.clone();
    // The body can contain caller-owned controls. Keep their layout through
    // exit, but send focus back outside the panel and cover its hit targets.
    let guard = window.use_keyed_state((id.clone(), "exiting-focus"), cx, |_, _| {
        let previous = Rc::new(RefCell::new(None::<WeakFocusHandle>));
        (previous, None::<Subscription>)
    });
    let target = window
        .focused(cx)
        .filter(|handle| !focus.contains(handle, window));
    guard.update(cx, |(previous, guard), cx| {
        if let Some(target) = target {
            *previous.borrow_mut() = Some(target.downgrade());
        }
        if guard.is_none() {
            let previous = previous.clone();
            let state = state.downgrade();
            *guard = Some(window.on_focus_in(&focus, cx, move |window, cx| {
                // A new session can focus the container before the exit tree is redrawn.
                if state
                    .upgrade()
                    .is_some_and(|state| state.read(cx).is_open())
                {
                    return;
                }
                if let Some(handle) = previous
                    .borrow()
                    .as_ref()
                    .and_then(WeakFocusHandle::upgrade)
                {
                    handle.focus(window, cx);
                } else {
                    window.blur(cx);
                }
            }));
        }
    });
    panel
        .role(Role::Group)
        .track_focus(&focus)
        .capture_key_down(|_, _, cx| cx.stop_propagation())
        .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
        .child(div().absolute().inset_0().occlude())
}

/// Supplies the hit target shared by modal sheet and dialog scrims.
pub(super) fn modal_backdrop(
    scrim: Scrim,
    dismiss: impl Fn(&mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .absolute()
        .inset_0()
        .occlude()
        .on_any_mouse_down(move |event, window, cx| {
            cx.stop_propagation();
            if event.button == MouseButton::Left {
                dismiss(window, cx);
            }
        })
        .on_scroll(|_, _, cx| cx.stop_propagation())
        .child(scrim)
}

/// An accepted transition from an open to a closed Material dialog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DialogCloseReason {
    Confirm,
    Cancel,
    Escape,
    OutsideClick,
    WindowClose,
    Programmatic,
}

/// Window-relative presentation for a Material dialog.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DialogVariant {
    #[default]
    Basic,
    /// Uses a full-screen task surface in compact windows and a basic dialog
    /// at wider breakpoints, as required by Material dialog guidance.
    FullScreen,
}

/// User dismissal routes for a Material dialog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DialogProperties {
    pub dismiss_on_escape: bool,
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
    pub container: Option<Hsla>,
    pub icon: Option<Hsla>,
    pub title: Option<Hsla>,
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
    confirm_content: Option<ActionSlot>,
    dismiss_content: Option<ActionSlot>,
    cancel: SharedString,
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
    on_close: Option<ModalClosed>,
}

impl Dialog {
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
            confirm_content: None,
            dismiss_content: None,
            cancel: "Cancel".into(),
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
            on_close: None,
        }
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

    pub fn variant(mut self, variant: DialogVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    /// Replaces the visual heading, keeping the constructor's accessible name.
    pub fn title_content(mut self, title: impl IntoElement) -> Self {
        self.title_content = Some(title.into_any_element());
        self
    }

    pub fn show_title(mut self, show: bool) -> Self {
        self.show_title = show;
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

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

    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel = label.into();
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
    pub fn on_close(
        mut self,
        close: impl Fn(DialogCloseReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(Rc::new(close));
        self
    }

    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    pub fn colors(mut self, colors: DialogColors) -> Self {
        self.colors = colors;
        self
    }

    /// Selects a tonal Material surface when the container color uses the theme.
    pub fn tonal_elevation(mut self, elevation: Elevation) -> Self {
        self.tonal_elevation = Some(elevation);
        self
    }

    pub fn shadow_elevation(mut self, elevation: Elevation) -> Self {
        self.shadow_elevation = Some(elevation);
        self
    }

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

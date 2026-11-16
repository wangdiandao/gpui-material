use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, AnyWindowHandle, App, AppContext, Context, ElementId, Entity, FocusHandle, Hsla,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Render, RenderOnce, Role,
    ScrollHandle, SharedString, StatefulInteractiveElement, Styled, Subscription, WeakEntity,
    WeakFocusHandle, Window, WindowKind, WindowOptions, anchored, div, point,
    prelude::FluentBuilder, px,
};
use gpui_base::{DialogChangeReason, TestSupportExt, active_focus_trap};
use gpui_material_foundation::{
    MaterialRoot, Scrim, Surface, color, isolated_scroll, set_window_material_theme,
    window_material_theme,
};
use gpui_material_theme::{ColorRole, CornerRole, CornerShape, Direction, Elevation, TypeRole};

use crate::{Button, ButtonVariant, IconButton};

pub(super) fn exiting_panel<E: StatefulInteractiveElement + ParentElement>(
    panel: E,
    id: &ElementId,
    focus: &FocusHandle,
    window: &mut Window,
    cx: &mut App,
) -> E {
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
            *guard = Some(window.on_focus_in(focus, cx, move |window, cx| {
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
        .track_focus(focus)
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

struct NativeDialog {
    parent: AnyWindowHandle,
    child: AnyWindowHandle,
    _closed: Subscription,
}

/// Retains a dialog's visibility and focus scope across renders.
pub struct DialogState {
    pub(super) focus: FocusHandle,
    previous: Option<FocusHandle>,
    cancel_focus: FocusHandle,
    confirm_focus: FocusHandle,
    open: bool,
    generation: u64,
    last_close_reason: Option<DialogCloseReason>,
    native: Option<NativeDialog>,
    pub(super) scroll: ScrollHandle,
}

impl DialogState {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            previous: None,
            cancel_focus: cx.focus_handle(),
            confirm_focus: cx.focus_handle(),
            open: false,
            generation: 0,
            last_close_reason: None,
            native: None,
            scroll: ScrollHandle::new(),
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Returns the most recent accepted close, including programmatic closes.
    pub fn last_close_reason(&self) -> Option<DialogCloseReason> {
        self.last_close_reason
    }

    fn begin(
        &mut self,
        previous: Option<FocusHandle>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.previous = previous;
        self.open = true;
        self.generation = self.generation.wrapping_add(1);
        self.focus.focus(window, cx);
        cx.notify();
    }

    /// Focuses the modal container. A caller can then focus a child editor.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            self.begin(window.focused(cx), window, cx);
        }
    }

    /// Closes the dialog and restores its previous focus if it still owns focus.
    /// A programmatic close updates the reason without firing the user close callback.
    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.finish(DialogCloseReason::Programmatic, window, cx);
    }

    fn finish(&mut self, reason: DialogCloseReason, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        self.open = false;
        self.last_close_reason = Some(reason);
        let previous = self.previous.take();
        if let Some(native) = self.native.take() {
            if reason != DialogCloseReason::WindowClose {
                if native.child.window_id() == window.window_handle().window_id() {
                    window.remove_window();
                } else {
                    native
                        .child
                        .update(cx, |_, window, _| window.remove_window())
                        .ok();
                }
            }
            restore_parent(native.parent, previous, cx);
        } else if self.focus.contains_focused(window, cx) || self.focus.is_focused(window) {
            if let Some(previous) = previous {
                previous.focus(window, cx);
            } else {
                window.blur(cx);
            }
        }
        cx.notify();
    }
}

fn restore_parent(parent: AnyWindowHandle, previous: Option<FocusHandle>, cx: &mut App) {
    if let Some(previous) = previous {
        cx.defer(move |cx| {
            parent
                .update(cx, |_, window, cx| {
                    if window.focused(cx).is_none() {
                        previous.focus(window, cx);
                    }
                })
                .ok();
        });
    }
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

#[derive(Clone)]
pub(super) struct ModalSession {
    state: WeakEntity<DialogState>,
    generation: u64,
    dismiss_request: Option<ModalDismissRequest>,
    on_close: Option<ModalClosed>,
}

impl ModalSession {
    pub(super) fn new(
        state: &Entity<DialogState>,
        dismiss_request: Option<ModalDismissRequest>,
        on_close: Option<ModalClosed>,
        cx: &App,
    ) -> Self {
        Self {
            state: state.downgrade(),
            generation: state.read(cx).generation,
            dismiss_request,
            on_close,
        }
    }

    fn current(&self, window: &Window, cx: &App) -> bool {
        self.state.upgrade().is_some_and(|state| {
            let dialog = state.read(cx);
            dialog.open
                && dialog.generation == self.generation
                && dialog.focus.contains_focused(window, cx)
                && active_focus_trap(window, cx).is_none_or(|focus| focus == dialog.focus)
        })
    }

    pub(super) fn dismiss(
        &self,
        reason: DialogCloseReason,
        window: &mut Window,
        cx: &mut App,
    ) -> bool {
        self.current(window, cx)
            && self
                .dismiss_request
                .as_ref()
                .is_none_or(|request| request(reason, window, cx))
            && self.finish(reason, window, cx)
    }

    fn finish(&self, reason: DialogCloseReason, window: &mut Window, cx: &mut App) -> bool {
        let Some(state) = self.state.upgrade() else {
            return false;
        };
        let accepted = state.update(cx, |state, cx| {
            if !state.open || state.generation != self.generation {
                return false;
            }
            state.finish(reason, window, cx);
            true
        });
        if accepted && let Some(on_close) = &self.on_close {
            on_close(reason, window, cx);
        }
        accepted
    }
}

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
        let theme = window_material_theme(parent, cx).clone();
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

type BuildNativeDialog = Box<dyn Fn(&Entity<DialogState>, &mut Window, &mut App) -> Dialog>;

struct NativeDialogView {
    state: Entity<DialogState>,
    build: BuildNativeDialog,
}

impl Render for NativeDialogView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child((self.build)(&self.state, window, cx))
    }
}

impl RenderOnce for Dialog {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state.read(cx);
        let native = state.native.is_some();
        if let Some(native) = &state.native
            && native.child.window_id() != window.window_handle().window_id()
        {
            return div().into_any_element();
        }
        let open = state.open;
        let focus = state.focus.clone();
        let scroll = state.scroll.clone();
        let cancel_focus = state.cancel_focus.clone();
        let confirm_focus = state.confirm_focus.clone();
        let session = ModalSession::new(
            &self.state,
            self.dismiss_request.take(),
            self.on_close.take(),
            cx,
        );
        let presence = crate::motion::presence(&self.id, open, 200, window, cx);
        if !presence.should_render() {
            return div().into_any_element();
        }
        let progress = if native { 1. } else { presence.progress };
        let viewport = window.viewport_size();
        let full_screen = self.variant == DialogVariant::FullScreen && viewport.width < px(600.);
        let window_filling = full_screen || native;
        let margin = if window_filling { px(0.) } else { px(48.) };
        let available_width = (viewport.width - margin).max(px(0.));
        let width = if full_screen {
            available_width
        } else {
            self.width.min(available_width)
        };
        let max_height = (viewport.height - margin).max(px(0.));
        let motion_offset = px((1. - progress) * if full_screen { 32. } else { 8. });
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let colors = &theme.colors;
        let icon_color = self
            .colors
            .icon
            .unwrap_or_else(|| color(colors.color(ColorRole::Secondary)));
        let title_color = self
            .colors
            .title
            .unwrap_or_else(|| color(colors.color(ColorRole::OnSurface)));
        let text_color = self
            .colors
            .text
            .unwrap_or_else(|| color(colors.color(ColorRole::OnSurfaceVariant)));

        let confirm_action = DialogAction {
            session: session.clone(),
            reason: DialogCloseReason::Confirm,
            validate: self
                .confirm
                .as_ref()
                .map(|(_, validator)| validator.clone()),
        };
        let dismiss_action = DialogAction {
            session: session.clone(),
            reason: DialogCloseReason::Cancel,
            validate: None,
        };
        let keyboard_confirmation =
            self.confirm.is_some() && self.confirm_content.is_none() && !self.unstructured;
        let body = self
            .body
            .map(|body| body(confirm_action.clone(), dismiss_action.clone()));
        let role = if self.tonal_elevation.is_some() {
            ColorRole::Surface
        } else {
            ColorRole::SurfaceContainerHigh
        };
        let mut surface = Surface::new(role)
            .flex()
            .flex_col()
            .max_h(max_height)
            .content_color(text_color);
        if let Some(container) = self.colors.container {
            surface = surface.container_color(container);
        } else if self.unstructured && self.tonal_elevation.is_none() {
            surface = surface.container_color(gpui::transparent_black());
        }
        if let Some(tonal) = self.tonal_elevation {
            surface = surface.tonal_elevation(tonal);
        }
        if let Some(shape) = self.shape.or_else(|| {
            (!full_screen && !self.unstructured && !native).then(|| CornerRole::ExtraLarge.shape())
        }) {
            surface = surface.shape(shape);
        }
        let shadow =
            self.shadow_elevation
                .unwrap_or(if full_screen || self.unstructured || native {
                    Elevation::Level0
                } else {
                    Elevation::Level3
                });
        surface = surface.shadow_elevation(shadow);
        if self.unstructured {
            surface = surface
                .min_w(px(280.).min(width))
                .max_w(width)
                .children(body);
        } else {
            let icon = self
                .icon
                .map(|icon| div().text_color(icon_color).child(icon));
            let has_icon = icon.is_some();
            let heading = self.show_title.then(|| {
                let heading = self
                    .title_content
                    .unwrap_or_else(|| self.title.clone().into_any_element());
                super::text_style(
                    div()
                        .min_w_0()
                        .text_color(title_color)
                        .when(has_icon && !full_screen, Styled::text_center)
                        .child(heading),
                    if full_screen {
                        TypeRole::TitleLarge
                    } else {
                        TypeRole::HeadlineSmall
                    },
                    1.,
                    window,
                    cx,
                )
            });
            let supporting = self.description.map(|description| {
                super::text_style(
                    div().id("description").role(Role::Label).child(description),
                    TypeRole::BodyMedium,
                    1.,
                    window,
                    cx,
                )
            });
            let confirm_for_button = confirm_action.clone();
            let confirm = match self.confirm_content {
                Some(content) => Some(content(confirm_for_button)),
                None => self.confirm.map(|(label, _)| {
                    Button::new("confirm", label)
                        .track_focus(&confirm_focus)
                        .variant(ButtonVariant::Text)
                        .on_activate(move |window, cx| confirm_for_button.activate(window, cx))
                        .into_any_element()
                }),
            };
            let cancel = match self.dismiss_content {
                Some(content) => Some(content(dismiss_action)),
                None if self.show_cancel && full_screen => Some(
                    IconButton::new(
                        "cancel",
                        self.cancel,
                        crate::Icon::new(crate::IconName::Close),
                    )
                    .track_focus(&cancel_focus)
                    .on_activate(move |window, cx| dismiss_action.activate(window, cx))
                    .into_any_element(),
                ),
                None if self.show_cancel => Some(
                    Button::new("cancel", self.cancel)
                        .track_focus(&cancel_focus)
                        .variant(ButtonVariant::Text)
                        .on_activate(move |window, cx| dismiss_action.activate(window, cx))
                        .into_any_element(),
                ),
                None => None,
            };
            let content = isolated_scroll("dialog-content", &scroll)
                .test_support()
                .min_h_0()
                .flex()
                .flex_col()
                .gap(px(16.))
                .children(supporting)
                .children(body);
            surface = surface.w(width);
            if full_screen {
                surface = surface
                    .h(viewport.height)
                    .child(
                        div()
                            .id("dialog-app-bar")
                            .test_support()
                            .flex()
                            .flex_none()
                            .items_center()
                            .h(px(56.))
                            .px(px(4.))
                            .gap(px(8.))
                            .when(rtl, Styled::flex_row_reverse)
                            .children(cancel)
                            .children(icon)
                            .child(div().flex_1().min_w_0().children(heading))
                            .children(confirm),
                    )
                    .child(content.flex_1().p(px(24.)));
            } else {
                surface = surface
                    .p(px(24.))
                    .children(icon.map(|icon| {
                        div()
                            .flex_none()
                            .flex()
                            .justify_center()
                            .mb(px(16.))
                            .child(icon)
                    }))
                    .children(heading.map(|heading| div().flex_none().mb(px(16.)).child(heading)))
                    .child(content)
                    .children((cancel.is_some() || confirm.is_some()).then(|| {
                        div()
                            .flex()
                            .flex_none()
                            .mt(px(24.))
                            .when(rtl, Styled::flex_row_reverse)
                            .justify_end()
                            .flex_wrap()
                            .gap(px(8.))
                            .children(cancel)
                            .children(confirm)
                    }));
            }
        }

        let panel = div()
            .id(self.id.clone())
            .test_support()
            .relative()
            .max_w(available_width)
            .max_h(max_height)
            .opacity(progress)
            .mt(motion_offset)
            .when(open, |panel| panel.occlude())
            .child(surface);
        let backdrop = Scrim::new((self.id.clone(), "scrim"))
            .opacity(if native {
                0.
            } else {
                self.scrim_opacity * progress
            })
            .when_some(self.scrim_color, Scrim::color);
        if !open {
            return gpui::deferred(
                anchored().position(point(px(0.), px(0.))).child(
                    div()
                        .id((self.id.clone(), "dialog-exit"))
                        .absolute()
                        .top_0()
                        .left_0()
                        .w(viewport.width)
                        .h(viewport.height)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(backdrop)
                        .child(exiting_panel(panel, &self.id, &focus, window, cx)),
                ),
            )
            .with_priority(10 + self.layer)
            .into_any_element();
        }
        if native {
            let session = session.clone();
            window.on_window_should_close(cx, move |window, cx| {
                session.dismiss(DialogCloseReason::WindowClose, window, cx)
            });
        }
        let properties = self.properties;
        let outside = session.clone();
        let keyboard_focus = focus.clone();
        gpui_base::Dialog::new(cx)
            .focus_handle(focus)
            .layer(self.layer, true)
            .accessibility_label(self.title)
            .close_on_backdrop_press(false)
            .backdrop(modal_backdrop(backdrop, move |window, cx| {
                if !native && properties.dismiss_on_outside_click {
                    outside.dismiss(DialogCloseReason::OutsideClick, window, cx);
                }
            }))
            .on_ok(move |_, window, cx| {
                if keyboard_confirmation && keyboard_focus.is_focused(window) {
                    confirm_action.activate(window, cx);
                } else {
                    // Let a focused control handle Enter through its own key listener.
                    cx.propagate();
                }
                false
            })
            .on_open_change(move |_, reason, window, cx| {
                if reason == DialogChangeReason::Cancel && properties.dismiss_on_escape {
                    session.dismiss(DialogCloseReason::Escape, window, cx);
                }
            })
            .popup(panel)
            .into_any_element()
    }
}

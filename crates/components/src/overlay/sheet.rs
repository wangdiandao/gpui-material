use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, AppContext, Context, DragMoveEvent, Edges, ElementId, Entity, Hsla,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Render, RenderOnce, Role,
    SharedString, Size, StatefulInteractiveElement, Styled, Window, anchored, div, point,
    prelude::FluentBuilder, px, radians,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{
    DragCapture, Scrim, Surface, animate, color, isolated_scroll, window_material_theme,
};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Direction, Elevation, MotionKind, MotionSpec, MotionSpeed,
    TypeRole,
};

use crate::{Button, ButtonVariant, Divider, Icon, IconButton, IconName, ScaffoldInsets, Text};

use super::bottom_sheet::{
    BottomHandle, BottomSheetLayout, BottomSheetState, BottomSheetThresholds, BottomSheetValue,
};

use super::dialog::{
    DialogCloseReason, DialogProperties, DialogState, ModalClosed, ModalDismissRequest,
    ModalSession, exiting_panel, modal_backdrop,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SheetEdge {
    #[default]
    Bottom,
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SheetMode {
    #[default]
    Standard,
    Modal,
}

/// Placement of a side sheet relative to its window edge or layout slot.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SheetPlacement {
    #[default]
    Docked,
    Detached,
}

/// Logical detached margins in pixels. The default is 16 px on each side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SheetMargins {
    pub top: Pixels,
    pub bottom: Pixels,
    pub start: Pixels,
    pub end: Pixels,
}

impl Default for SheetMargins {
    fn default() -> Self {
        Self {
            top: px(16.),
            bottom: px(16.),
            start: px(16.),
            end: px(16.),
        }
    }
}

impl SheetMargins {
    fn fit(self, viewport: Size<Pixels>, rtl: bool) -> Edges<Pixels> {
        let (start, end) =
            fit_margins(self.start, self.end, (viewport.width - px(48.)).max(px(0.)));
        let (top, bottom) = fit_margins(
            self.top,
            self.bottom,
            (viewport.height - px(48.)).max(px(0.)),
        );
        let (left, right) = if rtl { (end, start) } else { (start, end) };
        Edges {
            top,
            bottom,
            left,
            right,
        }
    }
}

fn fit_margins(a: Pixels, b: Pixels, available: Pixels) -> (Pixels, Pixels) {
    let sanitize = |margin: Pixels| {
        if f32::from(margin).is_finite() {
            margin.clamp(px(0.), available)
        } else {
            px(0.)
        }
    };
    let (a, b) = (sanitize(a), sanitize(b));
    if a + b > available {
        let factor = available / (a + b);
        (a * factor, b * factor)
    } else {
        (a, b)
    }
}

/// Optional local colors for the sheet and its adornments.
#[derive(Clone, Copy, Debug, Default)]
pub struct SheetColors {
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub header: Option<Hsla>,
    pub divider: Option<Hsla>,
    pub scrim: Option<Hsla>,
}

/// Activation for a close control supplied by the caller.
#[derive(Clone)]
pub struct SheetClose(CloseAction);

#[derive(Clone)]
enum CloseAction {
    Modal(ModalSession),
    Standard(Option<ModalDismissRequest>, Option<ModalClosed>),
}

impl SheetClose {
    /// Requests dismissal from the sheet's Close action.
    pub fn activate(&self, window: &mut Window, cx: &mut App) {
        self.request(window, cx);
    }

    pub(super) fn request(&self, window: &mut Window, cx: &mut App) -> bool {
        let reason = DialogCloseReason::Cancel;
        match &self.0 {
            CloseAction::Modal(session) => session.dismiss(reason, window, cx),
            CloseAction::Standard(request, Some(closed))
                if request
                    .as_ref()
                    .is_none_or(|request| request(reason, window, cx)) =>
            {
                closed(reason, window, cx);
                true
            }
            CloseAction::Standard(_, _) => false,
        }
    }
}

type CloseSlot = Box<dyn FnOnce(SheetClose) -> AnyElement>;

type Change = Rc<dyn Fn(bool, &mut Window, &mut App)>;

struct SheetDrag;

impl Render for SheetDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(Default)]
struct Gesture {
    origin: Option<Pixels>,
    capture: Option<DragCapture>,
    offset: Pixels,
}

fn drag_handle(
    id: &ElementId,
    gesture: Rc<RefCell<Gesture>>,
    travel: Pixels,
    expanded: bool,
    change: Change,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let tint = color(
        window_material_theme(window, cx)
            .colors
            .color(ColorRole::OnSurfaceVariant),
    );
    let focus = window
        .use_keyed_state((id.clone(), "drag-focus"), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let down = gesture.clone();
    let start = gesture.clone();
    let release = gesture.clone();
    let outside = gesture.clone();
    let moving = gesture.clone();
    let cancel = gesture;
    let drag_focus = focus.clone();
    let release_change = change.clone();
    let settle = move |gesture: &RefCell<Gesture>,
                       y: Pixels,
                       change: &Change,
                       window: &mut Window,
                       cx: &mut App| {
        let active = std::mem::take(&mut *gesture.borrow_mut());
        window.refresh();
        if active.capture.is_none() {
            return;
        }
        if let Some(origin) = active.origin {
            let distance = y - origin;
            if distance.abs() >= px(32.) && expanded == (distance > px(0.)) {
                change(!expanded, window, cx);
            }
        }
    };
    div()
        .id("drag-handle")
        .test_support()
        .track_focus(&focus)
        .w_full()
        .h(px(48.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .cursor_ns_resize()
        .child(drag_handle_mark(tint))
        .on_mouse_down(MouseButton::Left, move |event, _, _| {
            down.borrow_mut().origin = Some(event.position.y);
        })
        .on_drag(SheetDrag, move |_, _, window, cx| {
            drag_focus.focus(window, cx);
            let weak = Rc::downgrade(&start);
            start.borrow_mut().capture = Some(DragCapture::watch(window, cx, move |_, _| {
                if let Some(state) = weak.upgrade() {
                    *state.borrow_mut() = Gesture::default();
                }
            }));
            cx.stop_propagation();
            cx.new(|_| SheetDrag)
        })
        .on_drag_move(move |event: &DragMoveEvent<SheetDrag>, window, _| {
            let mut state = moving.borrow_mut();
            if state.capture.is_some()
                && let Some(origin) = state.origin
            {
                let offset = event.event.position.y - origin;
                state.offset = if expanded {
                    offset.clamp(px(0.), travel)
                } else {
                    offset.clamp(-travel, px(0.))
                };
                window.refresh();
            }
        })
        .on_mouse_up(MouseButton::Left, move |event, window, cx| {
            settle(&release, event.position.y, &release_change, window, cx);
        })
        .on_mouse_up_out(MouseButton::Left, move |event, window, cx| {
            settle(&outside, event.position.y, &change, window, cx);
        })
        .capture_key_down(move |event, window, cx| {
            if event.keystroke.key == "escape"
                && let Some(capture) = std::mem::take(&mut *cancel.borrow_mut()).capture
            {
                capture.cancel(window, cx);
                window.refresh();
                cx.stop_propagation();
            }
        })
        .into_any_element()
}

fn drag_handle_mark(tint: gpui::Hsla) -> gpui::Div {
    div().w(px(32.)).h(px(4.)).rounded_full().bg(tint)
}

enum SheetBinding {
    Immediate(Entity<DialogState>),
    Retained(Entity<BottomSheetState>),
}

/// A docked or modal Material sheet. Standard sheets take part in parent layout.
/// The caller owns the bottom sheet's expanded value and handles changes.
#[derive(IntoElement)]
pub struct Sheet {
    id: ElementId,
    state: SheetBinding,
    title: SharedString,
    content: AnyElement,
    edge: SheetEdge,
    mode: SheetMode,
    extent: Pixels,
    expansion: Option<(bool, Change)>,
    placement: SheetPlacement,
    margins: SheetMargins,
    header: Option<AnyElement>,
    back: Option<AnyElement>,
    close_content: Option<CloseSlot>,
    close_label: SharedString,
    show_close: bool,
    footer: Option<AnyElement>,
    footer_divider: bool,
    edge_divider: bool,
    properties: DialogProperties,
    on_dismiss_request: Option<ModalDismissRequest>,
    on_close: Option<ModalClosed>,
    layer: usize,
    shape: Option<CornerShape>,
    container_role: Option<ColorRole>,
    content_role: Option<ColorRole>,
    colors: SheetColors,
    tonal_elevation: Elevation,
    shadow_elevation: Option<Elevation>,
    scrim_opacity: f32,
    presence: Option<gpui_base::motion::PresenceSample>,
    layout_size: Option<Size<Pixels>>,
    peek_height: Option<Pixels>,
    max_width: Pixels,
    drag_enabled: bool,
    show_handle: bool,
    handle_content: Option<AnyElement>,
    handle_label: SharedString,
    thresholds: BottomSheetThresholds,
    content_insets: ScaffoldInsets,
}

impl Sheet {
    pub fn new(
        id: impl Into<ElementId>,
        state: &Entity<DialogState>,
        title: impl Into<SharedString>,
        content: impl IntoElement,
    ) -> Self {
        Self::with_state(
            id.into(),
            SheetBinding::Immediate(state.clone()),
            title.into(),
            content.into_any_element(),
        )
    }

    /// Creates a bottom sheet with retained stops and state-owned visibility.
    pub fn bottom(
        id: impl Into<ElementId>,
        state: &Entity<BottomSheetState>,
        title: impl Into<SharedString>,
        content: impl IntoElement,
    ) -> Self {
        Self::with_state(
            id.into(),
            SheetBinding::Retained(state.clone()),
            title.into(),
            content.into_any_element(),
        )
    }

    fn with_state(
        id: ElementId,
        state: SheetBinding,
        title: SharedString,
        content: AnyElement,
    ) -> Self {
        Self {
            id,
            state,
            title,
            content,
            edge: SheetEdge::Bottom,
            mode: SheetMode::Standard,
            extent: px(320.),
            expansion: None,
            placement: SheetPlacement::Docked,
            margins: SheetMargins::default(),
            header: None,
            back: None,
            close_content: None,
            close_label: "Close".into(),
            show_close: true,
            footer: None,
            footer_divider: false,
            edge_divider: false,
            properties: DialogProperties::default(),
            on_dismiss_request: None,
            on_close: None,
            layer: 0,
            shape: None,
            container_role: None,
            content_role: None,
            colors: SheetColors::default(),
            tonal_elevation: Elevation::Level0,
            shadow_elevation: None,
            scrim_opacity: 0.32,
            presence: None,
            layout_size: None,
            peek_height: None,
            max_width: px(640.),
            drag_enabled: true,
            show_handle: true,
            handle_content: None,
            handle_label: "Resize bottom sheet".into(),
            thresholds: BottomSheetThresholds::default(),
            content_insets: ScaffoldInsets::default(),
        }
    }

    pub(crate) fn in_scaffold(mut self, size: Size<Pixels>) -> Self {
        self.edge = SheetEdge::Bottom;
        self.mode = SheetMode::Standard;
        self.layout_size = Some(size);
        self
    }

    /// Sets the retained bottom sheet's partial height. It fits the expanded extent.
    pub fn peek_height(mut self, height: Pixels) -> Self {
        if f32::from(height).is_finite() {
            self.peek_height = Some(height.max(px(48.)));
        }
        self
    }

    /// Sets the bottom sheet's maximum width in logical pixels.
    pub fn max_width(mut self, width: Pixels) -> Self {
        if f32::from(width).is_finite() && width > px(0.) {
            self.max_width = width;
        }
        self
    }

    pub fn drag_enabled(mut self, enabled: bool) -> Self {
        self.drag_enabled = enabled;
        self
    }

    pub fn drag_thresholds(mut self, thresholds: BottomSheetThresholds) -> Self {
        self.thresholds = thresholds.normalized();
        self
    }

    /// Replaces the grip decoration of the retained sheet's resize control.
    pub fn handle_content(mut self, content: impl IntoElement) -> Self {
        self.handle_content = Some(content.into_any_element());
        self
    }

    /// Names the resize action for keyboard and assistive technology.
    pub fn handle_label(mut self, label: impl Into<SharedString>) -> Self {
        self.handle_label = label.into();
        self
    }

    /// Uses a text height action in place of the grip when multiple stops are available.
    pub fn show_handle(mut self, show: bool) -> Self {
        self.show_handle = show;
        self
    }

    /// Sets logical padding within the bottom sheet's scrolling content.
    pub fn content_insets(mut self, insets: ScaffoldInsets) -> Self {
        self.content_insets = insets.normalized();
        self
    }

    pub fn edge(mut self, edge: SheetEdge) -> Self {
        if matches!(self.state, SheetBinding::Immediate(_)) {
            self.edge = edge;
        }
        self
    }

    pub fn mode(mut self, mode: SheetMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets expanded height for a bottom sheet, or width for a side sheet.
    pub fn extent(mut self, extent: Pixels) -> Self {
        self.extent = extent.max(px(48.));
        self
    }

    /// Enables a keyboard-operable expand and collapse action on a bottom sheet.
    pub fn expandable(
        mut self,
        expanded: bool,
        on_change: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.expansion = Some((expanded, Rc::new(on_change)));
        self
    }

    pub fn placement(mut self, placement: SheetPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn margins(mut self, margins: SheetMargins) -> Self {
        self.margins = margins;
        self
    }

    /// Replaces the visible heading while preserving the accessible sheet name.
    pub fn header_content(mut self, heading: impl IntoElement) -> Self {
        self.header = Some(heading.into_any_element());
        self
    }

    /// Adds a named back action. Navigation ownership remains with the caller.
    pub fn back(
        mut self,
        label: impl Into<SharedString>,
        on_back: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.back = Some(
            IconButton::new(
                "back",
                label,
                Icon::new(IconName::ArrowForward)
                    .rotate(radians(std::f32::consts::PI))
                    .mirror_in_rtl(true),
            )
            .on_activate(on_back)
            .into_any_element(),
        );
        self
    }

    pub fn back_content(mut self, back: impl IntoElement) -> Self {
        self.back = Some(back.into_any_element());
        self
    }

    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = label.into();
        self
    }

    pub fn show_close(mut self, show: bool) -> Self {
        self.show_close = show;
        self
    }

    /// Builds a custom close control. Connect its activation to the supplied action.
    pub fn close_content<E: IntoElement>(
        mut self,
        close: impl FnOnce(SheetClose) -> E + 'static,
    ) -> Self {
        self.close_content = Some(Box::new(move |action| close(action).into_any_element()));
        self
    }

    pub fn footer(mut self, actions: impl IntoElement) -> Self {
        self.footer = Some(actions.into_any_element());
        self
    }

    pub fn footer_divider(mut self, visible: bool) -> Self {
        self.footer_divider = visible;
        self
    }

    /// Adds a divider between a docked Standard side sheet and its adjacent content.
    pub fn edge_divider(mut self, visible: bool) -> Self {
        self.edge_divider = visible;
        self
    }

    pub fn properties(mut self, properties: DialogProperties) -> Self {
        self.properties = properties;
        self
    }

    /// Return false to veto a user-initiated close.
    pub fn on_dismiss_request(
        mut self,
        request: impl Fn(DialogCloseReason, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.on_dismiss_request = Some(Rc::new(request));
        self
    }

    /// Receives accepted modal closes or requests to remove a Standard sheet.
    pub fn on_close(
        mut self,
        close: impl Fn(DialogCloseReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(Rc::new(close));
        self
    }

    /// Sets the stacking layer of a nested modal sheet.
    pub fn layer(mut self, layer: usize) -> Self {
        self.layer = layer;
        self
    }

    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    pub fn container_role(mut self, role: ColorRole) -> Self {
        self.container_role = Some(role);
        self
    }

    pub fn content_role(mut self, role: ColorRole) -> Self {
        self.content_role = Some(role);
        self
    }

    pub fn colors(mut self, colors: SheetColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn tonal_elevation(mut self, elevation: Elevation) -> Self {
        self.tonal_elevation = elevation;
        self
    }

    pub fn shadow_elevation(mut self, elevation: Elevation) -> Self {
        self.shadow_elevation = Some(elevation);
        self
    }

    /// Reuses the enclosing component's sampled presence.
    pub(crate) fn sampled_presence(mut self, presence: gpui_base::motion::PresenceSample) -> Self {
        self.presence = Some(presence);
        self
    }

    pub fn scrim_opacity(mut self, opacity: f32) -> Self {
        self.scrim_opacity = if opacity.is_nan() {
            0.32
        } else {
            opacity.clamp(0., 1.)
        };
        self
    }
}

impl RenderOnce for Sheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let modal = self.mode == SheetMode::Modal;
        let viewport = window.viewport_size();
        let available = self.layout_size.unwrap_or(viewport);
        let bottom = self.edge == SheetEdge::Bottom;
        let bottom_width = available.width.min(self.max_width);
        let bottom_left = (viewport.width - bottom_width) / 2.;
        let (dialog, retained) = match &self.state {
            SheetBinding::Immediate(dialog) => (dialog.clone(), None),
            SheetBinding::Retained(state) => (state.read(cx).dialog.clone(), Some(state.clone())),
        };
        let bottom_state = retained.as_ref().filter(|_| bottom);
        let mut on_dismiss_request = self.on_dismiss_request;
        let mut on_close = self.on_close;
        let retained_layout = bottom_state.map(|state| {
            let expanded = self.extent.min(available.height);
            let partial = self
                .peek_height
                .unwrap_or_else(|| {
                    if modal {
                        available.height / 2.
                    } else {
                        px(56.)
                    }
                })
                .min(expanded);
            let layout = BottomSheetLayout {
                available,
                expanded,
                partial,
                mode: self.mode,
                drag_enabled: self.drag_enabled && self.show_handle,
            };
            let height = state.update(cx, |state, cx| {
                state.configure(layout, window, cx);
                state.visible_height()
            });
            let state = state.downgrade();
            let request = on_dismiss_request.take();
            on_dismiss_request = Some(Rc::new(move |reason, window, cx| {
                request
                    .as_ref()
                    .is_none_or(|request| request(reason, window, cx))
                    && state
                        .update(cx, |state, cx| state.accept_user_close(window, cx))
                        .unwrap_or(false)
            }));
            if !modal && on_close.is_none() {
                on_close = Some(Rc::new(|_, _, _| {}));
            }
            (height, partial.max(px(1.)))
        });
        let open = !modal || dialog.read(cx).is_open();
        let presence = (modal && retained_layout.is_none()).then(|| {
            self.presence
                .unwrap_or_else(|| crate::motion::presence(&self.id, open, 240, window, cx))
        });
        let hidden = bottom_state.is_some_and(|state| {
            let state = state.read(cx);
            state.target_value() == BottomSheetValue::Hidden && state.visible_height() <= px(0.)
        });
        if hidden || presence.is_some_and(|presence| !presence.should_render()) {
            return div().into_any_element();
        }
        let visibility = if let Some((height, peek)) = retained_layout {
            (height / peek).clamp(0., 1.)
        } else {
            presence.map_or(1., |presence| presence.progress)
        };
        let theme = window_material_theme(window, cx).clone();
        let left = match self.edge {
            SheetEdge::Start => theme.layout.direction == Direction::Ltr,
            SheetEdge::End => theme.layout.direction == Direction::Rtl,
            SheetEdge::Bottom => false,
        };
        let detached = !bottom && self.placement == SheetPlacement::Detached;
        let rtl = theme.layout.direction == Direction::Rtl;
        let margins = if detached {
            self.margins.fit(viewport, rtl)
        } else {
            Edges::default()
        };
        let available_width = (viewport.width - margins.left - margins.right).max(px(0.));
        let target = if bottom {
            self.extent.min(available.height)
        } else {
            self.extent.min(px(400.)).min(available_width)
        };
        let extent = if !bottom && !modal {
            px(animate(
                (self.id.clone(), "side-sheet-width"),
                f32::from(target),
                theme.motion.spec(
                    MotionKind::Spatial,
                    MotionSpeed::Default,
                    cx.reduce_motion(),
                ),
                window,
                cx,
            ))
            .clamp(px(0.), available_width)
        } else {
            target
        };

        let expanded = !bottom
            || self
                .expansion
                .as_ref()
                .is_none_or(|(expanded, _)| *expanded);
        let focus = dialog.read(cx).focus.clone();
        let scroll = dialog.read(cx).scroll.clone();
        let session = modal
            .then(|| ModalSession::new(&dialog, on_dismiss_request.clone(), on_close.clone(), cx));
        let gesture = window
            .use_keyed_state((self.id.clone(), "drag"), cx, |_, _| {
                Rc::new(RefCell::new(Gesture::default()))
            })
            .read(cx)
            .clone();
        // The 48dp handle and 48dp header actions must remain operable at rest.
        let collapsed = px(48. + 8. + 48. + 16.).min(extent);
        let travel = extent - collapsed;
        let height = if let Some((height, _)) = retained_layout {
            height
        } else if self.expansion.is_some() && bottom {
            let drag = gesture.borrow();
            let tracked = drag.capture.is_some();
            let target = if expanded { extent } else { collapsed } - drag.offset;
            let policy = if tracked {
                MotionSpec::Instant
            } else {
                theme.motion.spec(
                    MotionKind::Spatial,
                    MotionSpeed::Default,
                    cx.reduce_motion(),
                )
            };
            px(animate(
                (self.id.clone(), "sheet-height"),
                f32::from(target),
                policy,
                window,
                cx,
            ))
            .clamp(collapsed, extent)
        } else {
            extent
        };
        let offscreen = if retained_layout.is_some() {
            px(0.)
        } else {
            -(if bottom { height } else { extent }) * (1. - visibility)
        };
        let handle = self
            .expansion
            .as_ref()
            .filter(|_| bottom && retained_layout.is_none())
            .map(|(expanded, change)| {
                if open {
                    drag_handle(
                        &self.id,
                        gesture,
                        travel,
                        *expanded,
                        change.clone(),
                        window,
                        cx,
                    )
                } else {
                    let tint = color(theme.colors.color(ColorRole::OnSurfaceVariant));
                    div()
                        .w_full()
                        .h(px(48.))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(drag_handle_mark(tint))
                        .into_any_element()
                }
            });
        let show_default_close = (modal || on_close.is_some())
            && bottom_state.is_none_or(|state| {
                state
                    .read(cx)
                    .available_values()
                    .contains(&BottomSheetValue::Hidden)
            });
        let close_action = SheetClose(match session.clone() {
            Some(session) => CloseAction::Modal(session),
            None => CloseAction::Standard(on_dismiss_request, on_close),
        });
        let handle = if let Some(state) = bottom_state
            .filter(|state| self.show_handle || state.read(cx).available_values().len() > 1)
        {
            Some(
                BottomHandle {
                    id: self.id.clone(),
                    state: state.clone(),
                    close: close_action.clone(),
                    decoration: self.handle_content,
                    label: self.handle_label,
                    grip: self.show_handle,
                    thresholds: self.thresholds,
                    tint: color(theme.colors.color(ColorRole::OnSurfaceVariant)),
                    active: open,
                }
                .render(window, cx),
            )
        } else {
            handle
        };
        let show_chrome = retained_layout.is_none() || handle.is_none() || height >= px(120.);
        let body_insets = self.content_insets.resolve(theme.layout.direction);
        let close = self
            .show_close
            .then(|| match self.close_content {
                Some(content) => Some(content(close_action)),
                None if show_default_close && bottom => Some(
                    Button::new("close", self.close_label)
                        .variant(ButtonVariant::Text)
                        .on_activate(move |window, cx| close_action.activate(window, cx))
                        .into_any_element(),
                ),
                None if show_default_close => Some(
                    IconButton::new("close", self.close_label, Icon::new(IconName::Close))
                        .on_activate(move |window, cx| close_action.activate(window, cx))
                        .into_any_element(),
                ),
                None => None,
            })
            .flatten();
        let container_role = self.container_role.unwrap_or(if !bottom && !modal {
            ColorRole::Surface
        } else {
            ColorRole::SurfaceContainerLow
        });
        let shade = |role| color(theme.colors.color(role));
        let content = self.colors.content.or_else(|| self.content_role.map(shade));
        let side_color = shade(ColorRole::OnSurfaceVariant);
        let default_title = if bottom {
            Text::new(self.title.clone())
                .id("sheet-heading")
                .role(TypeRole::TitleMedium)
        } else {
            Text::new(self.title.clone())
                .id("sheet-heading")
                .role(TypeRole::TitleMedium)
                .wrap()
        };
        let header = div()
            .id("sheet-header")
            .test_support()
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .flex_wrap()
            .gap(if bottom { px(8.) } else { px(12.) })
            .when(!bottom && rtl, Styled::flex_row_reverse)
            .children(self.back)
            .child(
                div()
                    .flex_1()
                    .min_w(px(56.))
                    .max_w_full()
                    .when_some(
                        self.colors.header.or((!bottom).then_some(side_color)),
                        Styled::text_color,
                    )
                    .child(
                        self.header
                            .unwrap_or_else(|| default_title.into_any_element()),
                    ),
            )
            .when_some(
                self.expansion.filter(|_| bottom),
                |header, (expanded, change)| {
                    header.child(
                        Button::new("expand", if expanded { "Collapse" } else { "Expand" })
                            .variant(ButtonVariant::Text)
                            .on_activate(move |window, cx| {
                                if open {
                                    change(!expanded, window, cx);
                                }
                            }),
                    )
                },
            )
            .children(close);
        let divider = |divider: Divider| match self.colors.divider {
            Some(color) => divider.color_override(color),
            None => divider,
        };
        let footer = self.footer.map(|footer| {
            div()
                .id("sheet-footer")
                .test_support()
                .flex()
                .flex_col()
                .flex_none()
                .min_h(px(72.))
                .when(self.footer_divider, |footer| {
                    footer.child(
                        div()
                            .id("sheet-footer-divider")
                            .test_support()
                            .child(divider(Divider::horizontal())),
                    )
                })
                .child(
                    div()
                        .pt(px(16.))
                        .flex_1()
                        .flex()
                        .items_center()
                        .when(rtl, Styled::justify_end)
                        .child(footer),
                )
        });
        let shape = self.shape.unwrap_or_else(|| {
            if bottom {
                CornerRole::ExtraLargeTop.shape()
            } else if detached {
                CornerRole::Large.shape()
            } else if modal {
                if self.edge == SheetEdge::Start {
                    CornerRole::LargeEnd.shape()
                } else {
                    CornerRole::LargeStart.shape()
                }
            } else {
                CornerRole::None.shape()
            }
        });
        let elevation = self
            .shadow_elevation
            .unwrap_or(if bottom || modal || detached {
                Elevation::Level1
            } else {
                Elevation::Level0
            });
        let surface = Surface::new(container_role)
            .shape(shape)
            .shadow_elevation(elevation)
            .tonal_elevation(self.tonal_elevation)
            .when_some(content, Surface::content_color)
            .when_some(self.colors.container, Surface::container_color)
            .flex()
            .flex_col()
            .gap(px(8.))
            .px(if bottom { px(16.) } else { px(24.) })
            .pb(if bottom { px(16.) } else { px(24.) })
            .when(handle.is_none(), |surface| surface.pt(px(16.)))
            .children(handle)
            .when_else(
                bottom,
                |surface| surface.w_full().h(height).overflow_hidden(),
                |surface| surface.h_full().flex_1().min_w_0().overflow_hidden(),
            )
            .children(show_chrome.then_some(header))
            .when(show_chrome && (expanded || height > collapsed), |surface| {
                surface.child(
                    isolated_scroll("sheet-content", &scroll)
                        .test_support()
                        .flex_1()
                        .child(if self.content_insets == ScaffoldInsets::default() {
                            self.content
                        } else {
                            div()
                                .pt(body_insets.top)
                                .pr(body_insets.right)
                                .pb(body_insets.bottom)
                                .pl(body_insets.left)
                                .child(self.content)
                                .into_any_element()
                        }),
                )
            })
            .children(footer.filter(|_| retained_layout.is_none() || height >= px(192.)));
        let edge_separator = (!bottom && !modal && !detached && self.edge_divider).then(|| {
            div()
                .id("sheet-edge-divider")
                .test_support()
                .h_full()
                .flex_none()
                .child(divider(Divider::vertical()))
        });
        let panel = div()
            .id(self.id.clone())
            .test_support()
            .when(open, |panel| {
                panel
                    .role(if modal { Role::Dialog } else { Role::Group })
                    .aria_label(self.title)
                    .when(modal, |panel| {
                        panel.a11y_synthetic_children(|tree| tree.parent_node().set_modal())
                    })
                    .occlude()
            })
            .when(bottom && !modal, |panel| {
                panel.w_full().max_w(bottom_width).mx_auto()
            })
            .when(!bottom, |panel| {
                panel
                    .flex()
                    .w_full()
                    .min_w_0()
                    .when(left, Styled::flex_row_reverse)
                    .when(!modal, Styled::h_full)
            })
            .when(modal, |panel| {
                panel.absolute().when_else(
                    bottom,
                    |panel| panel.bottom(offscreen).left(bottom_left).w(bottom_width),
                    |panel| {
                        panel
                            .w(extent)
                            .top(margins.top)
                            .h((viewport.height - margins.top - margins.bottom).max(px(0.)))
                            .when_else(
                                left,
                                |panel| panel.left(margins.left + offscreen),
                                |panel| panel.right(margins.right + offscreen),
                            )
                    },
                )
            })
            .children(edge_separator)
            .child(surface);
        if !modal {
            return if bottom {
                panel.into_any_element()
            } else {
                div()
                    .w(extent + margins.left + margins.right)
                    .max_w_full()
                    .min_w_0()
                    .h_full()
                    .pt(margins.top)
                    .pb(margins.bottom)
                    .pl(margins.left)
                    .pr(margins.right)
                    .child(panel)
                    .into_any_element()
            };
        }

        let layer = 9usize.saturating_add(self.layer);
        let scrim = Scrim::new((self.id.clone(), "scrim"))
            .opacity(self.scrim_opacity * visibility)
            .when_some(self.colors.scrim, Scrim::color);
        if let Some(session) = session.filter(|_| open) {
            let properties = self.properties;
            let outside = session.clone();
            // Scope Base's host and focus trap IDs to this sheet, including
            // when nested sheets render as siblings.
            gpui::deferred(
                div().id((self.id, "sheet-root")).child(
                    gpui_base::Sheet::new(cx)
                        .focus_handle(focus)
                        .overlay_interactive(false)
                        .overlay(modal_backdrop(scrim, move |window, cx| {
                            if properties.dismiss_on_outside_click {
                                outside.dismiss(DialogCloseReason::OutsideClick, window, cx);
                            }
                        }))
                        .surface(div().absolute().inset_0().overflow_hidden().child(panel))
                        .request_close(move |window, cx| {
                            cx.stop_propagation();
                            if properties.dismiss_on_escape {
                                session.dismiss(DialogCloseReason::Escape, window, cx);
                            }
                        }),
                ),
            )
            .with_priority(layer)
            .into_any_element()
        } else {
            gpui::deferred(
                anchored().position(point(px(0.), px(0.))).child(
                    div()
                        .id((self.id.clone(), "sheet-exit"))
                        .absolute()
                        .top_0()
                        .left_0()
                        .w(viewport.width)
                        .h(viewport.height)
                        .overflow_hidden()
                        .child(scrim)
                        .child(exiting_panel(panel, &self.id, &dialog, window, cx)),
                ),
            )
            .with_priority(layer)
            .into_any_element()
        }
    }
}

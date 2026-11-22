use std::rc::Rc;

use gpui::{
    AnyElement, App, Edges, ElementId, Entity, Hsla, IntoElement, Pixels, SharedString, Size,
    Window, px, radians,
};
use gpui_material_theme::{ColorRole, CornerShape, Elevation};

use crate::{Icon, IconButton, IconName, ScaffoldInsets};

use super::bottom_sheet::{BottomSheetState, BottomSheetThresholds};
use super::dialog::{
    DialogCloseReason, DialogProperties, DialogState, ModalClosed, ModalDismissRequest,
    ModalSession,
};

mod gesture;
mod render;

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
    show_header: bool,
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
    max_width: Option<Pixels>,
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
            show_header: true,
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
            max_width: None,
            drag_enabled: true,
            show_handle: true,
            handle_content: None,
            handle_label: "Resize bottom sheet".into(),
            thresholds: BottomSheetThresholds::default(),
            content_insets: ScaffoldInsets::default(),
        }
    }

    #[cfg_attr(not(feature = "navigation"), allow(dead_code))]
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

    /// Sets the maximum width in logical pixels (defaults to 640 dp for bottom sheets and 400 dp for side sheets).
    pub fn max_width(mut self, width: Pixels) -> Self {
        if f32::from(width).is_finite() && width > px(0.) {
            self.max_width = Some(width);
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

    /// Shows or hides the visible header row while keeping the constructor's name as the accessible label.
    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
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
    #[cfg_attr(not(feature = "navigation"), allow(dead_code))]
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

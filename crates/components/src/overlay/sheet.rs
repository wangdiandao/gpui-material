use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, Hsla, IntoElement, Pixels, SharedString, Size, Window, px,
    radians,
};
use gpui_material_theme::{ColorRole, CornerShape, Elevation, LogicalEdges};

use crate::{Icon, IconButton, IconName};

use super::bottom_sheet::{BottomSheetState, BottomSheetThresholds};
use super::dialog::{
    DialogCloseReason, DialogProperties, DialogState, ModalClosed, ModalDismissRequest,
    ModalSession,
};

mod chrome;
mod geometry;
mod render;

/// Edge of the window or layout container to which a [`Sheet`] attaches.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SheetEdge {
    /// Attach to the bottom edge.
    #[default]
    Bottom,
    /// Attach to the logical start edge (left in LTR, right in RTL).
    Start,
    /// Attach to the logical end edge (right in LTR, left in RTL).
    End,
}

/// Interaction mode of a [`Sheet`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SheetMode {
    /// Non-blocking docked sheet that participates in parent layout.
    #[default]
    Standard,
    /// Blocking overlay sheet presented above a scrim with modal focus trapping.
    Modal,
}

/// Placement of a side sheet relative to its window edge or layout slot.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SheetPlacement {
    /// Flush against the window or container edge.
    #[default]
    Docked,
    /// Inset from the edges using [`LogicalEdges`] margins with rounded corners on all sides.
    Detached,
}

/// Optional local colors for the sheet and its adornments.
#[derive(Clone, Copy, Debug, Default)]
pub struct SheetColors {
    /// Background color override for the sheet container.
    pub container: Option<Hsla>,
    /// Foreground color override for the sheet body content.
    pub content: Option<Hsla>,
    /// Foreground color override for the sheet header title.
    pub header: Option<Hsla>,
    /// Color override for header, footer, and edge dividers.
    pub divider: Option<Hsla>,
    /// Color override for the modal backdrop scrim.
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

pub(super) type Change = Rc<dyn Fn(bool, &mut Window, &mut App)>;

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
    margins: LogicalEdges,
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
    on_closed: Option<ModalClosed>,
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
    content_insets: LogicalEdges,
}

impl Sheet {
    /// Creates a sheet bound to a [`DialogState`] with the given accessible title and body content.
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
            margins: LogicalEdges::all(16.),
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
            on_closed: None,
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
            content_insets: LogicalEdges::ZERO,
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

    /// Enables or disables pointer drag resizing on a retained bottom sheet (default `true`).
    pub fn drag_enabled(mut self, enabled: bool) -> Self {
        self.drag_enabled = enabled;
        self
    }

    /// Sets the distance and velocity thresholds used when settling a released drag.
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
    pub fn content_insets(mut self, insets: LogicalEdges) -> Self {
        self.content_insets = insets.sanitize();
        self
    }

    /// Sets which edge the sheet attaches to (default [`SheetEdge::Bottom`]). Ignored for retained bottom sheets.
    pub fn edge(mut self, edge: SheetEdge) -> Self {
        if matches!(self.state, SheetBinding::Immediate(_)) {
            self.edge = edge;
        }
        self
    }

    /// Sets whether the sheet is standard (docked) or modal (default [`SheetMode::Standard`]).
    pub fn mode(mut self, mode: SheetMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets expanded height for a bottom sheet, or width for a side sheet.
    /// Non-finite values are ignored.
    pub fn extent(mut self, extent: Pixels) -> Self {
        if f32::from(extent).is_finite() {
            self.extent = extent.max(px(48.));
        }
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

    /// Sets whether a side sheet is docked to the edge or detached with margins (default [`SheetPlacement::Docked`]).
    pub fn placement(mut self, placement: SheetPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Sets the logical inset margins used when [`SheetPlacement::Detached`] is active.
    pub fn margins(mut self, margins: LogicalEdges) -> Self {
        self.margins = margins.sanitize();
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

    /// Replaces the leading back button with custom content.
    pub fn back_content(mut self, back: impl IntoElement) -> Self {
        self.back = Some(back.into_any_element());
        self
    }

    /// Overrides the accessible label of the default close button (default `"Close"`).
    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = label.into();
        self
    }

    /// Shows or hides the default close button in the sheet header (default `true`).
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

    /// Adds a fixed footer slot at the bottom of the sheet.
    pub fn footer(mut self, actions: impl IntoElement) -> Self {
        self.footer = Some(actions.into_any_element());
        self
    }

    /// Shows or hides a horizontal divider above the sheet footer (default `false`).
    pub fn footer_divider(mut self, visible: bool) -> Self {
        self.footer_divider = visible;
        self
    }

    /// Adds a divider between a docked Standard side sheet and its adjacent content.
    pub fn edge_divider(mut self, visible: bool) -> Self {
        self.edge_divider = visible;
        self
    }

    /// Configures Escape and outside-click dismissal behavior for modal sheets.
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
    pub fn on_closed(
        mut self,
        close: impl Fn(DialogCloseReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_closed = Some(Rc::new(close));
        self
    }

    /// Sets the stacking layer of a nested modal sheet.
    pub fn layer(mut self, layer: usize) -> Self {
        self.layer = layer;
        self
    }

    /// Overrides the corner shape of the sheet container.
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Overrides the theme [`ColorRole`] used for the sheet container background.
    pub fn container_role(mut self, role: ColorRole) -> Self {
        self.container_role = Some(role);
        self
    }

    /// Overrides the theme [`ColorRole`] used for the sheet foreground content.
    pub fn content_role(mut self, role: ColorRole) -> Self {
        self.content_role = Some(role);
        self
    }

    /// Overrides the sheet's container, content, header, divider, and scrim colors.
    pub fn colors(mut self, colors: SheetColors) -> Self {
        self.colors = colors;
        self
    }

    /// Sets the tonal elevation applied to the sheet surface.
    pub fn tonal_elevation(mut self, elevation: Elevation) -> Self {
        self.tonal_elevation = elevation;
        self
    }

    /// Overrides the shadow elevation cast by the sheet surface.
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

    /// Sets the modal backdrop scrim opacity, clamped to 0–1. `NaN` falls back to the default (`0.32`).
    pub fn scrim_opacity(mut self, opacity: f32) -> Self {
        self.scrim_opacity = if opacity.is_nan() {
            0.32
        } else {
            opacity.clamp(0., 1.)
        };
        self
    }
}

mod bottom_sheet;
mod dialog;
mod menu;
mod modal;
mod sheet;
mod snackbar;
mod tooltip;

pub use bottom_sheet::{BottomSheetState, BottomSheetThresholds, BottomSheetValue};
pub use dialog::{
    Dialog, DialogAction, DialogCloseReason, DialogColors, DialogProperties, DialogState,
    DialogVariant,
};
pub use menu::{Menu, MenuColor, MenuItem, MenuItemStyle};
pub use sheet::{Sheet, SheetClose, SheetColors, SheetEdge, SheetMode, SheetPlacement};
pub use snackbar::{
    Snackbar, SnackbarColors, SnackbarHost, SnackbarId, SnackbarParts, SnackbarResult,
};
pub(crate) use tooltip::init as init_tooltip;
pub use tooltip::{
    Label, Tooltip, TooltipCaret, TooltipColors, TooltipPlacement, TooltipPositioner, TooltipState,
};

/// Wrapping overlay copy uses the same theme font and type role as shaped
/// single-line labels. Keep animation scale separate from component geometry.
fn text_style<E: gpui::Styled>(
    element: E,
    role: gpui_material_theme::TypeRole,
    scale: f32,
    window: &gpui::Window,
    cx: &gpui::App,
) -> E {
    let style = role.style(gpui_material_theme::TypographyEmphasis::Baseline);
    gpui_material_foundation::text_style(element, style, scale, window, cx)
}

pub(crate) fn presence(
    id: &gpui::ElementId,
    visible: bool,
    duration_ms: u64,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) -> gpui_base::motion::PresenceSample {
    use std::time::Duration;

    use gpui_base::motion::{
        Easing, MotionStatus, Presence, PresencePhase, PresenceSample, Transition,
    };

    const ABSENT: PresenceSample = PresenceSample {
        phase: PresencePhase::Absent,
        progress: 0.,
        status: MotionStatus::Finished,
    };
    let mounted =
        window.use_keyed_state((id.clone(), "material-presence-mounted"), cx, |_, _| false);
    let was_mounted = *mounted.read(cx);
    if !visible && !was_mounted {
        return ABSENT;
    }
    let sample = Presence::new((id.clone(), "material-presence"), visible)
        .transition(Transition::new(Duration::from_millis(duration_ms)).easing(Easing::EaseOut))
        .sample(window, cx);
    let render = visible || sample.progress > 0.;
    if was_mounted != render {
        mounted.update(cx, |mounted, _| *mounted = render);
    }
    if render { sample } else { ABSENT }
}

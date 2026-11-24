mod appearance;
mod interaction;
mod label;
mod render;
mod surface;

use std::{rc::Rc, time::Duration};

use gpui::{
    AnyElement, App, Bounds, ElementId, Entity, FocusHandle, IntoElement, KeyBinding, Pixels,
    SharedString, Size, Window, px,
};
use gpui_material_theme::{CornerShape, Direction, Elevation};

use appearance::Appearance;

pub use appearance::{TooltipCaret, TooltipColors, TooltipPlacement};
pub use gpui_base::Positioner as TooltipPositioner;
pub use interaction::TooltipState;
pub use label::Label;

use crate::control_style::finite_length;

gpui::actions!(
    material_tooltip,
    [
        FocusTooltipAction,
        FocusTooltipTrigger,
        DismissTooltip,
        ActivateTooltipAction
    ]
);

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", FocusTooltipAction, Some("MaterialTooltipTrigger")),
        KeyBinding::new(
            "shift-tab",
            FocusTooltipTrigger,
            Some("MaterialTooltipContent"),
        ),
        KeyBinding::new("escape", DismissTooltip, Some("MaterialTooltip")),
        KeyBinding::new(
            "enter",
            ActivateTooltipAction,
            Some("MaterialTooltipAction"),
        ),
        KeyBinding::new(
            "space",
            ActivateTooltipAction,
            Some("MaterialTooltipAction"),
        ),
    ]);
}

pub(super) type TooltipCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub(super) type PositionProvider =
    Box<dyn Fn(Bounds<Pixels>, Size<Pixels>, Direction) -> TooltipPositioner>;

pub(super) struct TooltipAction {
    pub(super) label: SharedString,
    pub(super) focus: FocusHandle,
    pub(super) callback: TooltipCallback,
}

/// A transient plain or rich Material tooltip, or a persistent rich tooltip
/// with caller-owned visibility.
#[derive(IntoElement)]
pub struct Tooltip {
    pub(super) id: ElementId,
    pub(super) trigger: Option<AnyElement>,
    pub(super) text: SharedString,
    pub(super) title: Option<SharedString>,
    pub(super) shortcut: Option<SharedString>,
    pub(super) action: Option<TooltipAction>,
    pub(super) title_content: Option<AnyElement>,
    pub(super) body_content: Option<AnyElement>,
    pub(super) action_content: Option<AnyElement>,
    pub(super) link: Option<(SharedString, TooltipCallback)>,
    pub(super) rich: bool,
    pub(super) trigger_focus: Option<FocusHandle>,
    pub(super) focus_target: Option<FocusHandle>,
    pub(super) focusable: bool,
    pub(super) enable_user_input: bool,
    pub(super) persistent: Option<Entity<TooltipState>>,
    pub(super) label_visible: Option<bool>,
    pub(super) on_dismiss_request: Option<TooltipCallback>,
    pub(super) placement: TooltipPlacement,
    pub(super) offset: Pixels,
    positioner: Option<PositionProvider>,
    appearance: Appearance,
    show_delay: Duration,
    pub(super) hide_delay: Duration,
}

impl Tooltip {
    /// Creates a plain tooltip anchored to `trigger` with the given descriptive text.
    pub fn plain(
        id: impl Into<ElementId>,
        trigger: impl IntoElement,
        text: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            trigger: Some(trigger.into_any_element()),
            text: text.into(),
            title: None,
            shortcut: None,
            action: None,
            title_content: None,
            body_content: None,
            action_content: None,
            link: None,
            rich: false,
            trigger_focus: None,
            focus_target: None,
            focusable: false,
            enable_user_input: true,
            persistent: None,
            label_visible: None,
            on_dismiss_request: None,
            placement: TooltipPlacement::Above,
            offset: px(4.),
            positioner: None,
            appearance: Appearance::default(),
            show_delay: Duration::from_millis(500),
            hide_delay: Duration::from_millis(200),
        }
    }

    /// Creates a rich tooltip anchored to `trigger` with a title and body text.
    pub fn rich(
        id: impl Into<ElementId>,
        trigger: impl IntoElement,
        title: impl Into<SharedString>,
        text: impl Into<SharedString>,
    ) -> Self {
        Self {
            title: Some(title.into()),
            rich: true,
            ..Self::plain(id, trigger, text)
        }
    }

    /// Appends a subtle formatted desktop shortcut hint beside plain tooltip
    /// text or in a rich tooltip header/body while keeping the constructor's
    /// text as the accessible description.
    pub fn shortcut(mut self, text: impl Into<SharedString>) -> Self {
        self.shortcut = Some(text.into());
        self
    }

    /// Adds a text button to a rich tooltip.
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        focus: &FocusHandle,
        callback: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.rich = true;
        self.action = Some(TooltipAction {
            label: label.into(),
            focus: focus.clone(),
            callback: Rc::new(callback),
        });
        self
    }

    /// Replaces the rich tooltip title with custom content.
    pub fn title_content(mut self, content: impl IntoElement) -> Self {
        self.rich = true;
        self.title_content = Some(content.into_any_element());
        self
    }

    /// Replaces the tooltip body text with custom content while keeping the constructor text as the accessible description.
    pub fn body_content(mut self, content: impl IntoElement) -> Self {
        self.body_content = Some(content.into_any_element());
        self
    }

    /// Replaces the standard action with caller-authored interactive content.
    pub fn action_content(mut self, content: impl IntoElement) -> Self {
        self.rich = true;
        self.action_content = Some(content.into_any_element());
        self
    }

    /// Adds a semantic link. The callback owns the destination and navigation.
    pub fn link(
        mut self,
        label: impl Into<SharedString>,
        callback: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.rich = true;
        self.link = Some((label.into(), Rc::new(callback)));
        self
    }

    /// Overrides the pointer hover delay before showing a transient tooltip (default 500 ms).
    pub fn show_delay(mut self, delay: Duration) -> Self {
        self.show_delay = delay;
        self
    }

    /// Overrides the pointer leave grace delay before hiding a transient tooltip (default 200 ms).
    pub fn hide_delay(mut self, delay: Duration) -> Self {
        self.hide_delay = delay;
        self
    }

    /// Uses the trigger's focus handle for keyboard activation and restoration.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.trigger_focus = Some(focus.clone());
        self
    }

    /// Uses the caller's first focus target inside the tooltip.
    pub fn focus_target(mut self, focus: &FocusHandle) -> Self {
        self.focus_target = Some(focus.clone());
        self
    }

    /// Moves focus to the tooltip when it opens. The default preserves focus.
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    /// Enables automatic trigger input for the chosen visibility mode.
    pub fn enable_user_input(mut self, enable: bool) -> Self {
        self.enable_user_input = enable;
        self
    }

    /// Uses caller-owned visibility for a persistent rich tooltip.
    pub fn persistent(mut self, state: &Entity<TooltipState>) -> Self {
        self.rich = true;
        self.persistent = Some(state.clone());
        self
    }

    /// Handles Escape or a persistent tooltip's outside press. A persistent
    /// tooltip with a handler closes when the owner accepts the request.
    pub fn on_dismiss_request(
        mut self,
        callback: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss_request = Some(Rc::new(callback));
        self
    }

    /// Sets the preferred side relative to the trigger (default [`TooltipPlacement::Above`]).
    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Gap between the trigger and surface in logical pixels.
    pub fn offset(mut self, offset: Pixels) -> Self {
        self.offset = finite_length(offset);
        self
    }

    /// Replaces the standard placement with a positioner built from the trigger
    /// bounds, viewport size, and direction. The host applies viewport margins.
    pub fn positioner(
        mut self,
        positioner: impl Fn(Bounds<Pixels>, Size<Pixels>, Direction) -> TooltipPositioner + 'static,
    ) -> Self {
        self.positioner = Some(Box::new(positioner));
        self
    }

    /// Renders a pointer caret pointing toward the trigger.
    pub fn caret(mut self, caret: TooltipCaret) -> Self {
        self.appearance.caret = Some(caret);
        self
    }

    /// Overrides the maximum width of the tooltip surface in logical pixels.
    pub fn max_width(mut self, width: Pixels) -> Self {
        self.appearance.max_width = Some(finite_length(width));
        self
    }

    /// Overrides the corner shape of the tooltip surface.
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.appearance.shape = Some(shape);
        self
    }

    /// Overrides the tooltip's container, content, title, and action colors.
    pub fn colors(mut self, colors: TooltipColors) -> Self {
        self.appearance.colors = colors;
        self
    }

    /// Sets the tonal elevation used to tint the tooltip surface.
    pub fn tonal_elevation(mut self, elevation: Elevation) -> Self {
        self.appearance.tonal_elevation = elevation;
        self
    }

    /// Overrides the shadow elevation cast by the tooltip surface.
    pub fn shadow_elevation(mut self, elevation: Elevation) -> Self {
        self.appearance.shadow_elevation = Some(elevation);
        self
    }
}

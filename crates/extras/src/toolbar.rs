use std::rc::Rc;

use gpui::{
    App, ElementId, FocusHandle, Hsla, InteractiveElement, StatefulInteractiveElement, Styled,
    Window, prelude::FluentBuilder,
};
use gpui_base::Button;
use gpui_material_theme::StateLayer;

mod bar;
mod fab_menu;

pub use bar::{
    Toolbar, ToolbarAction, ToolbarArrangement, ToolbarColor, ToolbarColors, ToolbarElevations,
    ToolbarExitDirection, ToolbarFabPosition, ToolbarFling, ToolbarOrientation, ToolbarPadding,
    ToolbarPlacement, ToolbarScrollBehavior, ToolbarScrollState,
};
pub use fab_menu::{
    FabMenu, FabMenuAlignment, FabMenuItem, FabMenuItemColors, FabMenuTriggerAppearance,
    FabMenuTriggerStyle,
};

type Invoke = Rc<dyn Fn(&mut Window, &mut App)>;

fn focus_handle(
    id: &ElementId,
    part: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> FocusHandle {
    window
        .use_keyed_state((id.clone(), part), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone()
}

fn activate_key(
    event: &gpui::KeyDownEvent,
    window: &mut Window,
    cx: &mut App,
    command: &dyn Fn(&mut Window, &mut App),
) {
    if !event.keystroke.modifiers.modified()
        && matches!(event.keystroke.key.as_str(), "enter" | "space")
    {
        if !event.is_held {
            command(window, cx);
        }
        cx.stop_propagation();
    }
}

fn style_button(
    button: Button,
    background: Hsla,
    foreground: Hsla,
    ring: Hsla,
    enabled: bool,
) -> Button {
    button
        .bg(background)
        .text_color(foreground)
        .border_2()
        .border_color(gpui::transparent_black())
        .when(enabled, |button| {
            button
                .hover(move |style| {
                    style.bg(background.blend(foreground.opacity(StateLayer::Hover.opacity())))
                })
                .active(move |style| {
                    style.bg(background.blend(foreground.opacity(StateLayer::Pressed.opacity())))
                })
                .focus_visible(move |style| style.border_color(ring))
        })
}

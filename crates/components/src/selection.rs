mod base;
mod checkbox;
mod radio;
mod switch;

use std::rc::Rc;

use gpui::{
    AnyElement, App, Div, Hsla, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{color, control_feedback::FeedbackState, current_material_theme};
use gpui_material_theme::{ColorRole, Density, Direction, StateLayer};

use crate::action::ripple_ink;

pub use checkbox::{Checkbox, CheckboxColors, CheckboxState, CheckboxStateColors};
pub use radio::{Radio, RadioColors, RadioGroup};
pub use switch::{Switch, SwitchColors, SwitchStateColors};

fn tone(role: ColorRole, window: &Window, cx: &App) -> Hsla {
    color(current_material_theme(window, cx).colors.color(role))
}

fn selection_tone(selected: bool, disabled: bool, window: &Window, cx: &App) -> Hsla {
    if disabled {
        tone(ColorRole::OnSurface, window, cx).opacity(0.38)
    } else if selected {
        tone(ColorRole::Primary, window, cx)
    } else {
        tone(ColorRole::OnSurfaceVariant, window, cx)
    }
}

fn row(direction: Direction, show_label: bool) -> Div {
    div()
        .flex()
        .items_center()
        .when(show_label, Styled::gap_2)
        .when(direction == Direction::Rtl, Styled::flex_row_reverse)
}

fn selection_label(
    label: SharedString,
    content: Option<AnyElement>,
    disabled: bool,
    window: &Window,
    cx: &App,
) -> impl IntoElement + use<> {
    div()
        .id("selection-label")
        .test_support()
        .min_w_0()
        .when(content.is_none(), Styled::truncate)
        .when(disabled, |element| {
            element.text_color(tone(ColorRole::OnSurface, window, cx).opacity(0.38))
        })
        .child(content.unwrap_or_else(|| label.into_any_element()))
}

#[allow(clippy::too_many_arguments)]
fn selection_target(
    id: SharedString,
    indicator: impl IntoElement,
    accent: Hsla,
    enabled: bool,
    ripple: Rc<std::cell::RefCell<FeedbackState>>,
    direction: Direction,
    density: Density,
    reduced_motion: bool,
) -> Div {
    let static_pressed = enabled && reduced_motion && ripple.borrow().pressed();
    let hit_target = px(density.interactive_size(40.));
    let state_layer = px(density.visual_size(40.).max(24.));
    div()
        .size(hit_target)
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .id("selection-indicator")
                .relative()
                .size(state_layer)
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .when(enabled, |target| {
                    let hover = id.clone();
                    target
                        .when(static_pressed, |target| {
                            target.bg(accent.opacity(StateLayer::Pressed.opacity()))
                        })
                        .when(!static_pressed, |target| {
                            target.group_hover(hover, |style| {
                                style.bg(accent.opacity(StateLayer::Hover.opacity()))
                            })
                        })
                        .when(reduced_motion, |target| {
                            target.group_active(id, |style| {
                                style.bg(accent.opacity(StateLayer::Pressed.opacity()))
                            })
                        })
                })
                .child(ripple_ink(
                    ripple,
                    accent,
                    gpui_material_theme::CornerRole::Full.shape(),
                    direction,
                ))
                .child(indicator),
        )
}

fn focus_style(color: Hsla) -> impl FnOnce(gpui::StyleRefinement) -> gpui::StyleRefinement {
    move |style| style.border_color(color)
}

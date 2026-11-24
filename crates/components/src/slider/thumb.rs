use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, Bounds, ElementId, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Role, SharedString, StatefulInteractiveElement, Styled, Window, canvas, div,
    point, prelude::FluentBuilder, px,
};
use gpui_base::{
    AxisExt, TestSupportExt,
    slider::{SliderState, SliderValue},
};
use gpui_material_foundation::{RangeThumb, current_material_theme, range_semantics, shape_text};
use gpui_material_theme::{TypeRole, TypographyEmphasis};

use super::geometry::{Geometry, value_label_origin};
use super::gesture::Gesture;
use super::layout::TrackLayout;
use super::palette::SliderPalette;
use super::{FormatValue, SliderThumbStyle, thumb_id, thumb_index};

#[allow(clippy::too_many_arguments)]
fn render_value_label(
    which: RangeThumb,
    opacity: f32,
    value: SliderValue,
    format_value: Option<&FormatValue>,
    geometry: Geometry,
    palette: SliderPalette,
    window: &Window,
    cx: &App,
) -> Option<AnyElement> {
    if opacity <= 0. {
        return None;
    }
    let numeric_value = if which == RangeThumb::Start {
        value.start()
    } else {
        value.end()
    };
    let text = format_value.map_or_else(
        || format!("{numeric_value:.2}").into(),
        |format| format(numeric_value),
    );
    let theme = current_material_theme(window, cx);
    let foreground = palette.inverse_on_surface.opacity(opacity);
    let background = palette.inverse_surface.opacity(opacity);
    let horizontal = geometry.axis.is_horizontal();
    let line = shape_text(
        text,
        TypeRole::LabelLarge.style(TypographyEmphasis::Baseline),
        &theme.typography,
        1.,
        foreground,
        window,
        cx,
    )?;
    Some(
        div()
            .absolute()
            .inset_0()
            .size_full()
            .child(gpui::deferred(
                canvas(
                    |_, _, _| (),
                    move |thumb, _, window, cx| {
                        let size = gpui::size(
                            (line.line().width.ceil() + px(16.)).max(px(48.)),
                            line.line_height().max(px(44.)),
                        );
                        let visible = window.content_mask().bounds;
                        if !visible.contains(&thumb.center()) {
                            return;
                        }
                        let origin = value_label_origin(
                            thumb,
                            size,
                            visible,
                            horizontal,
                            geometry.direction,
                        );
                        let bounds = Bounds::new(origin, size);
                        window.paint_quad(gpui::quad(
                            bounds,
                            px(6.),
                            background,
                            px(0.),
                            background,
                            gpui::BorderStyle::Solid,
                        ));
                        let _ = line.paint(
                            origin
                                + point(
                                    (size.width - line.line().width) / 2.,
                                    (size.height - line.line_height()) / 2.,
                                ),
                            gpui::TextAlign::Left,
                            None,
                            window,
                            cx,
                        );
                    },
                )
                .size_full(),
            ))
            .into_any_element(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_thumb(
    id: &ElementId,
    which: RangeThumb,
    position: f32,
    name: SharedString,
    focus: &FocusHandle,
    content: Option<AnyElement>,
    style: SliderThumbStyle,
    label_opacity: f32,
    value: SliderValue,
    format_value: Option<&FormatValue>,
    state: &Entity<SliderState>,
    gesture: &Rc<RefCell<Gesture>>,
    geometry: Geometry,
    layout: TrackLayout,
    palette: SliderPalette,
    disabled: bool,
    window: &Window,
    cx: &App,
) -> impl IntoElement {
    let horizontal = geometry.axis.is_horizontal();
    let handle_size = layout.handle_sizes[thumb_index(which)];
    let handle_width = handle_size.width;
    let half_handle = handle_width / 2.;
    let cross = handle_size.height;
    let (width, height) = if horizontal {
        (handle_width, cross)
    } else {
        (cross, handle_width)
    };
    let tint = if disabled {
        style.disabled_color
    } else {
        style.color
    }
    .unwrap_or(palette.thumb);
    let focus_color = style.focused_color.unwrap_or(palette.ring);
    let keyboard_focused = !disabled
        && window.is_window_active()
        && focus.is_focused(window)
        && window.last_input_was_keyboard();
    let label = render_value_label(
        which,
        label_opacity,
        value,
        format_value,
        geometry,
        palette,
        window,
        cx,
    );
    let custom = content.is_some();
    let visual_id = ElementId::from((
        id.clone(),
        if which == RangeThumb::Start {
            "start-visual"
        } else {
            "end-visual"
        },
    ));
    let visual_thumb = geometry
        .place(
            div()
                .absolute()
                .when(horizontal, |thumb| {
                    thumb
                        .top((layout.target - height) / 2.)
                        .ml(-half_handle)
                        .when(geometry.reversed(), |thumb| {
                            thumb.ml(px(0.)).mr(-half_handle)
                        })
                })
                .when(!horizontal, |thumb| {
                    thumb.left((layout.target - width) / 2.).when_else(
                        geometry.reversed(),
                        |thumb| thumb.mt(-half_handle),
                        |thumb| thumb.mb(-half_handle),
                    )
                }),
            position,
        )
        .id(visual_id)
        .test_support()
        .w(width)
        .h(height)
        .when_else(
            custom,
            |thumb| {
                thumb
                    .text_color(tint)
                    .when(keyboard_focused, |thumb| {
                        thumb.border_1().border_color(focus_color)
                    })
                    .children(content.map(|art| div().size_full().child(art)))
            },
            |thumb| {
                thumb
                    .rounded_full()
                    .bg(if keyboard_focused { focus_color } else { tint })
            },
        )
        .when(!disabled, |thumb| {
            let hover = gesture.clone();
            thumb.on_hover(move |hovered, window, cx| {
                {
                    let mut hover = hover.borrow_mut();
                    hover.feedback[thumb_index(which)].hovered = *hovered;
                }
                hover.borrow().record(window, cx);
            })
        })
        .children(label);

    // The accessible Role::Slider node spans the full track interaction area
    // rather than the 4 px thumb strip so UIA hit-testing and screen readers
    // see the full slider bounds.
    let accessible = div()
        .id(thumb_id(id, which))
        .absolute()
        .inset_0()
        .size_full()
        .when(!disabled, |node| node.role(Role::Slider))
        .when(!disabled, |node| node.aria_label(name))
        .child(visual_thumb);

    let accessible = if disabled {
        accessible
    } else {
        let keyboard_state = state.clone();
        range_semantics(accessible, state, which, geometry.axis, cx).capture_key_down(
            move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let Some(sign) = geometry.reversed_key_delta(&event.keystroke.key) else {
                    return;
                };
                keyboard_state.update(cx, |state, cx| {
                    state.step_thumb(
                        which == RangeThumb::Start,
                        sign * state.accessibility_step(),
                        window,
                        cx,
                    );
                });
                cx.stop_propagation();
            },
        )
    };
    accessible.test_support().when(!disabled, |node| {
        node.track_focus(&focus.clone().tab_stop(true))
    })
}

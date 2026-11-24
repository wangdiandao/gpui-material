use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    AnyElement, BorderStyle, Corners, Div, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, SharedString, Stateful, Styled, canvas, div, prelude::FluentBuilder, px,
    quad,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{
    InteractionFeedback, InteractionState, elevation_shadow_pair, resolve_edges,
};
use gpui_material_theme::{CornerShape, Density, Direction, LayoutPolicy, LogicalEdges, TypeRole};

use crate::{Button, ButtonVariant, ControlSize, Text, action::FeedbackState};

use super::long_press::LongPress;
use super::style::{ListItemAlignment, RowColors};
use super::{SecondaryAction, Slot};

impl Slot {
    fn render(self, id: &'static str, role: TypeRole, tint: Hsla, wrap: bool) -> AnyElement {
        let content = match self {
            Self::Text(value) => Text::new(value)
                .id(id)
                .role(role)
                .when(wrap, Text::wrap)
                .into_any_element(),
            Self::Custom(element) => element,
        };
        inherit_color((ElementId::from(id), "slot"), content, Some(tint))
    }
}

pub(super) fn list_item_min_height(
    has_overline: bool,
    has_supporting: bool,
    three_lines: bool,
    interactive: bool,
    density: Density,
) -> f32 {
    let baseline = match (has_overline, has_supporting) {
        _ if three_lines => 88.,
        (false, false) => 56.,
        _ => 72.,
    };
    if interactive {
        density.interactive_size(baseline)
    } else {
        density.visual_size(baseline)
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_visual_canvas(
    radii: [f32; 4],
    full: f32,
    direction: Direction,
    enabled: bool,
    elevation: f32,
    shadow: Hsla,
    container: Hsla,
    content_tint: Hsla,
    ring: Hsla,
    interaction: InteractionState,
    layer: f32,
    reduced: bool,
    feedback: &Rc<RefCell<FeedbackState>>,
) -> impl IntoElement {
    let geometry = feedback.clone();
    let ink = feedback.clone();
    canvas(
        move |bounds, window, _| {
            let extent = f32::from(bounds.size.width.min(bounds.size.height)) * 0.5;
            let shape = CornerShape::Radii(radii.map(|radius| radius + (extent - radius) * full));
            let [top_left, top_right, bottom_right, bottom_left] = shape
                .radii(
                    bounds.size.width.into(),
                    bounds.size.height.into(),
                    direction,
                )
                .map(px);
            if enabled {
                geometry
                    .borrow_mut()
                    .set_rounded_shape(bounds, shape, direction, window);
            } else {
                geometry.borrow_mut().set_shape(None, window);
            }
            Corners {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            }
        },
        move |bounds, corners, window, cx| {
            if let Some(shadows) = elevation_shadow_pair(elevation, 1., shadow) {
                window.paint_drop_shadows(bounds, corners, &shadows);
            }
            window.paint_quad(quad(
                bounds,
                corners,
                container,
                px(0.),
                gpui::transparent_black(),
                BorderStyle::Solid,
            ));
            let feedback = ink.borrow();
            if let Some(shape) = feedback.shape() {
                // Base's outer ring keeps the existing list focus geometry.
                let mut visual_interaction = interaction;
                visual_interaction.focus_visible = false;
                InteractionFeedback::new(visual_interaction, content_tint, ring)
                    .paint_with_opacity(
                        shape,
                        feedback.ripple(),
                        layer,
                        cx.background_executor().now(),
                        reduced,
                        window,
                    );
            }
        },
    )
    .absolute()
    .inset_0()
}

pub(super) fn render_content_slot(
    label: SharedString,
    content: Option<AnyElement>,
    overline: Option<Slot>,
    supporting: Option<Slot>,
    colors: &RowColors,
    text_height: Option<Rc<Cell<Pixels>>>,
) -> Div {
    div()
        .relative()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .overflow_hidden()
        .text_color(colors.content)
        .children(
            overline
                .map(|slot| slot.render("overline", TypeRole::LabelSmall, colors.overline, false)),
        )
        .child(
            div()
                .id("headline-slot")
                .test_support()
                .child(content.unwrap_or_else(|| Text::new(label).id("label").into_any_element())),
        )
        .children(
            supporting.map(|slot| {
                slot.render("supporting", TypeRole::BodyMedium, colors.supporting, true)
            }),
        )
        .when_some(text_height, |slot, text_height| {
            slot.child(
                canvas(
                    move |bounds, window, _| {
                        if text_height.replace(bounds.size.height) != bounds.size.height {
                            window.refresh();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            )
        })
}

fn align_row<E: Styled>(row: E, alignment: ListItemAlignment) -> E {
    match alignment {
        ListItemAlignment::Start => row.items_start(),
        ListItemAlignment::End => row.items_end(),
        _ => row.items_center(),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_surface(
    height: f32,
    three_lines: bool,
    alignment: ListItemAlignment,
    padding: Option<LogicalEdges>,
    layout: LayoutPolicy,
    visual: impl IntoElement,
    leading: Option<AnyElement>,
    content: Div,
    trailing: Option<AnyElement>,
    footer: Option<AnyElement>,
    long_press: Option<&SecondaryAction>,
    long_press_target: Option<&Rc<LongPress>>,
    long_press_button: Option<ElementId>,
    colors: &RowColors,
    enabled: bool,
    dragged: bool,
) -> Stateful<Div> {
    let padding = padding
        .unwrap_or_else(|| LogicalEdges::symmetric(16., if three_lines { 12. } else { 10. }));
    let edges = resolve_edges(padding, layout.direction);
    let press_button = long_press.zip(long_press_button).map(|(press, id)| {
        let invoke = press.handler.clone();
        Button::new(id, press.label.clone())
            .variant(ButtonVariant::Text)
            .size(ControlSize::ExtraSmall)
            .disabled(!enabled || dragged)
            .on_activate(move |window, cx| invoke(window, cx))
    });
    let alignment = if alignment == ListItemAlignment::Auto {
        if three_lines {
            ListItemAlignment::Start
        } else {
            ListItemAlignment::Center
        }
    } else {
        alignment
    };
    let row_slots = align_row(div().flex().w_full().gap(px(12.)), alignment)
        .when(layout.direction == Direction::Rtl, Styled::flex_row_reverse)
        .children(leading.map(|element| inherit_color("leading-slot", element, colors.leading)))
        .child(content)
        .children(trailing.map(|element| inherit_color("trailing-slot", element, colors.trailing)))
        .children(press_button.map(|button| div().id("long-press-slot").child(button)));

    let surface = div()
        .id("list-surface")
        .relative()
        .min_h(px(height))
        .w_full()
        .flex()
        .flex_col()
        .pl(edges.left)
        .pr(edges.right)
        .pt(edges.top)
        .pb(edges.bottom)
        .child(visual)
        .children(
            long_press_target
                .filter(|_| enabled && !dragged)
                .map(|press| press.target()),
        );
    let surface = match alignment {
        ListItemAlignment::Start => surface.justify_start(),
        ListItemAlignment::End => surface.justify_end(),
        _ => surface.justify_center(),
    };
    if let Some(footer) = footer {
        surface
            .gap(px(6.))
            .child(row_slots)
            .child(div().id("footer-slot").w_full().child(footer))
    } else {
        surface.child(row_slots)
    }
}

fn inherit_color(id: impl Into<ElementId>, element: AnyElement, color: Option<Hsla>) -> AnyElement {
    // Each slot owns a separate identity scope, including elements that use
    // implicit IDs such as Text and Icon.
    div()
        .id(id)
        .test_support()
        .when_some(color, Styled::text_color)
        .child(element)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_item_min_height_respects_density_and_interactive_floor() {
        assert_eq!(
            list_item_min_height(false, false, false, false, Density::Standard),
            56.
        );
        assert_eq!(
            list_item_min_height(false, true, false, false, Density::Standard),
            72.
        );
        assert_eq!(
            list_item_min_height(true, true, true, false, Density::Standard),
            88.
        );

        let compact_visual = list_item_min_height(false, false, false, false, Density::Compact);
        let compact_interactive = list_item_min_height(false, false, false, true, Density::Compact);
        assert!(compact_visual <= 56.);
        assert!(compact_interactive >= 48.);
        assert!(compact_interactive >= compact_visual);

        let comfortable = list_item_min_height(false, false, false, false, Density::Comfortable);
        assert!(comfortable >= 56.);
    }
}

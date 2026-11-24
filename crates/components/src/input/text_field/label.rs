use gpui::StatefulInteractiveElement;
use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, Bounds, FocusHandle, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, Role, SharedString, Styled, canvas, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;

use super::{TextFieldLabelAlignment, metrics::FieldMetrics};

pub(super) struct LabelConfig {
    pub(super) label: SharedString,
    pub(super) label_content: Option<AnyElement>,
    pub(super) label_above: bool,
    pub(super) has_cutout: bool,
    pub(super) required: bool,
    pub(super) disabled: bool,
    pub(super) rtl: bool,
    pub(super) has_leading: bool,
    pub(super) trailing_clearance: f32,
    pub(super) floating: f32,
    pub(super) expanded_alignment: TextFieldLabelAlignment,
    pub(super) minimized_alignment: TextFieldLabelAlignment,
    pub(super) label_color: Hsla,
}

pub(super) fn build_label(
    config: LabelConfig,
    metrics: &FieldMetrics,
    focus: &FocusHandle,
    measure_cutout: Option<Rc<Cell<Option<Bounds<Pixels>>>>>,
) -> (Option<AnyElement>, Option<AnyElement>) {
    let label_alignment = if config.floating >= 0.5 {
        config.minimized_alignment
    } else {
        config.expanded_alignment
    };
    let label_padding = if config.has_cutout { px(4.) } else { px(0.) };
    let label_left = (metrics.padding.left - label_padding).max(px(0.));
    let label_right = (metrics.padding.right - label_padding).max(px(0.));
    let leading_inset = if config.has_leading {
        56. * (1. - config.floating)
    } else {
        0.
    };
    let trailing_inset = config.trailing_clearance * (1. - config.floating);
    let (left_inset, right_inset) = if config.rtl {
        (trailing_inset, leading_inset)
    } else {
        (leading_inset, trailing_inset)
    };
    let label_content = config
        .label_content
        .unwrap_or_else(|| config.label.into_any_element());
    let label_container = div()
        .id("label")
        .test_support()
        .when_else(
            config.label_above,
            |label| label.pl(label_left).pr(label_right),
            |label| {
                label
                    .absolute()
                    .top(px(metrics.label_top))
                    .left(label_left + px(left_inset))
                    .right(label_right + px(right_inset))
            },
        )
        .text_size(px(metrics.label_size))
        .line_height(px(metrics.label_height))
        .text_color(config.label_color)
        .flex()
        .map(|label| match label_alignment {
            TextFieldLabelAlignment::Center => label.justify_center(),
            TextFieldLabelAlignment::Start if config.rtl => label.justify_end(),
            TextFieldLabelAlignment::End if !config.rtl => label.justify_end(),
            _ => label,
        })
        .child(
            div()
                .id("label-text")
                .test_support()
                .min_w_0()
                .max_w_full()
                .relative()
                .truncate()
                .when(config.has_cutout, |label| label.px(px(4.)))
                .role(Role::Label)
                .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
                .when(config.required, |label| {
                    label
                        .flex()
                        .items_baseline()
                        .gap(px(4.))
                        .when(config.rtl, Styled::flex_row_reverse)
                })
                .child(label_content)
                .children(config.required.then_some("*"))
                .when_some(measure_cutout, |label, measure_cutout| {
                    label.child(
                        canvas(
                            move |bounds, _, _| measure_cutout.set(Some(bounds)),
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .inset_0(),
                    )
                }),
        )
        .when(!config.disabled, |label| {
            let focus = focus.clone();
            label.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                focus.focus(window, cx);
            })
        });
    if config.label_above {
        (Some(label_container.into_any_element()), None)
    } else {
        (None, Some(label_container.into_any_element()))
    }
}

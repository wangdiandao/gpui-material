use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderOnce, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_material_foundation::{InteractionSource, current_material_theme};
use gpui_material_theme::{Direction, LogicalEdges, TypeRole, TypographyEmphasis};

use crate::action::{Action, ActionControl, InvokeAction, icon_slot};
use crate::{Icon, IconName};

use super::{
    Chip, ChipVariant,
    palette::{build_chip_layout, build_chip_style, resolve_palette},
    split::{measure_height, render_split_chip, split_action_style},
};

#[allow(clippy::too_many_arguments)]
fn build_leading_slot(
    avatar: Option<AnyElement>,
    leading: Option<AnyElement>,
    filter_mark: f32,
    spacing: Pixels,
    leading_color: Hsla,
    direction: Direction,
) -> Option<AnyElement> {
    avatar
        .map(|avatar| {
            div()
                .size(px(24.))
                .flex_none()
                .rounded_full()
                .overflow_hidden()
                .text_color(leading_color)
                .child(icon_slot(avatar, 24.))
                .into_any_element()
        })
        .or_else(|| {
            leading.map(|leading| {
                div()
                    .flex_none()
                    .text_color(leading_color)
                    .child(icon_slot(leading, 18.))
                    .into_any_element()
            })
        })
        .or_else(|| {
            (filter_mark > 0.).then(|| {
                div()
                    .w((px(18.) + spacing) * filter_mark)
                    .flex_none()
                    .overflow_hidden()
                    .opacity(filter_mark)
                    .text_color(leading_color)
                    .when(direction == Direction::Rtl, |slot| {
                        slot.flex().justify_end()
                    })
                    .child(icon_slot(
                        Icon::new(IconName::Check).into_any_element(),
                        18.,
                    ))
                    .into_any_element()
            })
        })
}

#[allow(clippy::too_many_arguments)]
fn build_primary_content(
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    label: AnyElement,
    padding: LogicalEdges,
    spacing: Pixels,
    filter_mark: f32,
    split: bool,
    direction: Direction,
    visual_height: &Rc<Cell<Pixels>>,
) -> AnyElement {
    let content = div()
        .relative()
        .flex()
        .items_center()
        .min_w_0()
        .text_size(px(14.))
        .when(direction == Direction::Rtl, Styled::flex_row_reverse)
        .pt(px(padding.top))
        .pb(px(padding.bottom));
    let mut content = logical_edges(
        content,
        px(padding.start),
        if split { spacing } else { px(padding.end) },
        direction,
    )
    .children(leading)
    .when(filter_mark == 0., |row| row.gap(spacing))
    .child(label)
    .children(trailing.map(|trailing| {
        if filter_mark > 0. {
            logical_edges(div(), spacing, px(0.), direction)
                .child(trailing)
                .into_any_element()
        } else {
            trailing
        }
    }));
    if split {
        content = content.child(measure_height(visual_height.clone()));
    }
    content.into_any_element()
}

pub(super) fn logical_edges(element: Div, start: Pixels, end: Pixels, direction: Direction) -> Div {
    if direction == Direction::Rtl {
        element.pl(end).pr(start)
    } else {
        element.pl(start).pr(end)
    }
}

impl RenderOnce for Chip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let selectable = matches!(self.variant, ChipVariant::Filter | ChipVariant::Input);
        let selected = selectable && self.selected;
        let tonal = selectable && self.tonal;
        let elevated = self.elevated && self.variant != ChipVariant::Input;
        let split = selectable && self.remove.is_some();
        let (direction, density) = {
            let theme = current_material_theme(window, cx);
            (theme.layout.direction, theme.layout.density)
        };
        let palette = resolve_palette(
            &self.id,
            self.variant,
            selectable,
            selected,
            tonal,
            self.tone,
            elevated,
            self.disabled,
            self.colors,
            self.border,
            window,
            cx,
        );
        let style = build_chip_style(
            self.variant,
            selectable,
            selected,
            tonal,
            elevated,
            self.shapes,
            palette.foreground,
        );
        let layout = build_chip_layout(
            split,
            self.tab_stop,
            self.dragged,
            self.shapes,
            self.elevations,
            palette,
        );

        let avatar = (self.variant == ChipVariant::Input)
            .then_some(self.avatar)
            .flatten();
        let has_avatar = avatar.is_some();
        let filter_mark = if self.variant == ChipVariant::Filter && self.leading.is_none() {
            palette.selection
        } else {
            0.
        };
        let leading = build_leading_slot(
            avatar,
            self.leading,
            filter_mark,
            self.spacing,
            palette.leading,
            direction,
        );
        let trailing = self
            .trailing
            .filter(|_| self.variant != ChipVariant::Suggestion)
            .map(|content| {
                div()
                    .flex_none()
                    .text_color(palette.trailing)
                    .text_size(px(18.))
                    .child(content)
                    .into_any_element()
            });
        let padding = self.padding.unwrap_or_else(|| {
            let start = if has_avatar {
                4.
            } else if filter_mark > 0. {
                16. - 8. * filter_mark
            } else if leading.is_some() {
                8.
            } else {
                16.
            };
            let end = if split {
                f32::from(self.spacing)
            } else if trailing.is_some() {
                8.
            } else {
                16.
            };
            LogicalEdges::new(0., end, 0., start)
        });
        let label = self.content.unwrap_or_else(|| {
            crate::text::single_line(
                self.label.clone(),
                TypeRole::LabelLarge.style(TypographyEmphasis::Baseline),
                1.,
                palette.label,
                (None, 1., 1.),
                window,
                cx,
            )
            .into_any_element()
        });
        let row = div()
            .id(self.id.clone())
            .relative()
            .flex()
            .min_w_0()
            .items_center()
            .when(direction == Direction::Rtl, Styled::flex_row_reverse);
        let (trailing, remove_content) = if split {
            (None, trailing)
        } else {
            (trailing, None)
        };
        let visual_height = Rc::new(Cell::new(px(density.visual_size(style.height))));
        let content = build_primary_content(
            leading,
            trailing,
            label,
            padding,
            self.spacing,
            filter_mark,
            split,
            direction,
            &visual_height,
        );
        let primary = ActionControl {
            id: ElementId::from((self.id.clone(), "activate")),
            label: self.label,
            focus: self.focus,
            disabled: self.disabled,
            action: if selectable {
                Action::Toggle {
                    selected,
                    on_change: self.on_change,
                }
            } else {
                Action::Invoke(self.on_activate.map(InvokeAction::Full))
            },
            style: if split {
                split_action_style(style, 0.)
            } else {
                style
            },
            content,
        };
        let tracker = self
            .interaction_source
            .as_ref()
            .map(|source| InteractionSource::track(source, &self.id, window, cx));
        if let Some((label, on_remove)) = self.remove.filter(|_| selectable) {
            render_split_chip(
                self.id,
                row,
                primary,
                label,
                on_remove,
                remove_content,
                self.disabled,
                self.dragged,
                selected,
                direction,
                padding,
                self.padding,
                style,
                layout,
                palette,
                self.elevations,
                visual_height,
                tracker,
                window,
                cx,
            )
        } else {
            row.child(primary.render_with_tracker(1., layout, tracker, window, cx))
        }
    }
}

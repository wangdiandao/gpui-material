use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, Entity, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, Role, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_base::TestSupportExt;

use super::{
    DropdownOption, OptionContent, Select, interaction::choose, state::ExposedDropdownState,
};

pub(super) struct OptionRowContext {
    pub(super) state: Entity<ExposedDropdownState>,
    pub(super) on_select: Option<Select>,
    pub(super) option_content: Option<OptionContent>,
    pub(super) open: bool,
    pub(super) parent: gpui::EntityId,
    pub(super) ink: Hsla,
    pub(super) selected_bg: Hsla,
    pub(super) selected_ink: Hsla,
    pub(super) min_height: Pixels,
    pub(super) horizontal_padding: Pixels,
}

pub(super) fn render_option_row(
    choice: DropdownOption,
    index: usize,
    total: usize,
    current: bool,
    ctx: &OptionRowContext,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let content = ctx.option_content.as_ref().map_or_else(
        || choice.label.clone().into_any_element(),
        |content| content(&choice, current, window, cx),
    );
    let ink = ctx.ink;
    let selected = ctx.selected_bg;
    let selected_ink = ctx.selected_ink;
    div()
        .id(choice.id.clone())
        .test_support()
        .when(ctx.open, |row| {
            row.role(Role::ListBoxOption)
                .aria_label(choice.label.clone())
                .aria_selected(current)
                .aria_position_in_set(index + 1)
                .aria_size_of_set(total)
        })
        .when(current, |row| row.bg(selected).text_color(selected_ink))
        .when(ctx.open && current, |row| row.aria_active_descendant())
        .when(ctx.open, |row| {
            row.hover(move |style| {
                style.bg(if current {
                    selected.blend(selected_ink.opacity(0.08))
                } else {
                    ink.opacity(0.08)
                })
            })
        })
        .min_h(ctx.min_height)
        .w_full()
        .px(ctx.horizontal_padding)
        .flex()
        .items_center()
        .child(content)
        .when(ctx.open, |row| {
            let state = ctx.state.clone();
            let callback = ctx.on_select.clone();
            let parent = ctx.parent;
            let select_row = Rc::new(move |window: &mut Window, cx: &mut App| {
                choose(&state, &choice, &callback, window, cx);
                cx.notify(parent);
            });
            let accessible_select = select_row.clone();
            row.on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                accessible_select(window, cx);
            })
            .on_click(move |_, window, cx| {
                select_row(window, cx);
            })
        })
        .into_any_element()
}

use std::rc::Rc;

use gpui::{
    AccessibleAction, App, ElementId, Entity, EntityInputHandler, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px, transparent_black,
};
use gpui_base::{
    Button as BaseButton, TestSupportExt,
    input::{Enter as InputEnter, MoveDown, MoveEnd, MoveHome, MoveUp},
};
use gpui_material_components::{
    ControlSize, Dialog, Divider, Icon, IconButton, IconName, Text, TextField,
    TextFieldColorOverrides, TextFieldColors,
};
use gpui_material_foundation::{
    Surface, color, current_material_strings, current_material_theme, isolated_scroll,
};
use gpui_material_theme::{ColorRole, CornerRole, Direction, Elevation, TypeRole};

use super::{
    CommandItem, CommandPalette, CommandPaletteState, SelectHandler,
    state::{NavStep, matches_command, resolve_active_id, step_active_id},
};

fn execute_command(
    item: CommandItem,
    state: &Entity<CommandPaletteState>,
    on_select: Option<&SelectHandler>,
    close_on_select: bool,
    window: &mut Window,
    cx: &mut App,
) {
    if item.is_disabled() {
        return;
    }
    if close_on_select {
        state.update(cx, |state, cx| state.close(window, cx));
    }
    state.update(cx, |state, cx| state.emit_execute(item.clone(), cx));
    if let Some(on_select) = on_select {
        on_select(&item, window, cx);
    }
}

impl RenderOnce for CommandPalette {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let rtl = direction == Direction::Rtl;
        let density = theme.layout.density;

        let on_surface = color(theme.colors.color(ColorRole::OnSurface));
        let on_surface_variant = color(theme.colors.color(ColorRole::OnSurfaceVariant));
        let outline_variant = color(theme.colors.color(ColorRole::OutlineVariant));
        let secondary_container = color(theme.colors.color(ColorRole::SecondaryContainer));
        let on_secondary_container = color(theme.colors.color(ColorRole::OnSecondaryContainer));
        let surface_highest = color(theme.colors.color(ColorRole::SurfaceContainerHighest));
        let disabled_fg = on_surface.opacity(0.38);

        let divider_color = self.colors.divider.unwrap_or(outline_variant);
        let group_heading_color = self.colors.group_heading.unwrap_or(on_surface_variant);
        let active_bg = self
            .colors
            .active_item_container
            .unwrap_or(secondary_container);
        let active_fg = self
            .colors
            .active_item_content
            .unwrap_or(on_secondary_container);
        let shortcut_bg = self.colors.shortcut_container.unwrap_or(surface_highest);
        let shortcut_fg = self.colors.shortcut_content.unwrap_or(on_surface_variant);
        let surface_shape = self.shape.unwrap_or_else(|| CornerRole::ExtraLarge.shape());

        let strings = current_material_strings(window, cx);
        let placeholder = self
            .placeholder
            .unwrap_or_else(|| SharedString::from(strings.search_prompt.clone()));
        let empty_label = self
            .empty_label
            .unwrap_or_else(|| SharedString::from(strings.no_results.clone()));
        let close_label = self
            .close_label
            .unwrap_or_else(|| SharedString::from(strings.close.clone()));
        let accessible_label = self
            .accessible_label
            .unwrap_or_else(|| SharedString::from("Command palette"));
        drop(strings);

        let (dialog_state, editor, scroll, stored_query, stored_active) = {
            let model = self.state.read(cx);
            (
                model.dialog_state().clone(),
                model.editor().clone(),
                model.scroll_handle().clone(),
                model.query().clone(),
                model.active_command_id().cloned(),
            )
        };
        let query = if editor.update(cx, |input, cx| {
            input.marked_text_range(window, cx).is_none()
        }) {
            editor.read(cx).value()
        } else {
            stored_query
        };

        let filtered: Vec<CommandItem> = self
            .items
            .into_iter()
            .filter(|item| match &self.filter {
                Some(predicate) => predicate(item, query.as_ref()),
                None => matches_command(item, query.as_ref()),
            })
            .collect();
        let active_id = resolve_active_id(&filtered, stored_active.as_ref());

        let palette_id = self.id.clone();
        let state_for_keys = self.state.clone();
        let state_for_close = self.state.clone();
        let filtered_for_keys = Rc::new(filtered.clone());
        let on_select = self.on_select;
        let on_select_for_keys = on_select.clone();
        let close_on_select = self.close_on_select;
        let max_results_height = self.max_results_height;
        let container_override = self.colors.container;
        let footer = self.footer;
        let row_min_h = px(density.interactive_size(48.0));
        let row_radius = CornerRole::Medium.shape().radii(48., 48., direction)[0];
        let badge_radius = CornerRole::Small.shape().radii(24., 24., direction)[0];
        let input_colors = TextFieldColors {
            unfocused: TextFieldColorOverrides {
                container: Some(transparent_black()),
                indicator: Some(transparent_black()),
                ..Default::default()
            },
            focused: TextFieldColorOverrides {
                container: Some(transparent_black()),
                indicator: Some(transparent_black()),
                ..Default::default()
            },
            hovered_indicator: Some(transparent_black()),
            ..Default::default()
        };

        let mut dialog =
            Dialog::unstructured(self.id.clone(), &dialog_state, accessible_label.clone())
                .width(self.width)
                .layer(self.layer)
                .properties(self.properties)
                .scrim_opacity(self.scrim_opacity)
                .when_some(self.colors.scrim, Dialog::scrim_color);

        if let Some(on_dismiss_request) = self.on_dismiss_request {
            dialog = dialog.on_dismiss_request(move |reason, window, cx| {
                on_dismiss_request(reason, window, cx)
            });
        }

        let caller_on_closed = self.on_closed;
        dialog = dialog.on_closed(move |reason, window, cx| {
            state_for_close.update(cx, |state, cx| state.emit_close(reason, cx));
            if let Some(on_closed) = &caller_on_closed {
                on_closed(reason, window, cx);
            }
        });

        let state_for_rows = self.state.clone();
        dialog.body_content(move |_, dismiss| {
            let input_id = ElementId::from((palette_id.clone(), "input"));
            let close_id = ElementId::from((palette_id.clone(), "close"));
            let results_id = ElementId::from((palette_id.clone(), "results"));
            let empty_id = ElementId::from((palette_id.clone(), "empty"));

            let header = div()
                .id((palette_id.clone(), "header"))
                .test_support()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .px(px(12.))
                .py(px(6.))
                .when(rtl, Styled::flex_row_reverse)
                .child(
                    div()
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .pl(px(4.))
                        .text_color(on_surface_variant)
                        .child(Icon::new(IconName::Search).size(px(20.))),
                )
                .child(
                    div().flex_1().min_w_0().child(
                        TextField::new(input_id, accessible_label.clone(), &editor)
                            .show_label(false)
                            .placeholder_content(
                                placeholder.clone(),
                                Text::new(placeholder)
                                    .role(TypeRole::BodyLarge)
                                    .color_override(on_surface_variant),
                            )
                            .clearable(true)
                            .indicator_width(px(0.), px(0.))
                            .colors(input_colors),
                    ),
                )
                .child(
                    IconButton::standard(close_id, close_label, Icon::new(IconName::Close))
                        .size(ControlSize::ExtraSmall)
                        .on_activate(move |window, cx| dismiss.activate(window, cx)),
                );

            let results_body = if filtered.is_empty() {
                div()
                    .id(empty_id)
                    .test_support()
                    .role(Role::Status)
                    .aria_label(empty_label.clone())
                    .w_full()
                    .py(px(32.))
                    .px(px(16.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        Text::new(empty_label)
                            .role(TypeRole::BodyMedium)
                            .color_override(on_surface_variant),
                    )
                    .into_any_element()
            } else {
                let mut list = div()
                    .id((palette_id.clone(), "list"))
                    .test_support()
                    .role(Role::ListBox)
                    .w_full()
                    .p(px(8.))
                    .flex()
                    .flex_col()
                    .gap(px(2.));

                let mut last_group: Option<SharedString> = None;
                for item in filtered {
                    if let Some(group) = item.group_label()
                        && last_group.as_ref() != Some(group)
                    {
                        last_group = Some(group.clone());
                        list = list.child(
                            div()
                                .w_full()
                                .px(px(12.))
                                .pt(px(8.))
                                .pb(px(4.))
                                .flex()
                                .when(rtl, Styled::justify_end)
                                .child(
                                    Text::new(group.clone())
                                        .role(TypeRole::LabelMedium)
                                        .color_override(group_heading_color),
                                ),
                        );
                    }

                    let disabled = item.is_disabled();
                    let is_active = !disabled && active_id.as_ref() == Some(item.id());
                    let row_id = ElementId::from((palette_id.clone(), item.id().clone()));
                    let fg = if disabled {
                        disabled_fg
                    } else if is_active {
                        active_fg
                    } else {
                        on_surface
                    };
                    let sub_fg = if disabled {
                        disabled_fg
                    } else if is_active {
                        active_fg.opacity(0.84)
                    } else {
                        on_surface_variant
                    };
                    let icon_fg = if disabled {
                        disabled_fg
                    } else if is_active {
                        active_fg
                    } else {
                        on_surface_variant
                    };

                    let mut row = BaseButton::new(row_id)
                        .role(Role::ListBoxOption)
                        .accessibility_label(item.label().clone())
                        .disabled(disabled)
                        .focusable(false)
                        .aria_selected(is_active)
                        .w_full()
                        .min_h(row_min_h)
                        .px(px(12.))
                        .py(px(8.))
                        .rounded(px(row_radius))
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_start()
                        .gap(px(12.))
                        .when(rtl, Styled::flex_row_reverse)
                        .when(is_active, |this| this.bg(active_bg))
                        .when(!disabled && !is_active, |this| {
                            this.cursor_pointer()
                                .hover(|style| style.bg(on_surface.opacity(0.08)))
                        })
                        .when(!disabled && is_active, Styled::cursor_pointer)
                        .children(item.icon_name().map(|icon| {
                            div()
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(icon_fg)
                                .child(Icon::new(icon).size(px(20.)))
                        }))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .gap(px(2.))
                                .when(rtl, Styled::items_end)
                                .child(
                                    Text::new(item.label().clone())
                                        .role(TypeRole::BodyLarge)
                                        .emphasized(is_active)
                                        .color_override(fg)
                                        .wrap(),
                                )
                                .children(item.supporting_text().map(|supporting| {
                                    Text::new(supporting.clone())
                                        .role(TypeRole::BodySmall)
                                        .color_override(sub_fg)
                                        .wrap()
                                })),
                        )
                        .children(item.shortcut_label().map(|shortcut| {
                            div()
                                .flex_none()
                                .px(px(8.))
                                .py(px(3.))
                                .rounded(px(badge_radius))
                                .bg(shortcut_bg)
                                .border_1()
                                .border_color(outline_variant)
                                .child(
                                    Text::new(shortcut.clone())
                                        .role(TypeRole::LabelSmall)
                                        .color_override(if disabled {
                                            disabled_fg
                                        } else {
                                            shortcut_fg
                                        }),
                                )
                        }));

                    if !disabled {
                        let click_item = item.clone();
                        let click_state = state_for_rows.clone();
                        let click_select = on_select.clone();
                        let a11y_item = item;
                        let a11y_state = state_for_rows.clone();
                        let a11y_select = on_select.clone();
                        row = row
                            .on_click(move |_, window, cx| {
                                execute_command(
                                    click_item.clone(),
                                    &click_state,
                                    click_select.as_ref(),
                                    close_on_select,
                                    window,
                                    cx,
                                );
                                cx.stop_propagation();
                            })
                            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                execute_command(
                                    a11y_item.clone(),
                                    &a11y_state,
                                    a11y_select.as_ref(),
                                    close_on_select,
                                    window,
                                    cx,
                                );
                            });
                    }

                    list = list.child(row);
                }

                isolated_scroll(results_id, &scroll)
                    .test_support()
                    .w_full()
                    .max_h(max_results_height)
                    .child(list)
                    .into_any_element()
            };

            let mut surface = Surface::new(ColorRole::SurfaceContainerHigh)
                .w_full()
                .flex()
                .flex_col()
                .shape(surface_shape)
                .shadow_elevation(Elevation::Level3)
                .child(header)
                .child(Divider::horizontal().color_override(divider_color))
                .child(results_body)
                .children(footer.map(|footer| {
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .child(Divider::horizontal().color_override(divider_color))
                        .child(div().w_full().px(px(16.)).py(px(8.)).child(footer))
                }));
            if let Some(container) = container_override {
                surface = surface.container_color(container);
            }

            let step_action = {
                let editor = editor.clone();
                let filtered = filtered_for_keys.clone();
                let state = state_for_keys.clone();
                move |step: NavStep, window: &mut Window, cx: &mut App| -> bool {
                    if editor.update(cx, |input, cx| {
                        input.marked_text_range(window, cx).is_some()
                    }) {
                        return false;
                    }
                    let next = step_active_id(&filtered, state.read(cx).active_command_id(), step);
                    state.update(cx, |state, cx| {
                        state.set_active_command(next, cx);
                    });
                    window.refresh();
                    true
                }
            };
            let execute_active = {
                let editor = editor.clone();
                let filtered = filtered_for_keys.clone();
                let state = state_for_keys.clone();
                let on_select = on_select_for_keys.clone();
                move |window: &mut Window, cx: &mut App| -> bool {
                    if editor.update(cx, |input, cx| {
                        input.marked_text_range(window, cx).is_some()
                    }) {
                        return false;
                    }
                    let active = resolve_active_id(&filtered, state.read(cx).active_command_id());
                    if let Some(active) = active
                        && let Some(item) = filtered
                            .iter()
                            .find(|item| !item.is_disabled() && item.id() == &active)
                            .cloned()
                    {
                        execute_command(
                            item,
                            &state,
                            on_select.as_ref(),
                            close_on_select,
                            window,
                            cx,
                        );
                    }
                    true
                }
            };

            let step_down = step_action.clone();
            let step_up = step_action.clone();
            let step_home = step_action.clone();
            let step_end = step_action.clone();
            let exec_action = execute_active.clone();

            div()
                .id((palette_id, "shell"))
                .test_support()
                .w_full()
                .capture_action(move |_: &MoveDown, window, cx| {
                    if step_down(NavStep::Next, window, cx) {
                        cx.stop_propagation();
                    }
                })
                .capture_action(move |_: &MoveUp, window, cx| {
                    if step_up(NavStep::Previous, window, cx) {
                        cx.stop_propagation();
                    }
                })
                .capture_action(move |_: &MoveHome, window, cx| {
                    if step_home(NavStep::First, window, cx) {
                        cx.stop_propagation();
                    }
                })
                .capture_action(move |_: &MoveEnd, window, cx| {
                    if step_end(NavStep::Last, window, cx) {
                        cx.stop_propagation();
                    }
                })
                .capture_action(move |_: &InputEnter, window, cx| {
                    if exec_action(window, cx) {
                        cx.stop_propagation();
                    }
                })
                .capture_key_down(move |event, window, cx| {
                    if event.keystroke.modifiers.modified() {
                        return;
                    }
                    let handled = match event.keystroke.key.as_str() {
                        "down" => step_action(NavStep::Next, window, cx),
                        "up" => step_action(NavStep::Previous, window, cx),
                        "home" => step_action(NavStep::First, window, cx),
                        "end" => step_action(NavStep::Last, window, cx),
                        "enter" => execute_active(window, cx),
                        _ => false,
                    };
                    if handled {
                        cx.stop_propagation();
                    }
                })
                .child(surface)
        })
    }
}

use gpui::{
    AccessibleAction, AnyElement, App, Axis, ElementId, FocusHandle, Hsla, InteractiveElement,
    IntoElement, ParentElement, Pixels, RenderOnce, Role, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Button as BaseButton, TestSupportExt};
use gpui_material_foundation::{
    InteractionSource, Surface, color,
    control_feedback::{activate_feedback, feedback_target, ripple_state},
    current_material_theme, move_roving_focus,
};
use gpui_material_theme::{ColorRole, CornerShape, Direction, StateLayer, TypeRole};

use crate::{
    Badge, Divider, Sheet, SheetColors, SheetEdge, SheetMode, Text,
    navigation::items::{NavigationItem, Select, row_state_for},
};

use super::{
    NavigationDrawer, NavigationDrawerColors, NavigationDrawerEntry, NavigationDrawerMode,
};

#[derive(Default)]
struct DrawerScrollState {
    scroll: ScrollHandle,
}

#[derive(Clone, Copy)]
struct ResolvedDrawerColors {
    container: Option<Hsla>,
    content: Option<Hsla>,
    header: Hsla,
    divider: Option<Hsla>,
    active_indicator: Hsla,
    active_content: Hsla,
    inactive_content: Hsla,
    disabled_content: Hsla,
    state_layer: Hsla,
    ring: Hsla,
}

impl NavigationDrawerColors {
    fn resolve(self, window: &Window, cx: &App) -> ResolvedDrawerColors {
        use ColorRole::*;
        let theme = current_material_theme(window, cx);
        let role = |r| color(theme.colors.color(r));
        ResolvedDrawerColors {
            container: self.container,
            content: self.content,
            header: self.header.unwrap_or_else(|| role(OnSurfaceVariant)),
            divider: self.divider,
            active_indicator: self
                .active_indicator
                .unwrap_or_else(|| role(SecondaryContainer)),
            active_content: self
                .active_content
                .unwrap_or_else(|| role(OnSecondaryContainer)),
            inactive_content: self
                .inactive_content
                .or(self.content)
                .unwrap_or_else(|| role(OnSurfaceVariant)),
            disabled_content: role(OnSurface).opacity(0.38),
            state_layer: role(OnSurface),
            ring: role(Secondary),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_destination(
    collection_id: &ElementId,
    mut item: NavigationItem,
    selected: bool,
    position: usize,
    total: usize,
    focus: &FocusHandle,
    is_tab_stop: bool,
    on_select: Option<Select>,
    colors: &ResolvedDrawerColors,
    row_height: Pixels,
    font_scale: f32,
    rtl: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let disabled = item.disabled;
    let id: ElementId = (collection_id.clone(), item.id.clone()).into();
    let description = item.badge.as_mut().and_then(Badge::take_accessible_label);

    let indicator_bg = if selected {
        item.colors
            .selected_indicator
            .unwrap_or(colors.active_indicator)
    } else {
        gpui::transparent_black()
    };
    let icon_color = if disabled {
        item.colors.disabled_icon.unwrap_or(colors.disabled_content)
    } else if selected {
        item.colors.selected_icon.unwrap_or(colors.active_content)
    } else {
        item.colors
            .unselected_icon
            .unwrap_or(colors.inactive_content)
    };
    let label_color = if disabled {
        item.colors
            .disabled_label
            .unwrap_or(colors.disabled_content)
    } else if selected {
        item.colors
            .selected_label_start
            .or(item.colors.selected_label_top)
            .unwrap_or(colors.active_content)
    } else {
        item.colors
            .unselected_label
            .unwrap_or(colors.inactive_content)
    };

    let feedback = item.interaction_source.as_ref().map(|source| {
        let feedback = ripple_state(&id, !disabled, window, cx);
        let tracker = InteractionSource::track(source, &id, window, cx);
        feedback
            .borrow_mut()
            .track(!disabled, focus, Some(tracker), window, cx);
        feedback
    });

    let label_elem = item.label_content.unwrap_or_else(|| {
        Text::new(item.label.clone())
            .id((id.clone(), "drawer-label"))
            .role(if selected {
                TypeRole::LabelLarge
            } else {
                TypeRole::TitleSmall
            })
            .font_scale(font_scale)
            .into_any_element()
    });

    let icon_elem = item.icon.map(|icon| {
        div()
            .flex_none()
            .text_size(px(24.))
            .text_color(icon_color)
            .child(icon.size(px(24.)).inherit_color())
    });

    let badge_elem = item.badge.map(|badge| div().flex_none().child(badge));

    let row = div()
        .w_full()
        .min_h(row_height)
        .px(px(16.))
        .rounded_full()
        .bg(indicator_bg)
        .flex()
        .items_center()
        .gap(px(12.))
        .when(rtl, Styled::flex_row_reverse)
        .children(icon_elem)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .flex()
                .when(rtl, Styled::justify_end)
                .text_color(label_color)
                .child(label_elem),
        )
        .children(badge_elem);

    let state_layer = colors.state_layer;
    let ring = colors.ring;
    let item_id = item.id.clone();

    BaseButton::new(id)
        .role(Role::ListBoxOption)
        .accessibility_label(item.label)
        .when_some(description, |button, desc| button.aria_description(desc))
        .aria_selected(selected)
        .aria_position_in_set(position)
        .aria_size_of_set(total)
        .track_focus(focus)
        .tab_stop(is_tab_stop)
        .disabled(disabled)
        .w_full()
        .rounded_full()
        .border_2()
        .border_color(gpui::transparent_black())
        .when(!disabled, |button| {
            let activate = on_select.clone();
            let click_id = item_id.clone();
            let click_feedback = feedback.clone();
            let a11y_activate = on_select;
            let a11y_id = item_id;
            button
                .hover(move |style| style.bg(state_layer.opacity(StateLayer::Hover.opacity())))
                .active(move |style| style.bg(state_layer.opacity(StateLayer::Pressed.opacity())))
                .focus_visible(move |style| style.border_color(ring))
                .when_some(feedback, |button, fb| {
                    feedback_target(button, fb, true, Some(focus.clone()))
                })
                .when_some(activate, |button, handler| {
                    button.on_click(move |event, window, cx| {
                        cx.stop_propagation();
                        if let Some(fb) = &click_feedback {
                            activate_feedback(event, fb, window, cx);
                        }
                        handler(&click_id, window, cx);
                    })
                })
                .when_some(a11y_activate, |button, handler| {
                    button.on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                        handler(&a11y_id, window, cx);
                    })
                })
        })
        .child(row)
        .into_any_element()
}

impl RenderOnce for NavigationDrawer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let rtl = direction == Direction::Rtl;
        let row_height = px(theme.layout.density.interactive_size(56.));
        let viewport_width = window.viewport_size().width;
        let resolved_width = self.width.min(viewport_width.max(px(0.)));
        let colors = self.colors.resolve(window, cx);

        let (
            collection_id,
            nav_label,
            initial_items,
            selected,
            on_select,
            font_scale,
            retained_state,
        ) = self.items.into_drawer_parts();

        let mut combined: Vec<NavigationDrawerEntry> =
            Vec::with_capacity(initial_items.len() + self.entries.len());
        for item in initial_items {
            if self.dividers_before.contains(&item.id) {
                combined.push(NavigationDrawerEntry::Divider);
            }
            combined.push(NavigationDrawerEntry::item(item));
        }
        for entry in self.entries {
            if let NavigationDrawerEntry::Item(ref item) = entry
                && self.dividers_before.contains(&item.id)
            {
                combined.push(NavigationDrawerEntry::Divider);
            }
            combined.push(entry);
        }

        let nav_state = retained_state.unwrap_or_else(|| row_state_for(&collection_id, window, cx));
        let scroll = window
            .use_keyed_state((self.id.clone(), "navigation-drawer-scroll"), cx, |_, _| {
                DrawerScrollState::default()
            })
            .read(cx)
            .scroll
            .clone();

        let item_ids: Vec<(SharedString, bool)> = combined
            .iter()
            .filter_map(|entry| match entry {
                NavigationDrawerEntry::Item(item) => Some((item.id.clone(), !item.disabled)),
                _ => None,
            })
            .collect();
        let ordered_ids: Vec<SharedString> = item_ids.iter().map(|(id, _)| id.clone()).collect();
        let handles = nav_state.update(cx, |state, cx| state.sync_focus_map(&ordered_ids, cx));

        let enabled_items: Vec<(SharedString, FocusHandle)> = item_ids
            .iter()
            .filter(|(_, enabled)| *enabled)
            .map(|(id, _)| (id.clone(), handles[id].clone()))
            .collect();

        let tab_stop = enabled_items
            .iter()
            .find(|(_, handle)| handle.is_focused(window))
            .map(|(id, _)| id.clone())
            .or_else(|| {
                selected
                    .as_ref()
                    .filter(|sel| enabled_items.iter().any(|(id, _)| id == *sel))
                    .cloned()
            })
            .or_else(|| enabled_items.first().map(|(id, _)| id.clone()));

        let nav_handles: Vec<FocusHandle> =
            item_ids.iter().map(|(id, _)| handles[id].clone()).collect();
        let nav_enabled: Vec<bool> = item_ids.iter().map(|(_, enabled)| *enabled).collect();
        let total = item_ids.len();

        let mut rows: Vec<AnyElement> = Vec::with_capacity(combined.len());
        let mut dest_row_indices: Vec<usize> = Vec::with_capacity(total);
        let mut destination_index = 0usize;
        for (entry_index, entry) in combined.into_iter().enumerate() {
            match entry {
                NavigationDrawerEntry::Item(item) => {
                    let item = *item;
                    destination_index += 1;
                    let is_selected = selected.as_ref() == Some(&item.id);
                    let handle = &handles[&item.id];
                    let is_tab_stop = tab_stop.as_ref() == Some(&item.id);
                    dest_row_indices.push(rows.len());
                    rows.push(render_destination(
                        &collection_id,
                        item,
                        is_selected,
                        destination_index,
                        total,
                        handle,
                        is_tab_stop,
                        on_select.clone(),
                        &colors,
                        row_height,
                        font_scale,
                        rtl,
                        window,
                        cx,
                    ));
                }
                NavigationDrawerEntry::Header(title) => {
                    rows.push(
                        div()
                            .id((self.id.clone(), format!("drawer-header-{entry_index}")))
                            .w_full()
                            .px(px(16.))
                            .pt(px(16.))
                            .pb(px(8.))
                            .flex()
                            .when(rtl, Styled::justify_end)
                            .text_color(colors.header)
                            .child(
                                Text::new(title)
                                    .id((
                                        self.id.clone(),
                                        format!("drawer-header-text-{entry_index}"),
                                    ))
                                    .role(TypeRole::TitleSmall)
                                    .font_scale(font_scale),
                            )
                            .into_any_element(),
                    );
                }
                NavigationDrawerEntry::Divider => {
                    let mut divider = Divider::horizontal().inset(px(16.), px(16.));
                    if let Some(color) = colors.divider {
                        divider = divider.color_override(color);
                    }
                    rows.push(
                        div()
                            .id((self.id.clone(), format!("drawer-divider-{entry_index}")))
                            .w_full()
                            .py(px(8.))
                            .child(divider)
                            .into_any_element(),
                    );
                }
            }
        }

        let is_modal = self.mode == NavigationDrawerMode::Modal;
        let title = self.headline.clone().unwrap_or_else(|| nav_label.clone());

        let key_scroll = scroll.clone();
        let destination_list = div()
            .id((self.id.clone(), "drawer-destinations"))
            .test_support()
            .role(Role::ListBox)
            .aria_label(nav_label)
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .px(px(12.))
            .py(px(8.))
            .flex()
            .flex_col()
            .gap(px(2.))
            .on_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                if !nav_handles.iter().any(|handle| handle.is_focused(window)) {
                    return;
                }
                let items: Vec<_> = nav_handles
                    .iter()
                    .zip(&nav_enabled)
                    .map(|(handle, enabled)| (handle, *enabled))
                    .collect();
                if let Some(target) = move_roving_focus(
                    &items,
                    &event.keystroke.key,
                    Axis::Vertical,
                    direction,
                    window,
                    cx,
                ) {
                    if let Some(&row_index) = dest_row_indices.get(target) {
                        key_scroll.scroll_to_item(row_index);
                    }
                    cx.stop_propagation();
                    window.refresh();
                }
            })
            .children(rows);

        let panel = div()
            .id((self.id.clone(), "drawer-panel"))
            .test_support()
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .when(!is_modal, |panel| {
                panel.children(self.headline.map(|headline| {
                    div()
                        .id((self.id.clone(), "drawer-headline"))
                        .px(px(28.))
                        .pt(px(18.))
                        .pb(px(10.))
                        .flex()
                        .when(rtl, Styled::justify_end)
                        .text_color(colors.header)
                        .child(
                            Text::new(headline)
                                .id((self.id.clone(), "drawer-headline-text"))
                                .role(TypeRole::TitleSmall)
                                .font_scale(font_scale),
                        )
                }))
            })
            .children(self.header.map(|header| {
                div()
                    .id((self.id.clone(), "drawer-custom-header"))
                    .flex_none()
                    .px(px(16.))
                    .pt(px(12.))
                    .pb(px(8.))
                    .child(header)
            }))
            .child(destination_list)
            .children(self.footer.map(|footer| {
                div()
                    .id((self.id.clone(), "drawer-footer"))
                    .flex_none()
                    .px(px(16.))
                    .py(px(12.))
                    .child(footer)
            }));

        if is_modal {
            let Some(state) = self.modal_state else {
                return div().id(self.id).into_any_element();
            };
            let open = state.read(cx).is_open();
            let presence_id = (self.id.clone(), "drawer-modal").into();
            let presence = crate::overlay::presence(&presence_id, open, 240, window, cx);
            if !presence.should_render() {
                return div().id(self.id).into_any_element();
            }
            return Sheet::new(self.id, &state, title, panel)
                .mode(SheetMode::Modal)
                .edge(SheetEdge::Start)
                .extent(resolved_width)
                .properties(self.modal_properties)
                .sampled_presence(presence)
                .colors(SheetColors {
                    container: self.colors.container,
                    content: self.colors.content,
                    header: self.colors.header,
                    divider: self.colors.divider,
                    scrim: self.colors.scrim,
                })
                .when_some(self.colors.scrim_opacity, Sheet::scrim_opacity)
                .when_some(self.shape, Sheet::shape)
                .when_some(self.on_dismiss_request, |sheet, cb| {
                    sheet.on_dismiss_request(move |reason, window, cx| cb(reason, window, cx))
                })
                .when_some(self.on_closed, |sheet, cb| {
                    sheet.on_closed(move |reason, window, cx| cb(reason, window, cx))
                })
                .into_any_element();
        }

        let default_shape = self.shape.unwrap_or(CornerShape::Radii([0., 16., 16., 0.]));
        let surface = Surface::new(ColorRole::SurfaceContainerLow)
            .shape(default_shape)
            .when_some(colors.container, Surface::container_color)
            .when_some(colors.content, Surface::content_color)
            .size_full()
            .overflow_hidden()
            .child(panel);

        div()
            .id(self.id)
            .test_support()
            .role(Role::Navigation)
            .aria_label(title)
            .w(resolved_width)
            .h_full()
            .flex_none()
            .child(surface)
            .into_any_element()
    }
}

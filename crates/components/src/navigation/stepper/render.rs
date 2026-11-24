use std::collections::HashMap;

use gpui::{
    AccessibleAction, AnyElement, App, Axis, ElementId, FocusHandle, Hsla, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Button as BaseButton, TestSupportExt};
use gpui_material_foundation::{color, current_material_theme, move_roving_focus, text_style};
use gpui_material_theme::{ColorRole, Direction, StateLayer, TypeRole, TypographyEmphasis};

use crate::{Icon, IconName, Text};

use super::{
    StepItem, StepSelect, StepStatus, Stepper, StepperColors, StepperOrientation,
    resolve_step_status, step_is_interactive,
};

#[derive(Default)]
struct StepperFocusState {
    handles: HashMap<SharedString, FocusHandle>,
}

#[derive(Clone, Copy)]
struct ResolvedColors {
    active_container: Hsla,
    active_content: Hsla,
    completed_container: Hsla,
    completed_content: Hsla,
    upcoming_container: Hsla,
    upcoming_content: Hsla,
    error_container: Hsla,
    error_content: Hsla,
    disabled_container: Hsla,
    disabled_content: Hsla,
    connector: Hsla,
    active_connector: Hsla,
    label: Hsla,
    muted_label: Hsla,
    error_label: Hsla,
    supporting: Hsla,
    state_layer: Hsla,
    ring: Hsla,
}

impl StepperColors {
    fn resolve(self, window: &Window, cx: &App) -> ResolvedColors {
        use ColorRole::*;
        let theme = current_material_theme(window, cx);
        let role = |r| color(theme.colors.color(r));
        let on_surface = role(OnSurface);
        ResolvedColors {
            active_container: self.active_container.unwrap_or_else(|| role(Primary)),
            active_content: self.active_content.unwrap_or_else(|| role(OnPrimary)),
            completed_container: self.completed_container.unwrap_or_else(|| role(Primary)),
            completed_content: self.completed_content.unwrap_or_else(|| role(OnPrimary)),
            upcoming_container: self
                .upcoming_container
                .unwrap_or_else(|| role(SurfaceContainerHighest)),
            upcoming_content: self
                .upcoming_content
                .unwrap_or_else(|| role(OnSurfaceVariant)),
            error_container: self.error_container.unwrap_or_else(|| role(Error)),
            error_content: self.error_content.unwrap_or_else(|| role(OnError)),
            disabled_container: on_surface.opacity(0.12),
            disabled_content: on_surface.opacity(0.38),
            connector: self.connector.unwrap_or_else(|| role(OutlineVariant)),
            active_connector: self.active_connector.unwrap_or_else(|| role(Primary)),
            label: self.label.unwrap_or(on_surface),
            muted_label: role(OnSurfaceVariant),
            error_label: role(Error),
            supporting: self.supporting.unwrap_or_else(|| role(OnSurfaceVariant)),
            state_layer: on_surface,
            ring: role(Secondary),
        }
    }
}

fn badge_colors(status: StepStatus, colors: &ResolvedColors) -> (Hsla, Hsla) {
    match status {
        StepStatus::Active => (colors.active_container, colors.active_content),
        StepStatus::Completed => (colors.completed_container, colors.completed_content),
        StepStatus::Error => (colors.error_container, colors.error_content),
        StepStatus::Upcoming => (colors.upcoming_container, colors.upcoming_content),
        StepStatus::Disabled => (colors.disabled_container, colors.disabled_content),
    }
}

fn label_color(status: StepStatus, colors: &ResolvedColors) -> Hsla {
    match status {
        StepStatus::Active | StepStatus::Completed => colors.label,
        StepStatus::Error => colors.error_label,
        StepStatus::Upcoming => colors.muted_label,
        StepStatus::Disabled => colors.disabled_content,
    }
}

fn render_badge(
    index: usize,
    status: StepStatus,
    custom_icon: Option<AnyElement>,
    colors: &ResolvedColors,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let (bg, fg) = badge_colors(status, colors);
    let glyph = if let Some(icon) = custom_icon {
        div().text_size(px(16.)).child(icon).into_any_element()
    } else {
        match status {
            StepStatus::Completed => Icon::new(IconName::Check).size(px(16.)).into_any_element(),
            StepStatus::Error => Icon::new(IconName::Error).size(px(16.)).into_any_element(),
            StepStatus::Active | StepStatus::Upcoming | StepStatus::Disabled => text_style(
                div(),
                TypeRole::LabelMedium.style(TypographyEmphasis::Baseline),
                1.,
                window,
                cx,
            )
            .child((index + 1).to_string())
            .into_any_element(),
        }
    };

    div()
        .flex_none()
        .size(px(28.))
        .rounded_full()
        .bg(bg)
        .text_color(fg)
        .flex()
        .items_center()
        .justify_center()
        .child(glyph)
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_step_header(
    stepper_id: &ElementId,
    index: usize,
    count: usize,
    item_id: SharedString,
    label: SharedString,
    supporting: Option<SharedString>,
    status: StepStatus,
    custom_icon: Option<AnyElement>,
    interactive: bool,
    focus: FocusHandle,
    is_tab_stop: bool,
    on_select: Option<StepSelect>,
    colors: &ResolvedColors,
    rtl: bool,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let badge = render_badge(index, status, custom_icon, colors, window, cx);
    let title_color = label_color(status, colors);
    let supporting_color = if status == StepStatus::Disabled {
        colors.disabled_content
    } else if status == StepStatus::Error {
        colors.error_label
    } else {
        colors.supporting
    };

    let step_elem_id: ElementId = (stepper_id.clone(), item_id).into();

    let labels = div()
        .flex()
        .flex_col()
        .min_w_0()
        .when(rtl, Styled::items_end)
        .child(
            div().text_color(title_color).child(
                Text::new(label.clone())
                    .id((step_elem_id.clone(), "label"))
                    .role(if status == StepStatus::Active {
                        TypeRole::TitleSmall
                    } else {
                        TypeRole::BodyMedium
                    })
                    .wrap(),
            ),
        )
        .children(supporting.map(|text| {
            div().text_color(supporting_color).child(
                Text::new(text)
                    .id((step_elem_id.clone(), "supporting"))
                    .role(TypeRole::BodySmall)
                    .wrap(),
            )
        }));

    let row = div()
        .flex()
        .items_center()
        .gap(px(10.))
        .px(px(6.))
        .py(px(6.))
        .rounded(px(12.))
        .when(rtl, Styled::flex_row_reverse)
        .child(badge)
        .child(labels);

    let aria_name: SharedString = format!("{label}, {}", status.accessible_label()).into();

    if interactive {
        let state_layer = colors.state_layer;
        let ring = colors.ring;
        let click_select = on_select.clone();
        let a11y_select = on_select;
        BaseButton::new(step_elem_id)
            .role(Role::Button)
            .accessibility_label(aria_name)
            .aria_selected(status == StepStatus::Active)
            .aria_position_in_set(index + 1)
            .aria_size_of_set(count)
            .track_focus(&focus)
            .tab_stop(is_tab_stop)
            .rounded(px(12.))
            .border_2()
            .border_color(gpui::transparent_black())
            .hover(move |style| style.bg(state_layer.opacity(StateLayer::Hover.opacity())))
            .active(move |style| style.bg(state_layer.opacity(StateLayer::Pressed.opacity())))
            .focus_visible(move |style| style.border_color(ring))
            .when_some(click_select, |button, handler| {
                button.on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    handler(index, window, cx);
                })
            })
            .when_some(a11y_select, |button, handler| {
                button.on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                    handler(index, window, cx);
                })
            })
            .child(row)
            .into_any_element()
    } else {
        div()
            .id(step_elem_id)
            .test_support()
            .role(Role::ListItem)
            .aria_label(aria_name)
            .aria_selected(status == StepStatus::Active)
            .aria_position_in_set(index + 1)
            .aria_size_of_set(count)
            .child(row)
            .into_any_element()
    }
}

impl RenderOnce for Stepper {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let rtl = direction == Direction::Rtl;
        let colors = self.colors.resolve(window, cx);
        let count = self.steps.len();

        let focus_state = window.use_keyed_state((self.id.clone(), "stepper-focus"), cx, |_, _| {
            StepperFocusState::default()
        });
        let handles: Vec<FocusHandle> = focus_state.update(cx, |state, cx| {
            let mut previous = std::mem::take(&mut state.handles);
            let mut ordered = Vec::with_capacity(self.steps.len());
            for step in &self.steps {
                let handle = previous
                    .remove(&step.id)
                    .unwrap_or_else(|| cx.focus_handle());
                ordered.push(handle.clone());
                state.handles.insert(step.id.clone(), handle);
            }
            ordered
        });

        let has_on_select = self.on_select.is_some();
        let resolved: Vec<(StepStatus, bool)> = self
            .steps
            .iter()
            .enumerate()
            .map(|(index, step)| {
                let status = resolve_step_status(step.status, index, self.active_step);
                let interactive =
                    step_is_interactive(status, step.editable, self.non_linear, has_on_select);
                (status, interactive)
            })
            .collect();

        let tab_stop = handles
            .iter()
            .enumerate()
            .position(|(ix, handle)| resolved[ix].1 && handle.is_focused(window))
            .or_else(|| {
                resolved
                    .iter()
                    .position(|(status, interactive)| *interactive && *status == StepStatus::Active)
            })
            .or_else(|| resolved.iter().position(|(_, interactive)| *interactive));

        let axis = match self.orientation {
            StepperOrientation::Horizontal => Axis::Horizontal,
            StepperOrientation::Vertical => Axis::Vertical,
        };
        let nav_handles = handles.clone();
        let nav_enabled: Vec<bool> = resolved
            .iter()
            .map(|(_, interactive)| *interactive)
            .collect();

        let container = div()
            .id(self.id.clone())
            .test_support()
            .role(Role::List)
            .aria_label(self.accessible_label)
            .w_full()
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
                if move_roving_focus(&items, &event.keystroke.key, axis, direction, window, cx)
                    .is_some()
                {
                    cx.stop_propagation();
                    window.refresh();
                }
            });

        match self.orientation {
            StepperOrientation::Horizontal => {
                let mut children: Vec<AnyElement> = Vec::with_capacity(count.saturating_mul(2));
                for (index, step) in self.steps.into_iter().enumerate() {
                    let StepItem {
                        id: item_id,
                        label,
                        supporting,
                        icon,
                        ..
                    } = step;
                    let (status, interactive) = resolved[index];
                    let header = render_step_header(
                        &self.id,
                        index,
                        count,
                        item_id,
                        label,
                        supporting,
                        status,
                        icon,
                        interactive,
                        handles[index].clone(),
                        tab_stop == Some(index),
                        self.on_select.clone(),
                        &colors,
                        rtl,
                        window,
                        cx,
                    );
                    children.push(header);
                    if index + 1 < count {
                        let line_color = if status == StepStatus::Completed {
                            colors.active_connector
                        } else {
                            colors.connector
                        };
                        children.push(
                            div()
                                .id((self.id.clone(), format!("connector-{index}")))
                                .flex_1()
                                .min_w(px(16.))
                                .h(px(2.))
                                .mx(px(4.))
                                .rounded_full()
                                .bg(line_color)
                                .into_any_element(),
                        );
                    }
                }
                container
                    .flex()
                    .items_center()
                    .when(rtl, Styled::flex_row_reverse)
                    .children(children)
            }
            StepperOrientation::Vertical => {
                let mut rows: Vec<AnyElement> = Vec::with_capacity(count);
                for (index, step) in self.steps.into_iter().enumerate() {
                    let StepItem {
                        id: item_id,
                        label,
                        supporting,
                        icon,
                        content,
                        ..
                    } = step;
                    let (status, interactive) = resolved[index];
                    let show_content = content.is_some()
                        && (status == StepStatus::Active || self.active_step == Some(index));
                    let has_connector = index + 1 < count;
                    let header = render_step_header(
                        &self.id,
                        index,
                        count,
                        item_id.clone(),
                        label,
                        supporting,
                        status,
                        icon,
                        interactive,
                        handles[index].clone(),
                        tab_stop == Some(index),
                        self.on_select.clone(),
                        &colors,
                        rtl,
                        window,
                        cx,
                    );
                    let line_color = if status == StepStatus::Completed {
                        colors.active_connector
                    } else {
                        colors.connector
                    };
                    let body = (show_content || has_connector).then(|| {
                        let rail = div()
                            .w(px(40.))
                            .flex_none()
                            .flex()
                            .justify_center()
                            .py(px(2.))
                            .when(has_connector, |rail| {
                                rail.child(
                                    div()
                                        .w(px(2.))
                                        .h_full()
                                        .min_h(px(20.))
                                        .rounded_full()
                                        .bg(line_color),
                                )
                            });
                        div()
                            .flex()
                            .w_full()
                            .when(rtl, Styled::flex_row_reverse)
                            .child(rail)
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .when(show_content, |slot| slot.py(px(8.)).children(content)),
                            )
                    });
                    rows.push(
                        div()
                            .id((self.id.clone(), format!("step-row-{index}")))
                            .flex()
                            .flex_col()
                            .w_full()
                            .child(header)
                            .children(body)
                            .into_any_element(),
                    );
                }
                container.flex().flex_col().children(rows)
            }
        }
    }
}

use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, ElementId, Entity, FocusHandle, Hsla,
    InteractiveElement as _, IntoElement, ParentElement as _, Pixels, Role, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_base::{
    Button as BaseButton, Date as BaseDate, TestSupportExt as _, actions::Confirm as ConfirmAction,
};
use gpui_material_components::{
    Button, ButtonColors, ButtonVariant, DialogCloseReason, Icon, IconButton, IconName, Text,
};
use gpui_material_foundation::{color, current_material_theme, text_style};
use gpui_material_theme::{ColorRole, Direction, TypeRole, TypographyEmphasis};
use jiff::{ToSpan as _, civil::Date};

use super::{
    state::{DatePickerState, MonthCache, calendar_start, first_of_month},
    types::{DatePickerColors, DatePickerLocale, DockedActions},
};

const YEARS_PER_PAGE: i32 = 12;

const DAY_LABELS: [&str; 31] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17",
    "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31",
];

fn year_page(year: i32, first: i32) -> i32 {
    first + (year - first) / YEARS_PER_PAGE * YEARS_PER_PAGE
}

fn year_choices(
    id: &ElementId,
    state: &Entity<DatePickerState>,
    from: i32,
    colors: DatePickerColors,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let model = state.read(cx);
    let first = i32::from(model.limits.bounds.start().year());
    let last = i32::from(model.limits.bounds.end().year());
    let displayed = model.displayed_month();
    let today = model.calendar.read(cx).today();
    let selected_year = i32::from(displayed.year());
    let direction = current_material_theme(window, cx).layout.direction;
    let rtl = direction == Direction::Rtl;
    let page_change = |delta: i32| {
        let state = state.clone();
        move |_: &mut Window, cx: &mut App| {
            state.update(cx, |model, cx| {
                let next = Some((from + delta).clamp(first, last));
                if model.year_page != next {
                    model.year_page = next;
                    cx.notify();
                }
            });
        }
    };
    let items = (from..(from + YEARS_PER_PAGE).min(last + 1)).map(|year| {
        let state = state.clone();
        let selected = selected_year == year;
        let enabled = model.limits.accepts_year(year);
        let content = if selected {
            colors.selected_year
        } else if i32::from(today.year()) == year {
            colors.current_year
        } else {
            colors.year
        };
        let container = selected.then_some(colors.selected_year_container).flatten();
        Button::new((id.clone(), format!("year-{year}")), year.to_string())
            .variant(if selected {
                ButtonVariant::Filled
            } else {
                ButtonVariant::Text
            })
            .disabled(!enabled)
            .colors(ButtonColors {
                content,
                container,
                disabled_content: if selected {
                    colors.disabled_selected_year
                } else {
                    colors.disabled_year
                },
                disabled_container: selected
                    .then_some(colors.disabled_selected_year_container)
                    .flatten(),
                ..ButtonColors::default()
            })
            .on_activate(move |window, cx| {
                state.update(cx, |model, cx| {
                    if model.limits.accepts_year(year)
                        && let Ok(year_i16) = i16::try_from(year)
                        && let Ok(target) = Date::new(year_i16, displayed.month(), 1)
                    {
                        model.set_displayed_month(target, cx);
                        model.day.focus(window, cx);
                    }
                });
            })
    });
    let switch = state.clone();
    div()
        .id((id.clone(), "year-list"))
        .test_support()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .when(rtl, gpui::Styled::flex_row_reverse)
                .child(
                    Button::new((id.clone(), "previous-years"), "Earlier years")
                        .variant(ButtonVariant::Text)
                        .disabled(from <= first)
                        .colors(ButtonColors {
                            content: colors.navigation,
                            ..ButtonColors::default()
                        })
                        .on_activate(page_change(-YEARS_PER_PAGE)),
                )
                .child(
                    Button::new((id.clone(), "return-month"), "Show month")
                        .variant(ButtonVariant::Text)
                        .colors(ButtonColors {
                            content: colors.subheading,
                            ..ButtonColors::default()
                        })
                        .on_activate(move |window, cx| {
                            switch.update(cx, |model, cx| {
                                model.year_page = None;
                                model.day.focus(window, cx);
                                cx.notify();
                            });
                        }),
                )
                .child(
                    Button::new((id.clone(), "next-years"), "Later years")
                        .variant(ButtonVariant::Text)
                        .disabled(from + YEARS_PER_PAGE > last)
                        .colors(ButtonColors {
                            content: colors.navigation,
                            ..ButtonColors::default()
                        })
                        .on_activate(page_change(YEARS_PER_PAGE)),
                ),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(4.))
                .when(rtl, gpui::Styled::flex_row_reverse)
                .children(items),
        )
        .into_any_element()
}

fn activate_day(
    state: &Entity<DatePickerState>,
    docked: Option<&DockedActions>,
    date: Date,
    window: &mut Window,
    cx: &mut App,
) {
    if docked.is_some_and(|actions| !actions.current(cx)) {
        return;
    }
    let picked = state.update(cx, |model, cx| model.select_day(date, cx));
    if let (Some(actions), Some(value)) = (docked, picked) {
        actions.finish(DialogCloseReason::Confirm, Some(value), window, cx);
    } else {
        state.read(cx).day.clone().focus(window, cx);
    }
}

struct CalendarContext<'a> {
    id: &'a ElementId,
    state: &'a Entity<DatePickerState>,
    locale: &'a DatePickerLocale,
    cache: &'a MonthCache,
    docked: &'a Option<DockedActions>,
    colors: DatePickerColors,
    month: Date,
    start: Date,
    draft: BaseDate,
    today: Date,
    cursor: Date,
    direction: Direction,
    cell: Pixels,
    focus_cursor: bool,
    tones: CalendarTones,
}

#[derive(Clone, Copy)]
struct CalendarTones {
    on_primary: Hsla,
    primary: Hsla,
    on_surface: Hsla,
    on_secondary_container: Hsla,
    secondary_container: Hsla,
    secondary: Hsla,
}

impl CalendarTones {
    fn from_window(window: &Window, cx: &App) -> Self {
        let theme = current_material_theme(window, cx);
        let tone = |role| color(theme.colors.color(role));
        Self {
            on_primary: tone(ColorRole::OnPrimary),
            primary: tone(ColorRole::Primary),
            on_surface: tone(ColorRole::OnSurface),
            on_secondary_container: tone(ColorRole::OnSecondaryContainer),
            secondary_container: tone(ColorRole::SecondaryContainer),
            secondary: tone(ColorRole::Secondary),
        }
    }
}

fn calendar_header(ctx: &CalendarContext<'_>, model: &DatePickerState) -> impl IntoElement {
    let rtl = ctx.direction == Direction::Rtl;
    let direction = ctx.direction;
    let first_day = ctx.locale.first_day;
    let month = ctx.month;
    let move_month = |target: Entity<DatePickerState>, key| {
        move |_: &mut Window, cx: &mut App| {
            target.update(cx, |model, cx| {
                model.navigate(key, false, direction, first_day, cx);
            });
        }
    };
    let show_years = ctx.state.clone();
    let navigation = ButtonColors {
        content: ctx.colors.navigation,
        ..ButtonColors::default()
    };
    div()
        .flex()
        .items_center()
        .justify_between()
        .when(rtl, gpui::Styled::flex_row_reverse)
        .child(
            IconButton::new(
                (ctx.id.clone(), "previous-month"),
                "Previous month",
                Icon::new(IconName::ChevronRight)
                    .rotate(gpui::Radians(std::f32::consts::PI))
                    .mirror_in_rtl(true),
            )
            .colors(navigation)
            .disabled(month <= first_of_month(*model.limits.bounds.start()))
            .on_activate(move_month(ctx.state.clone(), "pageup")),
        )
        .child(
            Button::new(
                (ctx.id.clone(), "year-toggle"),
                format!("Choose year, {}", ctx.cache.month_label),
            )
            .variant(ButtonVariant::Text)
            .colors(ButtonColors {
                content: ctx.colors.subheading,
                ..ButtonColors::default()
            })
            .content(
                Text::new(ctx.cache.month_label.clone())
                    .id("current-month")
                    .role(TypeRole::TitleSmall),
            )
            .on_activate(move |_, cx| {
                show_years.update(cx, |model, cx| {
                    model.year_page = Some(year_page(
                        i32::from(month.year()),
                        i32::from(model.limits.bounds.start().year()),
                    ));
                    cx.notify();
                });
            }),
        )
        .child(
            IconButton::new(
                (ctx.id.clone(), "next-month"),
                "Next month",
                Icon::new(IconName::ChevronRight).mirror_in_rtl(true),
            )
            .colors(navigation)
            .disabled(month >= first_of_month(*model.limits.bounds.end()))
            .on_activate(move_month(ctx.state.clone(), "pagedown")),
        )
}

fn weekday_row(ctx: &CalendarContext<'_>) -> impl IntoElement {
    let rtl = ctx.direction == Direction::Rtl;
    let weekdays = (0_i64..7).map(|offset| {
        let weekday = ctx
            .start
            .checked_add(offset.days())
            .unwrap_or(ctx.start)
            .weekday();
        div()
            .size(ctx.cell)
            .flex()
            .items_center()
            .justify_center()
            .when_some(ctx.colors.weekday, gpui::Styled::text_color)
            .child(
                Text::new((ctx.locale.weekday)(weekday))
                    .id(("weekday", weekday.to_monday_zero_offset() as usize))
                    .role(TypeRole::BodySmall),
            )
    });
    div()
        .flex()
        .when(rtl, gpui::Styled::flex_row_reverse)
        .children(weekdays)
}

fn day_colors(
    colors: DatePickerColors,
    tones: CalendarTones,
    enabled: bool,
    active: bool,
    range: bool,
    is_today: bool,
) -> (Hsla, Hsla) {
    match (enabled, active) {
        (true, true) => (
            colors.selected_day.unwrap_or(tones.on_primary),
            colors.selected_day_container.unwrap_or(tones.primary),
        ),
        (false, true) => (
            colors
                .disabled_selected_day
                .unwrap_or_else(|| tones.on_surface.opacity(0.38)),
            colors
                .disabled_selected_day_container
                .unwrap_or_else(|| tones.on_surface.opacity(0.12)),
        ),
        (true, false) => (
            if range {
                colors.range_day.unwrap_or(tones.on_secondary_container)
            } else if is_today {
                colors.today.unwrap_or(tones.primary)
            } else {
                colors.day.unwrap_or(tones.on_surface)
            },
            gpui::transparent_black(),
        ),
        (false, false) => (
            colors
                .disabled_day
                .unwrap_or_else(|| tones.on_surface.opacity(0.38)),
            gpui::transparent_black(),
        ),
    }
}

fn day_cell(
    ctx: &CalendarContext<'_>,
    model: &DatePickerState,
    offset: i64,
    window: &Window,
) -> AnyElement {
    let Some(date) = ctx
        .start
        .checked_add(offset.days())
        .ok()
        .filter(|date| date.month() == ctx.month.month())
    else {
        return div().size(ctx.cell).into_any_element();
    };
    let rtl = ctx.direction == Direction::Rtl;
    let enabled = model.limits.accepts(date);
    let active = ctx.draft.is_active(&date);
    let range = ctx.draft.is_in_range(&date);
    let leading = range && ctx.draft.start() == Some(date);
    let trailing = range && ctx.draft.end() == Some(date);
    let focus = ctx.cursor == date;
    let is_today = date == ctx.today;
    let show_focus_ring = focus && ctx.focus_cursor && window.last_input_was_keyboard();
    let (ink, selected_background) =
        day_colors(ctx.colors, ctx.tones, enabled, active, range, is_today);
    let day_index = (date.day() - 1) as usize;
    let day_id = ctx.cache.day_ids[day_index].clone();
    let day_label = ctx.cache.day_labels[day_index].clone();
    let control = BaseButton::new((ctx.id.clone(), day_id))
        .role(Role::ListBoxOption)
        .aria_selected(range || active)
        .aria_description(if is_today { "Today" } else { "" })
        .accessibility_label(day_label)
        .focusable(false)
        .when(focus, |button| button.aria_active_descendant())
        .disabled(!enabled)
        .size(ctx.cell)
        .when_some(
            enabled.then(|| {
                let state = ctx.state.clone();
                let actions = ctx.docked.clone();
                Rc::new(move |window: &mut Window, cx: &mut App| {
                    activate_day(&state, actions.as_ref(), date, window, cx);
                })
            }),
            |button, activate| {
                let click = activate.clone();
                button
                    .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                        activate(window, cx)
                    })
                    .on_click(move |_, window, cx| click(window, cx))
            },
        );
    div()
        .size(ctx.cell)
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .when(range, |cell_box| {
            cell_box.child(
                div()
                    .absolute()
                    .h(ctx.cell.min(px(40.)))
                    .top((ctx.cell - ctx.cell.min(px(40.))) / 2.)
                    .left(if if rtl { trailing } else { leading } {
                        ctx.cell / 2.
                    } else {
                        px(0.)
                    })
                    .right(if if rtl { leading } else { trailing } {
                        ctx.cell / 2.
                    } else {
                        px(0.)
                    })
                    .bg(ctx
                        .colors
                        .range_container
                        .unwrap_or(ctx.tones.secondary_container)),
            )
        })
        .child(
            control.child(
                div()
                    .size(ctx.cell.min(px(40.)))
                    .rounded_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .when(active, |day| day.bg(selected_background))
                    .when(show_focus_ring, |day| {
                        day.border_2()
                            .border_color(ctx.colors.focus_outline.unwrap_or(ctx.tones.secondary))
                    })
                    .when(is_today && !active && !show_focus_ring, |day| {
                        day.border_1()
                            .border_color(ctx.colors.today_outline.unwrap_or(ctx.tones.primary))
                    })
                    .text_color(ink)
                    .child(SharedString::new_static(DAY_LABELS[day_index])),
            ),
        )
        .into_any_element()
}

fn calendar_grid(
    ctx: &CalendarContext<'_>,
    model: &DatePickerState,
    day_focus: FocusHandle,
    window: &Window,
) -> impl IntoElement {
    let rtl = ctx.direction == Direction::Rtl;
    let direction = ctx.direction;
    let first_day = ctx.locale.first_day;
    let keyboard_focus = day_focus.clone();
    let confirm_focus = day_focus.clone();
    let keys = ctx.state.clone();
    let key_actions = ctx.docked.clone();
    let key_selection = ctx.state.clone();
    let select = Rc::new(move |window: &mut Window, cx: &mut App| {
        let cursor = key_selection.read(cx).cursor;
        activate_day(&key_selection, key_actions.as_ref(), cursor, window, cx);
    });
    let confirm = select.clone();
    let rows = (0_i64..6)
        .map(|week| {
            div()
                .flex()
                .when(rtl, gpui::Styled::flex_row_reverse)
                .children((0_i64..7).map(|day| day_cell(ctx, model, week * 7 + day, window)))
        })
        .collect::<Vec<_>>();
    div()
        .id((ctx.id.clone(), "calendar-grid"))
        .test_support()
        .role(Role::ListBox)
        .aria_label(ctx.cache.month_label.clone())
        .track_focus(&day_focus)
        .tab_stop(true)
        .flex()
        .flex_col()
        .items_center()
        .on_action(move |_: &ConfirmAction, window, cx| {
            if confirm_focus.is_focused(window) {
                confirm(window, cx);
                cx.stop_propagation();
            } else {
                cx.propagate();
            }
        })
        .capture_key_down(move |event, window, cx| {
            if !keyboard_focus.is_focused(window) {
                return;
            }
            let modifiers = event.keystroke.modifiers;
            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                if modifiers.modified() {
                    return;
                }
                select(window, cx);
                cx.stop_propagation();
                return;
            }
            let allowed_modifiers = if matches!(event.keystroke.key.as_str(), "pageup" | "pagedown")
            {
                !modifiers.alt && !modifiers.shift && !modifiers.platform && !modifiers.function
            } else {
                !modifiers.modified()
            };
            if !allowed_modifiers {
                return;
            }
            if keys.update(cx, |model, cx| {
                model.navigate(
                    &event.keystroke.key,
                    modifiers.control,
                    direction,
                    first_day,
                    cx,
                )
            }) {
                cx.stop_propagation();
            }
        })
        .child(weekday_row(ctx))
        .children(rows)
}

pub(super) fn calendar(
    id: &ElementId,
    state: Entity<DatePickerState>,
    locale: DatePickerLocale,
    docked: Option<DockedActions>,
    presentation: (DatePickerColors, Pixels),
    window: &Window,
    cx: &App,
) -> AnyElement {
    let (colors, width) = presentation;
    let model = state.read(cx);
    if let Some(page) = model.year_page {
        return year_choices(id, &state, page, colors, window, cx);
    }
    let month = model.displayed_month();
    let cache = model.month_cache(month, &locale);
    let day_focus = model.day.clone();
    let ctx = CalendarContext {
        id,
        state: &state,
        locale: &locale,
        cache: &cache,
        docked: &docked,
        colors,
        month,
        start: calendar_start(month, locale.first_day),
        draft: model.calendar.read(cx).date(),
        today: model.calendar.read(cx).today(),
        cursor: model.cursor,
        direction: current_material_theme(window, cx).layout.direction,
        cell: (width / 7.).clamp(px(16.), px(48.)),
        focus_cursor: day_focus.is_focused(window),
        tones: CalendarTones::from_window(window, cx),
    };
    let header = calendar_header(&ctx, model);
    let grid = calendar_grid(&ctx, model, day_focus, window);
    text_style(
        div()
            .w(width)
            .max_w_full()
            .mx_auto()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(header)
            .child(grid),
        TypeRole::BodyLarge.style(TypographyEmphasis::Baseline),
        1.,
        window,
        cx,
    )
    .into_any_element()
}

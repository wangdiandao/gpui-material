use std::{cell::Cell, rc::Rc};

use gpui::{
    Anchor, AnyElement, App, Bounds, ElementId, Entity, FocusHandle, Hsla, InteractiveElement,
    IntoElement, ListSizingBehavior, ListState, ParentElement, Pixels, Role,
    StatefulInteractiveElement, Styled, Window, div, list, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{OverlayMotion, ScaledOverlay, ThemeScope, logical_anchor};
use gpui_material_theme::{Direction, MaterialTheme, TypeRole, TypographyEmphasis};

use crate::{MenuSurfaceStyle, menu_appearance::menu_pivot};

use super::{
    DropdownOption, OptionContent, Select,
    interaction::Interaction,
    option_row::{OptionRowContext, render_option_row},
    state::ExposedDropdownState,
};

pub(super) struct PopupGeometry {
    pub(super) anchor: Anchor,
    pub(super) height: Pixels,
    pub(super) width: Pixels,
    pub(super) gap: Pixels,
}

pub(super) fn resolve_popup_geometry(
    measured: Bounds<Pixels>,
    viewport: gpui::Size<Pixels>,
    direction: Direction,
    match_anchor_width: bool,
    popup_width: Pixels,
) -> PopupGeometry {
    let default_height = (viewport.height * 0.55).min(px(280.));
    let gap = px(4.);
    let margin = px(8.);
    let (anchor, height) = if measured.size.height > px(0.) {
        let below = (viewport.height - measured.bottom() - gap - margin).max(px(0.));
        let above = (measured.top() - gap - margin).max(px(0.));
        if below < default_height && above > below {
            (Anchor::BottomLeft, default_height.min(above))
        } else {
            (Anchor::TopLeft, default_height.min(below))
        }
    } else {
        (Anchor::TopLeft, default_height)
    };
    let anchor = logical_anchor(anchor, direction);
    let width = if match_anchor_width {
        if measured.size.width > px(0.) {
            measured.size.width
        } else {
            px(120.)
        }
    } else {
        popup_width
    };
    let width = width.min((viewport.width - margin * 2.).max(px(0.)));
    PopupGeometry {
        anchor,
        height,
        width,
        gap,
    }
}

pub(super) struct PopupContentConfig {
    pub(super) id: ElementId,
    pub(super) state: Entity<ExposedDropdownState>,
    pub(super) scroll: ListState,
    pub(super) open: bool,
    pub(super) options: Rc<[DropdownOption]>,
    pub(super) active: Option<ElementId>,
    pub(super) selected_option: Option<ElementId>,
    pub(super) options_label: String,
    pub(super) transfer_focus: bool,
    pub(super) popup_focus: FocusHandle,
    pub(super) popup_keys: Interaction,
    pub(super) outside: Interaction,
    pub(super) bounds: Rc<Cell<Bounds<Pixels>>>,
    pub(super) placement: Rc<Cell<(Bounds<Pixels>, Bounds<Pixels>)>>,
    pub(super) last_trigger: Bounds<Pixels>,
    pub(super) width: Pixels,
    pub(super) height: Pixels,
    pub(super) appearance: MenuSurfaceStyle,
    pub(super) container_color: Option<Hsla>,
    pub(super) on_select: Option<Select>,
    pub(super) option_content: Option<OptionContent>,
    pub(super) parent: gpui::EntityId,
    pub(super) theme: MaterialTheme,
    pub(super) ink: Hsla,
    pub(super) selected_bg: Hsla,
    pub(super) selected_ink: Hsla,
}

pub(super) fn build_popup_content(
    config: PopupContentConfig,
    motion: OverlayMotion,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let dp = move |value| motion.px(value);
    let spacing = dp(16.);
    let scroll_height =
        (config.height - config.appearance.popup_vertical_insets()).max(px(0.)) * motion.scale;
    let font_revision = gpui_material_foundation::font_cache_revision(cx);
    config.state.update(cx, |state, _| {
        state.sync_presented_list(
            &config.options,
            config.active.as_ref(),
            (
                config.width,
                window.rem_size(),
                config.theme.typography.clone(),
                font_revision,
            ),
        );
    });
    let scroll_region_id = (config.id.clone(), "options-scroll-region");
    let options = config.options;
    let active = config.active;
    let selected_option = config.selected_option;
    let open = config.open;
    let list_body = if options.is_empty() {
        div()
            .id(scroll_region_id)
            .test_support()
            .max_h(scroll_height)
            .overflow_hidden()
            .child(
                div()
                    .px(spacing * 0.75)
                    .py(spacing * 0.5)
                    .child("No matching options"),
            )
            .into_any_element()
    } else {
        let row_ctx = Rc::new(OptionRowContext {
            state: config.state,
            on_select: config.on_select,
            option_content: config.option_content,
            open,
            parent: config.parent,
            ink: config.ink,
            selected_bg: config.selected_bg,
            selected_ink: config.selected_ink,
            min_height: dp(48.),
            horizontal_padding: spacing * 0.75,
        });
        div()
            .id(scroll_region_id)
            .test_support()
            .max_h(scroll_height)
            .overflow_hidden()
            .child(
                list(config.scroll, move |index, window, cx| {
                    let choice = options[index].clone();
                    let current = active.as_ref().or(selected_option.as_ref()) == Some(&choice.id);
                    render_option_row(choice, index, options.len(), current, &row_ctx, window, cx)
                })
                .with_sizing_behavior(ListSizingBehavior::Infer)
                .w_full()
                .max_h(scroll_height),
            )
            .into_any_element()
    };
    let bounds = config.bounds;
    let popup_focus = config.popup_focus;
    let popup_keys = config.popup_keys;
    let outside = config.outside;
    let content = div()
        .id((config.id, if open { "options" } else { "options-exit" }))
        .test_support()
        .opacity(motion.alpha)
        .w(config.width * motion.scale)
        .text_color(config.ink)
        .max_h(config.height * motion.scale)
        .when(open, |list| {
            list.role(Role::ListBox)
                .aria_label(config.options_label)
                .when(config.transfer_focus, |list| {
                    list.track_focus(&popup_focus).tab_stop(false)
                })
                .capture_key_down(move |event, window, cx| popup_keys.keys(event, window, cx))
                .on_mouse_down_out(move |event, window, cx| {
                    if !bounds.get().contains(&event.position) {
                        outside.dismiss_outside(window, cx);
                    }
                })
        })
        .child(
            config
                .appearance
                .dropdown_surface(&config.theme, motion)
                .when_some(config.container_color, |surface, color| surface.bg(color))
                .w_full()
                .child(list_body),
        )
        .when(!open, |list| {
            list.child(div().absolute().inset_0().occlude())
        });
    let text_style = TypeRole::LabelLarge.style(TypographyEmphasis::Baseline);
    let content =
        gpui_material_foundation::text_style(content, text_style, motion.scale, window, cx);
    let placement = config.placement;
    let last_trigger = config.last_trigger;
    let scaled = ScaledOverlay::new(content, motion, move |menu| {
        menu_pivot(
            menu,
            if open {
                placement.get().1
            } else {
                last_trigger
            },
        )
    })
    .width(config.width);
    ThemeScope::inherit(scaled, window, cx)
}

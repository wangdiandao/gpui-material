use gpui::Hsla;
use gpui_material_foundation::color;
use gpui_material_theme::{ColorRole, MaterialTheme};

use super::{super::tabs::TabVariant, NavigationItem, Presentation};

pub(super) struct ItemPalette {
    pub(super) icon_role: ColorRole,
    pub(super) icon_color: Hsla,
    pub(super) label_color: Hsla,
    pub(super) selected_indicator: Hsla,
    pub(super) tab_indicator: Hsla,
}

pub(super) fn resolve_palette(
    presentation: Presentation,
    selected: bool,
    tab_content: Option<Hsla>,
    styled_item: bool,
    theme: &MaterialTheme,
    item: &NavigationItem,
    horizontal: bool,
) -> ItemPalette {
    let tone = |role| color(theme.colors.color(role));
    let foreground = if !selected {
        ColorRole::OnSurfaceVariant
    } else {
        match presentation {
            Presentation::Tabs(TabVariant::Primary, _) => ColorRole::Primary,
            Presentation::Tabs(TabVariant::Secondary, _) => ColorRole::OnSurface,
            _ => ColorRole::Secondary,
        }
    };
    let icon_role = if selected && !presentation.is_tab() {
        ColorRole::OnSecondaryContainer
    } else {
        foreground
    };
    let bar = presentation.item_colors();
    let fallback = bar.and_then(|colors| colors.content).or(tab_content);
    let (icon_color, label_color, selected_indicator) = if styled_item {
        let colors = item.colors;
        let resolve = |selected_color: Option<Hsla>,
                       unselected: Option<Hsla>,
                       disabled: Option<Hsla>,
                       active_role| {
            let unselected = unselected
                .or(fallback)
                .unwrap_or_else(|| tone(ColorRole::OnSurfaceVariant));
            if item.disabled {
                disabled.unwrap_or_else(|| unselected.opacity(0.38))
            } else if selected {
                selected_color
                    .or(fallback)
                    .unwrap_or_else(|| tone(active_role))
            } else {
                unselected
            }
        };
        let icon = resolve(
            colors.selected_icon,
            colors.unselected_icon,
            colors.disabled_icon,
            icon_role,
        );
        let label = resolve(
            if horizontal {
                colors.selected_label_start
            } else {
                colors.selected_label_top
            },
            colors.unselected_label,
            colors.disabled_label,
            foreground,
        );
        let selected_indicator = colors
            .selected_indicator
            .unwrap_or_else(|| tone(ColorRole::SecondaryContainer));
        (
            icon,
            label,
            selected_indicator.opacity(if item.disabled { 0.38 } else { 1. }),
        )
    } else {
        (
            tone(icon_role),
            tone(foreground),
            tone(ColorRole::SecondaryContainer),
        )
    };
    let tab_indicator = item
        .colors
        .selected_indicator
        .unwrap_or_else(|| tone(ColorRole::Primary))
        .opacity(if item.disabled && styled_item {
            0.38
        } else {
            1.
        });
    ItemPalette {
        icon_role,
        icon_color,
        label_color,
        selected_indicator,
        tab_indicator,
    }
}

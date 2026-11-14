use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role,
    ScrollHandle, SharedString, StatefulInteractiveElement, Styled, TextAlign, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{color, text_style, window_material_theme};
use gpui_material_theme::{ColorRole, Direction, TypeRole, TypographyEmphasis};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppBarVariant {
    #[default]
    Small,
    Centered,
    MediumFlexible,
    LargeFlexible,
}

/// Top app bar with caller-owned navigation, actions, and optional search.
/// Pass the content scroll handle to collapse flexible variants on scroll.
#[derive(IntoElement)]
pub struct AppBar {
    id: ElementId,
    title: SharedString,
    subtitle: Option<SharedString>,
    variant: AppBarVariant,
    navigation: Option<AnyElement>,
    actions: Option<AnyElement>,
    search: Option<AnyElement>,
    scroll: Option<ScrollHandle>,
    font_scale: f32,
}

impl AppBar {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            variant: AppBarVariant::Small,
            navigation: None,
            actions: None,
            search: None,
            scroll: None,
            font_scale: 1.,
        }
    }

    pub fn variant(mut self, variant: AppBarVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn navigation(mut self, element: impl IntoElement) -> Self {
        self.navigation = Some(element.into_any_element());
        self
    }

    pub fn actions(mut self, element: impl IntoElement) -> Self {
        self.actions = Some(element.into_any_element());
        self
    }

    /// Supplies the search control that replaces the title area in the first row.
    pub fn search(mut self, element: impl IntoElement) -> Self {
        self.search = Some(element.into_any_element());
        self
    }

    pub fn track_scroll(mut self, handle: &ScrollHandle) -> Self {
        self.scroll = Some(handle.clone());
        self
    }

    pub fn font_scale(mut self, scale: f32) -> Self {
        self.font_scale = if scale.is_finite() && scale > 0. {
            scale
        } else {
            1.
        };
        self
    }
}

impl RenderOnce for AppBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let centered = self.variant == AppBarVariant::Centered;
        let scroll = self
            .scroll
            .as_ref()
            .map_or(0., |scroll| f32::from(-scroll.offset().y))
            .max(0.);
        // AppBar{Medium,Large}FlexibleTokens in Compose Material 3.
        let expansion = match (self.variant, self.subtitle.is_some()) {
            (AppBarVariant::MediumFlexible, false) => 48.,
            (AppBarVariant::MediumFlexible, true) => 72.,
            (AppBarVariant::LargeFlexible, false) => 56.,
            (AppBarVariant::LargeFlexible, true) => 88.,
            _ => 0.,
        } * self.font_scale.max(1.);
        let remaining = (expansion - scroll).max(0.);
        let expanded = remaining > 0. && self.search.is_none();
        let color_for = |role| color(theme.colors.color(role));
        let title = |role: TypeRole, subtitle_role: TypeRole| {
            let align = if centered {
                TextAlign::Center
            } else if rtl {
                TextAlign::Right
            } else {
                TextAlign::Left
            };
            div()
                .flex()
                .flex_col()
                .min_w_0()
                .text_align(align)
                .child(
                    text_style(
                        div().truncate(),
                        role.style(TypographyEmphasis::Baseline),
                        self.font_scale,
                        window,
                        cx,
                    )
                    .text_color(color_for(ColorRole::OnSurface))
                    .child(self.title.clone()),
                )
                .children(self.subtitle.clone().map(|subtitle| {
                    text_style(
                        div().truncate(),
                        subtitle_role.style(TypographyEmphasis::Baseline),
                        self.font_scale,
                        window,
                        cx,
                    )
                    .text_color(color_for(ColorRole::OnSurfaceVariant))
                    .child(subtitle)
                }))
        };
        let top_title = if expanded && remaining > expansion / 2. {
            None
        } else {
            Some(title(TypeRole::TitleLarge, TypeRole::LabelMedium))
        };
        let expanded_title = (expanded && remaining > expansion / 2.).then(|| {
            title(
                if self.variant == AppBarVariant::LargeFlexible {
                    TypeRole::DisplaySmall
                } else {
                    TypeRole::HeadlineMedium
                },
                if self.variant == AppBarVariant::LargeFlexible {
                    TypeRole::TitleMedium
                } else {
                    TypeRole::LabelLarge
                },
            )
        });
        let background = crate::motion::mix(
            color_for(ColorRole::Surface),
            color_for(ColorRole::SurfaceContainer),
            (scroll / expansion.max(24.)).min(1.),
        );
        let has_search = self.search.is_some();
        let center_slots = centered && !has_search;
        div()
            .id(self.id)
            .test_support()
            .role(Role::Toolbar)
            .aria_label(self.title)
            .w_full()
            .flex_none()
            .flex()
            .flex_col()
            .bg(background)
            .child(
                div()
                    .flex()
                    .items_center()
                    .min_h(px(64.))
                    .px(px(4.))
                    .gap(px(4.))
                    .when(rtl, Styled::flex_row_reverse)
                    .child(
                        div()
                            .flex()
                            .min_w_0()
                            .when(center_slots, |lane| lane.flex_1())
                            .children(self.navigation),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .when_else(center_slots, |lane| lane.w_1_2(), |lane| lane.flex_1())
                            .children(self.search)
                            .children(top_title.filter(|_| !has_search)),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .min_w_0()
                            .when(center_slots, |lane| lane.flex_1())
                            .when(rtl, Styled::flex_row_reverse)
                            .children(self.actions),
                    ),
            )
            .when(expanded, |bar| {
                bar.child(
                    div()
                        .h(px(remaining))
                        .overflow_hidden()
                        .flex()
                        .flex_col()
                        .justify_end()
                        .px(px(16.))
                        .pb(px(4.))
                        .children(expanded_title),
                )
            })
    }
}

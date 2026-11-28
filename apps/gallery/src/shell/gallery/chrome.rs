use std::sync::{Arc, LazyLock};

use gpui::{
    App, Context, Image, ImageFormat, InteractiveElement, IntoElement, ObjectFit, ParentElement,
    Pixels, Role, StatefulInteractiveElement, Styled, StyledImage, Window, div, img,
    prelude::FluentBuilder, px, svg, transparent_black,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    ControlSize, Icon, IconButton, IconName, NavigationItem, NavigationItems, TabVariant, Tabs,
    TabsColors, Text, ThemeScope, ToggleButton, ToggleButtonVariant, Tooltip,
    foundation::{Surface, color, window_material_theme},
    theme::{Appearance, ColorRole, CornerShape, Density, Direction, TypeRole},
};

use super::{Gallery, REPOSITORY_URL};
use crate::{
    shell::catalog::{GalleryPage, TopSection, first_of, section_of},
    story::Story,
};

static BRAND_LOGO: LazyLock<Arc<Image>> = LazyLock::new(|| {
    Arc::new(Image::from_bytes(
        ImageFormat::Svg,
        include_bytes!("../../../assets/logo.svg").to_vec(),
    ))
});

impl Gallery {
    pub(super) fn render_section_tabs(
        &self,
        id_prefix: &'static str,
        window: &Window,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let active_section = section_of(self.current);
        let owner = cx.weak_entity();
        let destinations = TopSection::ALL
            .into_iter()
            .map(|section| NavigationItem::new(section.id(), section.title()));
        let items = NavigationItems::new(id_prefix, "Sections", destinations)
            .selected(active_section.id())
            .on_select(move |id, window, cx| {
                let Some(section) = TopSection::ALL
                    .into_iter()
                    .find(|candidate| candidate.id() == id.as_ref())
                else {
                    return;
                };
                let _ = owner.update(cx, |this, cx| {
                    this.navigate(first_of(section), window, cx);
                });
            });
        let mut compact = window_material_theme(window, cx).clone();
        compact.layout.density = Density::Compact;
        let tabs_width = if f32::from(window.viewport_size().width) >= 920. {
            px(324.)
        } else {
            px(264.)
        };
        div().w(tabs_width).flex_none().child(
            ThemeScope::new(
                Tabs::new(items)
                    .variant(TabVariant::Secondary)
                    .colors(TabsColors {
                        container: Some(transparent_black()),
                        divider: Some(transparent_black()),
                        ..Default::default()
                    }),
            )
            .theme(compact),
        )
    }

    fn render_brand(&self, window: &Window, cx: &Context<Self>) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let owner = cx.weak_entity();
        Surface::clickable(
            "gallery-brand",
            "GPUI Material Overview",
            ColorRole::SurfaceContainer,
        )
        .shape(CornerShape::uniform(12.))
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .py_1()
        .when(rtl, Styled::flex_row_reverse)
        .on_activate(move |window, cx| {
            let _ = owner.update(cx, |this, cx| {
                this.navigate(GalleryPage::Story(Story::Introduction), window, cx);
            });
        })
        .child(
            img(BRAND_LOGO.clone())
                .id("brand-mark")
                .size(px(28.))
                .object_fit(ObjectFit::Contain),
        )
        .child(
            Text::new("GPUI Material")
                .id("brand-title")
                .role(TypeRole::TitleSmall)
                .emphasized(true),
        )
    }

    pub(super) fn render_app_bar(
        &self,
        single_pane: bool,
        show_catalog: bool,
        margin: Pixels,
        window: &Window,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let (label, icon) = if show_catalog {
            ("Hide components", IconName::LeftPanelClose)
        } else {
            ("Show components", IconName::LeftPanelOpen)
        };
        let owner = cx.weak_entity();
        div()
            .w_full()
            .flex_none()
            .px(margin)
            .py_2()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap_2()
            .when(rtl, Styled::flex_row_reverse)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when(rtl, Styled::flex_row_reverse)
                    .child(Tooltip::plain(
                        "browse-components-tooltip",
                        IconButton::new(
                            "browse-components",
                            label,
                            Icon::new(icon).mirror_in_rtl(true),
                        )
                        .on_activate(move |window, cx| {
                            let _ = owner.update(cx, |this, cx| this.toggle_catalog(window, cx));
                        }),
                        label,
                    ))
                    .when(!single_pane, |left| {
                        left.child(self.render_brand(window, cx))
                            .child(self.render_section_tabs("top-section", window, cx))
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when(rtl, Styled::flex_row_reverse)
                    .child(self.render_theme_toggle(window, cx))
                    .child(Tooltip::plain(
                        "repository-tooltip",
                        IconButton::new(
                            "open-repository",
                            "Open GitHub repository",
                            svg()
                                .data(include_bytes!("../../../assets/mark-github-24.svg"))
                                .size_full()
                                .text_color(color(theme.colors.color(ColorRole::OnSurfaceVariant))),
                        )
                        .on_activate(|_, cx| cx.open_url(REPOSITORY_URL)),
                        "Open GitHub repository",
                    ))
                    .child(
                        ToggleButton::new("toggle-motion", "Reduced motion", cx.reduce_motion())
                            .variant(ToggleButtonVariant::Outlined)
                            .size(ControlSize::ExtraSmall)
                            .on_change(|next, _, cx| cx.set_reduce_motion(next)),
                    ),
            )
    }

    fn render_theme_toggle(&self, window: &Window, cx: &Context<Self>) -> Tooltip {
        let (label, icon, appearance) = match window_material_theme(window, cx).colors.appearance()
        {
            Appearance::Light => ("Switch to dark theme", IconName::DarkMode, Appearance::Dark),
            Appearance::Dark => (
                "Switch to light theme",
                IconName::LightMode,
                Appearance::Light,
            ),
        };
        let owner = cx.weak_entity();
        let button = IconButton::new("toggle-theme", label, Icon::new(icon)).on_activate(
            move |window, cx| {
                let _ = owner.update(cx, |this, cx| {
                    this.set_appearance(appearance, window, cx);
                });
            },
        );
        Tooltip::plain("theme-tooltip", button, label)
    }

    pub(super) fn render_status_bar(
        &self,
        margin: Pixels,
        window: &Window,
        cx: &App,
    ) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let appearance = theme.colors.appearance();
        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        let theme_status = format!("Window theme {appearance:?}");
        div()
            .w_full()
            .flex_none()
            .px(margin)
            .py_2()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_end()
            .gap_3()
            .when(rtl, Styled::flex_row_reverse)
            .child(
                div()
                    .id("window-theme")
                    .role(Role::Status)
                    .aria_label(theme_status.clone())
                    .test_support()
                    .child(
                        Text::new(theme_status)
                            .id("label")
                            .role(TypeRole::LabelMedium),
                    ),
            )
            .child(
                div()
                    .id("gallery-version")
                    .role(Role::Status)
                    .aria_label(version.clone())
                    .test_support()
                    .child(Text::new(version).id("label").role(TypeRole::LabelMedium)),
            )
    }
}

use std::{cell::Cell, rc::Rc, sync::OnceLock};

use gpui::{
    AnyElement, App, AppContext, ClipboardItem, Context, ElementId, Entity, Hsla,
    InteractiveElement, IntoElement, ParentElement, Pixels, Render, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    Button, ButtonVariant, Chip, ChipVariant, ControlSize, Icon, IconButton, IconName,
    IconToggleButton, MaterialShape, Slider, SliderState, Surface, Switch, Text, ToggleButton,
    ToggleButtonVariant,
    foundation::{SurfaceBorder, color, window_material_theme},
    theme::{Appearance, ColorRole, CornerRole, CornerShape, Direction, ThemePreset, TypeRole},
};

use super::{
    catalog::{CatalogGroup, GalleryPage, StylePage, component_count, entries_in_group},
    styles_examples::{ApplyPreset, cached_shape_canvas, preset_label},
};
use crate::Story;

const INSTALL_CMD: &str = "cargo add gpui-material";

const HERO_SHAPE_CYCLE: [MaterialShape; 8] = [
    MaterialShape::Cookie9Sided,
    MaterialShape::Flower,
    MaterialShape::Pill,
    MaterialShape::SoftBurst,
    MaterialShape::Arch,
    MaterialShape::Clover4Leaf,
    MaterialShape::Gem,
    MaterialShape::Sunny,
];

const HERO_SHAPE_ROLES: [ColorRole; 4] = [
    ColorRole::Primary,
    ColorRole::TertiaryContainer,
    ColorRole::Secondary,
    ColorRole::Tertiary,
];

const SWATCH_PRESETS: [ThemePreset; 8] = [
    ThemePreset::Baseline,
    ThemePreset::Vibrant,
    ThemePreset::Expressive,
    ThemePreset::Ocean,
    ThemePreset::Forest,
    ThemePreset::Sunset,
    ThemePreset::Spring,
    ThemePreset::Monochrome,
];

const LIVE_CHIPS: [&str; 3] = ["Desktop", "Expressive", "Adaptive"];

fn cached_swatch_pair(slot: usize, appearance: Appearance) -> (Hsla, Hsla) {
    static CACHE: OnceLock<[[(Hsla, Hsla); 2]; SWATCH_PRESETS.len()]> = OnceLock::new();
    let table = CACHE.get_or_init(|| {
        std::array::from_fn(|ix| {
            let preset = SWATCH_PRESETS[ix];
            let pair = |app| {
                let scheme = preset.colors(app);
                (
                    color(scheme.color(ColorRole::Primary)),
                    color(scheme.color(ColorRole::PrimaryContainer)),
                )
            };
            [pair(Appearance::Light), pair(Appearance::Dark)]
        })
    });
    let mode_ix = usize::from(appearance == Appearance::Dark);
    table[slot][mode_ix]
}

pub(crate) type NavigateFn = Rc<dyn Fn(GalleryPage, &mut Window, &mut App)>;
pub(crate) type SetAppearanceFn = Rc<dyn Fn(Appearance, &mut Window, &mut App)>;

pub(crate) struct OverviewView {
    on_navigate: NavigateFn,
    active_preset: Rc<dyn Fn(&App) -> ThemePreset>,
    on_select_preset: ApplyPreset,
    on_set_appearance: SetAppearanceFn,
    available_width: Rc<Cell<Pixels>>,
    hero_shapes: [usize; 4],
    live_switch: bool,
    live_chips: [bool; 3],
    live_toggles: [bool; 3],
    live_slider: Entity<SliderState>,
    copied: bool,
}

impl OverviewView {
    pub(crate) fn new(
        on_navigate: NavigateFn,
        active_preset: Rc<dyn Fn(&App) -> ThemePreset>,
        on_select_preset: ApplyPreset,
        on_set_appearance: SetAppearanceFn,
        available_width: Rc<Cell<Pixels>>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            on_navigate,
            active_preset,
            on_select_preset,
            on_set_appearance,
            available_width,
            hero_shapes: [0, 1, 2, 3],
            live_switch: true,
            live_chips: [true, true, false],
            live_toggles: [true, false, false],
            live_slider: cx.new(|_| SliderState::new().max(100.).default_value(64.)),
            copied: false,
        }
    }

    fn render_hero(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let on_navigate = self.on_navigate.clone();
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let shape_fills = HERO_SHAPE_ROLES.map(|role| color(theme.colors.color(role)));

        let owner = cx.weak_entity();
        let mut shape_tiles = Vec::with_capacity(4);
        for slot in 0..4 {
            let shape = HERO_SHAPE_CYCLE[self.hero_shapes[slot] % HERO_SHAPE_CYCLE.len()];
            let fill = shape_fills[slot];
            let mesh =
                cached_shape_canvas(("overview-hero-mesh", slot), shape, fill, 84., window, cx);
            let owner = owner.clone();
            shape_tiles.push(
                Surface::clickable(
                    ("overview-hero-shape", slot),
                    format!("Cycle shape: {}", shape.name()),
                    ColorRole::SurfaceContainerLow,
                )
                .shape(CornerShape::uniform(16.))
                .p_1()
                .on_activate(move |_, cx| {
                    let _ = owner.update(cx, |this, cx| {
                        this.hero_shapes[slot] =
                            (this.hero_shapes[slot] + 1) % HERO_SHAPE_CYCLE.len();
                        cx.notify();
                    });
                })
                .child(mesh),
            );
        }
        let mut tiles = shape_tiles.into_iter();
        let t0 = tiles.next().unwrap();
        let t1 = tiles.next().unwrap();
        let t2 = tiles.next().unwrap();
        let t3 = tiles.next().unwrap();

        let nav_components = on_navigate.clone();
        let nav_styles = on_navigate.clone();
        let nav_examples = on_navigate;

        div()
            .id("overview-hero")
            .w_full()
            .child(
                Surface::new(ColorRole::SurfaceContainerLow)
                    .w_full()
                    .rounded(px(28.))
                    .border_1()
                    .border_color(outline)
                    .p_6()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap_6()
                    .when(rtl, Styled::flex_row_reverse)
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(260.))
                            .flex()
                            .flex_col()
                            .gap_4()
                            .when(rtl, Styled::items_end)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .when(rtl, Styled::items_end)
                                    .child(
                                        Text::new("GPUI Material")
                                            .id("overview-hero-title")
                                            .role(TypeRole::DisplaySmall)
                                            .emphasized(true)
                                            .wrap(),
                                    )
                                    .child(
                                        Text::new(
                                            "Material 3 Expressive components, dynamic color schemes, 35 shapes, and adaptive desktop layouts built natively for GPUI.",
                                        )
                                        .id("overview-hero-subtitle")
                                        .role(TypeRole::BodyLarge)
                                        .color(ColorRole::OnSurfaceVariant)
                                        .wrap(),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap_2()
                                    .when(rtl, Styled::flex_row_reverse)
                                    .child(
                                        Button::new(
                                            "overview-cta-components",
                                            "Explore components",
                                        )
                                        .variant(ButtonVariant::Filled)
                                        .on_activate(move |window, cx| {
                                            nav_components(
                                                GalleryPage::Story(Story::Button),
                                                window,
                                                cx,
                                            );
                                        }),
                                    )
                                    .child(
                                        Button::new("overview-cta-styles", "Explore styles")
                                            .variant(ButtonVariant::FilledTonal)
                                            .on_activate(move |window, cx| {
                                                nav_styles(
                                                    GalleryPage::Style(StylePage::Color),
                                                    window,
                                                    cx,
                                                );
                                            }),
                                    )
                                    .child(
                                        Button::new("overview-cta-examples", "Adaptive example")
                                            .variant(ButtonVariant::Outlined)
                                            .on_activate(move |window, cx| {
                                                nav_examples(
                                                    GalleryPage::Story(Story::AdaptiveLayout),
                                                    window,
                                                    cx,
                                                );
                                            }),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex_none()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(div().flex().gap_2().child(t0).child(t1))
                            .child(div().flex().gap_2().child(t2).child(t3)),
                    ),
            )
            .into_any_element()
    }

    fn tile(
        id: &'static str,
        title: &'static str,
        subtitle: &'static str,
        window: &Window,
        cx: &App,
    ) -> gpui::Stateful<gpui::Div> {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        div()
            .id(id)
            .flex_1()
            .min_w(px(260.))
            .rounded(px(24.))
            .bg(color(theme.colors.color(ColorRole::SurfaceContainerLow)))
            .border_1()
            .border_color(color(theme.colors.color(ColorRole::OutlineVariant)))
            .p_5()
            .flex()
            .flex_col()
            .gap_4()
            .when(rtl, Styled::items_end)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .when(rtl, Styled::items_end)
                    .child(
                        Text::new(title)
                            .id("title")
                            .role(TypeRole::TitleMedium)
                            .emphasized(true),
                    )
                    .child(
                        Text::new(subtitle)
                            .id("subtitle")
                            .role(TypeRole::BodySmall)
                            .color(ColorRole::OnSurfaceVariant)
                            .wrap(),
                    ),
            )
    }

    fn render_theme_tile(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = window_material_theme(window, cx);
        let appearance = theme.colors.appearance();
        let active_preset = (self.active_preset)(cx);
        let outline = color(theme.colors.color(ColorRole::Outline));
        let on_primary = color(theme.colors.color(ColorRole::OnPrimary));
        let on_select = self.on_select_preset.clone();
        let on_appearance = self.on_set_appearance.clone();

        let swatches = SWATCH_PRESETS
            .into_iter()
            .enumerate()
            .map(|(slot, preset)| {
                let selected = preset == active_preset;
                let (primary, container) = cached_swatch_pair(slot, appearance);
                let on_select = on_select.clone();
                Surface::clickable(
                    (ElementId::from("overview-swatch"), preset_label(preset)),
                    format!("Use {} theme", preset_label(preset)),
                    ColorRole::SurfaceContainerLow,
                )
                .shape(CornerRole::Full.shape())
                .size(px(44.))
                .flex()
                .items_center()
                .justify_center()
                .on_activate(move |window, cx| {
                    on_select(preset, window, cx);
                })
                .child(
                    div()
                        .size(px(32.))
                        .rounded_full()
                        .bg(primary)
                        .border_2()
                        .border_color(if selected { outline } else { container })
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(selected, |swatch| {
                            swatch.child(
                                Icon::new(IconName::Check)
                                    .size(px(16.))
                                    .color_override(on_primary),
                            )
                        }),
                )
            });

        let to_light = on_appearance.clone();
        let to_dark = on_appearance;

        Self::tile(
            "overview-theme-tile",
            "Make it yours",
            "Pick a Material 3 palette preset and switch light or dark appearance live.",
            window,
            cx,
        )
        .child(div().flex().flex_wrap().gap_2().children(swatches))
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .child(
                    ToggleButton::new(
                        "overview-mode-light",
                        "Light",
                        appearance == Appearance::Light,
                    )
                    .variant(ToggleButtonVariant::Outlined)
                    .size(ControlSize::ExtraSmall)
                    .on_change(move |checked, window, cx| {
                        if checked {
                            to_light(Appearance::Light, window, cx);
                        }
                    }),
                )
                .child(
                    ToggleButton::new("overview-mode-dark", "Dark", appearance == Appearance::Dark)
                        .variant(ToggleButtonVariant::Outlined)
                        .size(ControlSize::ExtraSmall)
                        .on_change(move |checked, window, cx| {
                            if checked {
                                to_dark(Appearance::Dark, window, cx);
                            }
                        }),
                ),
        )
        .into_any_element()
    }

    fn render_live_tile(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let toggles: [(IconName, &'static str); 3] = [
            (IconName::Favorite, "Favorite"),
            (IconName::Notifications, "Notifications"),
            (IconName::Archive, "Archive"),
        ];
        let owner = cx.weak_entity();
        let switch_owner = owner.clone();

        Self::tile(
            "overview-live-tile",
            "Try it live",
            "Real interactive controls driven by the active theme and motion policy.",
            window,
            cx,
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            Switch::new("overview-live-switch", "Live switch", self.live_switch)
                                .show_label(false)
                                .on_change(move |checked, _, cx| {
                                    let _ = switch_owner.update(cx, |this, cx| {
                                        this.live_switch = checked;
                                        cx.notify();
                                    });
                                }),
                        )
                        .child(
                            Text::new("Expressive")
                                .id("switch-label")
                                .role(TypeRole::LabelMedium),
                        ),
                )
                .child(div().flex().items_center().gap_1().children(
                    toggles.into_iter().enumerate().map(|(ix, (icon, label))| {
                        let selected = self.live_toggles[ix];
                        let owner = owner.clone();
                        IconToggleButton::new(
                            ("overview-live-toggle", ix),
                            label,
                            Icon::new(icon).filled(selected),
                            selected,
                        )
                        .size(ControlSize::ExtraSmall)
                        .on_change(move |next, _, cx| {
                            let _ = owner.update(cx, |this, cx| {
                                this.live_toggles[ix] = next;
                                cx.notify();
                            });
                        })
                    }),
                )),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .children(LIVE_CHIPS.into_iter().enumerate().map(|(ix, label)| {
                    let owner = owner.clone();
                    Chip::new(("overview-live-chip", ix), label)
                        .variant(ChipVariant::Filter)
                        .selected(self.live_chips[ix])
                        .on_change(move |next, _, cx| {
                            let _ = owner.update(cx, |this, cx| {
                                this.live_chips[ix] = next;
                                cx.notify();
                            });
                        })
                })),
        )
        .child(
            Slider::new("overview-live-slider", "Preview value", &self.live_slider)
                .size(ControlSize::ExtraSmall),
        )
        .into_any_element()
    }

    fn render_install_tile(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = window_material_theme(window, cx);
        let copied = self.copied;
        let fact = |ix: usize, icon: IconName, text: &'static str| {
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(Icon::new(icon).size(px(18.)).color(ColorRole::Primary))
                .child(
                    Text::new(text)
                        .id(("fact", ix))
                        .role(TypeRole::BodySmall)
                        .color(ColorRole::OnSurfaceVariant),
                )
        };

        Self::tile(
            "overview-install-tile",
            "Add it to your app",
            "One facade crate with optional feature families for extras and dynamic color.",
            window,
            cx,
        )
        .child(
            div()
                .w_full()
                .rounded(px(12.))
                .bg(color(
                    theme.colors.color(ColorRole::SurfaceContainerHighest),
                ))
                .pl_3()
                .pr_1()
                .py_1()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    Text::new(INSTALL_CMD)
                        .id("overview-install-cmd")
                        .role(TypeRole::BodyMedium),
                )
                .child(
                    IconButton::new(
                        "overview-copy-install",
                        if copied { "Copied" } else { "Copy command" },
                        Icon::new(if copied {
                            IconName::Check
                        } else {
                            IconName::Description
                        }),
                    )
                    .size(ControlSize::ExtraSmall)
                    .on_activate({
                        let owner = cx.weak_entity();
                        move |_, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(
                                INSTALL_CMD.to_owned(),
                            ));
                            let _ = owner.update(cx, |this, cx| {
                                this.copied = true;
                                cx.notify();
                            });
                        }
                    }),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(fact(
                    0,
                    IconName::Check,
                    "Windows, macOS, and Linux native GPUI",
                ))
                .child(fact(
                    1,
                    IconName::Contrast,
                    "18 bundled theme presets + dynamic color",
                ))
                .child(fact(
                    2,
                    IconName::MotionPhotosOn,
                    "35 Expressive shapes & spring motion",
                )),
        )
        .into_any_element()
    }

    fn render_library_grid(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let thumb_bg = color(theme.colors.color(ColorRole::Surface));
        let on_surface = color(theme.colors.color(ColorRole::OnSurface));
        let on_surface_variant = color(theme.colors.color(ColorRole::OnSurfaceVariant));
        let available = f32::from(self.available_width.get());
        let cols = ((available + 12.) / 252.).floor().clamp(1., 4.) as u16;

        let groups = CatalogGroup::COMPONENT_GROUPS.into_iter().map(|group| {
            let entries: Vec<_> = entries_in_group(group).collect();
            let count_label = format!("{} components", entries.len());
            let (radius, accent_role, icon_role, icon) = group_visual(group);
            let accent = color(theme.colors.color(accent_role));
            let cards = entries.into_iter().map(|(_, entry)| {
                let on_navigate = self.on_navigate.clone();
                let page = entry.page;
                Surface::clickable(
                    (ElementId::from("overview-card"), page.id()),
                    page.name(),
                    ColorRole::SurfaceContainerLow,
                )
                .shape(CornerRole::Large.shape())
                .border(SurfaceBorder {
                    width: px(1.),
                    color: outline,
                })
                .w_full()
                .overflow_hidden()
                .flex()
                .flex_col()
                .on_activate(move |window, cx| {
                    on_navigate(page, window, cx);
                })
                .child(
                    div()
                        .h(px(84.))
                        .w_full()
                        .bg(thumb_bg)
                        .border_b_1()
                        .border_color(outline)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .size(px(48.))
                                .rounded(px(radius))
                                .bg(accent)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(Icon::new(icon).size(px(22.)).color(icon_role)),
                        ),
                )
                .child(
                    div()
                        .p_4()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .when(rtl, Styled::items_end)
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(on_surface)
                                .child(page.name()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(on_surface_variant)
                                .child(page.description()),
                        ),
                )
            });

            div()
                .id((ElementId::from("overview-group"), group.id()))
                .w_full()
                .flex()
                .flex_col()
                .gap_3()
                .when(rtl, Styled::items_end)
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_baseline()
                        .gap_3()
                        .when(rtl, Styled::flex_row_reverse)
                        .child(
                            Text::new(group.title())
                                .id("title")
                                .role(TypeRole::TitleLarge)
                                .emphasized(true),
                        )
                        .child(
                            Text::new(count_label)
                                .id("count")
                                .role(TypeRole::LabelMedium)
                                .color(ColorRole::OnSurfaceVariant),
                        ),
                )
                .child(
                    div()
                        .w_full()
                        .grid()
                        .grid_cols(cols)
                        .gap_3()
                        .children(cards),
                )
        });

        div()
            .id("overview-library")
            .w_full()
            .pt_4()
            .flex()
            .flex_col()
            .gap_6()
            .when(rtl, Styled::items_end)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .when(rtl, Styled::items_end)
                    .child(
                        Text::new("Browse the library")
                            .id("heading")
                            .role(TypeRole::HeadlineSmall)
                            .emphasized(true),
                    )
                    .child(
                        Text::new(format!(
                            "{} components organized into {} Material 3 groups. Select any card to open its interactive examples.",
                            component_count(),
                            CatalogGroup::COMPONENT_GROUPS.len()
                        ))
                        .id("subheading")
                        .role(TypeRole::BodyMedium)
                        .color(ColorRole::OnSurfaceVariant)
                        .wrap(),
                    ),
            )
            .children(groups)
            .into_any_element()
    }
}

fn group_visual(group: CatalogGroup) -> (f32, ColorRole, ColorRole, IconName) {
    match group {
        CatalogGroup::Actions => (
            24.,
            ColorRole::PrimaryContainer,
            ColorRole::OnPrimaryContainer,
            IconName::Add,
        ),
        CatalogGroup::SelectionInput => (
            16.,
            ColorRole::SecondaryContainer,
            ColorRole::OnSecondaryContainer,
            IconName::Edit,
        ),
        CatalogGroup::Navigation => (
            20.,
            ColorRole::TertiaryContainer,
            ColorRole::OnTertiaryContainer,
            IconName::ArrowForward,
        ),
        CatalogGroup::Containment => (
            12.,
            ColorRole::PrimaryContainer,
            ColorRole::OnPrimaryContainer,
            IconName::Inbox,
        ),
        CatalogGroup::Communication => (
            24.,
            ColorRole::SecondaryContainer,
            ColorRole::OnSecondaryContainer,
            IconName::Notifications,
        ),
        _ => (
            24.,
            ColorRole::PrimaryContainer,
            ColorRole::OnPrimaryContainer,
            IconName::Info,
        ),
    }
}

impl Render for OverviewView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("gallery-overview")
            .test_support()
            .w_full()
            .flex()
            .flex_col()
            .gap_5()
            .child(self.render_hero(window, cx))
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .gap_4()
                    .child(self.render_theme_tile(window, cx))
                    .child(self.render_live_tile(window, cx))
                    .child(self.render_install_tile(window, cx)),
            )
            .child(self.render_library_grid(window, cx))
    }
}

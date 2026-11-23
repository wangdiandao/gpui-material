use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, ClipboardItem, Context, ElementId, Hsla, InteractiveElement,
    IntoElement, ParentElement, Pixels, Render, Role, StatefulInteractiveElement, Styled, Window,
    canvas, div, px,
};
use gpui_material::{
    Button, ButtonVariant, Chip, ChipVariant, ControlSize, MaterialShape, Shape, Surface, Text,
    ToggleButton, ToggleButtonVariant,
    foundation::{animate_with_status, color, window_material_theme},
    theme::{
        ColorRole, CornerRole, Direction, MotionKind, MotionScheme, MotionSpec, MotionSpeed,
        ThemePreset, TypeRole, TypographyEmphasis,
    },
};

use crate::{
    catalog::StylePage,
    story::{section, stack},
};

pub(crate) type ApplyPreset = Rc<dyn Fn(ThemePreset, &mut Window, &mut App)>;

pub(crate) fn preset_label(preset: ThemePreset) -> &'static str {
    match preset {
        ThemePreset::Baseline => "Baseline",
        ThemePreset::TonalSpot => "Tonal spot",
        ThemePreset::Vibrant => "Vibrant",
        ThemePreset::Expressive => "Expressive",
        ThemePreset::Fidelity => "Fidelity",
        ThemePreset::Content => "Content",
        ThemePreset::Monochrome => "Monochrome",
        ThemePreset::Neutral => "Neutral",
        ThemePreset::HighContrast => "High contrast",
        ThemePreset::Ocean => "Ocean",
        ThemePreset::Forest => "Forest",
        ThemePreset::Spring => "Spring",
        ThemePreset::Sunset => "Sunset",
        ThemePreset::Desert => "Desert",
        ThemePreset::Summer => "Summer",
        ThemePreset::BrownBeige => "Brown beige",
        ThemePreset::SageIvory => "Sage ivory",
        ThemePreset::TealCaramel => "Teal caramel",
    }
}

#[derive(Default)]
struct CachedShape {
    key: Option<(MaterialShape, Bounds<Pixels>)>,
    shape: Option<Shape>,
}

/// Paints any of the 35 Material 3 Expressive shapes inside a square box,
/// caching the tessellated mesh across redraws while shape and bounds stay unchanged.
pub(crate) fn cached_shape_canvas(
    id: impl Into<ElementId>,
    shape: MaterialShape,
    fill: Hsla,
    side: f32,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let cache = window.use_keyed_state(id.into(), cx, |_, _| CachedShape::default());
    canvas(
        move |bounds, _, cx| {
            cache.update(cx, |cached, _| {
                let key = (shape, bounds);
                if cached.key != Some(key) {
                    cached.key = Some(key);
                    cached.shape = Shape::material(bounds, shape);
                }
                cached.shape.as_ref().map(|s| s.fill_path().clone())
            })
        },
        move |_, path, window, _| {
            if let Some(path) = path {
                window.paint_path(path, fill);
            }
        },
    )
    .size(px(side))
    .flex_none()
    .into_any_element()
}

pub(crate) struct StylesExamples {
    page: StylePage,
    active_preset: Rc<dyn Fn(&App) -> ThemePreset>,
    on_select_preset: ApplyPreset,
    emphasized: bool,
    selected_shape: MaterialShape,
    copied_hex: Option<String>,
    motion_scheme: MotionScheme,
    motion_target: bool,
}

impl StylesExamples {
    pub(crate) fn new(
        page: StylePage,
        active_preset: Rc<dyn Fn(&App) -> ThemePreset>,
        on_select_preset: ApplyPreset,
    ) -> Self {
        Self {
            page,
            active_preset,
            on_select_preset,
            emphasized: false,
            selected_shape: MaterialShape::Cookie9Sided,
            copied_hex: None,
            motion_scheme: MotionScheme::Expressive,
            motion_target: false,
        }
    }

    fn render_color(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = window_material_theme(window, cx);
        let active_preset = (self.active_preset)(cx);
        let on_select = self.on_select_preset.clone();

        let preset_chips = ThemePreset::ALL.into_iter().map(|preset| {
            let selected = preset == active_preset;
            let on_select = on_select.clone();
            Chip::new(
                (ElementId::from("style-preset"), preset_label(preset)),
                preset_label(preset),
            )
            .variant(ChipVariant::Filter)
            .selected(selected)
            .on_change(move |_, window, cx| {
                on_select(preset, window, cx);
            })
        });

        let owner = cx.weak_entity();
        let copied_hex = self.copied_hex.clone();
        let swatch = |title: &'static str, bg_role: ColorRole, fg_role: ColorRole| {
            let bg = theme.colors.color(bg_role);
            let [_, r, g, b] = bg.channels();
            let hex = format!("#{r:02X}{g:02X}{b:02X}");
            let copy_hex = hex.clone();
            let owner = owner.clone();
            let is_copied = copied_hex.as_deref() == Some(hex.as_str());
            div()
                .id((ElementId::from("color-swatch"), title))
                .role(Role::Button)
                .aria_label(format!("Copy {title} {hex}"))
                .cursor_pointer()
                .flex_1()
                .min_w(px(164.))
                .rounded(px(12.))
                .p_3()
                .bg(color(bg))
                .border_1()
                .border_color(color(theme.colors.color(ColorRole::OutlineVariant)))
                .flex()
                .flex_col()
                .justify_between()
                .gap_3()
                .on_click(move |_, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(copy_hex.clone()));
                    let _ = owner.update(cx, |this, cx| {
                        this.copied_hex = Some(copy_hex.clone());
                        cx.notify();
                    });
                })
                .child(
                    Text::new(title)
                        .id("label")
                        .role(TypeRole::LabelLarge)
                        .color(fg_role),
                )
                .child(
                    Text::new(if is_copied {
                        format!("{hex} · Copied")
                    } else {
                        hex
                    })
                    .id("hex")
                    .role(TypeRole::LabelSmall)
                    .color(fg_role),
                )
        };

        let group_row = |items: Vec<gpui::Stateful<gpui::Div>>| {
            div().w_full().flex().flex_wrap().gap_2().children(items)
        };

        stack()
            .child(
                section("Theme presets")
                    .description(
                        "Switch the active Material 3 palette across the entire Gallery while keeping the current light or dark appearance.",
                    )
                    .child(div().flex().flex_wrap().gap_2().children(preset_chips)),
            )
            .child(
                section("Accent roles")
                    .description(
                        "Primary, secondary, and tertiary roles with their container pairs.",
                    )
                    .child(group_row(vec![
                        swatch("Primary", ColorRole::Primary, ColorRole::OnPrimary),
                        swatch(
                            "Primary container",
                            ColorRole::PrimaryContainer,
                            ColorRole::OnPrimaryContainer,
                        ),
                        swatch("Secondary", ColorRole::Secondary, ColorRole::OnSecondary),
                        swatch(
                            "Secondary container",
                            ColorRole::SecondaryContainer,
                            ColorRole::OnSecondaryContainer,
                        ),
                        swatch("Tertiary", ColorRole::Tertiary, ColorRole::OnTertiary),
                        swatch(
                            "Tertiary container",
                            ColorRole::TertiaryContainer,
                            ColorRole::OnTertiaryContainer,
                        ),
                    ])),
            )
            .child(
                section("Status & error roles")
                    .description(
                        "Error and harmonized status roles resolved by the active color scheme.",
                    )
                    .child(group_row(vec![
                        swatch("Error", ColorRole::Error, ColorRole::OnError),
                        swatch(
                            "Error container",
                            ColorRole::ErrorContainer,
                            ColorRole::OnErrorContainer,
                        ),
                        swatch("Info", ColorRole::Info, ColorRole::OnInfo),
                        swatch(
                            "Info container",
                            ColorRole::InfoContainer,
                            ColorRole::OnInfoContainer,
                        ),
                        swatch("Success", ColorRole::Success, ColorRole::OnSuccess),
                        swatch(
                            "Success container",
                            ColorRole::SuccessContainer,
                            ColorRole::OnSuccessContainer,
                        ),
                        swatch("Warning", ColorRole::Warning, ColorRole::OnWarning),
                        swatch(
                            "Warning container",
                            ColorRole::WarningContainer,
                            ColorRole::OnWarningContainer,
                        ),
                    ])),
            )
            .child(
                section("Surface hierarchy")
                    .description(
                        "Tonal surface containers from lowest to highest, plus inverse and outline tokens.",
                    )
                    .child(group_row(vec![
                        swatch(
                            "Surface container lowest",
                            ColorRole::SurfaceContainerLowest,
                            ColorRole::OnSurface,
                        ),
                        swatch(
                            "Surface container low",
                            ColorRole::SurfaceContainerLow,
                            ColorRole::OnSurface,
                        ),
                        swatch(
                            "Surface container",
                            ColorRole::SurfaceContainer,
                            ColorRole::OnSurface,
                        ),
                        swatch(
                            "Surface container high",
                            ColorRole::SurfaceContainerHigh,
                            ColorRole::OnSurface,
                        ),
                        swatch(
                            "Surface container highest",
                            ColorRole::SurfaceContainerHighest,
                            ColorRole::OnSurface,
                        ),
                        swatch(
                            "Inverse surface",
                            ColorRole::InverseSurface,
                            ColorRole::InverseOnSurface,
                        ),
                    ])),
            )
            .into_any_element()
    }

    fn render_typography(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = window_material_theme(window, cx);
        let emphasis = if self.emphasized {
            TypographyEmphasis::Emphasized
        } else {
            TypographyEmphasis::Baseline
        };
        let roles: [(&str, TypeRole); 15] = [
            ("Display Large", TypeRole::DisplayLarge),
            ("Display Medium", TypeRole::DisplayMedium),
            ("Display Small", TypeRole::DisplaySmall),
            ("Headline Large", TypeRole::HeadlineLarge),
            ("Headline Medium", TypeRole::HeadlineMedium),
            ("Headline Small", TypeRole::HeadlineSmall),
            ("Title Large", TypeRole::TitleLarge),
            ("Title Medium", TypeRole::TitleMedium),
            ("Title Small", TypeRole::TitleSmall),
            ("Body Large", TypeRole::BodyLarge),
            ("Body Medium", TypeRole::BodyMedium),
            ("Body Small", TypeRole::BodySmall),
            ("Label Large", TypeRole::LabelLarge),
            ("Label Medium", TypeRole::LabelMedium),
            ("Label Small", TypeRole::LabelSmall),
        ];

        let rows = roles.into_iter().map(|(label, role)| {
            let style = role.style(emphasis);
            let metrics = format!(
                "{:.0} sp / {:.0} sp · w{} · tracking {:+.2}",
                style.size, style.line_height, style.weight, style.tracking
            );
            div()
                .id((ElementId::from("type-role"), label))
                .w_full()
                .py_3()
                .border_b_1()
                .border_color(color(theme.colors.color(ColorRole::OutlineVariant)))
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    Text::new(metrics)
                        .id("metrics")
                        .role(TypeRole::LabelSmall)
                        .color(ColorRole::OnSurfaceVariant),
                )
                .child(
                    Text::new(label)
                        .id("sample")
                        .role(role)
                        .emphasized(self.emphasized)
                        .wrap(),
                )
        });

        let owner_baseline = cx.weak_entity();
        let owner_emphasized = cx.weak_entity();

        stack()
            .child(
                section("Type scale")
                    .description(
                        "Toggle between Baseline and Emphasized Material 3 Expressive typography scales.",
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_2()
                            .child(
                                ToggleButton::new(
                                    "typography-baseline",
                                    "Baseline",
                                    !self.emphasized,
                                )
                                .variant(ToggleButtonVariant::Outlined)
                                .size(ControlSize::Small)
                                .on_change(move |checked, _, cx| {
                                    if checked {
                                        let _ = owner_baseline.update(cx, |this, cx| {
                                            this.emphasized = false;
                                            cx.notify();
                                        });
                                    }
                                }),
                            )
                            .child(
                                ToggleButton::new(
                                    "typography-emphasized",
                                    "Emphasized",
                                    self.emphasized,
                                )
                                .variant(ToggleButtonVariant::Outlined)
                                .size(ControlSize::Small)
                                .on_change(move |checked, _, cx| {
                                    if checked {
                                        let _ = owner_emphasized.update(cx, |this, cx| {
                                            this.emphasized = true;
                                            cx.notify();
                                        });
                                    }
                                }),
                            ),
                    )
                    .child(div().flex().flex_col().w_full().children(rows)),
            )
            .into_any_element()
    }

    fn render_shape(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = window_material_theme(window, cx);
        let surface_bg = color(theme.colors.color(ColorRole::Surface));
        let fill = color(theme.colors.color(ColorRole::PrimaryContainer));
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let polygon_fill = color(theme.colors.color(ColorRole::Primary));
        let corners: [(&str, CornerRole); 9] = [
            ("None (0 dp)", CornerRole::None),
            ("Extra small (4 dp)", CornerRole::ExtraSmall),
            ("Small (8 dp)", CornerRole::Small),
            ("Medium (12 dp)", CornerRole::Medium),
            ("Large (16 dp)", CornerRole::Large),
            ("Large increased (20 dp)", CornerRole::LargeIncreased),
            ("Extra large (28 dp)", CornerRole::ExtraLarge),
            ("Extra extra large (48 dp)", CornerRole::ExtraExtraLarge),
            ("Full", CornerRole::Full),
        ];

        let corner_cards = corners.into_iter().map(|(label, role)| {
            let [tl, tr, br, bl] = role.shape().radii(96., 64., Direction::Ltr);
            div()
                .id((ElementId::from("corner-role"), label))
                .w(px(148.))
                .p_3()
                .rounded(px(12.))
                .bg(surface_bg)
                .border_1()
                .border_color(outline)
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w(px(96.))
                        .h(px(64.))
                        .bg(fill)
                        .rounded_tl(px(tl))
                        .rounded_tr(px(tr))
                        .rounded_br(px(br))
                        .rounded_bl(px(bl)),
                )
                .child(
                    Text::new(label)
                        .id("label")
                        .role(TypeRole::LabelSmall)
                        .color(ColorRole::OnSurfaceVariant),
                )
        });

        let selected = self.selected_shape;
        let selected_outline = polygon_fill;
        let hero_shape = cached_shape_canvas(
            "style-shape-spotlight",
            selected,
            polygon_fill,
            112.,
            window,
            cx,
        );
        let owner = cx.weak_entity();
        let shape_cards: Vec<_> = MaterialShape::ALL
            .into_iter()
            .map(|shape| {
                let is_selected = shape == selected;
                let owner = owner.clone();
                let shape_el = cached_shape_canvas(
                    (ElementId::from("style-shape-mesh"), shape.name()),
                    shape,
                    polygon_fill,
                    56.,
                    window,
                    cx,
                );
                div()
                    .id((ElementId::from("shape-card"), shape.name()))
                    .role(Role::Button)
                    .aria_label(format!("Inspect shape {}", shape.name()))
                    .cursor_pointer()
                    .w(px(128.))
                    .p_3()
                    .rounded(px(12.))
                    .bg(surface_bg)
                    .border_1()
                    .border_color(if is_selected {
                        selected_outline
                    } else {
                        outline
                    })
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .on_click(move |_, _, cx| {
                        let _ = owner.update(cx, |this, cx| {
                            this.selected_shape = shape;
                            cx.notify();
                        });
                    })
                    .child(shape_el)
                    .child(
                        Text::new(shape.name())
                            .id("label")
                            .role(TypeRole::LabelSmall)
                            .color(ColorRole::OnSurfaceVariant),
                    )
            })
            .collect();

        stack()
            .child(
                section("Shape inspector")
                    .description("Select any of the 35 Expressive shapes below to inspect it at full scale.")
                    .child(
                        div()
                            .id("style-shape-spotlight-card")
                            .w_full()
                            .p_4()
                            .rounded(px(16.))
                            .bg(surface_bg)
                            .border_1()
                            .border_color(outline)
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_6()
                            .child(hero_shape)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        Text::new(format!("MaterialShape::{selected:?}"))
                                            .id("spotlight-title")
                                            .role(TypeRole::TitleMedium)
                                            .emphasized(true),
                                    )
                                    .child(
                                        Text::new(format!(
                                            "Shape::material(bounds, MaterialShape::{selected:?})"
                                        ))
                                        .id("spotlight-code")
                                        .role(TypeRole::BodySmall)
                                        .color(ColorRole::OnSurfaceVariant),
                                    ),
                            ),
                    ),
            )
            .child(
                section("Corner scale")
                    .description(
                        "Rounded corner tokens resolved with Material 3 logical corner shapes.",
                    )
                    .child(div().flex().flex_wrap().gap_3().children(corner_cards)),
            )
            .child(
                section("Expressive shapes (35)")
                    .description(
                        "All 35 Material 3 Expressive polygon shapes rendered through gpui-material-foundation.",
                    )
                    .child(div().flex().flex_wrap().gap_3().children(shape_cards)),
            )
            .into_any_element()
    }

    fn render_motion(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = window_material_theme(window, cx);
        let reduced = cx.reduce_motion();
        let target = if self.motion_target { 1.0 } else { 0.0 };
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let track_bg = color(theme.colors.color(ColorRole::SurfaceContainerHighest));
        let knob_bg = color(theme.colors.color(ColorRole::Primary));

        let specs: [(&str, MotionKind, MotionSpeed); 6] = [
            ("Spatial · Fast", MotionKind::Spatial, MotionSpeed::Fast),
            (
                "Spatial · Default",
                MotionKind::Spatial,
                MotionSpeed::Default,
            ),
            ("Spatial · Slow", MotionKind::Spatial, MotionSpeed::Slow),
            ("Effects · Fast", MotionKind::Effects, MotionSpeed::Fast),
            (
                "Effects · Default",
                MotionKind::Effects,
                MotionSpeed::Default,
            ),
            ("Effects · Slow", MotionKind::Effects, MotionSpeed::Slow),
        ];

        let tracks: Vec<_> = specs
            .into_iter()
            .enumerate()
            .map(|(ix, (label, kind, speed))| {
                let spec = self.motion_scheme.spec(kind, speed, reduced);
                let sample = animate_with_status(
                    ElementId::from(("style-motion-track", ix)),
                    target,
                    spec,
                    window,
                    cx,
                );
                let progress = sample.value.clamp(0.0, 1.0);
                let details = match self.motion_scheme.spec(kind, speed, false) {
                    MotionSpec::Spring(spring) => format!(
                        "damping {:.2} · stiffness {:.0}{}",
                        spring.damping_ratio,
                        spring.stiffness,
                        if reduced { " (reduced -> instant)" } else { "" }
                    ),
                    _ => "instant".into(),
                };
                div().id(("motion-track-row", ix)).w_full().child(
                    Surface::new(ColorRole::Surface)
                        .w_full()
                        .rounded(px(12.))
                        .border_1()
                        .border_color(outline)
                        .p_3()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .items_center()
                                .justify_between()
                                .gap_2()
                                .child(Text::new(label).id("label").role(TypeRole::TitleSmall))
                                .child(
                                    Text::new(details)
                                        .id("details")
                                        .role(TypeRole::LabelSmall)
                                        .color(ColorRole::OnSurfaceVariant),
                                ),
                        )
                        .child(
                            div()
                                .w(px(280.))
                                .h(px(20.))
                                .rounded_full()
                                .bg(track_bg)
                                .relative()
                                .child(
                                    div()
                                        .absolute()
                                        .top(px(2.))
                                        .left(px(2. + progress * 244.))
                                        .w(px(32.))
                                        .h(px(16.))
                                        .rounded_full()
                                        .bg(knob_bg),
                                ),
                        ),
                )
            })
            .collect();

        let owner_standard = cx.weak_entity();
        let owner_expressive = cx.weak_entity();
        let owner_play = cx.weak_entity();

        stack()
            .child(
                section("Spring physics")
                    .description(
                        "Compare Material 3 Standard and Expressive spring tokens. Spatial springs overshoot slightly in Expressive mode; Effects springs remain critically damped.",
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_2()
                            .child(
                                ToggleButton::new(
                                    "motion-standard",
                                    "Standard",
                                    self.motion_scheme == MotionScheme::Standard,
                                )
                                .variant(ToggleButtonVariant::Outlined)
                                .size(ControlSize::Small)
                                .on_change(move |checked, _, cx| {
                                    if checked {
                                        let _ = owner_standard.update(cx, |this, cx| {
                                            this.motion_scheme = MotionScheme::Standard;
                                            cx.notify();
                                        });
                                    }
                                }),
                            )
                            .child(
                                ToggleButton::new(
                                    "motion-expressive",
                                    "Expressive",
                                    self.motion_scheme == MotionScheme::Expressive,
                                )
                                .variant(ToggleButtonVariant::Outlined)
                                .size(ControlSize::Small)
                                .on_change(move |checked, _, cx| {
                                    if checked {
                                        let _ = owner_expressive.update(cx, |this, cx| {
                                            this.motion_scheme = MotionScheme::Expressive;
                                            cx.notify();
                                        });
                                    }
                                }),
                            )
                            .child(
                                Button::new("motion-play", "Trigger transition")
                                    .variant(ButtonVariant::Filled)
                                    .size(ControlSize::Small)
                                    .on_activate(move |_, cx| {
                                        let _ = owner_play.update(cx, |this, cx| {
                                            this.motion_target = !this.motion_target;
                                            cx.notify();
                                        });
                                    }),
                            ),
                    )
                    .child(div().flex().flex_col().gap_2().w_full().children(tracks)),
            )
            .into_any_element()
    }
}

impl Render for StylesExamples {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.page {
            StylePage::Color => self.render_color(window, cx),
            StylePage::Typography => self.render_typography(window, cx),
            StylePage::Shape => self.render_shape(window, cx),
            StylePage::Motion => self.render_motion(window, cx),
        }
    }
}

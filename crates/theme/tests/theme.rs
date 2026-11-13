use gpui_material_theme::{
    Appearance, Argb, ColorRole, ColorScheme, CornerRole, CornerShape, Density, Direction,
    Elevation, FontFamilies, LayoutPolicy, MaterialTheme, MotionKind, MotionScheme, MotionSpec,
    MotionSpeed, PolygonRole, PolygonSpec, Spring, StateLayer, TokenValue, TypeRole, Typeface,
    TypographyEmphasis, WidthClass, token,
};

#[test]
fn static_appearances_resolve_color_pairs_and_elevation() {
    let mut light = ColorScheme::baseline(Appearance::Light);
    let dark = ColorScheme::baseline(Appearance::Dark);

    assert_eq!(light.color(ColorRole::Primary).to_u32(), 0xff67_50a4);
    assert_eq!(dark.color(ColorRole::Primary).to_u32(), 0xffd0_bcff);
    assert_eq!(
        light.color(ColorRole::PrimaryFixed),
        dark.color(ColorRole::PrimaryFixed)
    );
    assert_eq!(
        light.pair(ColorRole::Primary).unwrap().foreground.to_u32(),
        0xffff_ffff
    );
    assert_eq!(
        dark.pair(ColorRole::SurfaceContainerHighest)
            .unwrap()
            .foreground,
        dark.color(ColorRole::OnSurface)
    );
    assert!(dark.pair(ColorRole::OnPrimary).is_none());
    assert_eq!(light.color(ColorRole::Scrim).to_u32(), 0xff00_0000);
    assert_eq!(
        light.surface_at_elevation(Elevation::Level0),
        light.color(ColorRole::Surface)
    );
    assert_eq!(
        light.surface_at_elevation(Elevation::Level1).to_u32(),
        0xfff6_eefa
    );
    assert_eq!(
        dark.surface_at_elevation(Elevation::Level1).to_u32(),
        0xff1e_1b24
    );

    let custom = Argb::from_rgb(1, 2, 3);
    light.set_color(ColorRole::OnPrimary, custom);
    assert_eq!(light.pair(ColorRole::Primary).unwrap().foreground, custom);
    light.set_color(ColorRole::OnPrimary, Argb::from_u32(0xffff_ffff));
    assert_eq!(light, ColorScheme::default());
}

#[test]
fn component_references_preserve_semantic_roles_and_state_shapes() {
    assert_eq!(
        token("md.comp.filled-button.container-color"),
        Some(TokenValue::ColorRole(ColorRole::Primary))
    );
    assert_eq!(
        token("md.comp.filled-button.disabled-label-text-opacity"),
        Some(TokenValue::Opacity(0.38))
    );
    assert_eq!(
        token("md.comp.filled-button.hovered-container-elevation"),
        Some(TokenValue::Dp(1.0))
    );
    assert_eq!(
        token("md.comp.button.small.container-shape-round"),
        Some(TokenValue::Corners(CornerShape::Full))
    );
    assert_eq!(
        token("md.comp.button.small.pressed-container-shape"),
        Some(TokenValue::Corners(CornerRole::Small.shape()))
    );
    assert_eq!(StateLayer::Hover.opacity(), 0.08);
    assert_eq!(StateLayer::Focus.opacity(), 0.1);
    assert_eq!(StateLayer::Pressed.opacity(), 0.1);
    assert_eq!(StateLayer::Dragged.opacity(), 0.16);
    assert_eq!(token("md.comp.unknown"), None);
}

#[test]
fn type_scale_preserves_metrics_and_font_preferences() {
    use TypographyEmphasis::{Baseline, Emphasized};

    let display = TypeRole::DisplayLarge.style(Baseline);
    assert_eq!(
        (
            display.typeface,
            display.size,
            display.line_height,
            display.tracking,
            display.weight
        ),
        (Typeface::Brand, 57.0, 64.0, -0.2, 400)
    );
    let body = TypeRole::BodyLarge.style(Emphasized);
    assert_eq!(
        (
            body.typeface,
            body.size,
            body.line_height,
            body.tracking,
            body.weight
        ),
        (Typeface::Plain, 16.0, 24.0, 0.15, 500)
    );
    for role in TypeRole::ALL {
        let baseline = role.style(Baseline);
        let emphasized = role.style(Emphasized);
        assert_eq!(baseline.size, emphasized.size);
        assert_eq!(baseline.line_height, emphasized.line_height);
        assert!(emphasized.weight > baseline.weight);
    }
    let mut theme = MaterialTheme::default();
    assert_eq!(theme.typography.families(Typeface::Plain).primary, "Roboto");
    theme.typography.plain = FontFamilies {
        primary: "Segoe UI".into(),
        fallback: vec!["Microsoft YaHei UI".into()],
    };
    assert_eq!(
        theme.typography.families(body.typeface).fallback,
        ["Microsoft YaHei UI"]
    );
    assert_eq!(
        theme.typography.families(display.typeface).primary,
        "Roboto"
    );

    let scaled = LayoutPolicy::scale_text(body, 2.0);
    assert_eq!(
        (
            scaled.size,
            scaled.line_height,
            scaled.tracking,
            scaled.weight
        ),
        (32.0, 48.0, 0.30, 500)
    );
    for scale in [f32::NAN, f32::INFINITY, 0.0, -1.0, f32::MAX] {
        assert_eq!(LayoutPolicy::scale_text(body, scale), body);
    }
}

#[test]
fn corners_and_polygon_endpoints_preserve_source_parameters() {
    assert_eq!(
        CornerRole::LargeStart
            .shape()
            .radii(100.0, 30.0, Direction::Ltr),
        [15.0, 0.0, 0.0, 15.0]
    );
    assert_eq!(
        CornerRole::LargeStart
            .shape()
            .radii(100.0, 30.0, Direction::Rtl),
        [0.0, 15.0, 15.0, 0.0]
    );
    assert_eq!(
        CornerRole::ExtraExtraLarge
            .shape()
            .radii(72.0, 120.0, Direction::Ltr),
        [36.0; 4]
    );
    assert_eq!(
        CornerRole::Full.shape().radii(30.0, 80.0, Direction::Rtl),
        [15.0; 4]
    );
    assert_eq!(
        CornerRole::Full
            .shape()
            .radii(f32::NAN, 80.0, Direction::Rtl),
        [0.0; 4]
    );

    assert_eq!(
        PolygonRole::Circle.spec(),
        PolygonSpec::Circle {
            vertices: 10,
            radius: 1.0
        }
    );
    assert_eq!(
        PolygonRole::Square.spec(),
        PolygonSpec::Rectangle {
            width: 1.0,
            height: 1.0,
            rounding_radius: 0.3,
            smoothing: 0.0,
        }
    );
    assert_eq!(
        PolygonRole::Triangle.spec(),
        PolygonSpec::Regular {
            vertices: 3,
            radius: 1.0,
            rounding_radius: 0.2,
            smoothing: 0.0,
            rotation_degrees: -90.0,
        }
    );
}

#[test]
fn schemes_distinguish_spatial_and_effects_and_reduce_motion() {
    use MotionKind::{Effects, Spatial};
    use MotionScheme::{Expressive, Standard};
    use MotionSpeed::{Default, Fast, Slow};

    for (speed, standard, expressive, effects) in [
        (Fast, 1400.0, (0.6, 800.0), 3800.0),
        (Default, 700.0, (0.8, 380.0), 1600.0),
        (Slow, 300.0, (0.8, 200.0), 800.0),
    ] {
        let spring = |damping_ratio, stiffness| {
            MotionSpec::Spring(Spring {
                damping_ratio,
                stiffness,
            })
        };
        assert_eq!(Standard.spec(Spatial, speed, false), spring(0.9, standard));
        assert_eq!(
            Expressive.spec(Spatial, speed, false),
            spring(expressive.0, expressive.1)
        );
        for scheme in [Standard, Expressive] {
            assert_eq!(scheme.spec(Effects, speed, false), spring(1.0, effects));
            for kind in [Spatial, Effects] {
                assert_eq!(scheme.spec(kind, speed, true), MotionSpec::Instant);
            }
        }
    }
    assert_eq!(
        token("md.sys.motion.duration-medium-2"),
        Some(TokenValue::Milliseconds(300))
    );
}

#[test]
fn layout_classes_use_available_width_and_density_keeps_hit_size() {
    for (threshold, below, at) in [
        (600.0, WidthClass::Compact, WidthClass::Medium),
        (840.0, WidthClass::Medium, WidthClass::Expanded),
        (1200.0, WidthClass::Expanded, WidthClass::Large),
        (1600.0, WidthClass::Large, WidthClass::ExtraLarge),
    ] {
        assert_eq!(WidthClass::from_width(threshold - 0.25), below);
        assert_eq!(WidthClass::from_width(threshold), at);
    }
    assert_eq!(WidthClass::from_width(f32::INFINITY), WidthClass::Compact);
    assert_eq!(Density::Compact.visual_size(40.0), 36.0);
    assert_eq!(Density::Standard.visual_size(40.0), 40.0);
    assert_eq!(Density::Comfortable.visual_size(40.0), 44.0);
    assert_eq!(Density::Compact.interactive_size(40.0), 48.0);
    assert_eq!(Density::Comfortable.interactive_size(64.0), 68.0);
    let rtl = LayoutPolicy {
        direction: Direction::Rtl,
        ..LayoutPolicy::default()
    };
    assert_eq!(rtl.horizontal(8, 16), (16, 8));
}

#[cfg(feature = "serde")]
#[test]
fn serde_round_trips_theme_overrides_and_keeps_baseline_roles() {
    let mut theme = MaterialTheme::baseline(Appearance::Dark);
    theme
        .colors
        .set_color(ColorRole::Primary, Argb::from_rgb(12, 34, 56));
    theme.motion = MotionScheme::Expressive;
    theme.layout = LayoutPolicy {
        density: Density::Compact,
        direction: Direction::Rtl,
    };
    let decoded: MaterialTheme =
        serde_json::from_str(&serde_json::to_string(&theme).unwrap()).unwrap();
    assert_eq!(decoded, theme);
    assert_eq!(
        decoded.colors.color(ColorRole::Surface),
        ColorScheme::baseline(Appearance::Dark).color(ColorRole::Surface)
    );
    let older: ColorScheme = serde_json::from_str(r#"{"appearance":"Dark"}"#).unwrap();
    assert_eq!(older, ColorScheme::baseline(Appearance::Dark));
}

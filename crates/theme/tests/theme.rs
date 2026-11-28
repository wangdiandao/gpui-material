use gpui_material_theme::{
    Appearance, AppearanceMode, Argb, ColorRole, ColorScheme, ColorSpec, CornerRole, CornerShape,
    Density, Direction, Elevation, FontFamilies, LayoutPolicy, LogicalEdges, MaterialShape,
    MaterialStrings, MaterialTheme, MotionKind, MotionScheme, MotionSpec, MotionSpeed, PolygonRole,
    PolygonSpec, Spring, StateLayer, TokenValue, TypeRole, Typeface, TypographyEmphasis,
    WidthClass, non_negative_f32, token,
};

#[test]
fn static_appearances_resolve_color_pairs_and_elevation() {
    const CONST_LIGHT: ColorScheme = ColorScheme::baseline(Appearance::Light);
    const CONST_PRIMARY: Argb = CONST_LIGHT.color(ColorRole::Primary);
    const CONST_APPEARANCE: Appearance = CONST_LIGHT.appearance();
    assert_eq!(CONST_PRIMARY.to_u32(), 0xff67_50a4);
    assert_eq!(CONST_APPEARANCE, Appearance::Light);

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
fn blends_color_roles_and_alpha_with_exact_endpoints() {
    let mut light = ColorScheme::baseline(Appearance::Light);
    let mut dark = ColorScheme::baseline(Appearance::Dark);
    light.set_color(ColorRole::Info, Argb::from_u32(0x0000_00ff));
    dark.set_color(ColorRole::Info, Argb::from_u32(0xffff_ff00));

    assert_eq!(light.blend(&dark, f32::NAN), light);
    assert_eq!(light.blend(&dark, -1.), light);
    assert_eq!(light.blend(&dark, 2.), dark);
    assert_eq!(light.blend(&dark, 0.49).appearance(), Appearance::Light);
    let halfway = light.blend(&dark, 0.5);
    assert_eq!(halfway.appearance(), Appearance::Dark);
    let info = halfway.pair(ColorRole::Info).unwrap();
    assert_eq!(info.background.to_u32(), 0x8080_8080);
    assert_eq!(info.foreground, halfway.color(ColorRole::OnInfo));
    for role in [
        ColorRole::InfoContainer,
        ColorRole::Success,
        ColorRole::SuccessContainer,
        ColorRole::Warning,
        ColorRole::WarningContainer,
    ] {
        assert!(halfway.pair(role).is_some());
    }
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
        let typography = gpui_material_theme::Typography::default().with_text_scale(2.);
        for emphasis in [Baseline, Emphasized] {
            let raw = role.style(emphasis);
            let scaled = typography.style(role, emphasis);
            assert_eq!(scaled.size, raw.size * 2.);
            assert_eq!(scaled.line_height, raw.line_height * 2.);
            assert_eq!(scaled.tracking, raw.tracking * 2.);
            assert_eq!(scaled.weight, raw.weight);
        }
    }
    let mut theme = MaterialTheme::default();
    assert_eq!(theme.typography.text_scale(), 1.);
    assert_eq!(
        theme
            .typography
            .clone()
            .with_text_scale(f32::NAN)
            .text_scale(),
        1.
    );
    assert_eq!(
        theme.typography.clone().with_text_scale(0.5).text_scale(),
        1.
    );
    assert_eq!(
        theme.typography.clone().with_text_scale(3.).text_scale(),
        2.
    );
    assert_eq!(
        theme.typography.families(Typeface::Plain).primary.as_ref(),
        "Roboto"
    );
    assert_eq!(
        theme
            .typography
            .families(Typeface::Plain)
            .fallback
            .iter()
            .map(|f| f.as_ref())
            .collect::<Vec<_>>(),
        [
            "SF Pro Text",
            "Segoe UI",
            "Noto Sans",
            "PingFang SC",
            "Microsoft YaHei UI",
            "Noto Sans CJK SC",
        ]
    );
    theme.typography.plain = FontFamilies::new("Inter", ["Noto Sans CJK SC"]);
    assert_eq!(
        theme
            .typography
            .families(body.typeface)
            .fallback
            .iter()
            .map(|f| f.as_ref())
            .collect::<Vec<_>>(),
        ["Noto Sans CJK SC"]
    );
    assert_eq!(
        theme.typography.families(display.typeface).primary.as_ref(),
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

    assert_eq!(MaterialShape::ALL.len(), 35);
    assert_eq!(
        MaterialShape::from(PolygonRole::Circle),
        MaterialShape::Circle
    );
    assert_eq!(
        MaterialShape::from(PolygonRole::Square),
        MaterialShape::Square
    );
    assert_eq!(
        MaterialShape::from(PolygonRole::Triangle),
        MaterialShape::Triangle
    );
    let mut names = std::collections::BTreeSet::new();
    let mut variants = std::collections::BTreeSet::new();
    for shape in MaterialShape::ALL {
        assert!(variants.insert(shape), "duplicate MaterialShape {shape:?}");
        assert!(
            names.insert(shape.name()),
            "duplicate name {}",
            shape.name()
        );
        assert!(!shape.name().is_empty());
    }
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
    theme.typography = theme.typography.with_text_scale(1.5);
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
    let mut serialized = serde_json::to_value(&decoded).unwrap();
    let overrides = serialized["colors"]["overrides"].as_object().unwrap();
    assert_eq!(overrides.len(), 1);
    assert!(overrides.contains_key("Primary"));
    serialized["typography"]
        .as_object_mut()
        .unwrap()
        .remove("text_scale");
    let older: MaterialTheme = serde_json::from_value(serialized.clone()).unwrap();
    assert_eq!(older.typography.text_scale(), 1.);
    serialized["typography"]["text_scale"] = serde_json::json!(2.25);
    let clamped: MaterialTheme = serde_json::from_value(serialized).unwrap();
    assert_eq!(clamped.typography.text_scale(), 2.);
}

#[test]
fn argb_color_role_on_role_corner_shape_and_fluent_theme_helpers() {
    let c = Argb::from_rgba(0x12, 0x34, 0x56, 0x80);
    assert_eq!(
        (c.alpha(), c.red(), c.green(), c.blue()),
        (0x80, 0x12, 0x34, 0x56)
    );
    assert_eq!(c.with_alpha(0xff), Argb::from_rgb(0x12, 0x34, 0x56));

    assert_eq!(ColorRole::Primary.on_role(), Some(ColorRole::OnPrimary));
    assert_eq!(
        ColorRole::SuccessContainer.on_role(),
        Some(ColorRole::OnSuccessContainer)
    );
    assert_eq!(ColorRole::Outline.on_role(), None);
    let baseline = ColorScheme::baseline(Appearance::Light);
    for role in ColorRole::ALL {
        assert_eq!(
            baseline.pair(role).map(|pair| pair.foreground),
            role.on_role().map(|fg| baseline.color(fg))
        );
    }

    let custom = baseline
        .clone()
        .with_color(ColorRole::Primary, Argb::from_rgb(1, 2, 3));
    assert_eq!(custom.color(ColorRole::Primary), Argb::from_rgb(1, 2, 3));

    assert_eq!(CornerShape::uniform(8.0), CornerShape::Radii([8.0; 4]));
    assert_eq!(
        CornerShape::top(12.0),
        CornerShape::Radii([12.0, 12.0, 0.0, 0.0])
    );
    assert_eq!(
        CornerShape::bottom(12.0),
        CornerShape::Radii([0.0, 0.0, 12.0, 12.0])
    );
    assert_eq!(
        CornerShape::start(12.0),
        CornerShape::Radii([12.0, 0.0, 0.0, 12.0])
    );
    assert_eq!(
        CornerShape::end(12.0),
        CornerShape::Radii([0.0, 12.0, 12.0, 0.0])
    );

    let theme = MaterialTheme::baseline(Appearance::Dark)
        .colors(custom.clone())
        .typography(gpui_material_theme::Typography::default().with_text_scale(1.25))
        .motion(MotionScheme::Expressive)
        .density(Density::Compact)
        .direction(Direction::Rtl);
    assert_eq!(theme.colors, custom);
    assert_eq!(theme.typography.text_scale(), 1.25);
    assert_eq!(theme.motion, MotionScheme::Expressive);
    assert_eq!(theme.layout.density, Density::Compact);
    assert_eq!(theme.layout.direction, Direction::Rtl);
}

#[test]
fn logical_edges_and_color_spec_sanitize_and_resolve() {
    assert_eq!(non_negative_f32(-5.0), 0.0);
    assert_eq!(non_negative_f32(-0.0), 0.0);
    assert_eq!(non_negative_f32(f32::NAN), 0.0);
    assert_eq!(non_negative_f32(f32::INFINITY), 0.0);
    assert_eq!(non_negative_f32(12.5), 12.5);

    let edges = LogicalEdges::new(4.0, 8.0, -2.0, f32::NAN).start(16.0);
    assert_eq!(edges.resolve(Direction::Ltr), [4.0, 8.0, 0.0, 16.0]);
    assert_eq!(edges.resolve(Direction::Rtl), [4.0, 16.0, 0.0, 8.0]);
    assert_eq!(
        LogicalEdges::symmetric(12.0, 6.0).resolve(Direction::Ltr),
        [6.0, 12.0, 6.0, 12.0]
    );
    assert_eq!(
        LogicalEdges::all(8.0).resolve(Direction::Rtl),
        [8.0, 8.0, 8.0, 8.0]
    );

    let scheme = ColorScheme::baseline(Appearance::Light);
    let role_spec: ColorSpec = ColorRole::Primary.into();
    let custom_argb = Argb::from_rgb(10, 20, 30);
    let custom_spec: ColorSpec = custom_argb.into();
    assert_eq!(role_spec.resolve(&scheme), scheme.color(ColorRole::Primary));
    assert_eq!(custom_spec.resolve(&scheme), custom_argb);
}

#[test]
fn appearance_mode_resolves_explicit_and_system_values() {
    const LIGHT_ON_DARK: Appearance = AppearanceMode::Light.resolve(Appearance::Dark);
    const DARK_ON_LIGHT: Appearance = AppearanceMode::Dark.resolve(Appearance::Light);
    const SYSTEM_LIGHT: Appearance = AppearanceMode::System.resolve(Appearance::Light);
    const SYSTEM_DARK: Appearance = AppearanceMode::System.resolve(Appearance::Dark);

    assert_eq!(AppearanceMode::default(), AppearanceMode::System);
    assert_eq!(LIGHT_ON_DARK, Appearance::Light);
    assert_eq!(
        AppearanceMode::Light.resolve(Appearance::Light),
        Appearance::Light
    );
    assert_eq!(DARK_ON_LIGHT, Appearance::Dark);
    assert_eq!(
        AppearanceMode::Dark.resolve(Appearance::Dark),
        Appearance::Dark
    );
    assert_eq!(SYSTEM_LIGHT, Appearance::Light);
    assert_eq!(SYSTEM_DARK, Appearance::Dark);
    assert_eq!(
        AppearanceMode::from(Appearance::Light),
        AppearanceMode::Light
    );
    assert_eq!(AppearanceMode::from(Appearance::Dark), AppearanceMode::Dark);
}

#[cfg(feature = "serde")]
#[test]
fn appearance_mode_round_trips_through_serde() {
    for mode in [
        AppearanceMode::Light,
        AppearanceMode::Dark,
        AppearanceMode::System,
    ] {
        let encoded = serde_json::to_string(&mode).unwrap();
        let decoded: AppearanceMode = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, mode);
    }
}

#[test]
fn material_strings_defaults_builders_and_arc_sharing() {
    let a = MaterialStrings::default();
    let b = MaterialStrings::default();
    assert_eq!(a, b);
    assert!(std::sync::Arc::ptr_eq(&a.dismiss, &b.dismiss));
    assert_eq!(a.dismiss.as_ref(), "Dismiss");
    assert_eq!(a.dismiss_message.as_ref(), "Dismiss message");
    assert_eq!(a.dismiss_overlay.as_ref(), "Dismiss overlay");
    assert_eq!(a.close.as_ref(), "Close");
    assert_eq!(a.close_actions.as_ref(), "Close actions");
    assert_eq!(a.close_search.as_ref(), "Close search");
    assert_eq!(a.cancel.as_ref(), "Cancel");
    assert_eq!(a.confirm.as_ref(), "OK");
    assert_eq!(a.clear.as_ref(), "Clear");
    assert_eq!(a.show_password.as_ref(), "Show password");
    assert_eq!(a.hide_password.as_ref(), "Hide password");
    assert_eq!(a.open_options.as_ref(), "Open options");
    assert_eq!(a.close_options.as_ref(), "Close options");
    assert_eq!(a.no_matching_options.as_ref(), "No matching options");
    assert_eq!(a.more_actions.as_ref(), "More actions");
    assert_eq!(a.app_bar_actions.as_ref(), "App bar actions");
    assert_eq!(a.bottom_app_bar.as_ref(), "Bottom app bar");
    assert_eq!(a.breadcrumb.as_ref(), "Breadcrumb");
    assert_eq!(a.more_breadcrumbs.as_ref(), "More breadcrumbs");
    assert_eq!(a.navigation.as_ref(), "Navigation");
    assert_eq!(a.steps.as_ref(), "Steps");
    assert_eq!(a.refresh.as_ref(), "Refresh");
    assert_eq!(a.resize_bottom_sheet.as_ref(), "Resize bottom sheet");
    assert_eq!(a.expand.as_ref(), "Expand");
    assert_eq!(a.collapse.as_ref(), "Collapse");
    assert_eq!(a.drop_target.as_ref(), "Drop target");
    assert_eq!(a.drop_files_here.as_ref(), "Drop files here");
    assert_eq!(a.searching.as_ref(), "Searching");
    assert_eq!(a.no_results.as_ref(), "No results");
    assert_eq!(a.search_prompt.as_ref(), "Enter a search term");
    assert_eq!(a.minimize.as_ref(), "Minimize");
    assert_eq!(a.maximize.as_ref(), "Maximize");
    assert_eq!(a.restore.as_ref(), "Restore");

    let custom = MaterialStrings::default()
        .dismiss("关闭")
        .dismiss_message("关闭提示")
        .dismiss_overlay("关闭遮罩")
        .close("关闭面板")
        .close_actions("关闭操作")
        .close_search("关闭搜索")
        .cancel("取消")
        .confirm("确定")
        .clear("清除")
        .show_password("显示密码")
        .hide_password("隐藏密码")
        .open_options("展开选项")
        .close_options("收起选项")
        .no_matching_options("无匹配项")
        .more_actions("更多操作")
        .app_bar_actions("应用栏操作")
        .bottom_app_bar("底部应用栏")
        .breadcrumb("面包屑")
        .more_breadcrumbs("更多路径")
        .navigation("导航")
        .steps("步骤")
        .refresh("刷新")
        .resize_bottom_sheet("调整底部抽屉高度")
        .expand("展开")
        .collapse("收起")
        .drop_target("拖放区域")
        .drop_files_here("将文件拖放到此处")
        .searching("搜索中")
        .no_results("无结果")
        .search_prompt("输入搜索关键词")
        .minimize("最小化")
        .maximize("最大化")
        .restore("还原");

    assert_ne!(custom, a);
    assert_eq!(custom.dismiss.as_ref(), "关闭");
    assert_eq!(custom.cancel.as_ref(), "取消");
    assert_eq!(custom.restore.as_ref(), "还原");
}

#[cfg(feature = "serde")]
#[test]
fn material_strings_round_trips_and_supports_partial_deserialization() {
    let custom = MaterialStrings::default()
        .dismiss("关闭")
        .cancel("取消")
        .show_password("显示密码")
        .hide_password("隐藏密码")
        .dismiss_overlay("关闭遮罩");
    let encoded = serde_json::to_string(&custom).unwrap();
    let decoded: MaterialStrings = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, custom);

    let partial: MaterialStrings =
        serde_json::from_str(r#"{"cancel":"Annuler","clear":"Effacer"}"#).unwrap();
    assert_eq!(partial.cancel.as_ref(), "Annuler");
    assert_eq!(partial.clear.as_ref(), "Effacer");
    assert_eq!(partial.dismiss, MaterialStrings::default().dismiss);
    assert_eq!(
        partial.dismiss_overlay,
        MaterialStrings::default().dismiss_overlay
    );
}

#![cfg(feature = "dynamic-color")]

use gpui_material_theme::{
    Appearance, Argb, ColorRole, ColorScheme, DynamicVariant, MaterialTheme, SpecVersion,
    ThemePreset, TokenValue, token,
};
use material_color_utils::{
    hct::hct_color::Hct, palettes::tonal_palette::TonalPalette, utils::color_utils,
};
use serde_json::Value;

fn reference() -> Value {
    serde_json::from_str(include_str!("mcu-spec-2021.json")).unwrap()
}

fn hex(value: &Value) -> u32 {
    u32::from_str_radix(value.as_str().unwrap(), 16).unwrap()
}

#[test]
fn tonal_spot_matches_every_pinned_role() {
    let vectors = reference();
    let roles: Vec<_> = vectors["roles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| {
            let name = format!("md.sys.color.{}", name.as_str().unwrap());
            let Some(TokenValue::ColorRole(role)) = token(&name) else {
                panic!("missing color role {name}");
            };
            (role, name)
        })
        .collect();
    assert_eq!(
        roles.iter().map(|(role, _)| *role).collect::<Vec<_>>(),
        ColorRole::ALL
            .into_iter()
            .filter(|role| !matches!(
                role,
                ColorRole::Info
                    | ColorRole::OnInfo
                    | ColorRole::InfoContainer
                    | ColorRole::OnInfoContainer
                    | ColorRole::Success
                    | ColorRole::OnSuccess
                    | ColorRole::SuccessContainer
                    | ColorRole::OnSuccessContainer
                    | ColorRole::Warning
                    | ColorRole::OnWarning
                    | ColorRole::WarningContainer
                    | ColorRole::OnWarningContainer
            ))
            .collect::<Vec<_>>()
    );
    let mut differences = Vec::new();
    for sample in vectors["schemes"].as_array().unwrap() {
        let seed = Argb::from_u32(hex(&sample[0]));
        let appearance = if sample[1].as_bool().unwrap() {
            Appearance::Dark
        } else {
            Appearance::Light
        };
        let contrast = sample[2].as_f64().unwrap();
        let expected = sample[3].as_array().unwrap();
        assert_eq!(expected.len(), roles.len());
        let scheme = MaterialTheme::from_seed(seed, appearance, contrast).colors;

        for ((role, name), expected) in roles.iter().zip(expected) {
            let actual = scheme.color(*role).to_u32();
            let expected = hex(expected);
            if actual != expected {
                differences.push(format!(
                    "{seed:?}/{appearance:?}/{contrast} {name}: {actual:08x} vs {expected:08x}"
                ));
            }
        }
    }
    assert!(
        differences.is_empty(),
        "{} differences:\n{}",
        differences.len(),
        differences
            .iter()
            .take(25)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn dynamic_variants_resolve_supported_specs_and_harmonized_status() {
    use Appearance::*;
    use DynamicVariant::*;
    use SpecVersion::*;

    let seed = Argb::from_u32(0xff42_85f4);
    for appearance in [Light, Dark] {
        for variant in [
            TonalSpot, Vibrant, Expressive, Fidelity, Content, Monochrome, Neutral,
        ] {
            let make = |spec| ColorScheme::from_seed_with(seed, appearance, 0., variant, spec);
            let old = make(Spec2021);
            let new = make(Spec2025);
            assert_eq!(
                old == new,
                matches!(variant, Fidelity | Content | Monochrome)
            );
            let distinct = [
                ColorRole::Error,
                ColorRole::Info,
                ColorRole::Success,
                ColorRole::Warning,
            ]
            .map(|role| new.color(role));
            for (i, color) in distinct.iter().enumerate() {
                assert!(!distinct[..i].contains(color), "{variant:?}/{appearance:?}");
            }
            for role in [
                ColorRole::Info,
                ColorRole::InfoContainer,
                ColorRole::Success,
                ColorRole::SuccessContainer,
                ColorRole::Warning,
                ColorRole::WarningContainer,
            ] {
                let pair = new.pair(role).unwrap();
                let tone = |argb: Argb| Hct::from(color_utils::Argb(argb.to_u32())).tone();
                let ratio = gpui_material_theme::contrast_ratio_of_tones(
                    tone(pair.foreground),
                    tone(pair.background),
                )
                .unwrap();
                assert!(ratio >= 4.5, "{variant:?}/{appearance:?}/{role:?} {ratio}");
            }
        }
    }
    assert_eq!(
        ThemePreset::Baseline.colors(Light),
        ColorScheme::baseline(Light)
    );
    assert_ne!(
        ThemePreset::Ocean.colors(Light),
        ThemePreset::Forest.colors(Light)
    );
    for preset in ThemePreset::ALL {
        assert_eq!(preset.theme(Dark).colors.appearance(), Dark);
    }
}
#[test]
fn validates_hct_palette_and_tone_contrast_boundaries() {
    let vectors = reference();
    for sample in vectors["hct"].as_array().unwrap() {
        let source = Hct::from_argb(color_utils::Argb(hex(&sample[0])));
        for (actual, expected) in [source.hue(), source.chroma(), source.tone()]
            .into_iter()
            .zip(&sample.as_array().unwrap()[1..4])
        {
            assert!((actual - expected.as_f64().unwrap()).abs() < 0.000_001);
        }
        let reconstructed = Hct::new(source.hue(), source.chroma(), source.tone());
        assert_eq!(reconstructed.to_argb().0, hex(&sample[4]));
    }
    let palette = TonalPalette::from_argb(color_utils::Argb(0xff00_00ff));
    assert_eq!(palette.tone(40).0, hex(&vectors["blue_tone_40"]));
    assert_eq!(
        gpui_material_theme::contrast_ratio_of_tones(-10.0, 110.0),
        vectors["out_of_range_contrast"].as_f64()
    );
    assert_eq!(
        gpui_material_theme::contrast_ratio_of_tones(f64::NAN, 20.0),
        None
    );
}

#[test]
fn clamps_seed_contrast_and_ignores_seed_alpha() {
    let red = Argb::from_rgb(255, 0, 0);
    let mode = Appearance::Dark;
    let make = |contrast| ColorScheme::from_seed(red, mode, contrast);
    assert_eq!(make(f64::INFINITY), make(1.0));
    assert_eq!(make(f64::NEG_INFINITY), make(-1.0));
    assert_eq!(make(f64::NAN), make(0.0));
    assert_eq!(
        ColorScheme::from_seed(Argb::from_u32(0x00ff_0000), mode, 0.0),
        make(0.0)
    );
    assert_eq!(
        make(0.0).color(ColorRole::PrimaryFixed),
        Argb::from_u32(0xffff_dad4)
    );
}

#[test]
fn custom_color_and_harmonize_resolve_m3_four_role_groups() {
    use gpui_material_theme::{CustomColorGroup, harmonize_color};

    let source = Argb::from_u32(0xff00_00ff);
    let design = Argb::from_u32(0x80ff_0000);
    let harmonized = harmonize_color(design, source);
    assert_eq!(harmonized.alpha(), 0xff);
    assert_ne!(harmonized, Argb::from_u32(0xffff_0000));
    assert_eq!(harmonize_color(source, source), source);

    for appearance in [Appearance::Light, Appearance::Dark] {
        let scheme = ColorScheme::from_seed(source, appearance, 0.0);
        let raw = scheme.custom_color(design, false);
        let shifted = scheme.custom_color(design, true);
        assert_ne!(raw.color, shifted.color);
        assert_ne!(raw.color, raw.on_color);
        assert_ne!(raw.color_container, raw.on_color_container);
        assert_eq!(raw.pair().background, raw.color);
        assert_eq!(raw.pair().foreground, raw.on_color);
        assert_eq!(raw.container_pair().background, raw.color_container);
        assert_eq!(raw.container_pair().foreground, raw.on_color_container);
        assert_eq!(
            scheme.custom_color_with_contrast(design, f64::NAN, true),
            shifted
        );
        let high = scheme.custom_color_with_contrast(design, 1.0, true);
        assert_eq!(
            scheme.custom_color_with_contrast(design, f64::INFINITY, true),
            high
        );
        assert_ne!(high, CustomColorGroup { ..shifted });
    }
}

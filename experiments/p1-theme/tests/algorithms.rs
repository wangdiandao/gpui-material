use material_color_utils::{
    contrast::contrast_utils::Contrast, dynamic::color_spec::SpecVersion,
    palettes::tonal_palette::TonalPalette, scheme::SchemeTonalSpot, utils::color_utils::Argb,
};

#[test]
fn matches_pinned_material_color_utilities_vectors() {
    let tone = TonalPalette::from_argb(Argb(0xff0000ff)).tone(40);
    let fixed = SchemeTonalSpot::builder(Argb(0xffff0000), true, 0.)
        .spec_version(SpecVersion::Spec2021)
        .build()
        .primary_fixed();
    let raw_ratio = Contrast::ratio_of_tones(-10., 110.);
    let ratio = ratio_of_tones(-10., 110.);
    println!("unadapted out-of-range contrast={raw_ratio}");

    println!(
        "blue tone 40={:08x}, red primaryFixed={:08x}, ratio={ratio}",
        tone.0, fixed.0
    );
    assert_eq!(tone, Argb(0xff343dff));
    assert_eq!(fixed, Argb(0xffffdad4));
    assert!((ratio - 21.).abs() < 0.001);
}

#[test]
fn tonal_spot_preserves_fixed_roles_and_responds_to_contrast() {
    let scheme = |dark, contrast| {
        SchemeTonalSpot::builder(Argb(0xffff0000), dark, contrast)
            .spec_version(SpecVersion::Spec2021)
            .build()
    };
    let light = scheme(false, 0.);
    let dark = scheme(true, 0.);

    assert_eq!(light.primary_fixed(), dark.primary_fixed());
    assert_eq!(light.primary_fixed_dim(), dark.primary_fixed_dim());
    assert_ne!(light.primary(), dark.primary());
    for dark in [false, true] {
        let ratio = |contrast| {
            let roles = scheme(dark, contrast);
            material_color_utils::get_contrast_ratio(roles.primary(), roles.on_primary())
        };
        let values = [ratio(-1.), ratio(0.), ratio(1.)];
        println!("dark={dark}: primary/on_primary {values:?}");
        assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(values[1] >= 4.5);
    }
}

fn ratio_of_tones(first: f64, second: f64) -> f64 {
    Contrast::ratio_of_tones(first.clamp(0., 100.), second.clamp(0., 100.))
}

use gpui::{Div, InteractiveElement, ParentElement, Stateful, Styled, div, px};
use gpui_material::theme::{ColorRole, TypeRole, TypographyEmphasis};
use gpui_material::{Divider, Text};

use crate::story::{section, stack};

pub(super) fn examples() -> Div {
    stack()
        .child(
            section("Type scale")
                .description("Five roles in a field journal. Each pair of numbers shows size and line height in sp.")
                .child(specimen("Display Large", TypeRole::DisplayLarge, "Field notes"))
                .child(Divider::horizontal())
                .child(specimen("Headline Medium", TypeRole::HeadlineMedium, "A walk by the river"))
                .child(Divider::horizontal())
                .child(specimen("Title Large", TypeRole::TitleLarge, "Along the east bank"))
                .child(Divider::horizontal())
                .child(specimen("Body Large", TypeRole::BodyLarge, "Morning light catches the reeds as the city wakes."))
                .child(Divider::horizontal())
                .child(specimen("Label Medium", TypeRole::LabelMedium, "ENTRY 024 · 08:30")),
        )
        .child(
            section("Emphasis and font scale")
                .description("A single Title Large sample, with one setting changed at a time.")
                .child(comparisons().children([
                    style_sample("Baseline · 100%", false, 1.),
                    style_sample("Emphasized · 100%", true, 1.),
                    style_sample("Baseline · 125%", false, 1.25),
                ])),
        )
        .child(
            section("Wrapping and truncation")
                .description("The same text in two previews capped at 224 px.")
                .child(comparisons().children([
                    comparison("Wrap", overflow_sample().wrap()),
                    comparison("Truncate", overflow_sample().truncate(px(224.))),
                ])),
        )
}

fn specimen(label: &'static str, role: TypeRole, sample: &'static str) -> Stateful<Div> {
    let style = role.style(TypographyEmphasis::Baseline);
    div()
        .id(label)
        .flex()
        .flex_wrap()
        .items_center()
        .gap_x_4()
        .gap_y_1()
        .py_1()
        .child(
            stack().w(px(116.)).gap_1().child(caption(label)).child(
                Text::new(format!("{} / {}", style.size, style.line_height))
                    .id("metrics")
                    .role(TypeRole::LabelMedium)
                    .color(ColorRole::OnSurfaceVariant),
            ),
        )
        .child(
            div()
                .flex_auto()
                .flex_basis(px(180.))
                .min_w_0()
                .child(Text::new(sample).id("sample").role(role).wrap()),
        )
}

fn comparisons() -> Div {
    div().flex().flex_wrap().gap_4()
}

fn comparison(label: &'static str, sample: Text) -> Stateful<Div> {
    stack()
        .id(label)
        .flex_auto()
        .flex_basis(px(164.))
        .gap_2()
        .child(caption(label))
        .child(div().max_w(px(224.)).child(sample.id("sample")))
}

fn caption(label: &'static str) -> Text {
    Text::new(label)
        .id("label")
        .role(TypeRole::LabelMedium)
        .color(ColorRole::Primary)
        .wrap()
}

fn style_sample(label: &'static str, emphasized: bool, scale: f32) -> Stateful<Div> {
    comparison(
        label,
        Text::new("River · 河岸")
            .role(TypeRole::TitleLarge)
            .emphasized(emphasized)
            .font_scale(scale)
            .wrap(),
    )
}

fn overflow_sample() -> Text {
    Text::new("Field notes from the east bank · 河岸观察札记").role(TypeRole::BodyLarge)
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn text_previews_compare_scale_and_overflow(cx: &mut gpui::TestAppContext) {
    use gpui::{AppContext, Context, IntoElement, Render, Window, WindowOptions, size};
    use gpui_base::test_support::find_all;
    use gpui_material::MaterialRoot;

    struct Example;
    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            examples()
        }
    }

    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| Example))
            .unwrap()
    });
    for width in [700., 300.] {
        cx.simulate_window_resize(handle.into(), size(px(width), px(960.)));
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let sample = |label: &'static str| {
                find_all(window, &[], &"sample".into())
                    .into_iter()
                    .find(|sample| sample.path().contains(&label.into()))
                    .unwrap()
            };
            let baseline = sample("Baseline · 100%");
            let emphasized = sample("Emphasized · 100%");
            let scaled = sample("Baseline · 125%");
            assert_eq!(baseline.label(), emphasized.label());
            assert_eq!(baseline.label(), scaled.label());
            assert!(baseline.bounds().size.height < scaled.bounds().size.height);
            if width == 700. {
                assert_eq!(baseline.bounds().top(), emphasized.bounds().top());
                assert_eq!(baseline.bounds().top(), scaled.bounds().top());
            } else {
                assert!(baseline.bounds().bottom() < emphasized.bounds().top());
                assert!(emphasized.bounds().bottom() < scaled.bounds().top());
            }
            let wrapped = sample("Wrap");
            let truncated = sample("Truncate");
            assert_eq!(wrapped.label(), truncated.label());
            assert!(wrapped.bounds().size.height > truncated.bounds().size.height);
            for preview in [baseline, scaled, wrapped, truncated] {
                assert!(preview.bounds().right() <= px(width));
            }
        })
        .unwrap();
    }
}

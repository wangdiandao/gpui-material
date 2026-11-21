use gpui::{
    AppContext, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
    Styled, TestAppContext, Window, div, px, rgb,
};
use gpui_base::{TestSupportExt, test_support::find};
use gpui_material::{
    Divider,
    foundation::{MaterialRoot, color, set_window_material_theme},
    theme::{Appearance, ColorRole, Direction, MaterialTheme},
};

#[gpui::test]
fn divider_strokes_follow_axis_theme_direction_and_device_scale(cx: &mut TestAppContext) {
    struct Example {
        before: FocusHandle,
        after: FocusHandle,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let literal = rgb(0x2bbbad);
            div()
                .flex()
                .flex_col()
                .w(px(120.))
                .id("divider-test")
                .tab_group()
                .test_support()
                .child(div().id("before").h(px(1.)).track_focus(&self.before))
                .child(Divider::horizontal().inset(px(10.), px(20.)))
                .child(
                    Divider::horizontal()
                        .inset(px(10.), px(20.))
                        .thickness(px(4.))
                        .color_override(literal)
                        .color(ColorRole::Primary),
                )
                .child(
                    Divider::horizontal()
                        .thickness(px(0.))
                        .color(ColorRole::Primary)
                        .color_override(literal),
                )
                .child(
                    div()
                        .flex()
                        .h(px(60.))
                        .id("vertical-dividers")
                        .test_support()
                        .child(Divider::vertical().inset(px(6.), px(14.)))
                        .child(
                            Divider::vertical()
                                .inset(px(6.), px(14.))
                                .thickness(px(4.))
                                .color(ColorRole::Primary),
                        )
                        .child(
                            Divider::vertical()
                                .thickness(px(0.))
                                .color_override(literal),
                        ),
                )
                .child(
                    Divider::horizontal()
                        .thickness(px(f32::NAN))
                        .color(ColorRole::Secondary),
                )
                .child(div().id("after").h(px(1.)).track_focus(&self.after))
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let example = cx.new(|cx| Example {
                before: cx.focus_handle().tab_index(0).tab_stop(true),
                after: cx.focus_handle().tab_index(1).tab_stop(true),
            });
            view = Some(example.clone());
            example
        })
        .unwrap();
        (handle, view.unwrap())
    });
    for appearance in [Appearance::Light, Appearance::Dark] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for scale in [1., 1.5, 2.] {
                cx.simulate_window_scale_factor_change(handle.into(), scale);
                cx.update_window(handle.into(), |_, window, cx| {
                    let mut theme = MaterialTheme::baseline(appearance);
                    theme.layout.direction = direction;
                    let default = color(theme.colors.color(ColorRole::OutlineVariant));
                    let primary = color(theme.colors.color(ColorRole::Primary));
                    let secondary = color(theme.colors.color(ColorRole::Secondary));
                    set_window_material_theme(Some(theme), window, cx);
                    window.refresh();
                    window.draw(cx).clear(cx);

                    let origin = find(window, &[], &"divider-test".into())
                        .unwrap()
                        .bounds()
                        .origin;
                    let horizontal_inset = if direction == Direction::Ltr {
                        px(10.)
                    } else {
                        px(20.)
                    };
                    let vertical_top = find(window, &[], &"vertical-dividers".into())
                        .unwrap()
                        .bounds()
                        .top();
                    let one_logical_pixel = window.pixel_snap(px(1.)).scale(scale);
                    let quads = window.painted_quads();
                    for (fill, on_screen_thickness) in
                        [(default, one_logical_pixel.0), (primary, 4. * scale)]
                    {
                        assert!(
                            quads.iter().any(|quad| {
                                quad.background == fill.into()
                                    && quad.bounds.left()
                                        == (origin.x + horizontal_inset).scale(scale)
                                    && quad.bounds.size.width == px(90.).scale(scale)
                                    && quad.bounds.size.height.0 == on_screen_thickness
                            }),
                            "{appearance:?} {direction:?} {scale}: horizontal {fill:?}"
                        );
                        assert!(
                            quads.iter().any(|quad| {
                                quad.background == fill.into()
                                    && quad.bounds.size.width.0 == on_screen_thickness
                                    && quad.bounds.top() == (vertical_top + px(6.)).scale(scale)
                                    && quad.bounds.size.height == px(40.).scale(scale)
                            }),
                            "{appearance:?} {direction:?} {scale}: vertical {fill:?}"
                        );
                    }

                    assert!(
                        quads.iter().any(|quad| {
                            quad.background == secondary.into()
                                && quad.bounds.size.width == px(120.).scale(scale)
                                && quad.bounds.size.height == one_logical_pixel
                        }),
                        "{appearance:?} {direction:?} {scale}: invalid thickness falls back"
                    );

                    let literal: gpui::Hsla = rgb(0x2bbbad).into();
                    for (width, height) in [(120. * scale, 1.), (1., 60. * scale)] {
                        assert!(
                            quads.iter().any(|quad| {
                                quad.background == literal.into()
                                    && (quad.bounds.size.width.0 - width).abs() < 0.01
                                    && (quad.bounds.size.height.0 - height).abs() < 0.01
                            }),
                            "{appearance:?} {direction:?} {scale}: hairline {width}x{height}"
                        );
                    }

                    let example = view.read(cx);
                    let before = example.before.clone();
                    let after = example.after.clone();
                    before.focus(window, cx);
                    window.focus_next(cx);
                    assert!(after.is_focused(window));
                })
                .unwrap();
            }
        }
    }
}

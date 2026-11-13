use gpui::{
    AppContext, Axis, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};
use gpui_material::{ControlSize, Slider, SliderKind, SliderState};

pub(crate) struct SliderExamples {
    level: Entity<SliderState>,
    range: Entity<SliderState>,
    centered: Entity<SliderState>,
}

impl SliderExamples {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            level: cx.new(|_| SliderState::new().max(10.).step(1.).default_value(3.)),
            range: cx.new(|_| SliderState::new().max(10.).step(1.).default_value(3.0..7.0)),
            centered: cx.new(|_| SliderState::new().min(-1.).max(1.).default_value(0.4)),
        }
    }
}

impl Render for SliderExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let level = self.level.read(cx).value().end();
        let range = self.range.read(cx).value();
        let balance = self.centered.read(cx).value().end();
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(div().text_lg().child("Sliders"))
            .child(div().child(format!("Level {level}")))
            .child(Slider::new("level", "Level", &self.level).icon("+"))
            .child(div().child(format!("Range {}–{}", range.start(), range.end())))
            .child(
                Slider::range(
                    "material-range",
                    "Minimum level",
                    "Maximum level",
                    &self.range,
                )
                .size(ControlSize::ExtraSmall),
            )
            .child(div().child(format!("Balance {balance:.2}")))
            .child(
                Slider::new("balance", "Balance", &self.centered)
                    .kind(SliderKind::Centered)
                    .axis(Axis::Vertical)
                    .size(ControlSize::Medium),
            )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{
        ElementId, InputEvent, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext, point,
        px, size,
    };
    use gpui_base::test_support::find;
    use gpui_material::foundation::MaterialRoot;

    #[gpui::test]
    fn material_range_uses_base_step_values(cx: &mut TestAppContext) {
        let (handle, view) = cx.update(|cx| {
            crate::init_gallery(cx);
            let mut view = None;
            let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                let sliders = cx.new(SliderExamples::new);
                view = Some(sliders.clone());
                sliders
            })
            .unwrap();
            (handle, view.unwrap())
        });
        cx.simulate_window_resize(handle.into(), size(px(640.), px(440.)));
        cx.update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);
            let state = view.read(cx).range.clone();
            let bounds = state.read(cx).bounds();
            let position = point(bounds.left() + bounds.size.width * 0.42, bounds.center().y);
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position,
                    click_count: 1,
                    first_mouse: false,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            window.refresh();
            window.draw(cx).clear(cx);
            window.dispatch_event(
                MouseUpEvent {
                    button: MouseButton::Left,
                    position,
                    click_count: 1,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(state.read(cx).value(), (4.0, 7.0).into());
            let id = ElementId::from((ElementId::from("material-range"), "start"));
            let start = find(window, &[], &id).unwrap();
            assert_eq!(start.label(), Some("Minimum level"));
            assert_eq!(start.focused(), Some(true));
        })
        .unwrap();
        cx.simulate_keystrokes(handle.into(), "right");
        cx.run_until_parked();
        cx.update(|cx| assert_eq!(view.read(cx).range.read(cx).value(), (5.0, 7.0).into()));
    }
}

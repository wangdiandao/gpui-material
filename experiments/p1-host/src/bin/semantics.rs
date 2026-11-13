use gpui::{
    AppContext, Context, Entity, IntoElement, Render, Role, Styled, TitlebarOptions, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_base::{
    Button, Checkbox, Slider, TestSupportExt, Theme,
    slider::{SliderState, SliderThumb, SliderTrack},
};
use p1_host::open_probe;

struct SemanticsProbe {
    activations: usize,
    checked: bool,
    range: Entity<SliderState>,
}

impl SemanticsProbe {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            activations: 0,
            checked: false,
            range: cx.new(|_| SliderState::new().max(10.).default_value(3.)),
        }
    }
}

impl Render for SemanticsProbe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::global(cx).tokens.colors;
        let owner = cx.entity().downgrade();
        let status = format!("已执行 {} 次", self.activations);
        div()
            .id("semantics")
            .role(Role::Group)
            .aria_label("操作示例")
            .size_full()
            .tab_group()
            .flex()
            .flex_col()
            .items_start()
            .gap_3()
            .p_4()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(
                Button::new("activate")
                    .accessibility_label("执行操作")
                    .border_1()
                    .border_color(colors.border)
                    .px_3()
                    .py_2()
                    .focus_visible(|style| style.border_color(colors.ring))
                    .child("执行操作")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.activations += 1;
                        cx.notify();
                    })),
            )
            .child(
                Checkbox::new("enabled")
                    .accessibility_label("选项")
                    .checked(self.checked)
                    .border_1()
                    .border_color(colors.border)
                    .px_3()
                    .py_2()
                    .focus_visible(|style| style.border_color(colors.ring))
                    .child(if self.checked {
                        "选项已选中"
                    } else {
                        "选项未选中"
                    })
                    .on_change(move |checked, _, _, cx| {
                        let _ = owner.update(cx, |this, cx| {
                            this.checked = checked == gpui_base::CheckboxState::Checked;
                            cx.notify();
                        });
                    }),
            )
            .child(
                Slider::new(&self.range)
                    .accessibility_label("级别")
                    .w_48()
                    .child(
                        SliderTrack::new(&self.range)
                            .w_full()
                            .h_4()
                            .bg(colors.muted)
                            .child(
                                SliderThumb::new(&self.range)
                                    .size_4()
                                    .rounded_full()
                                    .bg(colors.primary),
                            ),
                    ),
            )
            .child(
                div()
                    .id("result")
                    .role(Role::Status)
                    .test_support()
                    .aria_label(status.clone())
                    .child(status),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx| {
        gpui_base::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(480.), px(320.)), cx)),
            titlebar: Some(TitlebarOptions {
                title: Some("P1-04 语义探针".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        open_probe(cx, options, |_, cx| cx.new(SemanticsProbe::new)).unwrap();
    });
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn painted_nodes_report_roles_names_and_checkbox_value(cx: &mut gpui::TestAppContext) {
    use gpui_base::test_support;

    let (window, view) = cx.update(|cx| {
        gpui_base::init(cx);
        let view = cx.new(SemanticsProbe::new);
        let window = open_probe(cx, WindowOptions::default(), |_, _| view.clone()).unwrap();
        (window, view)
    });
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let activate = test_support::find(window, &[], &"activate".into()).unwrap();
        assert_eq!(activate.role(), Some(Role::Button));
        assert_eq!(activate.label(), Some("执行操作"));
        let enabled = test_support::find(window, &[], &"enabled".into()).unwrap();
        assert_eq!(enabled.role(), Some(Role::CheckBox));
        assert_eq!(enabled.checked(), Some(false));
        let range_id = ("slider", view.read(cx).range.entity_id()).into();
        let range = test_support::find(window, &[], &range_id).unwrap();
        assert_eq!(range.role(), Some(Role::Slider));
        assert_eq!(range.label(), Some("级别"));
        let result = test_support::find(window, &[], &"result".into()).unwrap();
        assert_eq!(result.role(), Some(Role::Status));
        assert_eq!(result.label(), Some("已执行 0 次"));
    })
    .unwrap();
}

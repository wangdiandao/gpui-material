use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, Role,
    StatefulInteractiveElement, Styled, Window, div,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    Button, ButtonShape, ButtonVariant, ControlSize, ExtendedFab, Fab, FabColor, FabSize,
    IconButton, IconButtonVariant, IconButtonWidth, IconToggleButton, ToggleButton,
    ToggleButtonVariant,
};

#[derive(Default)]
pub(crate) struct ActionExamples {
    count: usize,
    toggles: [bool; 4],
    icon_toggles: [bool; 4],
    collapsed: bool,
}

impl Render for ActionExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let button_variants = [
            ButtonVariant::Elevated,
            ButtonVariant::Filled,
            ButtonVariant::FilledTonal,
            ButtonVariant::Outlined,
            ButtonVariant::Text,
        ];
        let sizes = [
            ControlSize::ExtraSmall,
            ControlSize::Small,
            ControlSize::Medium,
            ControlSize::Large,
            ControlSize::ExtraLarge,
        ];
        let toggle_variants = [
            ToggleButtonVariant::Elevated,
            ToggleButtonVariant::Filled,
            ToggleButtonVariant::FilledTonal,
            ToggleButtonVariant::Outlined,
        ];
        let icon_variants = [
            IconButtonVariant::Standard,
            IconButtonVariant::Filled,
            IconButtonVariant::FilledTonal,
            IconButtonVariant::Outlined,
        ];
        let fab_sizes = [FabSize::Small, FabSize::Medium, FabSize::Large];
        let fab_colors = [FabColor::Primary, FabColor::Secondary, FabColor::Tertiary];

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_lg().child("Actions"))
            .child(
                row().children(
                    button_variants
                        .into_iter()
                        .enumerate()
                        .map(|(ix, variant)| {
                            let view = cx.entity();
                            Button::new(("action-variant", ix), format!("{variant:?}"))
                                .variant(variant)
                                .on_activate(move |_, cx| {
                                    view.update(cx, |this, cx| {
                                        this.count += 1;
                                        cx.notify();
                                    })
                                })
                        }),
                ),
            )
            .child(
                row().children(sizes.into_iter().enumerate().map(|(ix, size)| {
                    Button::new(("action-size", ix), format!("{size:?}"))
                        .size(size)
                        .shape(ButtonShape::Square)
                        .leading_icon("+")
                        .trailing_icon("›")
                })),
            )
            .child(
                row().children(
                    toggle_variants
                        .into_iter()
                        .enumerate()
                        .map(|(ix, variant)| {
                            let view = cx.entity();
                            ToggleButton::new(
                                ("toggle-variant", ix),
                                format!("{variant:?}"),
                                self.toggles[ix],
                            )
                            .variant(variant)
                            .on_change(move |value, _, cx| {
                                view.update(cx, |this, cx| {
                                    this.toggles[ix] = value;
                                    cx.notify();
                                })
                            })
                        }),
                ),
            )
            .child(
                row().children(icon_variants.into_iter().enumerate().map(|(ix, variant)| {
                    IconButton::new(("icon-variant", ix), format!("{variant:?} action"), "+")
                        .variant(variant)
                        .width(
                            [
                                IconButtonWidth::Narrow,
                                IconButtonWidth::Uniform,
                                IconButtonWidth::Wide,
                                IconButtonWidth::Uniform,
                            ][ix],
                        )
                })),
            )
            .child(
                row().children(icon_variants.into_iter().enumerate().map(|(ix, variant)| {
                    let view = cx.entity();
                    IconToggleButton::new(
                        ("icon-toggle-variant", ix),
                        format!("{variant:?} favorite"),
                        "♡",
                        self.icon_toggles[ix],
                    )
                    .selected_icon("♥")
                    .variant(variant)
                    .on_change(move |value, _, cx| {
                        view.update(cx, |this, cx| {
                            this.icon_toggles[ix] = value;
                            cx.notify();
                        })
                    })
                })),
            )
            .child(
                row().children(fab_sizes.into_iter().zip(fab_colors).enumerate().map(
                    |(ix, (size, color))| {
                        Fab::new(("fab-size", ix), format!("Create ({size:?})"), "+")
                            .size(size)
                            .color(color)
                    },
                )),
            )
            .child(
                row().children(fab_sizes.into_iter().enumerate().map(|(ix, size)| {
                    let view = cx.entity();
                    ExtendedFab::new(
                        ("extended-fab-size", ix),
                        "Create a new document 新建文档",
                        "+",
                    )
                    .size(size)
                    .expanded(!self.collapsed)
                    .on_activate(move |_, cx| {
                        view.update(cx, |this, cx| {
                            this.collapsed = !this.collapsed;
                            cx.notify();
                        })
                    })
                })),
            )
            .child(
                row()
                    .child(Button::new("action-disabled", "Unavailable").disabled(true))
                    .child({
                        let label = format!("Activations: {}", self.count);
                        div()
                            .id("action-count")
                            .role(Role::Status)
                            .aria_label(label.clone())
                            .test_support()
                            .child(label)
                    }),
            )
    }
}

fn row() -> gpui::Div {
    div().flex().flex_wrap().items_center().gap_2()
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{
        AppContext, InputEvent, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext, px, size,
    };
    use gpui_base::test_support::find;
    use gpui_material::foundation::MaterialRoot;

    #[gpui::test]
    fn material_buttons_activate_once_and_toggles_update_the_owner(cx: &mut TestAppContext) {
        let (handle, view) = cx.update(|cx| {
            crate::init_gallery(cx);
            let mut view = None;
            let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                let actions = cx.new(|_| ActionExamples::default());
                view = Some(actions.clone());
                actions
            })
            .unwrap();
            (handle, view.unwrap())
        });
        cx.simulate_window_resize(handle.into(), size(px(1000.), px(850.)));
        let draw = |window: &mut Window, cx: &mut gpui::App| {
            window.refresh();
            window.draw(cx).clear(cx);
        };
        cx.update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            draw(window, cx);
            for (id, expected, destination) in [
                (gpui::ElementId::from(("action-variant", 0_usize)), 1, None),
                (
                    gpui::ElementId::from(("action-variant", 0_usize)),
                    1,
                    Some(gpui::point(px(999.), px(849.))),
                ),
                (gpui::ElementId::from("action-disabled"), 1, None),
                (gpui::ElementId::from(("toggle-variant", 0_usize)), 1, None),
            ] {
                let target = find(window, &[], &id).unwrap();
                assert_eq!(target.role(), Some(Role::Button));
                assert!(target.visible());
                let position = target.bounds().center();
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
                draw(window, cx);
                window.dispatch_event(
                    MouseUpEvent {
                        button: MouseButton::Left,
                        position: destination.unwrap_or(position),
                        click_count: 1,
                        modifiers: Default::default(),
                    }
                    .to_platform_input(),
                    cx,
                );
                draw(window, cx);
                assert_eq!(view.read(cx).count, expected);
            }
            assert!(view.read(cx).toggles[0]);
            let toggle = find(window, &[], &("toggle-variant", 0_usize).into()).unwrap();
            assert_eq!(toggle.focused(), Some(true));
        })
        .unwrap();
        cx.update_window(handle.into(), |_, window, cx| {
            let keystroke = gpui::Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
            window.dispatch_event(
                gpui::KeyDownEvent {
                    keystroke: keystroke.clone(),
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
            draw(window, cx);
            window.dispatch_event(gpui::KeyUpEvent { keystroke }.to_platform_input(), cx);
            draw(window, cx);
            assert!(!view.read(cx).toggles[0]);
            assert_eq!(view.read(cx).count, 1);
        })
        .unwrap();
    }
}

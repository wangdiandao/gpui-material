use gpui::{Context, IntoElement, ParentElement, Render, ScrollHandle, Styled, Window, div, px};
use gpui_material::foundation::isolated_scroll;
use gpui_material::theme::{ColorRole, TypeRole};
use gpui_material::{
    Badge, Button, ButtonVariant, Card, CardVariant, CircularProgress, Divider, Icon, IconName,
    LinearProgress, List, ListItem, ListVariant, LoadingIndicator, ProgressShape, Text,
};

use crate::story::{Story, section, stack};

pub(crate) struct DisplayExamples {
    story: Option<Story>,
    progress: f32,
    loading: bool,
    selected: bool,
    opened: bool,
    child_count: usize,
    indeterminate: bool,
    selected_file: Option<usize>,
    pinned_file: Option<usize>,
    files_scroll: ScrollHandle,
}

impl Default for DisplayExamples {
    fn default() -> Self {
        Self {
            story: None,
            progress: 0.35,
            loading: false,
            selected: false,
            opened: false,
            child_count: 0,
            indeterminate: false,
            selected_file: None,
            pinned_file: None,
            files_scroll: ScrollHandle::new(),
        }
    }
}

impl DisplayExamples {
    pub(crate) fn for_story(story: Story) -> Self {
        Self {
            story: Some(story),
            ..Self::default()
        }
    }

    fn shows(&self, story: Story) -> bool {
        self.story.is_none_or(|active| active == story)
    }
}

impl Render for DisplayExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let card = cx.entity();
        let child = cx.entity();
        let list = cx.entity();
        let progress = cx.entity();
        let loading = cx.entity();
        let mode = cx.entity();
        let linear = |id, name, shape| {
            let progress = LinearProgress::new(id, name).shape(shape);
            if self.indeterminate {
                progress
            } else {
                progress.value(self.progress)
            }
        };
        let circular = |id, name, shape| {
            let progress = CircularProgress::new(id, name).shape(shape);
            if self.indeterminate {
                progress
            } else {
                progress.value(self.progress)
            }
        };
        stack()
            .children(self.shows(Story::Text).then(|| {
                section("Typography and font scale")
                    .description("Compare the same mixed-script text at the default scale and at 125%.")
                    .child(
                        Text::new("Material typography 中文混排").id("baseline-text").wrap(),
                    )
                    .child(
                        Text::new("Material typography 中文混排")
                            .id("mixed-script-text")
                            .role(TypeRole::TitleMedium)
                            .emphasized(true)
                            .font_scale(1.25)
                            .wrap(),
                    )
            }))
            .children(self.shows(Story::Icon).then(|| {
                section("Outlined and filled symbols").child(
                    div().flex().flex_wrap().gap_4().items_center().children(
                        [20., 24., 40., 48.].into_iter().map(|side| {
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(Icon::new(IconName::Favorite).size(px(side)))
                                .child(
                                    Icon::new(IconName::Favorite)
                                        .filled(true)
                                        .size(px(side))
                                        .color(ColorRole::Primary),
                                )
                        }),
                    ),
                )
            }))
            .children(self.shows(Story::Icon).then(|| {
                section("Shared size and color").child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(
                            Button::new("icon-inherited", "Add document")
                                .leading_icon(Icon::new(IconName::Add)),
                        )
                        .child(
                            Button::new("icon-inherited-disabled", "Unavailable")
                                .leading_icon(Icon::new(IconName::Add))
                                .disabled(true),
                        ),
                )
            }))
            .children(self.shows(Story::Icon).then(|| {
                section("Directional and decorative SVGs")
                .description("The first arrow mirrors with right-to-left layout. The second retains its direction.")
                .child(
                    div()
                        .flex()
                        .gap_3()
                        .h_10()
                        .items_center()
                        .child(
                            Icon::new(IconName::ArrowForward)
                                .size(px(24.))
                                .mirror_in_rtl(true)
                                .label("forward", "Forward"),
                        )
                        .child(Divider::vertical().inset(px(4.), px(8.)))
                        .child(Icon::new(IconName::ArrowForward).size(px(24.))),
                )
            }))
            .children(self.shows(Story::Card).then(|| {
                section("Elevated, filled and outlined")
                    .description("Open the project card or activate its Pin button independently.")
                    .child(
                        Card::new("elevated-card")
                            .variant(CardVariant::Elevated)
                            .child(Text::new("Elevated card")),
                    )
                    .child(
                        Card::new("filled-card")
                            .on_activate("Open project", move |_, cx| {
                                card.update(cx, |this, cx| {
                                    this.opened = !this.opened;
                                    cx.notify();
                                });
                            })
                            .child(Text::new(if self.opened {
                                "Project open"
                            } else {
                                "Project"
                            }))
                            .child(
                                Button::new("card-child", format!("Pin ({})", self.child_count))
                                    .variant(ButtonVariant::Outlined)
                                    .on_activate(move |_, cx| {
                                        child.update(cx, |this, cx| {
                                            this.child_count += 1;
                                            cx.notify();
                                        })
                                    }),
                            ),
                    )
                    .child(
                        Card::new("outlined-card")
                            .variant(CardVariant::Outlined)
                            .disabled(true)
                            .child(Text::new("Unavailable card")),
                    )
            }))
            .children(self.shows(Story::Divider).then(|| {
                section("Horizontal and vertical dividers")
                    .child(Text::new("Workspace").id("divider-from").role(TypeRole::LabelLarge))
                    .child(Divider::horizontal().inset(px(16.), px(0.)))
                    .child(Text::new("Documents").id("divider-to").role(TypeRole::BodyMedium))
                    .child(
                        Divider::horizontal()
                            .thickness(px(3.))
                            .color(ColorRole::Primary),
                    )
                    .child(Divider::horizontal().thickness(px(0.)))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .h_10()
                            .child(Text::new("Before").id("divider-before"))
                            .child(Divider::vertical().inset(px(4.), px(8.)))
                            .child(Text::new("After").id("divider-after"))
                            .child(
                                Divider::vertical()
                                    .thickness(px(3.))
                                    .color(ColorRole::Primary),
                            )
                            .child(Divider::vertical().thickness(px(0.))),
                    )
            }))
            .children(self.shows(Story::Badge).then(|| {
                section("Dot, label and count badges")
                .description("The last badge caps the displayed count at 99+.")
                .child(
                    div()
                        .flex()
                        .items_center()
                        .flex_wrap()
                        .gap_5()
                        .p_2()
                        .child(Badge::dot(
                            Icon::new(IconName::Notifications)
                                .size(px(24.))
                                .label("new-activity", "New activity"),
                        ))
                        .child(Badge::count(
                            Icon::new(IconName::Inbox)
                                .size(px(24.))
                                .label("inbox", "Inbox"),
                            8,
                            99,
                        ))
                        .child(Badge::label(
                            Icon::new(IconName::Description)
                                .size(px(24.))
                                .label("new-file", "Files"),
                            "New",
                        ))
                        .child(Badge::count(
                            Icon::new(IconName::Archive)
                                .size(px(24.))
                                .label("archive-overflow", "More than 99 archived items"),
                            125,
                            99,
                        )),
                )
            }))
            .children(self.shows(Story::List).then(|| {
                section("Segmented list")
                .description("Select Documents to see the selected row treatment.")
                .child(
                    List::new("segmented-list")
                        .variant(ListVariant::Segmented)
                        .items([
                            ListItem::new("documents", "Documents")
                                .selected(self.selected)
                                .supporting_text("12 items")
                                .leading(Icon::new(IconName::Description).size(px(24.)))
                                .on_activate(move |_, cx| {
                                    list.update(cx, |this, cx| {
                                        this.selected = !this.selected;
                                        cx.notify();
                                    })
                                }),
                            ListItem::new("archives", "Archives")
                                .overline("Storage")
                                .supporting_text("Last updated today")
                                .trailing(Badge::count(
                                    Icon::new(IconName::Archive)
                                        .size(px(24.))
                                        .label("archive-status", "More than 99 archived items"),
                                    125,
                                    99,
                                )),
                            ListItem::new("shared", "Shared").trailing(Badge::dot(
                                Icon::new(IconName::Notifications)
                                    .size(px(24.))
                                    .label("shared-status", "New shared activity"),
                            )),
                        ]),
                )
            }))
            .children(self.shows(Story::Progress).then(|| {
                section("Linear and circular progress")
                    .description("Toggle between a measured transfer and one with no known completion time.")
                    .child(linear(
                        "linear-progress",
                        "Transfer progress",
                        ProgressShape::Straight,
                    ))
                    .child(linear(
                        "linear-wave",
                        "Wavy transfer progress",
                        ProgressShape::Wavy,
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_3()
                            .child(circular(
                                "circular-progress",
                                "Sync progress",
                                ProgressShape::Straight,
                            ))
                            .child(circular(
                                "circular-wave",
                                "Wavy sync progress",
                                ProgressShape::Wavy,
                            ))
                            .child(Button::new("advance-progress", "Advance")
                            .disabled(self.indeterminate)
                            .on_activate(
                                move |_, cx| {
                                    progress.update(cx, |this, cx| {
                                        this.progress = (this.progress + 0.25) % 1.;
                                        cx.notify();
                                    })
                                },
                            )),
                    )
                    .child(
                        Button::new(
                            "progress-mode",
                            if self.indeterminate {
                                "Use measured progress"
                            } else {
                                "Use indeterminate progress"
                            },
                        )
                        .on_activate(move |_, cx| {
                            mode.update(cx, |this, cx| {
                                this.indeterminate = !this.indeterminate;
                                cx.notify();
                            })
                        }),
                    )
            }))
            .children(self.shows(Story::LoadingIndicator).then(|| {
                section("Uncontained and contained").child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .child(
                            LoadingIndicator::new("loading", "Loading documents")
                                .active(self.loading),
                        )
                        .child(
                            LoadingIndicator::new("contained-loading", "Loading archives")
                                .contained(true)
                                .active(self.loading),
                        )
                        .child(
                            Button::new(
                                "toggle-loading",
                                if self.loading {
                                    "Stop loading"
                                } else {
                                    "Start loading"
                                },
                            )
                            .variant(ButtonVariant::Outlined)
                            .on_activate(move |_, cx| {
                                loading.update(cx, |this, cx| {
                                    this.loading = !this.loading;
                                    cx.notify();
                                })
                            }),
                        ),
                )
            }))
            .children(self.shows(Story::List).then(|| {
                section("Standard list")
                .description("Scroll for more files. Pinning a file leaves the row selection as it was.")
                .child(
                    isolated_scroll("files-scroll", &self.files_scroll)
                        .h_48()
                        .child(List::new("standard-list").items((1..=24).map(|file| {
                            let select = cx.entity();
                            let pin = cx.entity();
                            let pinned = self.pinned_file == Some(file);
                            ListItem::new(("file", file), format!("File {file}"))
                                .selected(self.selected_file == Some(file))
                                .on_activate(move |_, cx| {
                                    select.update(cx, |this, cx| {
                                        this.selected_file = Some(file);
                                        cx.notify();
                                    })
                                })
                                .trailing(
                                    Button::new(
                                        ("pin-file", file),
                                        if pinned {
                                            format!("Unpin file {file}")
                                        } else {
                                            format!("Pin file {file}")
                                        },
                                    )
                                    .variant(ButtonVariant::Text)
                                    .size(gpui_material::ControlSize::ExtraSmall)
                                    .on_activate(
                                        move |_, cx| {
                                            pin.update(cx, |this, cx| {
                                                this.pinned_file = (!pinned).then_some(file);
                                                cx.notify();
                                            })
                                        },
                                    ),
                                )
                        }))),
                )
            }))
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{
        AppContext, InputEvent, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext, size,
    };
    use gpui_base::test_support::find;
    use gpui_material::foundation::MaterialRoot;

    #[gpui::test]
    fn divider_strokes_follow_axis_theme_direction_and_device_scale(cx: &mut TestAppContext) {
        use gpui::{FocusHandle, InteractiveElement, rgb};
        use gpui_base::TestSupportExt;
        use gpui_material::{
            foundation::{color, set_window_material_theme},
            theme::{Appearance, Direction, MaterialTheme},
        };

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

    #[gpui::test]
    fn symbols_inherit_slot_size_and_keep_image_semantics(cx: &mut TestAppContext) {
        struct Example;
        impl Render for Example {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div()
                    .flex()
                    .children(
                        [16., 24., 40.]
                            .into_iter()
                            .enumerate()
                            .map(|(index, side)| {
                                div().text_size(px(side)).child(
                                    Icon::new(IconName::Favorite)
                                        .filled(index == 1)
                                        .label(("symbol", index), "Favorite"),
                                )
                            }),
                    )
                    .child(
                        div().text_size(px(40.)).child(
                            Icon::new(IconName::ArrowForward)
                                .size(px(20.))
                                .mirror_in_rtl(true)
                                .label("explicit-symbol", "Forward"),
                        ),
                    )
            }
        }
        let handle = cx.update(|cx| {
            crate::init_gallery(cx);
            MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                cx.new(|_| Example)
            })
            .unwrap()
        });
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            for (index, side) in [16., 24., 40.].into_iter().enumerate() {
                let icon = find(window, &[], &("symbol", index).into()).unwrap();
                assert_eq!(icon.bounds().size, size(px(side), px(side)));
                assert_eq!(icon.role(), Some(gpui::Role::Image));
            }
            let explicit = find(window, &[], &"explicit-symbol".into()).unwrap();
            assert_eq!(explicit.bounds().size, size(px(20.), px(20.)));
            assert_eq!(explicit.role(), Some(gpui::Role::Image));
        })
        .unwrap();
    }

    #[gpui::test]
    fn nested_card_action_does_not_activate_its_parent(cx: &mut TestAppContext) {
        let (handle, view) = cx.update(|cx| {
            crate::init_gallery(cx);
            let mut view = None;
            let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                let display = cx.new(|_| DisplayExamples::default());
                view = Some(display.clone());
                display
            })
            .unwrap();
            (handle, view.unwrap())
        });
        cx.simulate_window_resize(handle.into(), size(px(650.), px(870.)));
        cx.update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);
            let position = find(window, &[], &"card-child".into())
                .unwrap()
                .bounds()
                .center();
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
            assert_eq!(view.read(cx).child_count, 1);
            assert!(!view.read(cx).opened);
        })
        .unwrap();
    }
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn list_roves_skips_disabled_and_wraps_supporting_text(cx: &mut gpui::TestAppContext) {
    use gpui::{AppContext, WindowOptions};
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;
    struct Example;
    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(220.)).child(List::new("audit-list").items([
                ListItem::new("first", "First").on_activate(|_, _| {}),
                ListItem::new("selected", "Selected").selected(true).on_activate(|_, _| {}),
                ListItem::new("disabled", "Disabled").disabled(true).on_activate(|_, _| {}),
                ListItem::new("last", "Last").supporting_text("中文 supporting text that wraps across several lines and remains readable").on_activate(|_, _| {}),
            ]))
        }
    }
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| Example))
            .unwrap()
    });
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        for _ in 0..3 {
            window.refresh();
            window.draw(cx).clear(cx);
        }
        window.focus_next(cx);
    })
    .unwrap();
    let focused = |cx: &mut gpui::TestAppContext, id: &str| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(
                window,
                &[],
                &(gpui::ElementId::from(id.to_owned()), "activate").into(),
            )
            .unwrap()
            .focused()
        })
        .unwrap()
    };
    assert_eq!(focused(cx, "selected"), Some(true));
    cx.simulate_keystrokes(handle.into(), "down");
    assert_eq!(focused(cx, "last"), Some(true));
    cx.simulate_keystrokes(handle.into(), "down");
    assert_eq!(focused(cx, "first"), Some(true));
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let row = find(window, &[], &"last".into()).unwrap();
        let supporting = find(window, row.path(), &"supporting".into()).unwrap();
        assert!(
            supporting.bounds().size.height >= px(40.),
            "supporting text must wrap rather than truncate"
        );
        assert!(row.bounds().size.height >= px(88.));
        assert!(supporting.bounds().right() <= row.bounds().right());
    })
    .unwrap();
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn disabled_card_supplies_content_color_without_fading_custom_children(
    cx: &mut gpui::TestAppContext,
) {
    use gpui::{App, AppContext, Hsla, RenderOnce, WindowOptions};
    use gpui_material::{
        MaterialRoot,
        foundation::{color, window_material_theme},
        theme::ColorRole,
    };
    use std::{cell::Cell, rc::Rc};
    #[derive(gpui::IntoElement)]
    struct Probe(Rc<Cell<Hsla>>);
    impl RenderOnce for Probe {
        fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
            self.0.set(window.text_style().color);
            Text::new("Inherited card content")
        }
    }
    struct Example {
        inherited: Rc<Cell<Hsla>>,
        explicit: Rc<Cell<Hsla>>,
    }
    impl Render for Example {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let primary = color(
                window_material_theme(window, cx)
                    .colors
                    .color(ColorRole::Primary),
            );
            Card::new("disabled-card")
                .disabled(true)
                .child(Probe(self.inherited.clone()))
                .child(
                    div()
                        .text_color(primary)
                        .child(Probe(self.explicit.clone())),
                )
        }
    }
    let inherited = Rc::new(Cell::new(gpui::transparent_black()));
    let explicit = Rc::new(Cell::new(gpui::transparent_black()));
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| Example {
                inherited: inherited.clone(),
                explicit: explicit.clone(),
            })
        })
        .unwrap()
    });
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let theme = window_material_theme(window, cx);
        assert_eq!(
            inherited.get(),
            color(theme.colors.color(ColorRole::OnSurface)).opacity(0.38)
        );
        assert_eq!(
            explicit.get(),
            color(theme.colors.color(ColorRole::Primary))
        );
    })
    .unwrap();
}

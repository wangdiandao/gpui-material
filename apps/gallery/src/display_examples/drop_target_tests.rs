use std::path::PathBuf;

use gpui::{
    AppContext, Context, ElementId, ExternalPaths, FileDropEvent, Hsla, InputEvent,
    InteractiveElement, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Render, Role,
    StatefulInteractiveElement, Styled, TestAppContext, Window, div, px, rgb,
};
use gpui_base::test_support::find;
use gpui_material::{DropTarget, DropTargetColors, foundation::MaterialRoot};

#[derive(Clone, Debug, PartialEq, Eq)]
struct TagPayload {
    name: &'static str,
    allowed: bool,
}

struct DragChip;

impl Render for DragChip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size(px(20.)).child("drag")
    }
}

#[gpui::test]
fn drop_target_handles_files_typed_payloads_validation_activation_and_disabled(
    cx: &mut TestAppContext,
) {
    const ACTIVE_BG: u32 = 0x1b5e20;
    const INVALID_BG: u32 = 0xb71c1c;

    struct Example {
        dropped_files: Vec<PathBuf>,
        dropped_tags: Vec<&'static str>,
        activations: usize,
        disabled: bool,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let on_files = cx.entity();
            let on_tag = cx.entity();
            let on_act = cx.entity();
            let colors = DropTargetColors {
                active_container: Some(rgb(ACTIVE_BG).into()),
                invalid_container: Some(rgb(INVALID_BG).into()),
                ..Default::default()
            };
            div()
                .id("drop-scene")
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .child(
                    div()
                        .id("drag-valid")
                        .size(px(40.))
                        .on_drag(
                            TagPayload {
                                name: "valid",
                                allowed: true,
                            },
                            |_, _, _, cx| cx.new(|_| DragChip),
                        )
                        .child("Valid"),
                )
                .child(
                    div()
                        .id("drag-invalid")
                        .size(px(40.))
                        .on_drag(
                            TagPayload {
                                name: "blocked",
                                allowed: false,
                            },
                            |_, _, _, cx| cx.new(|_| DragChip),
                        )
                        .child("Blocked"),
                )
                .child(
                    DropTarget::files("file-zone")
                        .title("Drop PNG files")
                        .disabled(self.disabled)
                        .colors(colors)
                        .can_drop(|paths, _, _| {
                            paths
                                .paths()
                                .iter()
                                .all(|path| path.extension().is_some_and(|ext| ext == "png"))
                        })
                        .on_drop(move |paths, _, cx| {
                            let list = paths.paths().to_vec();
                            on_files.update(cx, |this, cx| {
                                this.dropped_files.extend(list);
                                cx.notify();
                            });
                        })
                        .on_activate(move |_, cx| {
                            on_act.update(cx, |this, cx| {
                                this.activations += 1;
                                cx.notify();
                            });
                        }),
                )
                .child(
                    DropTarget::<TagPayload>::new("tag-zone")
                        .title("Drop tag")
                        .disabled(self.disabled)
                        .colors(colors)
                        .can_drop(|payload, _, _| payload.allowed)
                        .on_drop(move |payload, _, cx| {
                            let name = payload.name;
                            on_tag.update(cx, |this, cx| {
                                this.dropped_tags.push(name);
                                cx.notify();
                            });
                        }),
                )
        }
    }

    let mut view_entity = None;
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|_| Example {
                dropped_files: Vec::new(),
                dropped_tags: Vec::new(),
                activations: 0,
                disabled: false,
            });
            view_entity = Some(entity.clone());
            entity
        })
        .unwrap()
    });
    let view = view_entity.unwrap();

    let draw = |window: &mut Window, cx: &mut gpui::App| {
        window.refresh();
        window.draw(cx).clear(cx);
    };

    // 1. Verify accessible roles and click/keyboard activation on file-zone.
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        draw(window, cx);

        let activate_id: ElementId = (ElementId::from("file-zone"), "activate").into();
        let activate_btn = find(window, &[], &activate_id).unwrap();
        assert_eq!(activate_btn.role(), Some(Role::Button));
        assert_eq!(activate_btn.label(), Some("Drop PNG files"));

        let body_id: ElementId = (ElementId::from("tag-zone"), "body").into();
        let tag_body = find(window, &[], &body_id).unwrap();
        assert_eq!(tag_body.role(), Some(Role::Region));
        assert_eq!(tag_body.label(), Some("Drop tag"));

        let center = activate_btn.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: center,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        assert_eq!(view.read(cx).activations, 1);
    })
    .unwrap();

    // 2. Verify ExternalPaths file drop validation: rejected .txt shows invalid feedback & skips on_drop; accepted .png fires on_drop.
    cx.update_window(handle.into(), |_, window, cx| {
        let zone_center = find(window, &[], &"file-zone".into())
            .unwrap()
            .bounds()
            .center();

        // Rejected file drag-over & drop.
        let txt_paths = ExternalPaths(vec![PathBuf::from("notes.txt")].into());
        window.dispatch_event(
            FileDropEvent::Entered {
                position: zone_center,
                paths: txt_paths.clone(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        let invalid_fill: Hsla = rgb(INVALID_BG).into();
        assert!(
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.background == invalid_fill.into())
        );

        window.dispatch_event(
            FileDropEvent::Submit {
                position: zone_center,
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        assert!(view.read(cx).dropped_files.is_empty());

        // Accepted .png file drag-over & drop.
        let png_paths = ExternalPaths(vec![PathBuf::from("avatar.png")].into());
        window.dispatch_event(
            FileDropEvent::Entered {
                position: zone_center,
                paths: png_paths,
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        let active_fill: Hsla = rgb(ACTIVE_BG).into();
        assert!(
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.background == active_fill.into())
        );

        window.dispatch_event(
            FileDropEvent::Submit {
                position: zone_center,
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        assert_eq!(view.read(cx).dropped_files, [PathBuf::from("avatar.png")]);
    })
    .unwrap();

    // 3. Verify typed in-app drag payload (accepted vs rejected) and disabled suppression.
    cx.update_window(handle.into(), |_, window, cx| {
        let tag_center = find(window, &[], &"tag-zone".into())
            .unwrap()
            .bounds()
            .center();

        // Start a valid drag programmatically via pointer drag from drag-valid to tag-zone.
        let valid_pos = gpui::point(px(30.), px(30.));
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: valid_pos,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseMoveEvent {
                position: tag_center,
                pressed_button: Some(MouseButton::Left),
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: tag_center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        assert_eq!(view.read(cx).dropped_tags, ["valid"]);

        // Dragging a rejected typed payload paints invalid feedback and skips on_drop.
        let invalid_pos = gpui::point(px(30.), px(82.));
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: invalid_pos,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseMoveEvent {
                position: tag_center,
                pressed_button: Some(MouseButton::Left),
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        let invalid_fill: Hsla = rgb(INVALID_BG).into();
        assert!(
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.background == invalid_fill.into())
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: tag_center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        assert_eq!(view.read(cx).dropped_tags, ["valid"]);

        // Keyboard Enter activates focused drop target.
        let activate_id: ElementId = (ElementId::from("file-zone"), "activate").into();
        let center = find(window, &[], &activate_id).unwrap().bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: center,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        let keystroke = Keystroke::parse("enter").unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
        draw(window, cx);
        assert_eq!(view.read(cx).activations, 3);

        // Disable targets and verify drops are ignored.
        view.update(cx, |this, cx| {
            this.disabled = true;
            cx.notify();
        });
        draw(window, cx);

        let file_center = find(window, &[], &"file-zone".into())
            .unwrap()
            .bounds()
            .center();
        let png_paths = ExternalPaths(vec![PathBuf::from("ignored.png")].into());
        window.dispatch_event(
            FileDropEvent::Entered {
                position: file_center,
                paths: png_paths,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            FileDropEvent::Submit {
                position: file_center,
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        assert_eq!(view.read(cx).dropped_files.len(), 1);
    })
    .unwrap();
}

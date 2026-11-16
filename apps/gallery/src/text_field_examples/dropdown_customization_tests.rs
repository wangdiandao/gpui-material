use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    AppContext, Bounds, ElementId, EntityInputHandler as _, FocusHandle, ListAlignment, ListOffset,
    ListState, Modifiers, Pixels, Point, Role, TestAppContext, VisualTestContext, WindowOptions,
    point, rgb, size,
};
use gpui_base::test_support::{ElementSnapshot, find};
use gpui_material::MaterialRoot;

use super::*;

struct Fixture {
    state: Entity<ExposedDropdownState>,
    editor: Entity<TextFieldState>,
    scroll: ListState,
    options: Vec<DropdownOption>,
    editable: bool,
    open: bool,
    accepts: bool,
    primary_enabled: bool,
    secondary_enabled: bool,
    width: Pixels,
    match_width: bool,
    renders: Rc<Cell<usize>>,
    changes: Vec<bool>,
    dismissals: usize,
    outside: FocusHandle,
}

impl Render for Fixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let changes = cx.weak_entity();
        let dismissals = changes.clone();
        let renders = self.renders.clone();
        let anchor = if self.editable {
            DropdownAnchor::PrimaryEditable
        } else {
            DropdownAnchor::PrimaryNotEditable
        };
        div()
            .w_full()
            .flex()
            .items_start()
            .gap_3()
            .child(
                div().w(self.width).child(
                    ExposedDropdown::new(
                        "custom-dropdown",
                        "Custom destination",
                        &self.state,
                        self.options.clone(),
                    )
                    .primary_anchor(anchor)
                    .primary_anchor_enabled(self.primary_enabled)
                    .secondary_anchor("Choose preset", |open| {
                        Icon::new(if open {
                            IconName::Remove
                        } else {
                            IconName::Add
                        })
                    })
                    .secondary_anchor_enabled(self.secondary_enabled)
                    .editor(|field| {
                        field
                            .variant(TextFieldVariant::Filled)
                            .leading(div().id("editor-decoration").size(px(16.)).test_support())
                    })
                    .open(self.open)
                    .on_open_change(move |open, _, cx| {
                        let _ = changes.update(cx, |view, cx| {
                            view.changes.push(open);
                            if view.accepts {
                                view.open = open;
                            }
                            cx.notify();
                        });
                    })
                    .on_dismiss(move |_, cx| {
                        let _ = dismissals.update(cx, |view, _| view.dismissals += 1);
                    })
                    .option_content(move |option, _, _, _| {
                        renders.set(renders.get() + 1);
                        div()
                            .id("option-decoration")
                            .test_support()
                            .min_h(px(36.))
                            .child(format!("Sample {}", option.label))
                    })
                    .match_anchor_width(self.match_width)
                    .popup_width(px(230.))
                    .appearance(MenuSurfaceStyle {
                        shape: Some(CornerShape::Radii([16., 12., 8., 6.])),
                        border: Some((px(2.), ColorRole::Outline)),
                        tonal_elevation: Elevation::Level1,
                        shadow_elevation: Some(Elevation::Level3),
                        ..Default::default()
                    })
                    .container_color(rgb(0xdba9ef).into()),
                ),
            )
            .child(Button::new("outside", "Outside").track_focus(&self.outside))
    }
}

fn open(
    cx: &mut TestAppContext,
    editable: bool,
    count: usize,
) -> (gpui::WindowHandle<gpui_base::Root>, Entity<Fixture>) {
    cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut fixture = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let editor = cx.new(|cx| TextFieldState::new(window, cx).default_value("Original"));
            let state =
                cx.new(|cx| ExposedDropdownState::with_editor(&editor, editable, window, cx));
            let scroll = ListState::new(0, ListAlignment::Top, px(120.));
            state.update(cx, |state, cx| state.set_scroll_state(&scroll, cx));
            let view = cx.new(|cx| Fixture {
                state,
                editor,
                scroll,
                options: (0..count)
                    .map(|index| DropdownOption::new(index, format!("Option {index}")))
                    .collect(),
                editable,
                open: false,
                accepts: true,
                primary_enabled: true,
                secondary_enabled: true,
                width: px(300.),
                match_width: false,
                renders: Rc::new(Cell::new(0)),
                changes: Vec::new(),
                dismissals: 0,
                outside: cx.focus_handle(),
            });
            fixture = Some(view.clone());
            view
        })
        .unwrap();
        (window, fixture.unwrap())
    })
}

fn draw(cx: &mut TestAppContext, handle: gpui::WindowHandle<gpui_base::Root>) {
    for _ in 0..2 {
        cx.update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            window.simulate_next_frame(cx);
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.executor().advance_clock(Duration::from_millis(20));
        cx.run_until_parked();
    }
}

fn field(window: &Window) -> ElementSnapshot {
    let id = (ElementId::from("custom-dropdown"), "combobox").into();
    let trigger = find(window, &[], &id).unwrap();
    find(window, trigger.path(), &"value".into()).unwrap()
}

fn click(
    cx: &mut TestAppContext,
    handle: gpui::WindowHandle<gpui_base::Root>,
    position: Point<Pixels>,
) {
    VisualTestContext::from_window(handle.into(), cx)
        .simulate_click(position, Modifiers::default());
    draw(cx, handle);
}

fn field_click(cx: &mut TestAppContext, handle: gpui::WindowHandle<gpui_base::Root>) {
    let position = cx
        .update_window(handle.into(), |_, window, _| {
            field(window).bounds().center()
        })
        .unwrap();
    click(cx, handle, position);
}

#[gpui::test]
fn dropdown_selection_only_popup_focus_and_controlled_dismissal(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, false, 8);
    cx.update(|cx| view.update(cx, |view, _| view.accepts = false));
    draw(cx, handle);
    field_click(cx, handle);
    cx.update(|cx| {
        let view = view.read(cx);
        assert_eq!(view.changes, [true]);
        assert!(!view.state.read(cx).is_open());
    });

    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.accepts = true;
            cx.notify();
        })
    });
    field_click(cx, handle);
    cx.update_window(handle.into(), |_, window, _| {
        let menu = find(window, &[], &"options".into()).unwrap();
        assert_eq!(menu.role(), Some(Role::ListBox));
        assert_eq!(menu.label(), Some("Custom destination options"));
        assert_eq!(menu.focused(), Some(true));
        assert_eq!(field(window).focused(), Some(false));
    })
    .unwrap();
    cx.simulate_keystrokes(handle.into(), "down enter");
    draw(cx, handle);
    cx.update_window(handle.into(), |_, window, cx| {
        assert_eq!(field(window).focused(), Some(true));
        assert_eq!(field(window).value(), Some("Option 0"));
        assert!(find(window, &[], &"options".into()).is_none());
        let view = view.read(cx);
        assert_eq!(
            view.state.read(cx).selected_option_id(),
            Some(&0usize.into())
        );
        assert_eq!(view.changes, [true, true, false]);
        assert_eq!(view.dismissals, 1);
    })
    .unwrap();

    field_click(cx, handle);
    cx.simulate_keystrokes(handle.into(), "escape");
    draw(cx, handle);
    cx.update_window(handle.into(), |_, window, cx| {
        assert_eq!(field(window).focused(), Some(true));
        assert_eq!(field(window).value(), Some("Option 0"));
        let view = view.read(cx);
        assert_eq!(view.dismissals, 2);
        assert_eq!(view.changes, [true, true, false, true, false]);
    })
    .unwrap();

    for open in [true, false] {
        cx.update(|cx| {
            view.update(cx, |view, cx| {
                view.open = open;
                cx.notify();
            })
        });
        draw(cx, handle);
    }
    cx.update_window(handle.into(), |_, window, cx| {
        assert_eq!(field(window).focused(), Some(true));
        assert_eq!(view.read(cx).changes.len(), 5);
        assert_eq!(view.read(cx).dismissals, 2);
    })
    .unwrap();
    field_click(cx, handle);
    let outside = cx
        .update_window(handle.into(), |_, window, _| {
            find(window, &[], &"outside".into())
                .unwrap()
                .bounds()
                .center()
        })
        .unwrap();
    click(cx, handle, outside);
    cx.update_window(handle.into(), |_, window, cx| {
        let view = view.read(cx);
        assert!(view.outside.is_focused(window));
        assert_eq!(view.dismissals, 3);
        assert_eq!(view.changes.len(), 7);
        assert!(!view.state.read(cx).is_open());
    })
    .unwrap();
}

#[gpui::test]
fn dropdown_editable_secondary_pointer_preserves_field_and_disabled_anchors_preserve_input(
    cx: &mut TestAppContext,
) {
    let (handle, view) = open(cx, true, 8);
    draw(cx, handle);
    let secondary = cx
        .update_window(handle.into(), |_, window, _| {
            let button = find(window, &[], &"toggle-options".into()).unwrap();
            assert_eq!(button.label(), Some("Choose preset"));
            button.bounds().center()
        })
        .unwrap();
    click(cx, handle, secondary);
    cx.update_window(handle.into(), |_, window, cx| {
        let view = view.read(cx);
        assert!(view.state.read(cx).is_open());
        assert_eq!(field(window).focused(), Some(true));
        assert_eq!(field(window).role(), Some(Role::EditableComboBox));
        view.editor.clone().update(cx, |editor, cx| {
            editor.replace_and_mark_text_in_range(None, "ime", Some(3..3), window, cx);
            assert!(editor.marked_text_range(window, cx).is_some());
        });
    })
    .unwrap();
    draw(cx, handle);
    cx.simulate_keystrokes(handle.into(), "enter");
    draw(cx, handle);
    cx.update_window(handle.into(), |_, window, cx| {
        let view = view.read(cx);
        assert!(view.state.read(cx).is_open());
        assert!(view.state.read(cx).selected_option_id().is_none());
        let scroll = find(window, &[], &"options-scroll-region".into()).unwrap();
        assert!(
            find(window, scroll.path(), &0usize.into())
                .unwrap()
                .visible()
        );
        view.editor
            .clone()
            .update(cx, |editor, cx| editor.unmark_text(window, cx));
    })
    .unwrap();
    cx.simulate_keystrokes(handle.into(), "escape");
    draw(cx, handle);
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.primary_enabled = false;
            view.secondary_enabled = false;
            cx.notify();
        })
    });
    draw(cx, handle);
    field_click(cx, handle);
    cx.simulate_input(handle.into(), "x");
    click(cx, handle, secondary);
    cx.update_window(handle.into(), |_, window, cx| {
        let view = view.read(cx);
        assert!(view.editor.read(cx).value().ends_with('x'));
        assert!(!view.state.read(cx).is_open());
        assert_eq!(view.changes, [true, false]);
        assert_eq!(field(window).role(), Some(Role::TextInput));
        assert!(find(window, &[], &"toggle-options".into()).is_none());
    })
    .unwrap();
}

#[gpui::test]
fn dropdown_custom_surface_width_and_virtual_scroll_survive_rebuilding(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, false, 500);
    cx.simulate_window_resize(handle.into(), size(px(660.), px(360.)));
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.open = true;
            cx.notify();
        })
    });
    draw(cx, handle);
    let popup = |window: &Window| find(window, &[], &"options".into()).unwrap().bounds();
    cx.update_window(handle.into(), |_, window, cx| {
        let bounds = popup(window);
        assert_eq!(bounds.size.width, px(230.));
        assert!(bounds.top() > field(window).bounds().bottom());
        assert!(bounds.bottom() <= window.viewport_size().height - px(8.));
        let scroll = find(window, &[], &"options-scroll-region".into()).unwrap();
        let first = find(window, scroll.path(), &0usize.into()).unwrap();
        assert_eq!(first.role(), Some(Role::ListBoxOption));
        assert_eq!(first.label(), Some("Option 0"));
        assert!(find(window, first.path(), &"option-decoration".into()).is_some());
        assert!(view.read(cx).renders.get() < 500);

        let tint = gpui::Hsla::from(rgb(0xdba9ef));
        let quads = window.painted_quads();
        let quad = quads
            .iter()
            .find(|quad| quad.background.as_solid() == Some(tint))
            .unwrap();
        assert_eq!(
            quad.corner_radii.top_left,
            px(16.).scale(window.scale_factor())
        );
        assert!(quads.iter().any(|border| {
            border.bounds == quad.bounds
                && border.border_widths.top == px(2.).scale(window.scale_factor())
        }));
        view.read(cx).scroll.scroll_to(ListOffset {
            item_ix: 50,
            offset_in_item: px(0.),
        });
    })
    .unwrap();
    draw(cx, handle);
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.options
                .insert(0, DropdownOption::new("inserted", "Inserted"));
            view.width = px(260.);
            view.match_width = true;
            cx.notify();
        })
    });
    draw(cx, handle);
    cx.update_window(handle.into(), |_, window, cx| {
        let view = view.read(cx);
        assert_eq!(
            view.state
                .read(cx)
                .scroll_state()
                .logical_scroll_top()
                .item_ix,
            51
        );
        assert_eq!(popup(window).size.width, field(window).bounds().size.width);
        let scroll = find(window, &[], &"options-scroll-region".into()).unwrap();
        assert!(
            find(window, scroll.path(), &50usize.into())
                .unwrap()
                .visible()
        );
        let actual = Bounds::new(point(px(0.), px(0.)), window.viewport_size());
        assert!(actual.contains(&popup(window).center()));
    })
    .unwrap();
}

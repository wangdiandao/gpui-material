use std::time::Duration;

use gpui::{
    AppContext, Context, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, Styled, Window, WindowOptions, div,
};
use gpui_material::{
    Accordion, AccordionItem, AccordionMode, AccordionVariant, Collapsible, Icon, IconName,
    MaterialRoot, MaterialTheme, Pagination, ReduceMotionPreference, Text,
    set_reduce_motion_preference, set_window_material_theme,
    test::TestWindowExt,
    theme::{Density, Direction},
};

struct CollapsibleHarness {
    open: bool,
    disabled_open: bool,
    focus: FocusHandle,
    toggles: usize,
}

impl Render for CollapsibleHarness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let disabled_entity = cx.entity();
        div()
            .p_4()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                Collapsible::new("col", "Details", self.open)
                    .leading(div().id("col-leading").child(Icon::new(IconName::Info)))
                    .track_focus(&self.focus)
                    .content(div().id("col-body").child(Text::new("Expanded body")))
                    .on_open_change(move |next, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.open = next;
                            this.toggles += 1;
                            cx.notify();
                        });
                    }),
            )
            .child(
                Collapsible::new("col-disabled", "Locked", self.disabled_open)
                    .disabled(true)
                    .content(Text::new("Hidden"))
                    .on_open_change(move |next, _, cx| {
                        disabled_entity.update(cx, |this, cx| {
                            this.disabled_open = next;
                            cx.notify();
                        });
                    }),
            )
    }
}

#[gpui::test]
fn collapsible_pointer_keyboard_disabled_and_reduced_motion(cx: &mut gpui::TestAppContext) {
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        set_reduce_motion_preference(ReduceMotionPreference::Reduce, cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|cx| CollapsibleHarness {
                open: false,
                disabled_open: false,
                focus: cx.focus_handle(),
                toggles: 0,
            });
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, view.unwrap())
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);

        let trigger = ElementId::from((ElementId::from("col"), "trigger"));
        let content = ElementId::from((ElementId::from("col"), "content"));
        assert_eq!(window.find(trigger.clone()).expanded(), Some(false));
        assert!(window.try_find(content.clone()).is_none());

        // Pointer click expands immediately under reduced motion.
        window.click(trigger.clone(), cx);
        assert!(view.read(cx).open);
        assert_eq!(window.find(trigger.clone()).expanded(), Some(true));
        assert!(window.try_find(content.clone()).is_some());

        // Keyboard Space collapses; Enter expands again.
        let focus = view.read(cx).focus.clone();
        focus.focus(window, cx);
        window.render_frame(cx);
        window.press("space", cx);
        assert!(!view.read(cx).open);
        assert!(window.try_find(content.clone()).is_none());

        window.press("enter", cx);
        assert!(view.read(cx).open);
        assert_eq!(view.read(cx).toggles, 3);

        // Disabled collapsible ignores click.
        let disabled_trigger = ElementId::from((ElementId::from("col-disabled"), "trigger"));
        assert_eq!(window.find(disabled_trigger.clone()).disabled(), Some(true));
        window.click(disabled_trigger, cx);
        assert!(!view.read(cx).disabled_open);
    })
    .unwrap();
}

#[gpui::test]
fn collapsible_rtl_mirrors_header_layout(cx: &mut gpui::TestAppContext) {
    use gpui_material::test::TestSupportExt;

    struct RtlHarness;
    impl Render for RtlHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w_full().p_4().child(
                Collapsible::new("rtl-col", "Title", true)
                    .leading(div().id("rtl-lead").test_support().child("L"))
                    .trailing(div().id("rtl-trail").test_support().child("T"))
                    .content(Text::new("Body")),
            )
        }
    }

    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            set_window_material_theme(
                Some(MaterialTheme::default().direction(Direction::Rtl)),
                window,
                cx,
            );
            cx.new(|_| RtlHarness)
        })
        .unwrap()
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        let lead_x = window.find("rtl-lead").bounds().origin.x;
        let trail_x = window.find("rtl-trail").bounds().origin.x;
        assert!(
            lead_x > trail_x,
            "in RTL leading slot ({lead_x:?}) must appear right of trailing slot ({trail_x:?})"
        );
    })
    .unwrap();
}

struct AccordionHarness {
    mode: AccordionMode,
    variant: AccordionVariant,
    expanded: Vec<SharedString>,
    focus_a: FocusHandle,
    focus_c: FocusHandle,
}

impl Render for AccordionHarness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        div().p_4().child(
            Accordion::new(
                "acc",
                [
                    AccordionItem::new("a", "Alpha")
                        .track_focus(&self.focus_a)
                        .content(Text::new("Alpha content")),
                    AccordionItem::new("b", "Beta")
                        .disabled(true)
                        .content(Text::new("Beta content")),
                    AccordionItem::new("c", "Gamma")
                        .track_focus(&self.focus_c)
                        .content(Text::new("Gamma content")),
                ],
            )
            .accessible_label("Sections")
            .mode(self.mode)
            .variant(self.variant)
            .expanded(self.expanded.clone())
            .on_change(move |ids, _, cx| {
                entity.update(cx, |this, cx| {
                    this.expanded = ids;
                    cx.notify();
                });
            }),
        )
    }
}

#[gpui::test]
fn accordion_single_multiple_and_roving_focus(cx: &mut gpui::TestAppContext) {
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        set_reduce_motion_preference(ReduceMotionPreference::Reduce, cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|cx| AccordionHarness {
                mode: AccordionMode::Single,
                variant: AccordionVariant::Standard,
                expanded: vec!["a".into()],
                focus_a: cx.focus_handle(),
                focus_c: cx.focus_handle(),
            });
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, view.unwrap())
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);

        let id_a = ElementId::from((ElementId::from("acc"), SharedString::from("a")));
        let id_b = ElementId::from((ElementId::from("acc"), SharedString::from("b")));
        let id_c = ElementId::from((ElementId::from("acc"), SharedString::from("c")));
        let panel_a = ElementId::from((ElementId::from("acc"), "a-panel"));
        let panel_c = ElementId::from((ElementId::from("acc"), "c-panel"));

        assert_eq!(window.find(id_a.clone()).expanded(), Some(true));
        assert!(window.try_find(panel_a.clone()).is_some());
        assert!(window.try_find(panel_c.clone()).is_none());

        // Disabled middle item ignores click and cannot expand.
        assert_eq!(window.find(id_b.clone()).expanded(), Some(false));
        window.click(id_b, cx);
        assert_eq!(view.read(cx).expanded, vec![SharedString::from("a")]);

        // Roving focus: Down from "a" skips disabled "b" and lands on "c".
        let focus_a = view.read(cx).focus_a.clone();
        focus_a.focus(window, cx);
        window.render_frame(cx);
        window.press("down", cx);
        assert!(view.read(cx).focus_c.is_focused(window));

        // Pressing Enter toggles "c" open and collapses "a" in Single mode.
        window.press("enter", cx);
        assert_eq!(view.read(cx).expanded, vec![SharedString::from("c")]);
        assert!(window.try_find(panel_a.clone()).is_none());
        assert!(window.try_find(panel_c.clone()).is_some());

        // Home returns to "a", End returns to "c".
        window.press("home", cx);
        assert!(view.read(cx).focus_a.is_focused(window));
        window.press("end", cx);
        assert!(view.read(cx).focus_c.is_focused(window));

        // Switch to Multiple + Segmented: both "a" and "c" can stay expanded.
        view.update(cx, |this, cx| {
            this.mode = AccordionMode::Multiple;
            this.variant = AccordionVariant::Segmented;
            cx.notify();
        });
        window.render_frame(cx);
        window.click(id_a, cx);
        assert_eq!(
            view.read(cx).expanded,
            vec![SharedString::from("a"), SharedString::from("c")]
        );
        assert!(window.try_find(panel_a).is_some());
        assert!(window.try_find(panel_c).is_some());

        // SingleRequired keeps open item expanded when clicked again.
        view.update(cx, |this, cx| {
            this.mode = AccordionMode::SingleRequired;
            this.expanded = vec!["c".into()];
            cx.notify();
        });
        window.render_frame(cx);
        window.click(id_c, cx);
        assert_eq!(view.read(cx).expanded, vec![SharedString::from("c")]);
    })
    .unwrap();
}

struct PaginationHarness {
    current: usize,
    total: usize,
    disabled: bool,
    changes: Vec<usize>,
}

impl Render for PaginationHarness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        div().p_4().child(
            Pagination::new("pag", self.current, self.total)
                .accessible_label("Catalog pages")
                .disabled(self.disabled)
                .on_change(move |page, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.current = page;
                        this.changes.push(page);
                        cx.notify();
                    });
                }),
        )
    }
}

#[gpui::test]
fn pagination_pointer_keyboard_boundaries_ellipsis_and_rtl(cx: &mut gpui::TestAppContext) {
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            set_window_material_theme(
                Some(MaterialTheme::default().density(Density::Compact)),
                window,
                cx,
            );
            let entity = cx.new(|_| PaginationHarness {
                current: 1,
                total: 12,
                disabled: false,
                changes: Vec::new(),
            });
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, view.unwrap())
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);

        let prev = ElementId::from((ElementId::from("pag"), "previous"));
        let next = ElementId::from((ElementId::from("pag"), "next"));
        let page1 = ElementId::from((ElementId::from("pag"), "page-1"));
        let page2 = ElementId::from((ElementId::from("pag"), "page-2"));
        let page12 = ElementId::from((ElementId::from("pag"), "page-12"));

        // On page 1, Previous is disabled and Next is enabled; ellipsis exists for 12 pages.
        assert_eq!(window.find(prev.clone()).disabled(), Some(true));
        assert_ne!(window.find(next.clone()).disabled(), Some(true));
        assert!(window.try_find(page12.clone()).is_some());
        assert_eq!(window.find(page1.clone()).selected(), Some(true));

        // Clicking disabled Previous does nothing.
        window.click(prev.clone(), cx);
        assert_eq!(view.read(cx).current, 1);

        // Clicking Next moves to page 2.
        window.click(next.clone(), cx);
        assert_eq!(view.read(cx).current, 2);
        assert_eq!(window.find(page2.clone()).selected(), Some(true));

        // Clicking page 12 jumps to last page and disables Next.
        window.click(page12.clone(), cx);
        assert_eq!(view.read(cx).current, 12);
        assert_eq!(window.find(next.clone()).disabled(), Some(true));
        assert_ne!(window.find(prev.clone()).disabled(), Some(true));

        // Clicking Previous moves back to 11.
        window.click(prev.clone(), cx);
        assert_eq!(view.read(cx).current, 11);

        // Keyboard navigation: focus page-11, press Right -> page-12, press Enter -> selects 12.
        window.click(ElementId::from((ElementId::from("pag"), "page-11")), cx);
        window.press("right", cx);
        window.press("enter", cx);
        assert_eq!(view.read(cx).current, 12);

        // Disabled pagination ignores clicks.
        view.update(cx, |this, cx| {
            this.disabled = true;
            cx.notify();
        });
        window.render_frame(cx);
        window.click(prev.clone(), cx);
        assert_eq!(view.read(cx).current, 12);

        // RTL mirrors layout so Previous appears right of Next.
        view.update(cx, |this, cx| {
            this.disabled = false;
            this.current = 5;
            cx.notify();
        });
        set_window_material_theme(
            Some(MaterialTheme::default().direction(Direction::Rtl)),
            window,
            cx,
        );
        window.render_frame(cx);
        let prev_x = window.find(prev).bounds().origin.x;
        let next_x = window.find(next).bounds().origin.x;
        assert!(
            prev_x > next_x,
            "in RTL previous ({prev_x:?}) must appear right of next ({next_x:?})"
        );
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(16));
}

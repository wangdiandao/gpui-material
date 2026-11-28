use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext as _, Bounds, Context, ElementId, Entity, FocusHandle, InteractiveElement as _,
    IntoElement, ParentElement as _, Point, Render, Role, ScrollStrategy, SharedString, Size,
    Styled as _, Subscription, TestAppContext, Window, WindowBounds, WindowHandle, WindowOptions,
    div, px, size,
};
use gpui_base::Root;
use gpui_material::{
    MaterialRoot, MaterialTheme, ThemeScope,
    data_views::{
        ColumnAlignment, DataTable, DataTableCell, DataTableColumn, DataTableRow, SortDirection,
        Tree, TreeEvent, TreeNode, TreeState, VirtualList, VirtualListScrollHandle,
        VirtualListVariant,
    },
    test::{TestQueryExt as _, TestSupportExt as _, TestWindowExt as _},
    theme::{Density, Direction, LogicalEdges},
};

use crate::Story;

fn open_test_window<V: Render>(
    cx: &mut TestAppContext,
    width: f32,
    height: f32,
    build: impl FnOnce(&mut Window, &mut gpui::App) -> Entity<V>,
) -> (WindowHandle<Root>, Entity<V>) {
    cx.update(|cx| {
        gpui_material::init(MaterialTheme::default(), cx);
        let mut view_entity = None;
        let bounds = Bounds {
            origin: Point::default(),
            size: size(px(width), px(height)),
        };
        let handle = MaterialRoot::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            cx,
            |window, cx| {
                let entity = build(window, cx);
                view_entity = Some(entity.clone());
                entity
            },
        )
        .expect("open test window");
        (handle, view_entity.unwrap())
    })
}

struct TableTestView {
    sort_col: Option<SharedString>,
    sort_dir: Option<SortDirection>,
    selected: Option<SharedString>,
    sorts: Vec<(SharedString, SortDirection)>,
    selections: Vec<SharedString>,
    row_focus: [FocusHandle; 3],
}

impl TableTestView {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            sort_col: None,
            sort_dir: None,
            selected: None,
            sorts: Vec::new(),
            selections: Vec::new(),
            row_focus: [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
        }
    }
}

impl Render for TableTestView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sort_view = cx.entity();
        let select_view = cx.entity();
        let name_dir = (self.sort_col.as_deref() == Some("name"))
            .then_some(self.sort_dir)
            .flatten();

        div().size_full().p_4().child(
            DataTable::new("test-table")
                .accessible_label("Services")
                .caption("Service status")
                .columns([
                    DataTableColumn::new("name", "Name")
                        .weight(2.0)
                        .sortable(true)
                        .sort_direction(name_dir),
                    DataTableColumn::new("latency", "Latency")
                        .width(px(120.))
                        .alignment(ColumnAlignment::End),
                ])
                .rows([
                    DataTableRow::new(
                        "r1",
                        [DataTableCell::text("API"), DataTableCell::text("18 ms")],
                    )
                    .selected(self.selected.as_deref() == Some("r1"))
                    .track_focus(&self.row_focus[0]),
                    DataTableRow::new(
                        "r2",
                        [
                            DataTableCell::text("Archive"),
                            DataTableCell::text("120 ms"),
                        ],
                    )
                    .disabled(true)
                    .track_focus(&self.row_focus[1]),
                    DataTableRow::new(
                        "r3",
                        [DataTableCell::text("Billing"), DataTableCell::text("12 ms")],
                    )
                    .selected(self.selected.as_deref() == Some("r3"))
                    .track_focus(&self.row_focus[2]),
                ])
                .on_sort(move |col, dir, _, cx| {
                    sort_view.update(cx, |this, cx| {
                        this.sort_col = Some(col.clone());
                        this.sort_dir = Some(dir);
                        this.sorts.push((col.clone(), dir));
                        cx.notify();
                    });
                })
                .on_row_select(move |id, _, cx| {
                    select_view.update(cx, |this, cx| {
                        this.selected = Some(id.clone());
                        this.selections.push(id.clone());
                        cx.notify();
                    });
                }),
        )
    }
}

#[gpui::test]
fn data_table_supports_pointer_sorting_row_selection_disabled_rows_and_keyboard_roving(
    cx: &mut TestAppContext,
) {
    let (handle, view) = open_test_window(cx, 640., 420., |_, cx| cx.new(TableTestView::new));

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);

        assert_eq!(window.find("test-table").role(), Some(Role::Table));
        assert_eq!(window.find("test-table").label(), Some("Services"));

        let name_head = ElementId::from((ElementId::from("test-table"), "col-name"));
        let row_1 = ElementId::from((ElementId::from("test-table"), "row-r1"));
        let row_2 = ElementId::from((ElementId::from("test-table"), "row-r2"));
        let row_3 = ElementId::from((ElementId::from("test-table"), "row-r3"));

        // Pointer sort toggle: None -> Ascending -> Descending.
        window.click(name_head.clone(), cx);
        window.click(name_head.clone(), cx);
        assert_eq!(
            view.read(cx).sorts,
            vec![
                ("name".into(), SortDirection::Ascending),
                ("name".into(), SortDirection::Descending),
            ]
        );

        // Keyboard Enter on focused sortable column header cycles Descending -> Ascending.
        window.press("enter", cx);
        assert_eq!(view.read(cx).sort_dir, Some(SortDirection::Ascending));

        // Pointer click on enabled row selects it; clicking disabled row does nothing.
        window.click(row_1.clone(), cx);
        assert_eq!(view.read(cx).selected.as_deref(), Some("r1"));
        assert_eq!(window.find(row_1.clone()).selected(), Some(true));

        window.click(row_2.clone(), cx);
        assert_eq!(view.read(cx).selected.as_deref(), Some("r1"));
        assert_eq!(view.read(cx).selections, vec![SharedString::from("r1")]);

        // Roving keyboard navigation skips disabled row r2 and activates r3 with Space.
        let focus_r1 = view.read(cx).row_focus[0].clone();
        focus_r1.focus(window, cx);
        window.render_frame(cx);
        window.press("down", cx);
        assert!(view.read(cx).row_focus[2].is_focused(window));
        assert!(!view.read(cx).row_focus[1].is_focused(window));
        window.press("space", cx);
        assert_eq!(view.read(cx).selected.as_deref(), Some("r3"));
        assert_eq!(window.find(row_3).selected(), Some(true));

        // Home returns to first enabled row r1, End jumps to r3.
        window.press("home", cx);
        assert!(view.read(cx).row_focus[0].is_focused(window));
        window.press("end", cx);
        assert!(view.read(cx).row_focus[2].is_focused(window));
    })
    .unwrap();
}

struct TableLayoutProbe;

impl Render for TableLayoutProbe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let make_table = |id: &'static str| {
            DataTable::new(id)
                .columns([
                    DataTableColumn::new("first", "First").width(px(140.)),
                    DataTableColumn::new("second", "Second").width(px(140.)),
                ])
                .row(DataTableRow::new("row", ["A", "B"]))
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_2()
            .child(ThemeScope::new(make_table("ltr-compact")).density(Density::Compact))
            .child(ThemeScope::new(make_table("ltr-comfortable")).density(Density::Comfortable))
            .child(ThemeScope::new(make_table("rtl-table")).direction(Direction::Rtl))
    }
}

#[gpui::test]
fn data_table_respects_density_rtl_mirroring_and_reduced_motion(cx: &mut TestAppContext) {
    let (handle, _) = open_test_window(cx, 640., 480., |_, cx| {
        cx.set_reduce_motion(true);
        cx.new(|_| TableLayoutProbe)
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);

        let compact_row = window
            .find(ElementId::from((ElementId::from("ltr-compact"), "row-row")))
            .bounds();
        let comfortable_row = window
            .find(ElementId::from((
                ElementId::from("ltr-comfortable"),
                "row-row",
            )))
            .bounds();
        assert!(
            compact_row.size.height < comfortable_row.size.height,
            "compact height {:?} should be smaller than comfortable {:?}",
            compact_row.size.height,
            comfortable_row.size.height
        );

        let ltr_first = window
            .find(ElementId::from((
                ElementId::from("ltr-compact"),
                "col-first",
            )))
            .bounds();
        let ltr_second = window
            .find(ElementId::from((
                ElementId::from("ltr-compact"),
                "col-second",
            )))
            .bounds();
        assert!(ltr_first.left() < ltr_second.left());

        let rtl_first = window
            .find(ElementId::from((ElementId::from("rtl-table"), "col-first")))
            .bounds();
        let rtl_second = window
            .find(ElementId::from((
                ElementId::from("rtl-table"),
                "col-second",
            )))
            .bounds();
        assert!(
            rtl_first.left() > rtl_second.left(),
            "RTL first column {:?} should render to the right of second column {:?}",
            rtl_first,
            rtl_second
        );
    })
    .unwrap();
}

struct TreeTestView {
    tree: Entity<TreeState>,
    events: Rc<RefCell<Vec<TreeEvent>>>,
    selected_cb: Rc<RefCell<Vec<SharedString>>>,
    activated_cb: Rc<RefCell<Vec<SharedString>>>,
    _sub: Subscription,
    direction: Direction,
    density: Density,
}

impl TreeTestView {
    fn new(direction: Direction, density: Density, cx: &mut Context<Self>) -> Self {
        let nodes = vec![
            TreeNode::new("src", "src")
                .expanded(false)
                .child(TreeNode::new("src/disabled", "disabled.rs").disabled(true))
                .child(TreeNode::new("src/lib", "lib.rs")),
            TreeNode::new("locked", "locked").disabled(true),
            TreeNode::new("readme", "README.md"),
        ];
        let tree = cx.new(|cx| TreeState::new(cx).items(nodes, cx));
        let events = Rc::new(RefCell::new(Vec::new()));
        let captured = events.clone();
        let sub = cx.subscribe(&tree, move |_, _, event: &TreeEvent, _| {
            captured.borrow_mut().push(event.clone());
        });
        Self {
            tree,
            events,
            selected_cb: Rc::new(RefCell::new(Vec::new())),
            activated_cb: Rc::new(RefCell::new(Vec::new())),
            _sub: sub,
            direction,
            density,
        }
    }
}

impl Render for TreeTestView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let selected_cb = self.selected_cb.clone();
        let activated_cb = self.activated_cb.clone();
        div().size_full().p_4().child(
            ThemeScope::new(
                Tree::new("test-tree", &self.tree)
                    .accessible_label("Files")
                    .h(px(220.))
                    .on_select(move |id, _, _| selected_cb.borrow_mut().push(id.clone()))
                    .on_activate(move |id, _, _| activated_cb.borrow_mut().push(id.clone())),
            )
            .direction(self.direction)
            .density(self.density),
        )
    }
}

#[gpui::test]
fn tree_supports_pointer_click_disabled_nodes_wai_aria_keyboard_and_events(
    cx: &mut TestAppContext,
) {
    let (handle, view) = open_test_window(cx, 520., 360., |_, cx| {
        cx.new(|cx| TreeTestView::new(Direction::Ltr, Density::Standard, cx))
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);

        let tree_items = window.find_all_by_role(Role::TreeItem);
        assert_eq!(tree_items.len(), 3);

        // Clicking disabled node does not select or emit callbacks.
        assert_eq!(window.find(1usize).label(), Some("locked"));
        window.click(1usize, cx);
        assert_eq!(view.read(cx).tree.read(cx).selected_id(cx), None);
        assert!(view.read(cx).selected_cb.borrow().is_empty());

        // Clicking branch "src" selects and expands it.
        assert_eq!(window.find(0usize).label(), Some("src"));
        window.click(0usize, cx);
        assert_eq!(
            view.read(cx).tree.read(cx).selected_id(cx).as_deref(),
            Some("src")
        );
        assert!(view.read(cx).tree.read(cx).is_expanded("src"));
        assert_eq!(view.read(cx).tree.read(cx).visible_count(), 5);

        // Keyboard Right on expanded branch moves to first enabled child ("src/lib"), skipping "src/disabled".
        window.press("right", cx);
        assert_eq!(
            view.read(cx).tree.read(cx).selected_id(cx).as_deref(),
            Some("src/lib")
        );

        // Keyboard Down skips disabled "locked" and lands on "readme".
        window.press("down", cx);
        assert_eq!(
            view.read(cx).tree.read(cx).selected_id(cx).as_deref(),
            Some("readme")
        );

        // Space activates the selected leaf.
        window.press("space", cx);
        assert!(
            view.read(cx)
                .activated_cb
                .borrow()
                .iter()
                .any(|id| id.as_ref() == "readme")
        );

        // Home returns to "src", Left collapses it.
        window.press("home", cx);
        assert_eq!(
            view.read(cx).tree.read(cx).selected_id(cx).as_deref(),
            Some("src")
        );
        window.press("left", cx);
        assert!(!view.read(cx).tree.read(cx).is_expanded("src"));
        assert_eq!(view.read(cx).tree.read(cx).visible_count(), 3);
    })
    .unwrap();

    cx.read_entity(&view, |view, _| {
        let events = view.events.borrow();
        assert!(events.contains(&TreeEvent::Expanded("src".into())));
        assert!(events.contains(&TreeEvent::Collapsed("src".into())));
    });
}

#[gpui::test]
fn tree_mirrors_rtl_navigation_and_scales_density_with_reduced_motion(cx: &mut TestAppContext) {
    let (handle, view) = open_test_window(cx, 520., 360., |_, cx| {
        cx.set_reduce_motion(true);
        cx.new(|cx| TreeTestView::new(Direction::Rtl, Density::Compact, cx))
    });

    let compact_height = cx
        .update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            window.render_frame(cx);

            let first_bounds = window.find_by_label("src").bounds();
            let tree = view.read(cx).tree.clone();
            tree.update(cx, |state, cx| {
                state.select(Some("src"), cx);
                state.focus(window, cx);
            });
            window.render_frame(cx);

            // In RTL, Left expands and Right collapses.
            window.press("left", cx);
            assert!(
                tree.read(cx).is_expanded("src"),
                "Left arrow should expand branch in RTL"
            );
            window.press("left", cx);
            assert_eq!(
                tree.read(cx).selected_id(cx).as_deref(),
                Some("src/lib"),
                "Second Left arrow in RTL should step into first enabled child"
            );
            window.press("right", cx);
            assert_eq!(
                tree.read(cx).selected_id(cx).as_deref(),
                Some("src"),
                "Right arrow on child in RTL should jump to parent"
            );
            window.press("right", cx);
            assert!(
                !tree.read(cx).is_expanded("src"),
                "Right arrow on expanded branch in RTL should collapse it"
            );

            first_bounds.size.height
        })
        .unwrap();

    let (handle_comfy, _) = open_test_window(cx, 520., 360., |_, cx| {
        cx.new(|cx| TreeTestView::new(Direction::Ltr, Density::Comfortable, cx))
    });
    let comfortable_height = cx
        .update_window(handle_comfy.into(), |_, window, cx| {
            window.activate_window();
            window.render_frame(cx);
            window.find_by_label("src").bounds().size.height
        })
        .unwrap();
    assert!(
        compact_height < comfortable_height,
        "compact tree row {:?} should be shorter than comfortable {:?}",
        compact_height,
        comfortable_height
    );
}

struct VirtualListTestView {
    v_scroll: VirtualListScrollHandle,
    h_scroll: VirtualListScrollHandle,
    v_focus: FocusHandle,
    h_focus: FocusHandle,
    v_sizes: Rc<Vec<Size<gpui::Pixels>>>,
    h_sizes: Rc<Vec<Size<gpui::Pixels>>>,
    rendered_vertical: Rc<RefCell<Vec<usize>>>,
}

impl VirtualListTestView {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            v_scroll: VirtualListScrollHandle::new(),
            h_scroll: VirtualListScrollHandle::new(),
            v_focus: cx.focus_handle(),
            h_focus: cx.focus_handle(),
            v_sizes: Rc::new((0..500).map(|_| size(px(0.), px(40.))).collect()),
            h_sizes: Rc::new((0..80).map(|_| size(px(120.), px(60.))).collect()),
            rendered_vertical: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl Render for VirtualListTestView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rendered = self.rendered_vertical.clone();
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_3()
            .child(
                VirtualList::vertical(
                    cx.entity(),
                    "test-vlist",
                    self.v_sizes.clone(),
                    move |_, range, _, _| {
                        let mut log = rendered.borrow_mut();
                        log.clear();
                        log.extend(range.clone());
                        range
                            .map(|ix| {
                                div()
                                    .id(("vrow", ix))
                                    .test_support()
                                    .w_full()
                                    .h(px(40.))
                                    .child(format!("Row {ix}"))
                            })
                            .collect()
                    },
                )
                .accessible_label("Rows")
                .variant(VirtualListVariant::Contained)
                .track_scroll(&self.v_scroll)
                .track_focus(&self.v_focus)
                .h(px(200.)),
            )
            .child(
                ThemeScope::new(
                    VirtualList::horizontal(
                        cx.entity(),
                        "test-hlist",
                        self.h_sizes.clone(),
                        |_, range, _, _| {
                            range
                                .map(|ix| div().w(px(120.)).h(px(60.)).child(format!("Col {ix}")))
                                .collect()
                        },
                    )
                    .accessible_label("Horizontal strip")
                    .variant(VirtualListVariant::Contained)
                    .content_padding(LogicalEdges::all(4.0))
                    .item_gap(px(8.))
                    .track_scroll(&self.h_scroll)
                    .track_focus(&self.h_focus)
                    .h(px(80.)),
                )
                .direction(Direction::Rtl),
            )
    }
}

#[gpui::test]
fn virtual_list_virtualizes_large_collections_supports_scroll_handle_keyboard_and_rtl(
    cx: &mut TestAppContext,
) {
    let (handle, view) = open_test_window(cx, 520., 360., |_, cx| {
        cx.set_reduce_motion(true);
        cx.new(VirtualListTestView::new)
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);

        assert_eq!(window.find("test-vlist").role(), Some(Role::List));
        assert_eq!(window.find("test-vlist").label(), Some("Rows"));

        // Only a small visible window of the 500 items is rendered.
        let initial_rendered = view.read(cx).rendered_vertical.borrow().clone();
        assert!(!initial_rendered.is_empty());
        assert!(
            initial_rendered.len() < 20,
            "expected virtualized slice, got {} items",
            initial_rendered.len()
        );
        assert_eq!(initial_rendered[0], 0);

        // Scroll handle jumps deep into the collection.
        view.read(cx)
            .v_scroll
            .scroll_to_item(250, ScrollStrategy::Top);
        window.render_frame(cx);
        let jumped = view.read(cx).rendered_vertical.borrow().clone();
        assert!(
            jumped.contains(&250),
            "expected item 250 in visible slice, got {jumped:?}"
        );

        // Keyboard Home returns to top, Down steps downward, End jumps to bottom.
        let v_focus = view.read(cx).v_focus.clone();
        v_focus.focus(window, cx);
        window.render_frame(cx);
        window.press("home", cx);
        assert_eq!(view.read(cx).v_scroll.offset().y, px(0.));

        window.press("down", cx);
        assert!(view.read(cx).v_scroll.offset().y < px(0.));

        window.press("end", cx);
        let end_y = view.read(cx).v_scroll.offset().y;
        assert!(
            end_y < px(-1000.),
            "expected scroll near bottom, got {end_y:?}"
        );

        // RTL horizontal VirtualList advances on Left and retreats on Right.
        let h_focus = view.read(cx).h_focus.clone();
        h_focus.focus(window, cx);
        window.render_frame(cx);
        window.press("left", cx);
        let after_left = view.read(cx).h_scroll.offset().x;
        assert!(
            after_left < px(0.),
            "Left in RTL horizontal VirtualList should scroll forward, got {after_left:?}"
        );
        window.press("right", cx);
        assert!(view.read(cx).h_scroll.offset().x > after_left);
    })
    .unwrap();
}

#[gpui::test]
fn gallery_data_views_story_renders_and_handles_interaction(cx: &mut TestAppContext) {
    let (handle, _) = open_test_window(cx, 900., 900., |window, cx| {
        let story_view = Story::DataViews
            .new_view(window, cx)
            .expect("DataViews story constructs a view");
        cx.new(|_| StoryWrapper(story_view))
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);

        assert_eq!(window.find("services-table").role(), Some(Role::Table));
        assert_eq!(window.find("events-vlist").role(), Some(Role::List));
        assert_eq!(window.find("cards-hlist").role(), Some(Role::List));
        assert!(window.try_find("workspace-tree").is_some());
    })
    .unwrap();
}

struct StoryWrapper(gpui::AnyView);

impl Render for StoryWrapper {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.0.clone())
    }
}

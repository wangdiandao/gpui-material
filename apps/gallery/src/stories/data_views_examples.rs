use std::rc::Rc;

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Render, ScrollStrategy,
    SharedString, Size, Styled, Window, div, px, size,
};
use gpui_material::{
    Badge, Button, ButtonVariant, Icon, IconName, Text,
    data_views::{
        ColumnAlignment, DataTable, DataTableCell, DataTableColumn, DataTableRow, SortDirection,
        Tree, TreeNode, TreeState, VirtualList, VirtualListScrollHandle, VirtualListVariant,
    },
    foundation::{color, window_material_theme},
    theme::{ColorRole, LogicalEdges, TypeRole},
};

use crate::story::{section, stack};

#[derive(Clone)]
struct ServiceRecord {
    id: &'static str,
    name: &'static str,
    region: &'static str,
    latency_ms: u32,
    status: &'static str,
    disabled: bool,
}

const SERVICES: [ServiceRecord; 5] = [
    ServiceRecord {
        id: "api",
        name: "API Gateway",
        region: "us-east-1",
        latency_ms: 18,
        status: "Healthy",
        disabled: false,
    },
    ServiceRecord {
        id: "auth",
        name: "Identity Broker",
        region: "eu-central-1",
        latency_ms: 24,
        status: "Healthy",
        disabled: false,
    },
    ServiceRecord {
        id: "search",
        name: "Search Indexer",
        region: "ap-northeast-1",
        latency_ms: 41,
        status: "Degraded",
        disabled: false,
    },
    ServiceRecord {
        id: "archive",
        name: "Cold Archive",
        region: "us-west-2",
        latency_ms: 120,
        status: "Maintenance",
        disabled: true,
    },
    ServiceRecord {
        id: "billing",
        name: "Billing Worker",
        region: "eu-west-1",
        latency_ms: 12,
        status: "Healthy",
        disabled: false,
    },
];

fn sample_tree_nodes() -> Vec<TreeNode> {
    vec![
        TreeNode::new("crates", "crates")
            .icon(IconName::Archive)
            .expanded(true)
            .child(
                TreeNode::new("crates/material", "material")
                    .icon(IconName::Inbox)
                    .expanded(true)
                    .child(
                        TreeNode::new("crates/material/lib", "lib.rs").icon(IconName::Description),
                    )
                    .child(
                        TreeNode::new("crates/material/cargo", "Cargo.toml")
                            .icon(IconName::Description),
                    ),
            )
            .child(
                TreeNode::new("crates/extras", "extras")
                    .icon(IconName::Inbox)
                    .child(
                        TreeNode::new("crates/extras/data_views", "data_views.rs")
                            .icon(IconName::Description),
                    ),
            )
            .child(
                TreeNode::new("crates/legacy", "legacy (read-only)")
                    .icon(IconName::Error)
                    .disabled(true),
            ),
        TreeNode::new("docs", "docs")
            .icon(IconName::Archive)
            .child(TreeNode::new("docs/component", "component.md").icon(IconName::Description))
            .child(
                TreeNode::new("docs/architecture", "architecture.md").icon(IconName::Description),
            ),
    ]
}

const VIRTUAL_ITEMS_COUNT: usize = 250;

pub(crate) struct DataViewsExamples {
    sort_column: SharedString,
    sort_direction: SortDirection,
    selected_row: Option<SharedString>,
    tree: Entity<TreeState>,
    tree_status: SharedString,
    vlist_scroll: VirtualListScrollHandle,
    vlist_sizes: Rc<Vec<Size<gpui::Pixels>>>,
    hlist_scroll: VirtualListScrollHandle,
    hlist_sizes: Rc<Vec<Size<gpui::Pixels>>>,
}

impl DataViewsExamples {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let tree = cx.new(|cx| TreeState::new(cx).items(sample_tree_nodes(), cx));
        let vlist_sizes = Rc::new(
            (0..VIRTUAL_ITEMS_COUNT)
                .map(|ix| size(px(0.), if ix % 5 == 0 { px(56.) } else { px(42.) }))
                .collect(),
        );
        let hlist_sizes = Rc::new(
            (0..40)
                .map(|ix| size(if ix % 3 == 0 { px(160.) } else { px(124.) }, px(64.)))
                .collect(),
        );
        Self {
            sort_column: "name".into(),
            sort_direction: SortDirection::Ascending,
            selected_row: Some("api".into()),
            tree,
            tree_status: "Select or expand a node".into(),
            vlist_scroll: VirtualListScrollHandle::new(),
            vlist_sizes,
            hlist_scroll: VirtualListScrollHandle::new(),
            hlist_sizes,
        }
    }

    fn sorted_services(&self) -> Vec<ServiceRecord> {
        let mut records = SERVICES.to_vec();
        records.sort_by(|a, b| {
            let ord = match self.sort_column.as_ref() {
                "region" => a.region.cmp(b.region),
                "latency" => a.latency_ms.cmp(&b.latency_ms),
                "status" => a.status.cmp(b.status),
                _ => a.name.cmp(b.name),
            };
            match self.sort_direction {
                SortDirection::Ascending => ord,
                SortDirection::Descending => ord.reverse(),
            }
        });
        records
    }
}

impl Render for DataViewsExamples {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let surface_high = color(theme.colors.color(ColorRole::SurfaceContainerHigh));
        let surface_low = color(theme.colors.color(ColorRole::SurfaceContainerLow));

        let view = cx.entity();
        let sort_view = view.clone();
        let select_view = view.clone();
        let services = self.sorted_services();

        let col_dir = |id: &str| (self.sort_column.as_ref() == id).then_some(self.sort_direction);

        let table = DataTable::new("services-table")
            .accessible_label("Cluster services")
            .caption(
                "Click column headers to sort; use Up/Down/Enter/Space to navigate enabled rows.",
            )
            .striped(true)
            .columns([
                DataTableColumn::new("name", "Service")
                    .weight(2.0)
                    .sortable(true)
                    .sort_direction(col_dir("name")),
                DataTableColumn::new("region", "Region")
                    .weight(1.3)
                    .sortable(true)
                    .sort_direction(col_dir("region")),
                DataTableColumn::new("latency", "p95 Latency")
                    .width(px(120.))
                    .alignment(ColumnAlignment::End)
                    .sortable(true)
                    .sort_direction(col_dir("latency")),
                DataTableColumn::new("status", "Status")
                    .width(px(132.))
                    .alignment(ColumnAlignment::Center)
                    .sortable(true)
                    .sort_direction(col_dir("status")),
            ])
            .rows(services.into_iter().map(|service| {
                let selected = self.selected_row.as_deref() == Some(service.id);
                let badge = Badge::standalone_label(service.status);
                DataTableRow::new(
                    service.id,
                    [
                        DataTableCell::text(service.name),
                        DataTableCell::text(service.region).muted(true),
                        DataTableCell::text(format!("{} ms", service.latency_ms)),
                        DataTableCell::custom(badge),
                    ],
                )
                .selected(selected)
                .disabled(service.disabled)
            }))
            .on_sort(move |col, dir, _, cx| {
                sort_view.update(cx, |this, cx| {
                    this.sort_column = col.clone();
                    this.sort_direction = dir;
                    cx.notify();
                });
            })
            .on_row_select(move |row_id, _, cx| {
                select_view.update(cx, |this, cx| {
                    this.selected_row = Some(row_id.clone());
                    cx.notify();
                });
            });

        let tree_select_view = view.clone();
        let tree_toggle_view = view.clone();
        let expand_tree = self.tree.clone();
        let collapse_tree = self.tree.clone();

        let tree_controls = div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                Button::tonal("tree-expand-all", "Expand all").on_activate(move |_, cx| {
                    expand_tree.update(cx, |state, cx| state.expand_all(cx));
                }),
            )
            .child(
                Button::new("tree-collapse-all", "Collapse all")
                    .variant(ButtonVariant::Outlined)
                    .on_activate(move |_, cx| {
                        collapse_tree.update(cx, |state, cx| state.collapse_all(cx));
                    }),
            )
            .child(
                Text::new(self.tree_status.clone())
                    .id("tree-status")
                    .role(TypeRole::BodySmall)
                    .color(ColorRole::OnSurfaceVariant),
            );

        let tree = Tree::new("workspace-tree", &self.tree)
            .accessible_label("Workspace explorer")
            .h(px(240.))
            .on_select(move |id, _, cx| {
                tree_select_view.update(cx, |this, cx| {
                    this.tree_status = format!("Selected: {id}").into();
                    cx.notify();
                });
            })
            .on_toggle(move |id, expanded, _, cx| {
                tree_toggle_view.update(cx, |this, cx| {
                    this.tree_status =
                        format!("{}: {id}", if expanded { "Expanded" } else { "Collapsed" }).into();
                    cx.notify();
                });
            });

        let jump_top_scroll = self.vlist_scroll.clone();
        let jump_mid_scroll = self.vlist_scroll.clone();
        let jump_end_scroll = self.vlist_scroll.clone();

        let vlist_controls = div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                Button::new("vlist-top", "Jump to top")
                    .variant(ButtonVariant::Outlined)
                    .on_activate(move |_, _| {
                        jump_top_scroll.scroll_to_item(0, ScrollStrategy::Top);
                    }),
            )
            .child(
                Button::tonal("vlist-mid", "Item #125").on_activate(move |_, _| {
                    jump_mid_scroll.scroll_to_item(125, ScrollStrategy::Center);
                }),
            )
            .child(
                Button::new("vlist-bottom", "Scroll to bottom")
                    .variant(ButtonVariant::Outlined)
                    .on_activate(move |_, _| {
                        jump_end_scroll.scroll_to_bottom();
                    }),
            );

        let vlist = VirtualList::vertical(
            cx.entity(),
            "events-vlist",
            self.vlist_sizes.clone(),
            move |_, range, _, _| {
                range
                    .map(|ix| {
                        let is_group = ix % 5 == 0;
                        div()
                            .w_full()
                            .h(if is_group { px(56.) } else { px(42.) })
                            .px_3()
                            .flex()
                            .items_center()
                            .justify_between()
                            .bg(if is_group { surface_high } else { surface_low })
                            .border_b_1()
                            .border_color(outline)
                            .child(
                                Text::new(if is_group {
                                    format!("Batch checkpoint #{ix}")
                                } else {
                                    format!("Telemetry event #{ix}")
                                })
                                .role(if is_group {
                                    TypeRole::TitleSmall
                                } else {
                                    TypeRole::BodyMedium
                                }),
                            )
                            .child(
                                Text::new(format!("{:.1} ms", (ix % 17) as f32 * 1.4 + 2.0))
                                    .role(TypeRole::LabelSmall)
                                    .color(ColorRole::OnSurfaceVariant),
                            )
                    })
                    .collect()
            },
        )
        .accessible_label("Telemetry stream")
        .variant(VirtualListVariant::Contained)
        .track_scroll(&self.vlist_scroll)
        .h(px(220.));

        let hlist = VirtualList::horizontal(
            cx.entity(),
            "cards-hlist",
            self.hlist_sizes.clone(),
            move |_, range, _, _| {
                range
                    .map(|ix| {
                        let width = if ix % 3 == 0 { px(160.) } else { px(124.) };
                        div()
                            .w(width)
                            .h(px(64.))
                            .rounded(px(12.))
                            .bg(surface_high)
                            .border_1()
                            .border_color(outline)
                            .px_3()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Icon::new(IconName::Description).size(px(18.)))
                            .child(Text::new(format!("Card #{ix}")).role(TypeRole::LabelLarge))
                    })
                    .collect()
            },
        )
        .accessible_label("Horizontal virtual strip")
        .variant(VirtualListVariant::Contained)
        .content_padding(LogicalEdges::all(8.0))
        .item_gap(px(8.))
        .track_scroll(&self.hlist_scroll)
        .h(px(84.));

        stack()
            .child(
                section("Data table")
                    .description("Typed Material 3 columns with sorting indicators, density-aware heights, row selection, disabled rows, and roving keyboard focus.")
                    .child(table),
            )
            .child(
                section("Tree")
                    .description("Virtualized hierarchical tree with logical indentation, RTL-mirrored disclosure chevrons, disabled nodes, WAI-ARIA keyboard navigation, and DisplayScrollbar.")
                    .child(tree_controls)
                    .child(tree),
            )
            .child(
                section("Virtual list")
                    .description("Variable-size vertical and horizontal virtualized lists paired with Material DisplayScrollbar and keyboard scrolling.")
                    .child(vlist_controls)
                    .child(vlist)
                    .child(hlist),
            )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests;

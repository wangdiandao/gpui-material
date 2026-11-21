use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Canvas, Div, Edges, ElementId, InteractiveElement,
    IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window, canvas, div,
    prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::current_material_theme;
use gpui_material_theme::Direction;

use crate::{ScaffoldInsets, Sheet};

/// Positions a scaffold's floating action within the usable content width.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScaffoldFabPosition {
    Start,
    Center,
    #[default]
    End,
    /// At the logical end of the scaffold, over the bottom bar.
    EndOverlay,
}

/// Resolved body layout before the content builder applies its padding.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaffoldContentLayout {
    /// Window-local bounds after the scaffold has allocated its bars and rail.
    pub bounds: Bounds<Pixels>,
    /// Space already allocated to the bars and rail.
    pub occupied: ScaffoldInsets,
    /// Remaining insets for the body or a child inside its scroll area.
    pub padding: ScaffoldInsets,
    pub direction: Direction,
}

type ContentBuilder = Box<dyn FnOnce(ScaffoldContentLayout, &mut Window, &mut App) -> AnyElement>;

enum Content {
    Element(AnyElement),
    Builder(ContentBuilder),
}

/// Arranges the top app bar, leading rail, content and bottom bar. The floating
/// action and snackbar are anchored within the content region above bottom
/// navigation. Give the scaffold a bounded size and give its content a scroll
/// viewport if needed.
#[derive(IntoElement)]
pub struct Scaffold {
    id: ElementId,
    content: Content,
    app_bar: Option<AnyElement>,
    rail: Option<AnyElement>,
    bottom_bar: Option<AnyElement>,
    bottom_sheet: Option<Sheet>,
    fab: Option<AnyElement>,
    snackbar: Option<AnyElement>,
    fab_position: ScaffoldFabPosition,
    content_insets: ScaffoldInsets,
}

impl Scaffold {
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self::with_content(id.into(), Content::Element(content.into_any_element()))
    }

    /// Builds the body with its current layout. Apply `layout.padding` once;
    /// the scaffold has already allocated `layout.occupied`.
    pub fn with_layout<E: IntoElement>(
        id: impl Into<ElementId>,
        content: impl FnOnce(ScaffoldContentLayout, &mut Window, &mut App) -> E + 'static,
    ) -> Self {
        Self::with_content(
            id.into(),
            Content::Builder(Box::new(move |layout, window, cx| {
                content(layout, window, cx).into_any_element()
            })),
        )
    }

    fn with_content(id: ElementId, content: Content) -> Self {
        Self {
            id,
            content,
            app_bar: None,
            rail: None,
            bottom_bar: None,
            bottom_sheet: None,
            fab: None,
            snackbar: None,
            fab_position: ScaffoldFabPosition::default(),
            content_insets: ScaffoldInsets::default(),
        }
    }

    pub fn app_bar(mut self, element: impl IntoElement) -> Self {
        self.app_bar = Some(element.into_any_element());
        self
    }

    pub fn rail(mut self, element: impl IntoElement) -> Self {
        self.rail = Some(element.into_any_element());
        self
    }

    pub fn bottom_bar(mut self, element: impl IntoElement) -> Self {
        self.bottom_bar = Some(element.into_any_element());
        self
    }

    /// Places a Standard bottom sheet beneath the measured content column.
    pub fn bottom_sheet(mut self, sheet: Sheet) -> Self {
        self.bottom_sheet = Some(sheet);
        self
    }

    pub fn fab(mut self, element: impl IntoElement) -> Self {
        self.fab = Some(element.into_any_element());
        self
    }

    pub fn snackbar(mut self, element: impl IntoElement) -> Self {
        self.snackbar = Some(element.into_any_element());
        self
    }

    pub fn fab_position(mut self, position: ScaffoldFabPosition) -> Self {
        self.fab_position = position;
        self
    }

    /// Sets minimum distances from the scaffold's edges to usable content.
    /// Space occupied by bars and the rail counts toward those distances.
    pub fn content_insets(mut self, insets: ScaffoldInsets) -> Self {
        self.content_insets = insets.normalized();
        self
    }
}

impl RenderOnce for Scaffold {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let direction = current_material_theme(window, cx).layout.direction;
        let position = self.fab_position;
        let insets = self.content_insets;
        let (fab, snackbar, overlay) = match (position, self.fab) {
            (ScaffoldFabPosition::EndOverlay, Some(fab)) => {
                (None, None, Some((fab, self.snackbar)))
            }
            (_, fab) => (fab, self.snackbar, None),
        };
        let needs_layout = overlay.is_some()
            || insets != ScaffoldInsets::default()
            || matches!(self.content, Content::Builder(_));
        let geometry = Rc::new(ScaffoldGeometry::default());
        let body_geometry = geometry.clone();
        let body = div()
            .id("scaffold-content")
            .test_support()
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .min_h_0();
        let body =
            match self.content {
                Content::Element(content) if !needs_layout => body
                    .child(body_wrapper().child(content))
                    .children(standard_overlay(
                        fab,
                        snackbar,
                        position,
                        insets.resolve(direction),
                        direction,
                    )),
                content => body.child(
                    laid_out_slot(move |bounds, window, cx| {
                        let layout = body_geometry.resolve(bounds, insets, direction);
                        let padding = layout.padding.resolve(direction);
                        let content = match content {
                            Content::Element(content) => body_wrapper()
                                .pt(padding.top)
                                .pr(padding.right)
                                .pb(padding.bottom)
                                .pl(padding.left)
                                .child(content),
                            Content::Builder(build) => {
                                body_wrapper().child(build(layout, window, cx))
                            }
                        };
                        div().relative().flex().flex_col().child(content).children(
                            standard_overlay(fab, snackbar, position, padding, direction),
                        )
                    })
                    .size_full(),
                ),
            };
        let column = match self.bottom_sheet {
            Some(sheet) => div()
                .flex_1()
                .min_w_0()
                .min_h_0()
                .child(
                    laid_out_slot(move |bounds, _, _| {
                        div()
                            .flex()
                            .flex_col()
                            .child(body)
                            .child(div().flex_none().child(sheet.in_scaffold(bounds.size)))
                    })
                    .size_full(),
                )
                .into_any_element(),
            None => body.into_any_element(),
        };
        let root_geometry = geometry.clone();
        div()
            .id(self.id)
            .test_support()
            .size_full()
            .min_h_0()
            .min_w_0()
            .flex()
            .flex_col()
            .when(needs_layout, move |root| {
                root.relative().child(
                    canvas(
                        move |bounds, _, _| root_geometry.root.set(Some(bounds)),
                        |_, (), _, _| {},
                    )
                    .absolute()
                    .inset_0(),
                )
            })
            .children(self.app_bar.map(|bar| div().flex_none().child(bar)))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .when(direction == Direction::Rtl, Styled::flex_row_reverse)
                    .children(self.rail.map(|rail| div().flex_none().h_full().child(rail)))
                    .child(column),
            )
            .children(self.bottom_bar.map(|bar| div().flex_none().child(bar)))
            .children(overlay.map(move |(fab, snackbar)| {
                laid_out_slot(move |root, _, _| {
                    let layout = geometry
                        .layout
                        .get()
                        .unwrap_or_else(|| geometry.resolve(root, insets, direction));
                    end_overlay(fab, snackbar, root, layout, insets.bottom)
                })
                .absolute()
                .inset_0()
            }))
    }
}

fn body_wrapper() -> Div {
    div().flex_1().min_h_0().min_w_0().flex().flex_col()
}

#[derive(Default)]
struct ScaffoldGeometry {
    root: Cell<Option<Bounds<Pixels>>>,
    layout: Cell<Option<ScaffoldContentLayout>>,
}

impl ScaffoldGeometry {
    fn resolve(
        &self,
        bounds: Bounds<Pixels>,
        requested: ScaffoldInsets,
        direction: Direction,
    ) -> ScaffoldContentLayout {
        let root = self.root.get().unwrap_or(bounds);
        let occupied = ScaffoldInsets {
            top: (bounds.top() - root.top()).max(px(0.)),
            start: if direction == Direction::Rtl {
                (root.right() - bounds.right()).max(px(0.))
            } else {
                (bounds.left() - root.left()).max(px(0.))
            },
            bottom: (root.bottom() - bounds.bottom()).max(px(0.)),
            end: px(0.),
        };
        let top = (requested.top - occupied.top)
            .max(px(0.))
            .min(bounds.size.height);
        let start = (requested.start - occupied.start)
            .max(px(0.))
            .min(bounds.size.width);
        let padding = ScaffoldInsets {
            top,
            start,
            bottom: (requested.bottom - occupied.bottom)
                .max(px(0.))
                .min(bounds.size.height - top),
            end: requested.end.min(bounds.size.width - start),
        };
        let layout = ScaffoldContentLayout {
            bounds,
            occupied,
            padding,
            direction,
        };
        self.layout.set(Some(layout));
        layout
    }
}

// Resolve the body against the outer flex layout before building its contents.
// The resulting child is laid out, prepainted, and painted exactly once.
fn laid_out_slot(
    build: impl FnOnce(Bounds<Pixels>, &mut Window, &mut App) -> Div + 'static,
) -> Canvas<AnyElement> {
    canvas(
        move |bounds, window, cx| {
            let mut child = build(bounds, window, cx)
                .w(bounds.size.width)
                .h(bounds.size.height)
                .into_any_element();
            child.layout_as_root(bounds.size.map(AvailableSpace::Definite), window, cx);
            child.prepaint_at(bounds.origin, window, cx);
            child
        },
        |_, mut child, window, cx| child.paint(window, cx),
    )
}

fn fab_row(position: ScaffoldFabPosition, direction: Direction) -> Div {
    let row = div().flex();
    match (position, direction) {
        (ScaffoldFabPosition::Center, _) => row.justify_center(),
        (ScaffoldFabPosition::Start, Direction::Ltr)
        | (ScaffoldFabPosition::End | ScaffoldFabPosition::EndOverlay, Direction::Rtl) => {
            row.justify_start()
        }
        _ => row.justify_end(),
    }
}

fn standard_overlay(
    fab: Option<AnyElement>,
    snackbar: Option<AnyElement>,
    position: ScaffoldFabPosition,
    padding: Edges<Pixels>,
    direction: Direction,
) -> Option<Div> {
    (fab.is_some() || snackbar.is_some()).then(|| {
        div()
            .absolute()
            .left(padding.left)
            .right(padding.right)
            .bottom(padding.bottom)
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(8.))
            .p(px(16.))
            .children(fab.map(|fab| fab_row(position, direction).child(fab)))
            .children(snackbar)
    })
}

fn end_overlay(
    fab: AnyElement,
    snackbar: Option<AnyElement>,
    root: Bounds<Pixels>,
    layout: ScaffoldContentLayout,
    bottom_inset: Pixels,
) -> Div {
    let padding = layout.padding.resolve(layout.direction);
    let bottom = bottom_inset.min(root.size.height);
    let fab = fab_row(ScaffoldFabPosition::EndOverlay, layout.direction)
        .items_end()
        .pb(px(16.))
        .when(snackbar.is_some(), |fab| {
            // Reserve the taller of the bottom bar and the FAB band. The
            // button stays at the bottom of the band even with a tall bar.
            fab.min_h((layout.occupied.bottom + px(16.) - bottom).max(px(0.)))
                .pt(px(8.))
        })
        .child(fab);
    div().relative().child(
        div()
            .absolute()
            .left(layout.bounds.left() - root.left() + padding.left)
            .right(root.right() - layout.bounds.right() + padding.right)
            .bottom(bottom)
            .min_w_0()
            .flex()
            .flex_col()
            .px(px(16.))
            .children(snackbar)
            .child(fab),
    )
}

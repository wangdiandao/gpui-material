use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, ClickEvent, CursorStyle, ElementId, FocusHandle, Hsla,
    InteractiveElement, IntoElement, KeyDownEvent, ParentElement, Pixels, RenderOnce, SharedString,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, canvas, div,
    prelude::FluentBuilder, px, radians, transparent_black,
};
use gpui_base::{
    StyledExt as _, Table, TableBody, TableCaption, TableCell, TableHead, TableHeader, TableRow,
};
use gpui_material_components::{Icon, IconName, Text};
use gpui_material_foundation::{
    FocusRingPlacement, InteractionFeedback, InteractionState, ThemeScope, color,
    control_feedback::{FeedbackState, activate_feedback, feedback_target, ripple_state},
    current_material_theme, move_roving_focus, resolve_edges,
};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Density, Direction, LogicalEdges, TypeRole,
};

const DEFAULT_HEADER_HEIGHT: f32 = 48.0;
const DEFAULT_ROW_HEIGHT: f32 = 52.0;
const MIN_ROW_HEIGHT: f32 = 36.0;
const DEFAULT_CELL_PADDING: LogicalEdges = LogicalEdges::symmetric(16.0, 10.0);

type SortHandler = Rc<dyn Fn(&SharedString, SortDirection, &mut Window, &mut App)>;
type RowSelectHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;
type RowActivateHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// Horizontal alignment of a [`DataTable`] column or cell in logical reading order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColumnAlignment {
    /// Aligns content to the logical start edge (left in LTR, right in RTL).
    #[default]
    Start,
    /// Centers content horizontally.
    Center,
    /// Aligns content to the logical end edge (right in LTR, left in RTL).
    End,
}

/// Active sort order of a sortable [`DataTableColumn`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending order (for example, A to Z or lowest to highest).
    Ascending,
    /// Descending order (for example, Z to A or highest to lowest).
    Descending,
}

impl SortDirection {
    /// Returns the opposite sort direction.
    pub const fn toggled(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }

    /// Computes the next sort direction when activating a column header.
    pub const fn next_from(current: Option<Self>) -> Self {
        match current {
            None | Some(Self::Descending) => Self::Ascending,
            Some(Self::Ascending) => Self::Descending,
        }
    }
}

/// Optional per-instance color overrides for [`DataTable`].
///
/// Any `None` field falls back to the active window [`gpui_material_theme::MaterialTheme`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DataTableColors {
    /// Outer table container fill (defaults to `SurfaceContainerLow`).
    pub container: Option<Hsla>,
    /// Header strip fill (defaults to `SurfaceContainer`).
    pub header_container: Option<Hsla>,
    /// Header label color (defaults to `OnSurfaceVariant`).
    pub header_content: Option<Hsla>,
    /// Default body row fill (defaults to transparent over `container`).
    pub row_container: Option<Hsla>,
    /// Alternate row fill when [`DataTable::striped`] is enabled (defaults to `SurfaceContainerLowest`).
    pub striped_row_container: Option<Hsla>,
    /// Default body cell text color (defaults to `OnSurface`).
    pub row_content: Option<Hsla>,
    /// Selected row fill (defaults to `SecondaryContainer`).
    pub selected_row_container: Option<Hsla>,
    /// Selected row text and state-layer ink color (defaults to `OnSecondaryContainer`).
    pub selected_row_content: Option<Hsla>,
    /// Disabled row or header content color (defaults to `OnSurface` at 38% opacity).
    pub disabled_content: Option<Hsla>,
    /// Row divider color (defaults to `OutlineVariant`).
    pub divider: Option<Hsla>,
    /// Outer container border color (defaults to `OutlineVariant`).
    pub border: Option<Hsla>,
    /// Caption text color (defaults to `OnSurfaceVariant`).
    pub caption: Option<Hsla>,
    /// Active sort indicator icon tint (defaults to `Primary`).
    pub sort_indicator: Option<Hsla>,
}

/// Column schema for [`DataTable`].
#[derive(Clone, Debug, PartialEq)]
pub struct DataTableColumn {
    id: SharedString,
    label: SharedString,
    alignment: ColumnAlignment,
    width: Option<Pixels>,
    min_width: Option<Pixels>,
    weight: f32,
    sortable: bool,
    sort_direction: Option<SortDirection>,
    disabled: bool,
}

impl DataTableColumn {
    /// Creates a column with a stable semantic `id` and visible header `label`.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            alignment: ColumnAlignment::Start,
            width: None,
            min_width: None,
            weight: 1.0,
            sortable: false,
            sort_direction: None,
            disabled: false,
        }
    }

    /// Sets logical horizontal alignment for this column's header and cells.
    pub fn alignment(mut self, alignment: ColumnAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets a fixed logical width for this column. Non-finite or non-positive values are ignored.
    pub fn width(mut self, width: Pixels) -> Self {
        if f32::from(width).is_finite() && width > px(0.) {
            self.width = Some(width);
        }
        self
    }

    /// Sets a minimum logical width for this column. Non-finite or negative values are ignored.
    pub fn min_width(mut self, width: Pixels) -> Self {
        if f32::from(width).is_finite() && width >= px(0.) {
            self.min_width = Some(width);
        }
        self
    }

    /// Sets the relative flex weight when no fixed width is specified (defaults to `1.0`).
    pub fn weight(mut self, weight: f32) -> Self {
        if weight.is_finite() && weight > 0.0 {
            self.weight = weight;
        }
        self
    }

    /// Marks this column header as sortable.
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// Sets the current controlled [`SortDirection`] indicator for this column.
    ///
    /// Passing `Some(_)` also marks the column sortable.
    pub fn sort_direction(mut self, direction: Option<SortDirection>) -> Self {
        if direction.is_some() {
            self.sortable = true;
        }
        self.sort_direction = direction;
        self
    }

    /// Disables header sorting interaction for this column when `true`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns the column identifier.
    pub fn id(&self) -> &SharedString {
        &self.id
    }

    /// Returns the column header label.
    pub fn label(&self) -> &SharedString {
        &self.label
    }
}

enum CellContent {
    Text(SharedString),
    Custom(AnyElement),
}

/// Content for a single cell inside a [`DataTableRow`].
pub struct DataTableCell {
    content: CellContent,
    alignment: Option<ColumnAlignment>,
    muted: bool,
}

impl DataTableCell {
    /// Creates a plain-text table cell rendered with Material `BodyMedium` typography.
    pub fn text(value: impl Into<SharedString>) -> Self {
        Self {
            content: CellContent::Text(value.into()),
            alignment: None,
            muted: false,
        }
    }

    /// Creates a table cell containing an arbitrary GPUI element.
    pub fn custom(element: impl IntoElement) -> Self {
        Self {
            content: CellContent::Custom(element.into_any_element()),
            alignment: None,
            muted: false,
        }
    }

    /// Overrides the column's horizontal alignment for this cell.
    pub fn alignment(mut self, alignment: ColumnAlignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    /// Renders text cells with muted `OnSurfaceVariant` emphasis when enabled and unselected.
    pub fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }
}

impl From<&'static str> for DataTableCell {
    fn from(value: &'static str) -> Self {
        Self::text(value)
    }
}

impl From<String> for DataTableCell {
    fn from(value: String) -> Self {
        Self::text(value)
    }
}

impl From<SharedString> for DataTableCell {
    fn from(value: SharedString) -> Self {
        Self::text(value)
    }
}

/// One data row inside a [`DataTable`].
pub struct DataTableRow {
    id: SharedString,
    cells: Vec<DataTableCell>,
    selected: bool,
    disabled: bool,
    accessible_label: Option<SharedString>,
    on_activate: Option<RowActivateHandler>,
    focus: Option<FocusHandle>,
}

impl DataTableRow {
    /// Creates a row with a stable `id` and cell values corresponding to the table columns.
    pub fn new(
        id: impl Into<SharedString>,
        cells: impl IntoIterator<Item = impl Into<DataTableCell>>,
    ) -> Self {
        Self {
            id: id.into(),
            cells: cells.into_iter().map(Into::into).collect(),
            selected: false,
            disabled: false,
            accessible_label: None,
            on_activate: None,
            focus: None,
        }
    }

    /// Marks the row as selected using `SecondaryContainer` / `OnSecondaryContainer` styling.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Disables row selection, focus, and activation when `true`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets an explicit accessible name for the row.
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    /// Registers a row-specific activation callback invoked on pointer click or `Enter`/`Space`.
    pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }

    /// Attaches a caller-owned [`FocusHandle`] to this row when interactive.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }
}

/// Material 3 Expressive data table built over `gpui-base`'s semantic [`Table`] primitives.
///
/// The application owns row data, selection, and sorting state. Column headers and interactive
/// rows follow `theme.layout.density`, mirror cleanly in RTL, and provide Material state layers,
/// bounded press ripples (static fallback under reduced motion), and keyboard navigation.
#[derive(IntoElement)]
pub struct DataTable {
    id: ElementId,
    columns: Vec<DataTableColumn>,
    rows: Vec<DataTableRow>,
    caption: Option<SharedString>,
    accessible_label: Option<SharedString>,
    total_row_count: Option<usize>,
    total_column_count: Option<usize>,
    cell_padding: LogicalEdges,
    header_height: Option<Pixels>,
    row_height: Option<Pixels>,
    shape: Option<CornerShape>,
    colors: DataTableColors,
    show_row_dividers: bool,
    striped: bool,
    on_sort: Option<SortHandler>,
    on_row_select: Option<RowSelectHandler>,
    style: StyleRefinement,
}

impl DataTable {
    /// Creates an empty Material 3 data table with the given stable element ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            columns: Vec::new(),
            rows: Vec::new(),
            caption: None,
            accessible_label: None,
            total_row_count: None,
            total_column_count: None,
            cell_padding: DEFAULT_CELL_PADDING,
            header_height: None,
            row_height: None,
            shape: None,
            colors: DataTableColors::default(),
            show_row_dividers: true,
            striped: false,
            on_sort: None,
            on_row_select: None,
            style: StyleRefinement::default(),
        }
    }

    /// Appends a single column definition.
    pub fn column(mut self, column: DataTableColumn) -> Self {
        self.columns.push(column);
        self
    }

    /// Appends multiple column definitions.
    pub fn columns(mut self, columns: impl IntoIterator<Item = DataTableColumn>) -> Self {
        self.columns.extend(columns);
        self
    }

    /// Appends a single data row.
    pub fn row(mut self, row: DataTableRow) -> Self {
        self.rows.push(row);
        self
    }

    /// Appends multiple data rows.
    pub fn rows(mut self, rows: impl IntoIterator<Item = DataTableRow>) -> Self {
        self.rows.extend(rows);
        self
    }

    /// Adds a descriptive caption strip at the top of the table.
    pub fn caption(mut self, caption: impl Into<SharedString>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Sets the accessible name announced for the `Role::Table` root.
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    /// Overrides the total ARIA row count (useful when rendering a paginated or virtual slice).
    pub fn total_row_count(mut self, count: usize) -> Self {
        self.total_row_count = Some(count);
        self
    }

    /// Overrides the total ARIA column count.
    pub fn total_column_count(mut self, count: usize) -> Self {
        self.total_column_count = Some(count);
        self
    }

    /// Sets direction-aware cell padding in logical units (defaults to 16 dp horizontal, 10 dp vertical).
    pub fn cell_padding(mut self, padding: LogicalEdges) -> Self {
        self.cell_padding = padding.sanitize();
        self
    }

    /// Overrides the density-aware header row minimum height. Non-finite or non-positive values are ignored.
    pub fn header_height(mut self, height: Pixels) -> Self {
        if f32::from(height).is_finite() && height > px(0.) {
            self.header_height = Some(height);
        }
        self
    }

    /// Overrides the density-aware body row minimum height. Non-finite or non-positive values are ignored.
    pub fn row_height(mut self, height: Pixels) -> Self {
        if f32::from(height).is_finite() && height > px(0.) {
            self.row_height = Some(height);
        }
        self
    }

    /// Sets the container corner shape (defaults to `CornerRole::Medium`).
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Applies per-instance color overrides.
    pub fn colors(mut self, colors: DataTableColors) -> Self {
        self.colors = colors;
        self
    }

    /// Controls whether 1 dp dividers are drawn between body rows (defaults to `true`).
    pub fn show_row_dividers(mut self, show: bool) -> Self {
        self.show_row_dividers = show;
        self
    }

    /// Enables subtle alternating row container fills (defaults to `false`).
    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    /// Invoked when an enabled sortable column header is clicked or activated with `Enter`/`Space`.
    ///
    /// Receives the column ID and the requested next [`SortDirection`].
    pub fn on_sort(
        mut self,
        handler: impl Fn(&SharedString, SortDirection, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_sort = Some(Rc::new(handler));
        self
    }

    /// Invoked when an enabled body row is clicked or activated with `Enter`/`Space`.
    pub fn on_row_select(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_row_select = Some(Rc::new(handler));
        self
    }
}

impl Styled for DataTable {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

pub(crate) fn table_header_height(density: Density, override_height: Option<Pixels>) -> Pixels {
    override_height.unwrap_or_else(|| {
        px(density
            .visual_size(DEFAULT_HEADER_HEIGHT)
            .max(MIN_ROW_HEIGHT))
    })
}

pub(crate) fn table_row_height(density: Density, override_height: Option<Pixels>) -> Pixels {
    override_height
        .unwrap_or_else(|| px(density.visual_size(DEFAULT_ROW_HEIGHT).max(MIN_ROW_HEIGHT)))
}

struct ResolvedTableColors {
    container: Hsla,
    header_container: Hsla,
    header_content: Hsla,
    row_container: Hsla,
    striped_row_container: Hsla,
    row_content: Hsla,
    muted_content: Hsla,
    selected_row_container: Hsla,
    selected_row_content: Hsla,
    disabled_content: Hsla,
    divider: Hsla,
    border: Hsla,
    caption: Hsla,
    sort_indicator: Hsla,
    focus_ring: Hsla,
}

impl DataTableColors {
    fn resolve(self, scheme: &gpui_material_theme::ColorScheme) -> ResolvedTableColors {
        let tone = |role| color(scheme.color(role));
        ResolvedTableColors {
            container: self
                .container
                .unwrap_or_else(|| tone(ColorRole::SurfaceContainerLow)),
            header_container: self
                .header_container
                .unwrap_or_else(|| tone(ColorRole::SurfaceContainer)),
            header_content: self
                .header_content
                .unwrap_or_else(|| tone(ColorRole::OnSurfaceVariant)),
            row_container: self.row_container.unwrap_or_else(transparent_black),
            striped_row_container: self
                .striped_row_container
                .unwrap_or_else(|| tone(ColorRole::SurfaceContainerLowest)),
            row_content: self
                .row_content
                .unwrap_or_else(|| tone(ColorRole::OnSurface)),
            muted_content: tone(ColorRole::OnSurfaceVariant),
            selected_row_container: self
                .selected_row_container
                .unwrap_or_else(|| tone(ColorRole::SecondaryContainer)),
            selected_row_content: self
                .selected_row_content
                .unwrap_or_else(|| tone(ColorRole::OnSecondaryContainer)),
            disabled_content: self
                .disabled_content
                .unwrap_or_else(|| tone(ColorRole::OnSurface).opacity(0.38)),
            divider: self
                .divider
                .unwrap_or_else(|| tone(ColorRole::OutlineVariant)),
            border: self
                .border
                .unwrap_or_else(|| tone(ColorRole::OutlineVariant)),
            caption: self
                .caption
                .unwrap_or_else(|| tone(ColorRole::OnSurfaceVariant)),
            sort_indicator: self
                .sort_indicator
                .unwrap_or_else(|| tone(ColorRole::Primary)),
            focus_ring: tone(ColorRole::Secondary),
        }
    }
}

fn apply_column_sizing<E: Styled>(el: E, column: &DataTableColumn) -> E {
    let el = if let Some(width) = column.width {
        el.w(width).flex_none()
    } else {
        el.flex_1().min_w_0()
    };
    let el = match column.min_width {
        Some(min_width) => el.min_w(min_width),
        None => el,
    };
    if column.width.is_none() && (column.weight - 1.0).abs() > f32::EPSILON {
        let mut el = el;
        el.style().flex_grow = Some(column.weight);
        el
    } else {
        el
    }
}

fn apply_cell_alignment<E: Styled>(el: E, alignment: ColumnAlignment) -> E {
    // Because row containers use `flex_row_reverse` in RTL, `justify_start` aligns to the
    // logical start edge (left in LTR, right in RTL) and `justify_end` to the logical end.
    match alignment {
        ColumnAlignment::Start => el.justify_start(),
        ColumnAlignment::Center => el.justify_center(),
        ColumnAlignment::End => el.justify_end(),
    }
}

fn render_feedback_overlay(
    feedback: Rc<RefCell<FeedbackState>>,
    interaction: InteractionState,
    ink: Hsla,
    focus_ring: Hsla,
    direction: Direction,
    reduced_motion: bool,
) -> AnyElement {
    let corners = CornerRole::None.shape();
    let prepaint_feedback = feedback.clone();
    canvas(
        move |bounds, window, _| {
            prepaint_feedback
                .borrow_mut()
                .set_rounded_shape(bounds, corners, direction, window);
        },
        move |_, _, window, cx| {
            let state = feedback.borrow();
            let Some(shape) = state.shape() else {
                return;
            };
            let now = cx.background_executor().now();
            InteractionFeedback::new(interaction, ink, focus_ring)
                .focus_ring_placement(FocusRingPlacement::Inset)
                .paint(shape, state.ripple(), now, reduced_motion, window);
        },
    )
    .absolute()
    .inset_0()
    .into_any_element()
}

fn row_default_label(row: &DataTableRow) -> SharedString {
    if let Some(label) = &row.accessible_label {
        return label.clone();
    }
    let text_parts: Vec<&str> = row
        .cells
        .iter()
        .filter_map(|cell| match &cell.content {
            CellContent::Text(text) if !text.is_empty() => Some(text.as_ref()),
            _ => None,
        })
        .collect();
    if text_parts.is_empty() {
        row.id.clone()
    } else {
        text_parts.join(", ").into()
    }
}

impl RenderOnce for DataTable {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        window.with_element_namespace((self.id.clone(), "material-data-table"), |window| {
            let theme = current_material_theme(window, cx).clone();
            let direction = theme.layout.direction;
            let rtl = direction == Direction::Rtl;
            let density = theme.layout.density;
            let reduced_motion = cx.reduce_motion();
            let colors = self.colors.resolve(&theme.colors);
            let padding = resolve_edges(self.cell_padding, direction);
            let header_height = table_header_height(density, self.header_height);
            let row_height = table_row_height(density, self.row_height);
            let shape = self.shape.unwrap_or_else(|| CornerRole::Medium.shape());
            let radii = shape.radii(100.0, 100.0, direction);

            let total_rows = self
                .total_row_count
                .unwrap_or_else(|| self.rows.len() + usize::from(!self.columns.is_empty()));
            let total_columns = self.total_column_count.unwrap_or(self.columns.len());

            let on_sort = self.on_sort;
            let on_row_select = self.on_row_select;

            // Prepare roving focus handles for interactive body rows.
            let row_focus: Vec<FocusHandle> = self
                .rows
                .iter()
                .map(|row| {
                    row.focus.clone().unwrap_or_else(|| {
                        window
                            .use_keyed_state(
                                (ElementId::from(row.id.clone()), "table-row-focus"),
                                cx,
                                |_, cx| cx.focus_handle(),
                            )
                            .read(cx)
                            .clone()
                    })
                })
                .collect();
            let row_enabled: Vec<bool> = self
                .rows
                .iter()
                .map(|row| !row.disabled && (on_row_select.is_some() || row.on_activate.is_some()))
                .collect();
            let row_tab_stop = row_focus
                .iter()
                .enumerate()
                .position(|(ix, handle)| row_enabled[ix] && handle.is_focused(window))
                .or_else(|| {
                    self.rows
                        .iter()
                        .enumerate()
                        .position(|(ix, row)| row_enabled[ix] && row.selected)
                })
                .or_else(|| row_enabled.iter().position(|enabled| *enabled));

            let mut header_cells = Vec::with_capacity(self.columns.len());
            for (col_ix, column) in self.columns.iter().enumerate() {
                let head_id = ElementId::from((self.id.clone(), format!("col-{}", column.id)));
                let interactive = column.sortable && !column.disabled && on_sort.is_some();
                let head_focus = interactive.then(|| {
                    window
                        .use_keyed_state(
                            (ElementId::from(column.id.clone()), "table-head-focus"),
                            cx,
                            |_, cx| cx.focus_handle(),
                        )
                        .read(cx)
                        .clone()
                });
                let head_feedback = ripple_state(&head_id, interactive, window, cx);
                let head_interaction = head_focus.as_ref().map_or_else(
                    || {
                        let mut state = InteractionState::default();
                        state.disabled = column.disabled;
                        state
                    },
                    |focus| head_feedback.borrow().interaction(true, focus, window),
                );
                let head_text_color = if column.disabled {
                    colors.disabled_content
                } else {
                    colors.header_content
                };

                let next_dir = SortDirection::next_from(column.sort_direction);
                let col_id = column.id.clone();
                let trigger_sort = on_sort.clone().filter(|_| interactive).map(|handler| {
                    let action: RowActivateHandler = Rc::new(ThemeScope::bind_action(
                        move |window, cx| handler(&col_id, next_dir, window, cx),
                        window,
                        cx,
                    ));
                    action
                });

                let sort_icon = column.sort_direction.map(|dir| {
                    let mut icon = Icon::new(IconName::ArrowDropDown)
                        .size(px(18.))
                        .color_override(if column.disabled {
                            colors.disabled_content
                        } else {
                            colors.sort_indicator
                        });
                    if dir == SortDirection::Ascending {
                        icon = icon.rotate(radians(PI));
                    }
                    icon
                });

                let label_text = Text::new(column.label.clone())
                    .role(TypeRole::LabelLarge)
                    .emphasized(column.sort_direction.is_some())
                    .color_override(head_text_color)
                    .max_lines(1);

                let head = TableHead::new(head_id, col_ix + 1)
                    .relative()
                    .flex()
                    .when(rtl, Styled::flex_row_reverse)
                    .items_center()
                    .gap_1()
                    .min_h(header_height)
                    .pt(padding.top)
                    .pb(padding.bottom)
                    .pl(padding.left)
                    .pr(padding.right)
                    .overflow_hidden()
                    .aria_label(column.label.clone())
                    .when(interactive, |el| el.cursor(CursorStyle::PointingHand));
                let head = apply_column_sizing(head, column);
                let head = apply_cell_alignment(head, column.alignment);

                let head = if let (Some(focus), Some(trigger)) = (head_focus, trigger_sort) {
                    let click_feedback = head_feedback.clone();
                    let click_focus = focus.clone();
                    let click_trigger = trigger.clone();
                    let a11y_trigger = trigger;
                    let head = feedback_target(
                        head.track_focus(&focus)
                            .tab_stop(true)
                            .on_click(move |event: &ClickEvent, window, cx| {
                                if click_feedback.borrow().blocks_activation(event) {
                                    return;
                                }
                                click_focus.focus(window, cx);
                                activate_feedback(event, &click_feedback, window, cx);
                                cx.stop_propagation();
                                click_trigger(window, cx);
                            })
                            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                a11y_trigger(window, cx);
                            }),
                        head_feedback.clone(),
                        true,
                        Some(focus),
                    );
                    head.child(render_feedback_overlay(
                        head_feedback,
                        head_interaction,
                        head_text_color,
                        colors.focus_ring,
                        direction,
                        reduced_motion,
                    ))
                } else {
                    head
                };

                header_cells.push(head.child(label_text).children(sort_icon));
            }

            let row_count = self.rows.len();
            let mut body_rows = Vec::with_capacity(row_count);
            for (row_ix, row) in self.rows.into_iter().enumerate() {
                let row_id_str = row.id.clone();
                let row_elem_id = ElementId::from((self.id.clone(), format!("row-{}", row.id)));
                let row_label = row_default_label(&row);
                let is_selected = row.selected;
                let is_disabled = row.disabled;
                let interactive = row_enabled[row_ix];
                let focus = row_focus[row_ix].clone();
                let is_tab_stop = Some(row_ix) == row_tab_stop;

                let row_feedback = ripple_state(&row_elem_id, interactive, window, cx);
                let row_interaction = if interactive {
                    row_feedback.borrow().interaction(true, &focus, window)
                } else {
                    let mut state = InteractionState::default();
                    state.disabled = is_disabled;
                    state
                };

                let row_bg = if is_selected {
                    colors.selected_row_container
                } else if self.striped && row_ix % 2 == 1 {
                    colors.striped_row_container
                } else {
                    colors.row_container
                };
                let row_fg = if is_disabled {
                    colors.disabled_content
                } else if is_selected {
                    colors.selected_row_content
                } else {
                    colors.row_content
                };

                let row_activate = row.on_activate;
                let trigger_row = interactive.then(|| {
                    let on_select = on_row_select.clone();
                    let row_id = row_id_str.clone();
                    let action: RowActivateHandler = Rc::new(ThemeScope::bind_action(
                        move |window, cx| {
                            if let Some(on_select) = &on_select {
                                on_select(&row_id, window, cx);
                            }
                            if let Some(activate) = &row_activate {
                                activate(window, cx);
                            }
                        },
                        window,
                        cx,
                    ));
                    action
                });

                let mut cell_elements = Vec::with_capacity(self.columns.len());
                let mut cells_iter = row.cells.into_iter();
                for (col_ix, column) in self.columns.iter().enumerate() {
                    let cell_id =
                        ElementId::from((row_elem_id.clone(), format!("cell-{}", column.id)));
                    let cell = cells_iter.next();
                    let alignment = cell
                        .as_ref()
                        .and_then(|c| c.alignment)
                        .unwrap_or(column.alignment);
                    let cell_content = match cell {
                        Some(cell) => match cell.content {
                            CellContent::Text(text) => {
                                let text_color = if is_disabled {
                                    colors.disabled_content
                                } else if is_selected {
                                    colors.selected_row_content
                                } else if cell.muted {
                                    colors.muted_content
                                } else {
                                    colors.row_content
                                };
                                Text::new(text)
                                    .role(TypeRole::BodyMedium)
                                    .color_override(text_color)
                                    .max_lines(1)
                                    .into_any_element()
                            }
                            CellContent::Custom(el) => el,
                        },
                        None => div().into_any_element(),
                    };

                    let cell_el = TableCell::new(cell_id, col_ix + 1)
                        .flex()
                        .when(rtl, Styled::flex_row_reverse)
                        .items_center()
                        .min_h(row_height)
                        .pt(padding.top)
                        .pb(padding.bottom)
                        .pl(padding.left)
                        .pr(padding.right)
                        .overflow_hidden()
                        .child(cell_content);
                    let cell_el = apply_column_sizing(cell_el, column);
                    let cell_el = apply_cell_alignment(cell_el, alignment);
                    cell_elements.push(cell_el);
                }

                let is_last = row_ix + 1 == row_count;
                let row_el = TableRow::new(row_elem_id, row_ix + 2)
                    .relative()
                    .w_full()
                    .flex()
                    .when(rtl, Styled::flex_row_reverse)
                    .items_stretch()
                    .bg(row_bg)
                    .text_color(row_fg)
                    .aria_label(row_label)
                    .aria_selected(is_selected)
                    .when(self.show_row_dividers && !is_last, |el| {
                        el.border_b_1().border_color(colors.divider)
                    })
                    .when(interactive, |el| el.cursor(CursorStyle::PointingHand));

                let row_el = if let Some(trigger) = trigger_row {
                    let click_feedback = row_feedback.clone();
                    let click_focus = focus.clone();
                    let click_trigger = trigger.clone();
                    let a11y_trigger = trigger;
                    let row_el = feedback_target(
                        row_el
                            .track_focus(&focus)
                            .tab_stop(is_tab_stop)
                            .on_click(move |event: &ClickEvent, window, cx| {
                                if click_feedback.borrow().blocks_activation(event) {
                                    return;
                                }
                                click_focus.focus(window, cx);
                                activate_feedback(event, &click_feedback, window, cx);
                                cx.stop_propagation();
                                click_trigger(window, cx);
                            })
                            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                a11y_trigger(window, cx);
                            }),
                        row_feedback.clone(),
                        true,
                        Some(focus),
                    );
                    row_el.child(render_feedback_overlay(
                        row_feedback,
                        row_interaction,
                        row_fg,
                        colors.focus_ring,
                        direction,
                        reduced_motion,
                    ))
                } else {
                    row_el
                };

                body_rows.push(row_el.children(cell_elements));
            }

            let nav_focus = row_focus;
            let nav_enabled = row_enabled;

            let caption_el = self.caption.map(|caption| {
                TableCaption::new((self.id.clone(), "caption"))
                    .w_full()
                    .flex()
                    .when(rtl, Styled::flex_row_reverse)
                    .items_center()
                    .pt(padding.top)
                    .pb(padding.bottom)
                    .pl(padding.left)
                    .pr(padding.right)
                    .border_b_1()
                    .border_color(colors.divider)
                    .child(
                        Text::new(caption)
                            .role(TypeRole::LabelMedium)
                            .color_override(colors.caption)
                            .wrap(),
                    )
            });

            let header_el = (!self.columns.is_empty()).then(|| {
                TableHeader::new((self.id.clone(), "header"))
                    .w_full()
                    .bg(colors.header_container)
                    .border_b_1()
                    .border_color(colors.divider)
                    .child(
                        TableRow::new((self.id.clone(), "header-row"), 1)
                            .w_full()
                            .flex()
                            .when(rtl, Styled::flex_row_reverse)
                            .items_stretch()
                            .children(header_cells),
                    )
            });

            let body_el = TableBody::new((self.id.clone(), "body"))
                .w_full()
                .flex()
                .flex_col()
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if event.keystroke.modifiers.modified() {
                        return;
                    }
                    if !nav_focus.iter().any(|handle| handle.is_focused(window)) {
                        return;
                    }
                    let items: Vec<_> = nav_focus
                        .iter()
                        .zip(&nav_enabled)
                        .map(|(handle, enabled)| (handle, *enabled))
                        .collect();
                    if move_roving_focus(
                        &items,
                        &event.keystroke.key,
                        gpui::Axis::Vertical,
                        direction,
                        window,
                        cx,
                    )
                    .is_some()
                    {
                        cx.stop_propagation();
                        window.refresh();
                    }
                })
                .children(body_rows);

            Table::new(self.id)
                .row_count(total_rows)
                .column_count(total_columns)
                .when_some(self.accessible_label, |table, label| {
                    table.accessibility_label(label)
                })
                .w_full()
                .flex()
                .flex_col()
                .overflow_hidden()
                .bg(colors.container)
                .border_1()
                .border_color(colors.border)
                .rounded_tl(px(radii[0]))
                .rounded_tr(px(radii[1]))
                .rounded_br(px(radii[2]))
                .rounded_bl(px(radii[3]))
                .children(caption_el)
                .children(header_el)
                .child(body_el)
                .refine_style(&self.style)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_direction_cycles_and_toggles() {
        assert_eq!(SortDirection::next_from(None), SortDirection::Ascending);
        assert_eq!(
            SortDirection::next_from(Some(SortDirection::Ascending)),
            SortDirection::Descending
        );
        assert_eq!(
            SortDirection::next_from(Some(SortDirection::Descending)),
            SortDirection::Ascending
        );
        assert_eq!(
            SortDirection::Ascending.toggled(),
            SortDirection::Descending
        );
        assert_eq!(
            SortDirection::Descending.toggled(),
            SortDirection::Ascending
        );
    }

    #[test]
    fn density_scales_table_heights_and_respects_overrides() {
        let compact = table_row_height(Density::Compact, None);
        let standard = table_row_height(Density::Standard, None);
        let comfortable = table_row_height(Density::Comfortable, None);
        assert!(compact < standard);
        assert!(standard < comfortable);
        assert_eq!(table_row_height(Density::Compact, Some(px(64.))), px(64.));
        assert_eq!(
            table_header_height(Density::Standard, Some(px(40.))),
            px(40.)
        );
    }

    #[test]
    fn column_sanitizes_invalid_dimensions_and_sort_direction_implies_sortable() {
        let col = DataTableColumn::new("name", "Name")
            .width(px(-10.))
            .min_width(px(f32::NAN))
            .weight(-2.0)
            .sort_direction(Some(SortDirection::Ascending));
        assert_eq!(col.width, None);
        assert_eq!(col.min_width, None);
        assert_eq!(col.weight, 1.0);
        assert!(col.sortable);
        assert_eq!(col.sort_direction, Some(SortDirection::Ascending));
    }
}

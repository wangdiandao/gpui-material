use gpui::{
    AnyElement, App, ElementId, Entity, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, Role, ScrollHandle, SharedString, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{
    Surface, animate, current_material_strings, current_material_theme, resolve_edges,
};
use gpui_material_theme::{
    ColorRole, CornerRole, Direction, Elevation, LogicalEdges, MotionKind, MotionSpec, MotionSpeed,
};

use super::app_bar::{self, AppBarScrollBehavior, AppBarScrollState};

/// Optional pigment overrides for [`BottomAppBar`]. Omitted fields use the window theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BottomAppBarColors {
    /// Background color override for the bottom app bar surface.
    pub container: Option<Hsla>,
    /// Inherited foreground color override for bar actions and content.
    pub content: Option<Hsla>,
}

/// Material 3 Bottom App Bar (80 dp default height, `SurfaceContainer`, `Elevation::Level2`).
/// Mounts seamlessly in [`crate::Scaffold::bottom_bar`] or standalone layouts.
#[derive(IntoElement)]
pub struct BottomAppBar {
    id: ElementId,
    actions: Vec<AnyElement>,
    fab: Option<AnyElement>,
    height: Option<Pixels>,
    colors: BottomAppBarColors,
    padding: LogicalEdges,
    elevation: Elevation,
    scroll: Option<ScrollHandle>,
    scroll_behavior: Option<(AppBarScrollBehavior, Entity<AppBarScrollState>)>,
    accessible_label: Option<SharedString>,
}

impl BottomAppBar {
    /// Default Material 3 bottom app bar height (80 dp).
    pub const DEFAULT_HEIGHT: Pixels = px(80.);

    /// Default Material 3 bottom app bar padding (4 dp start, 16 dp end, 12 dp vertical).
    pub const DEFAULT_PADDING: LogicalEdges = LogicalEdges::new(12., 16., 12., 4.);

    /// Creates a bottom app bar with the given element ID and default Material 3 styling.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            actions: Vec::new(),
            fab: None,
            height: None,
            colors: BottomAppBarColors::default(),
            padding: Self::DEFAULT_PADDING,
            elevation: Elevation::Level2,
            scroll: None,
            scroll_behavior: None,
            accessible_label: None,
        }
    }

    /// Appends one leading action control (typically an `IconButton`).
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }

    /// Appends multiple leading action controls in logical start order.
    pub fn actions(mut self, actions: impl IntoIterator<Item = AnyElement>) -> Self {
        self.actions.extend(actions);
        self
    }

    /// Places a docked/embedded floating action button at the logical trailing end.
    pub fn fab(mut self, fab: impl IntoElement) -> Self {
        self.fab = Some(fab.into_any_element());
        self
    }

    /// Sets the bar height in logical pixels. Defaults to 80 dp scaled by theme density.
    pub fn height(mut self, height: Pixels) -> Self {
        let value = f32::from(height);
        if value.is_finite() && value > 0. {
            self.height = Some(height);
        }
        self
    }

    /// Overrides container and inherited content colors independently.
    pub fn colors(mut self, colors: BottomAppBarColors) -> Self {
        self.colors = colors;
        self
    }

    /// Overrides logical inner insets.
    pub fn padding(mut self, padding: LogicalEdges) -> Self {
        self.padding = padding.sanitize();
        self
    }

    /// Sets the surface tonal and shadow elevation. Defaults to `Elevation::Level2`.
    pub fn elevation(mut self, elevation: Elevation) -> Self {
        self.elevation = elevation;
        self
    }

    /// Links a GPUI content `ScrollHandle` for scroll-driven hiding when `scroll_behavior` is set.
    pub fn track_scroll(mut self, scroll: &ScrollHandle) -> Self {
        self.scroll = Some(scroll.clone());
        self
    }

    /// Configures hide-on-scroll behavior backed by a retained `Entity<AppBarScrollState>`.
    pub fn scroll_behavior(
        mut self,
        behavior: AppBarScrollBehavior,
        state: &Entity<AppBarScrollState>,
    ) -> Self {
        self.scroll_behavior = Some((behavior, state.clone()));
        self
    }

    /// Sets the accessible name for the `Role::Toolbar` landmark. Defaults to
    /// [`gpui_material_theme::MaterialStrings::bottom_app_bar`] (`"Bottom app bar"`).
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }
}

impl RenderOnce for BottomAppBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let rtl = direction == Direction::Rtl;
        let full_height = self.height.map_or_else(
            || {
                theme
                    .layout
                    .density
                    .interactive_size(f32::from(Self::DEFAULT_HEIGHT))
            },
            f32::from,
        );
        let full_height_px = px(full_height);

        let collapsed_fraction = if let Some((behavior, state)) = &self.scroll_behavior {
            let handle_offset = self
                .scroll
                .as_ref()
                .map(|handle| app_bar::length(-handle.offset().y).unwrap_or(0.));
            let position = state.update(cx, |state, _| {
                state.prepare(handle_offset, full_height, Some(*behavior), true)
            });
            let motion = if position.settling && !cx.reduce_motion() {
                theme
                    .motion
                    .spec(MotionKind::Spatial, MotionSpeed::Default, false)
            } else {
                MotionSpec::Instant
            };
            let fraction = animate(
                (self.id.clone(), "bottom-app-bar-collapse"),
                position.target,
                motion,
                window,
                cx,
            )
            .clamp(0., 1.);
            if state.update(cx, |state, _| state.display(position, fraction)) {
                let weak = state.downgrade();
                window.defer(cx, move |_, cx| {
                    let _ = weak.update(cx, |_, cx| cx.notify());
                });
            }
            fraction
        } else {
            0.
        };

        let visible_height = px((full_height * (1. - collapsed_fraction)).max(0.));
        let scope = window
            .use_keyed_state((self.id.clone(), "bottom-app-bar-focus"), cx, |_, cx| {
                cx.focus_handle().tab_stop(false)
            })
            .read(cx)
            .clone();

        if visible_height <= px(0.5) {
            if scope.contains_focused(window, cx) {
                let focus = window.focused(cx);
                window.defer(cx, move |window, cx| {
                    if window
                        .focused(cx)
                        .is_none_or(|current| Some(current) == focus)
                    {
                        window.focus_next(cx);
                    }
                });
            }
            return div()
                .id(self.id)
                .test_support()
                .w_full()
                .h(px(0.))
                .overflow_hidden()
                .into_any_element();
        }

        let edges = resolve_edges(self.padding, direction);

        let actions_row = div()
            .id((self.id.clone(), "actions"))
            .test_support()
            .flex()
            .items_center()
            .gap(px(4.))
            .when(rtl, Styled::flex_row_reverse)
            .children(self.actions);

        let bar_row = div()
            .w_full()
            .h(full_height_px)
            .pl(edges.left)
            .pr(edges.right)
            .pt(edges.top)
            .pb(edges.bottom)
            .flex()
            .items_center()
            .justify_between()
            .when(rtl, Styled::flex_row_reverse)
            .child(actions_row)
            .children(self.fab.map(|fab| {
                div()
                    .id((self.id.clone(), "fab-slot"))
                    .test_support()
                    .flex_none()
                    .child(fab)
            }));

        let surface = Surface::new(ColorRole::SurfaceContainer)
            .shape(CornerRole::None.shape())
            .tonal_elevation(self.elevation)
            .shadow_elevation(self.elevation)
            .when_some(self.colors.container, Surface::container_color)
            .when_some(self.colors.content, Surface::content_color)
            .w_full()
            .h(visible_height)
            .overflow_hidden()
            .child(bar_row);

        let accessible_label = self.accessible_label.unwrap_or_else(|| {
            SharedString::from(current_material_strings(window, cx).bottom_app_bar.clone())
        });

        div()
            .id(self.id)
            .test_support()
            .role(Role::Toolbar)
            .aria_label(accessible_label)
            .track_focus(&scope)
            .w_full()
            .h(visible_height)
            .flex_none()
            .overflow_hidden()
            .child(surface)
            .into_any_element()
    }
}

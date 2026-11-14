use gpui::{
    AnyElement, App, ElementId, Entity, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_material_foundation::{animate, color, window_material_theme};
use gpui_material_theme::{ColorRole, MotionKind, MotionSpeed, TypeRole};

use crate::{DialogState, Sheet, SheetEdge, SheetMode, Text};

use super::items::{NavigationItems, Presentation};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RailMode {
    #[default]
    Collapsed,
    Expanded,
    Hidden,
}

/// Material wide navigation rail. An optional modal uses the window's existing
/// sheet focus scope and restores the trigger on dismissal.
#[derive(IntoElement)]
pub struct NavigationRail {
    id: ElementId,
    items: NavigationItems,
    mode: RailMode,
    title: Option<SharedString>,
    header: Option<AnyElement>,
    fab: Option<AnyElement>,
    expanded_width: Pixels,
    modal: Option<Entity<DialogState>>,
}

impl NavigationRail {
    pub fn new(id: impl Into<ElementId>, items: NavigationItems) -> Self {
        Self {
            id: id.into(),
            items,
            mode: RailMode::Collapsed,
            title: None,
            header: None,
            fab: None,
            expanded_width: px(240.),
            modal: None,
        }
    }

    pub fn mode(mut self, mode: RailMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Commands such as the expand/collapse toggle, before destinations.
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    pub fn fab(mut self, fab: impl IntoElement) -> Self {
        self.fab = Some(fab.into_any_element());
        self
    }

    pub fn expanded_width(mut self, width: Pixels) -> Self {
        self.expanded_width = width.clamp(px(220.), px(360.));
        self
    }

    /// Uses DialogState::open and DialogState::close to control the modal
    /// expansion. A closed modal leaves the configured docked or hidden mode.
    pub fn modal(mut self, state: &Entity<DialogState>) -> Self {
        self.modal = Some(state.clone());
        self
    }
}

impl RenderOnce for NavigationRail {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let open_modal = self.modal.filter(|state| state.read(cx).is_open());
        let expanded = open_modal.is_some() || self.mode == RailMode::Expanded;
        let docked_width = match self.mode {
            RailMode::Collapsed => px(96.),
            RailMode::Expanded => self.expanded_width,
            RailMode::Hidden => px(0.),
        };
        let theme = window_material_theme(window, cx);
        let background = color(theme.colors.color(ColorRole::Surface));
        let motion = theme.motion.spec(
            MotionKind::Spatial,
            MotionSpeed::Default,
            cx.reduce_motion(),
        );
        let width = px(animate(
            (self.id.clone(), "rail-width"),
            docked_width.into(),
            motion,
            window,
            cx,
        ))
        .max(px(0.));
        let show_docked = self.mode != RailMode::Hidden;
        let title = self.title.unwrap_or_else(|| "Navigation".into());
        let mut docked = div()
            .w(width)
            .h_full()
            .flex_none()
            .overflow_hidden()
            .bg(background);
        if !show_docked && open_modal.is_none() {
            return docked.into_any_element();
        }
        let panel = div()
            .flex()
            .flex_col()
            .h_full()
            .min_h_0()
            .gap(px(4.))
            .when(open_modal.is_none(), |panel| {
                panel.pt(px(4.)).pb(px(8.)).px(px(4.))
            })
            .children(self.header)
            .children(self.fab)
            .when(expanded && open_modal.is_none(), |panel| {
                panel.child(
                    div()
                        .px(px(12.))
                        .child(Text::new(title.clone()).role(TypeRole::TitleMedium).wrap()),
                )
            })
            .child(
                self.items
                    .render_row(Presentation::Rail(expanded), window, cx),
            );
        if let Some(state) = open_modal {
            docked = docked.child(
                Sheet::new(self.id, &state, title, panel)
                    .mode(SheetMode::Modal)
                    .edge(SheetEdge::Start)
                    .extent(self.expanded_width),
            );
        } else {
            docked = docked.child(panel);
        }
        docked.into_any_element()
    }
}

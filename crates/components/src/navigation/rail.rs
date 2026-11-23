use std::{collections::HashMap, rc::Rc};

use gpui::{
    AnyElement, App, ElementId, Entity, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{Surface, animate, current_material_theme};
use gpui_material_theme::{ColorRole, CornerRole, CornerShape, MotionKind, MotionSpeed, TypeRole};

use crate::{
    DialogCloseReason, DialogProperties, DialogState, Sheet, SheetColors, SheetEdge, SheetMargins,
    SheetMode, SheetPlacement, Text,
};

use super::items::{NavigationItems, Presentation};

mod state;
pub use state::NavigationRailState;

/// Presentation mode of a docked [`NavigationRail`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RailMode {
    /// Narrow rail showing stacked icons and labels.
    #[default]
    Collapsed,
    /// Wide rail showing inline icons beside labels.
    Expanded,
    /// Hidden rail occupying zero width.
    Hidden,
}

/// Distribution of standard destinations along the vertical axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationRailArrangement {
    /// Pack destinations at the top.
    #[default]
    Start,
    /// Center destinations vertically.
    Center,
    /// Pack destinations at the bottom.
    End,
    /// Distribute destinations with equal space between them.
    SpaceBetween,
    /// Distribute destinations with equal space around each item.
    SpaceAround,
    /// Distribute destinations with equal space between and around them.
    SpaceEvenly,
}

/// Logical insets in pixels, also used inside a destination's indicator.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationRailPadding {
    /// Logical start inset in pixels.
    pub start: Pixels,
    /// Logical end inset in pixels.
    pub end: Pixels,
    /// Top inset in pixels.
    pub top: Pixels,
    /// Bottom inset in pixels.
    pub bottom: Pixels,
}

impl NavigationRailPadding {
    fn docked_default() -> Self {
        Self {
            start: px(4.),
            end: px(4.),
            top: px(4.),
            bottom: px(8.),
        }
    }
}

/// Per-mode shape overrides. Omitted values use the default docked or sheet shape.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationRailShapes {
    /// Corner shape override when the rail is collapsed.
    pub collapsed: Option<CornerShape>,
    /// Corner shape override when the rail is expanded.
    pub expanded: Option<CornerShape>,
    /// Corner shape override when the rail is presented as a modal sheet.
    pub modal: Option<CornerShape>,
}

/// Independent local colors and scrim opacity.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationRailColors {
    /// Background color override for the docked rail container.
    pub container: Option<Hsla>,
    /// Default foreground color override for docked rail content.
    pub content: Option<Hsla>,
    /// Background color override for the modal rail sheet container.
    pub modal_container: Option<Hsla>,
    /// Default foreground color override for modal rail sheet content.
    pub modal_content: Option<Hsla>,
    /// Color override for the modal backdrop scrim.
    pub scrim: Option<Hsla>,
    /// Opacity override for the modal backdrop scrim.
    pub scrim_opacity: Option<f32>,
}

/// Window-local side-sheet policy for the optional modal rail.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RailModalOptions {
    /// Escape and outside-click dismissal rules for the modal sheet.
    pub properties: DialogProperties,
    /// Stacking layer for nested modals.
    pub layer: usize,
    /// Whether the modal side sheet is docked or detached.
    pub placement: SheetPlacement,
    /// Logical margins applied when [`SheetPlacement::Detached`] is active.
    pub margins: SheetMargins,
    /// When `true`, hides the docked rail (zero width) while collapsed if a modal state is bound.
    pub hide_on_collapse: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct RailPresentation {
    pub expanded: bool,
    pub arrangement: NavigationRailArrangement,
    pub spacing: Pixels,
    pub content: Option<Hsla>,
}

type Destinations = Box<dyn FnOnce(RailMode) -> AnyElement>;
type DismissRequest = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App) -> bool>;
type Closed = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App)>;

/// Material wide navigation rail. The optional modal uses the side-sheet session.
#[derive(IntoElement)]
pub struct NavigationRail {
    id: ElementId,
    items: NavigationItems,
    mode: RailMode,
    state: Option<Entity<NavigationRailState>>,
    title: Option<SharedString>,
    header: Option<AnyElement>,
    fab: Option<AnyElement>,
    expanded_width: Pixels,
    modal: Option<Entity<DialogState>>,
    modal_options: RailModalOptions,
    destination_content: Option<Destinations>,
    dividers: HashMap<SharedString, AnyElement>,
    arrangement: NavigationRailArrangement,
    destination_spacing: Pixels,
    content_padding: Option<NavigationRailPadding>,
    header_spacing: Pixels,
    expanded_header_top_padding: Pixels,
    shapes: NavigationRailShapes,
    colors: NavigationRailColors,
    dismiss: Option<DismissRequest>,
    closed: Option<Closed>,
}

impl NavigationRail {
    /// Creates a collapsed navigation rail with the given element ID and controlled [`NavigationItems`].
    pub fn new(id: impl Into<ElementId>, items: NavigationItems) -> Self {
        Self {
            id: id.into(),
            items,
            mode: RailMode::Collapsed,
            state: None,
            title: None,
            header: None,
            fab: None,
            expanded_width: px(240.),
            modal: None,
            modal_options: RailModalOptions::default(),
            destination_content: None,
            dividers: HashMap::new(),
            arrangement: NavigationRailArrangement::default(),
            destination_spacing: px(4.),
            content_padding: None,
            header_spacing: px(4.),
            expanded_header_top_padding: px(0.),
            shapes: NavigationRailShapes::default(),
            colors: NavigationRailColors::default(),
            dismiss: None,
            closed: None,
        }
    }

    /// Sets the docked rail mode (default [`RailMode::Collapsed`]). Ignored when [`Self::state`] is bound.
    pub fn mode(mut self, mode: RailMode) -> Self {
        self.mode = mode;
        self
    }

    /// Uses retained presentation and transitions in place of the supplied mode.
    pub fn state(mut self, state: &Entity<NavigationRailState>) -> Self {
        self.state = Some(state.clone());
        self
    }

    /// Sets the accessible title used for the rail and its optional modal sheet (default `"Navigation"`).
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Places a custom header element at the top of the rail (such as a menu toggle button).
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    /// Places a floating action button below the header.
    pub fn fab(mut self, fab: impl IntoElement) -> Self {
        self.fab = Some(fab.into_any_element());
        self
    }

    /// Sets the expanded rail width in logical pixels, clamped to `220..=360` px (default 240 px).
    pub fn expanded_width(mut self, width: Pixels) -> Self {
        if f32::from(width).is_finite() {
            self.expanded_width = width.clamp(px(220.), px(360.));
        }
        self
    }

    /// Replaces the standard destination list. The caller owns its navigation semantics.
    pub fn destination_content<E: IntoElement + 'static>(
        mut self,
        build: impl FnOnce(RailMode) -> E + 'static,
    ) -> Self {
        self.destination_content = Some(Box::new(move |mode| build(mode).into_any_element()));
        self
    }

    /// Places a divider or separator after one standard destination.
    pub fn divider_after(
        mut self,
        destination: impl Into<SharedString>,
        divider: impl IntoElement,
    ) -> Self {
        self.dividers
            .insert(destination.into(), divider.into_any_element());
        self
    }

    /// Sets how destinations are vertically distributed within the rail (default [`NavigationRailArrangement::Start`]).
    pub fn arrangement(mut self, arrangement: NavigationRailArrangement) -> Self {
        self.arrangement = arrangement;
        self
    }

    /// Sets the vertical gap between destinations in logical pixels (default 4 px).
    pub fn destination_spacing(mut self, spacing: Pixels) -> Self {
        self.destination_spacing = length(spacing);
        self
    }

    /// Overrides the logical inner padding of the rail container.
    pub fn content_padding(mut self, padding: NavigationRailPadding) -> Self {
        self.content_padding = Some(padding);
        self
    }

    /// Sets the vertical spacing below the header and FAB slots (default 4 px).
    pub fn header_spacing(mut self, spacing: Pixels) -> Self {
        self.header_spacing = length(spacing);
        self
    }

    /// Sets extra top padding above the header when the rail is expanded.
    pub fn expanded_header_top_padding(mut self, padding: Pixels) -> Self {
        self.expanded_header_top_padding = length(padding);
        self
    }

    /// Overrides corner shapes for collapsed, expanded, and modal presentations.
    pub fn shapes(mut self, shapes: NavigationRailShapes) -> Self {
        self.shapes = shapes;
        self
    }

    /// Overrides container, content, and modal scrim colors.
    pub fn colors(mut self, colors: NavigationRailColors) -> Self {
        self.colors = colors;
        self
    }

    /// Uses DialogState for modal visibility and focus restoration.
    pub fn modal(mut self, state: &Entity<DialogState>) -> Self {
        self.modal = Some(state.clone());
        self
    }

    /// Configures side-sheet presentation and dismissal policy for the modal rail.
    pub fn modal_options(mut self, options: RailModalOptions) -> Self {
        self.modal_options = options;
        self
    }

    /// Intercepts user-initiated modal dismissal requests; return `false` to veto closing.
    pub fn on_dismiss_request(
        mut self,
        callback: impl Fn(DialogCloseReason, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.dismiss = Some(Rc::new(callback));
        self
    }

    /// Invoked once when an accepted modal dismissal closes the rail sheet.
    pub fn on_modal_close(
        mut self,
        callback: impl Fn(DialogCloseReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.closed = Some(Rc::new(callback));
        self
    }

    fn docked(&self, window: &mut Window, cx: &mut App) -> DockedFrame {
        let viewport = length(window.viewport_size().width);
        let collapse_width = if self.modal.is_some() && self.modal_options.hide_on_collapse {
            px(0.)
        } else {
            px(96.).min(viewport)
        };
        let expand_width = self.expanded_width.min(viewport);
        if let Some(state) = &self.state {
            let (weights, target, moving) = state.update(cx, |state, cx| {
                let weights = state.sample(cx);
                (weights, state.target_value(), state.is_animating())
            });
            if moving && window.is_visible() {
                window.request_animation_frame();
            }
            let width = collapse_width * weights.collapsed + expand_width * weights.expanded;
            let mode = if target == RailMode::Hidden && moving {
                if weights.expanded >= weights.collapsed {
                    RailMode::Expanded
                } else {
                    RailMode::Collapsed
                }
            } else {
                target
            };
            let size = weights.expanded + weights.collapsed;
            DockedFrame {
                width,
                mode,
                expansion: if size > 0. {
                    weights.expanded / size
                } else {
                    0.
                },
            }
        } else {
            let theme = current_material_theme(window, cx);
            let motion = theme.motion.spec(
                MotionKind::Spatial,
                MotionSpeed::Default,
                cx.reduce_motion(),
            );
            let target = match self.mode {
                RailMode::Collapsed => collapse_width,
                RailMode::Expanded => expand_width,
                RailMode::Hidden => px(0.),
            };
            let width = px(animate(
                (self.id.clone(), "rail-width"),
                target.into(),
                motion,
                window,
                cx,
            ))
            .clamp(px(0.), viewport);
            let span = expand_width - collapse_width;
            let expansion = if span > px(0.) {
                ((width - collapse_width) / span).clamp(0., 1.)
            } else {
                f32::from(self.mode == RailMode::Expanded)
            };
            DockedFrame {
                width,
                mode: self.mode,
                expansion,
            }
        }
    }
}

struct DockedFrame {
    width: Pixels,
    mode: RailMode,
    expansion: f32,
}

impl RenderOnce for NavigationRail {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let frame = self.docked(window, cx);
        let modal = self.modal.as_ref().and_then(|state| {
            let open = state.read(cx).is_open();
            let id = (self.id.clone(), "rail-modal").into();
            let presence = crate::motion::presence(&id, open, 240, window, cx);
            presence.should_render().then(|| (state.clone(), presence))
        });
        // Resolve before either placement so a destination retains its handle.
        let items = self.items.retain(window, cx);
        let scope = window
            .use_keyed_state((self.id.clone(), "rail-docked-focus"), cx, |_, cx| {
                cx.focus_handle().tab_stop(false)
            })
            .read(cx)
            .clone();
        let frame_id = (self.id.clone(), "rail-frame");
        let docked = div()
            .id(frame_id)
            .test_support()
            .w(frame.width)
            .h_full()
            .flex_none()
            .overflow_hidden();
        let modal_visible = modal.is_some();
        if !modal_visible && (frame.mode == RailMode::Hidden || frame.width <= px(0.)) {
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
            return docked.into_any_element();
        }
        let mode = if modal_visible {
            RailMode::Expanded
        } else {
            frame.mode
        };
        let expanded = mode == RailMode::Expanded;
        let title = self.title.unwrap_or_else(|| "Navigation".into());
        let theme = current_material_theme(window, cx);
        let viewport = window.viewport_size();
        let layout = theme.layout;
        let colors = self.colors;
        let content_color = if modal_visible {
            colors.modal_content
        } else {
            colors.content
        };
        let padding = self.content_padding.unwrap_or_else(|| {
            if modal_visible {
                NavigationRailPadding::default()
            } else {
                NavigationRailPadding::docked_default()
            }
        });
        let (left, right) = layout.horizontal(length(padding.start), length(padding.end));
        let destinations = match self.destination_content {
            Some(build) => div()
                .id("custom-destinations")
                .flex_1()
                .min_h_0()
                .child(build(mode))
                .into_any_element(),
            None => items.with_rail_dividers(self.dividers).render_row(
                Presentation::Rail(RailPresentation {
                    expanded,
                    arrangement: self.arrangement,
                    spacing: self.destination_spacing,
                    content: content_color,
                }),
                window,
                cx,
            ),
        };
        let panel = div()
            .id("rail-content")
            .test_support()
            .flex()
            .flex_col()
            .h_full()
            .min_h_0()
            .gap(self.header_spacing)
            .pt(length(padding.top))
            .pb(length(padding.bottom))
            .pl(left)
            .pr(right)
            .when_some(content_color, Styled::text_color)
            .children(self.header.map(|header| {
                div()
                    .id("rail-header")
                    .test_support()
                    .flex_none()
                    .pt(if expanded {
                        self.expanded_header_top_padding
                    } else {
                        px(0.)
                    })
                    .child(header)
            }))
            .children(
                self.fab
                    .map(|fab| div().id("rail-fab").flex_none().child(fab)),
            )
            .when(expanded && !modal_visible, |panel| {
                panel.child(
                    div().px(px(12.)).child(
                        Text::new(title.clone())
                            .id("rail-title")
                            .role(TypeRole::TitleMedium)
                            .wrap(),
                    ),
                )
            })
            .child(destinations);

        let shape = docked_shape(self.shapes, frame.expansion, frame.width, viewport.height);
        let surface = Surface::new(ColorRole::Surface)
            .shape(shape)
            .when_some(colors.container, Surface::container_color)
            .size_full()
            .overflow_hidden()
            .when_some(content_color, Surface::content_color);
        if let Some((state, presence)) = modal {
            let options = self.modal_options;
            return docked
                .child(
                    Surface::new(ColorRole::Surface)
                        .shape(shape)
                        .size_full()
                        .when_some(colors.container, Surface::container_color),
                )
                .child(
                    Sheet::new(self.id, &state, title, panel)
                        .mode(SheetMode::Modal)
                        .edge(SheetEdge::Start)
                        .extent(self.expanded_width)
                        .placement(options.placement)
                        .margins(options.margins)
                        .layer(options.layer)
                        .properties(options.properties)
                        .sampled_presence(presence)
                        .colors(SheetColors {
                            container: colors.modal_container,
                            content: colors.modal_content,
                            scrim: colors.scrim,
                            ..Default::default()
                        })
                        .when_some(colors.scrim_opacity, Sheet::scrim_opacity)
                        .when_some(self.shapes.modal.or(self.shapes.expanded), Sheet::shape)
                        .when_some(self.dismiss, |sheet, callback| {
                            sheet.on_dismiss_request(move |reason, window, cx| {
                                callback(reason, window, cx)
                            })
                        })
                        .when_some(self.closed, |sheet, callback| {
                            sheet.on_close(move |reason, window, cx| callback(reason, window, cx))
                        }),
                )
                .into_any_element();
        }

        docked
            .track_focus(&scope)
            .child(surface.child(panel))
            .into_any_element()
    }
}

fn docked_shape(
    shapes: NavigationRailShapes,
    expanded: f32,
    width: Pixels,
    height: Pixels,
) -> CornerShape {
    let collapsed = shapes.collapsed.unwrap_or_else(|| CornerRole::None.shape());
    let target = shapes.expanded.unwrap_or_else(|| CornerRole::None.shape());
    if expanded <= 0. || collapsed == target {
        return collapsed;
    }
    if expanded >= 1. {
        return target;
    }
    let resolve = |shape| match shape {
        CornerShape::Radii(radii) => radii,
        CornerShape::Full => [f32::from(width.min(height)) * 0.5; 4],
    };
    let a = resolve(collapsed);
    let b = resolve(target);
    CornerShape::Radii(std::array::from_fn(|i| a[i] + (b[i] - a[i]) * expanded))
}

pub(super) fn length(value: Pixels) -> Pixels {
    if f32::from(value).is_finite() {
        value.max(px(0.))
    } else {
        px(0.)
    }
}

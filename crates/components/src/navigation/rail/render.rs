use gpui::{
    App, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{Surface, animate, current_material_theme, resolve_edges};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, LogicalEdges, MotionKind, MotionSpeed, TypeRole,
};

use crate::{Sheet, SheetColors, SheetEdge, SheetMode, Text};

use super::{
    super::items::Presentation, NavigationRail, NavigationRailShapes, RailMode, RailPresentation,
    length,
};

struct DockedFrame {
    width: Pixels,
    mode: RailMode,
    expansion: f32,
}

impl NavigationRail {
    fn docked(&self, window: &mut Window, cx: &mut App) -> DockedFrame {
        let viewport = length(window.viewport_size().width);
        let collapse_width = if self.modal.is_some() && self.modal_options.hide_on_collapse {
            px(0.)
        } else {
            px(96.).min(viewport)
        };
        let expand_width = self.expanded_width.min(viewport);
        if let Some(state) = &self.state {
            let (weights, target, moving, changed) = state.update(cx, |state, cx| {
                let (weights, changed) = state.sample(cx);
                (weights, state.target_value(), state.is_animating(), changed)
            });
            if changed {
                let weak = state.downgrade();
                window.defer(cx, move |_, cx| {
                    let _ = weak.update(cx, |_, cx| cx.notify());
                });
            }
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

impl RenderOnce for NavigationRail {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let frame = self.docked(window, cx);
        let modal = self.modal.as_ref().and_then(|state| {
            let open = state.read(cx).is_open();
            let id = (self.id.clone(), "rail-modal").into();
            let presence = crate::overlay::presence(&id, open, 240, window, cx);
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
        let colors = self.colors;
        let content_color = if modal_visible {
            colors.modal_content
        } else {
            colors.content
        };
        let padding = self.content_padding.unwrap_or_else(|| {
            if modal_visible {
                LogicalEdges::ZERO
            } else {
                LogicalEdges::new(4., 4., 8., 4.)
            }
        });
        let edges = resolve_edges(padding, theme.layout.direction);
        let destinations = match self.destination_content {
            Some(build) => div()
                .id((self.id.clone(), "custom-destinations"))
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
            .id((self.id.clone(), "rail-content"))
            .test_support()
            .flex()
            .flex_col()
            .h_full()
            .min_h_0()
            .gap(self.header_spacing)
            .pt(edges.top)
            .pb(edges.bottom)
            .pl(edges.left)
            .pr(edges.right)
            .when_some(content_color, Styled::text_color)
            .children(self.header.map(|header| {
                div()
                    .id((self.id.clone(), "rail-header"))
                    .test_support()
                    .flex_none()
                    .pt(if expanded {
                        self.expanded_header_top_padding
                    } else {
                        px(0.)
                    })
                    .child(header)
            }))
            .children(self.fab.map(|fab| {
                div()
                    .id((self.id.clone(), "rail-fab"))
                    .flex_none()
                    .child(fab)
            }))
            .when(expanded && !modal_visible, |panel| {
                panel.child(
                    div().px(px(12.)).child(
                        Text::new(title.clone())
                            .id((self.id.clone(), "rail-title"))
                            .role(TypeRole::TitleMedium)
                            .wrap(),
                    ),
                )
            })
            .child(destinations)
            .children(self.footer.map(|footer| {
                div()
                    .id((self.id.clone(), "rail-footer"))
                    .test_support()
                    .flex_none()
                    .child(footer)
            }));

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
                            sheet.on_closed(move |reason, window, cx| callback(reason, window, cx))
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

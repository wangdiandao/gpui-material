use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels, Rgba,
    Role, StatefulInteractiveElement, Styled, Window, canvas, div, prelude::FluentBuilder, px,
};
use gpui_base::{Button, Popup};
use gpui_material_components::{FabSize, Icon, IconName};
use gpui_material_foundation::{ThemeScope, elevation_shadows};
use gpui_material_theme::{CornerShape, Direction, Elevation};

use super::{
    Expand, FabMenu, FabMenuTriggerAppearance, MenuFocus, MenuState, MenuStyle, activate_key,
    style_button,
};

pub(super) enum TriggerContent {
    Icon(AnyElement),
    Custom(AnyElement),
}

pub(super) struct TriggerRenderContext<'a> {
    pub(super) id: &'a ElementId,
    pub(super) state: &'a Rc<MenuState>,
    pub(super) focus: &'a MenuFocus,
    pub(super) request: &'a Expand,
    pub(super) visual: TriggerVisual,
    pub(super) style: MenuStyle,
    pub(super) shadow: Hsla,
    pub(super) scale: f32,
    pub(super) visibility_alpha: f32,
    pub(super) open: bool,
    pub(super) interactive: bool,
    pub(super) has_items: bool,
}

#[derive(Clone, Copy)]
pub(super) struct TriggerVisual {
    pub(super) size: f32,
    corners: [f32; 4],
    icon: f32,
    fill: Hsla,
    ink: Hsla,
    elevation: f32,
}

impl TriggerVisual {
    fn at(self, appearance: FabMenuTriggerAppearance, cap: f32, direction: Direction) -> Self {
        let dimension = |value: Option<Pixels>, default| {
            value
                .map(f32::from)
                .filter(|value| value.is_finite() && *value > 0.)
                .unwrap_or(default)
        };
        let size = dimension(appearance.size, self.size).min(cap);
        Self {
            size,
            corners: appearance
                .shape
                .unwrap_or(CornerShape::Radii(self.corners))
                .radii(size, size, direction),
            icon: dimension(appearance.icon_size, self.icon).min(size),
            fill: appearance.container_color.unwrap_or(self.fill),
            ink: appearance.content_color.unwrap_or(self.ink),
            elevation: appearance.elevation.map_or(self.elevation, Elevation::dp),
        }
    }

    fn interpolate(self, end: Self, fraction: f32) -> Self {
        let mix = |a, b| a + (b - a) * fraction;
        let color = |a, b| {
            if fraction == 0. {
                return a;
            }
            if fraction == 1. {
                return b;
            }
            let a = Rgba::from(a);
            let b = Rgba::from(b);
            Rgba {
                r: mix(a.r, b.r),
                g: mix(a.g, b.g),
                b: mix(a.b, b.b),
                a: mix(a.a, b.a),
            }
            .into()
        };
        Self {
            size: mix(self.size, end.size),
            corners: std::array::from_fn(|index| mix(self.corners[index], end.corners[index])),
            icon: mix(self.icon, end.icon),
            fill: color(self.fill, end.fill),
            ink: color(self.ink, end.ink),
            elevation: mix(self.elevation, end.elevation),
        }
    }

    fn decorate<E: Styled + ParentElement>(
        self,
        element: E,
        scale: f32,
        shadow: Hsla,
        content: AnyElement,
    ) -> E {
        let [tl, tr, br, bl] = self.corners.map(|radius| px(radius * scale));
        element
            .size(px(self.size * scale))
            .flex_none()
            .flex()
            .justify_center()
            .items_center()
            .rounded_tl(tl)
            .rounded_tr(tr)
            .rounded_br(br)
            .rounded_bl(bl)
            .shadow(elevation_shadows(self.elevation * scale, scale, shadow))
            .child(content)
    }
}

pub(super) fn icon_content(icon: AnyElement, side: Pixels) -> gpui::Div {
    div()
        .size(side)
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .overflow_hidden()
        .text_size(side)
        .line_height(side)
        .child(icon)
}

pub(super) fn decorative_content(id: ElementId, content: impl IntoElement) -> impl IntoElement {
    div()
        .id(id)
        .role(Role::Group)
        .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
        .min_w_0()
        .max_w_full()
        .overflow_hidden()
        .child(content)
}

impl FabMenu {
    pub(super) fn trigger_visual(
        &self,
        palette: (Hsla, Hsla, Hsla, Hsla),
        cap: f32,
        reveal: f32,
        direction: Direction,
    ) -> TriggerVisual {
        let (fill, ink, strong_fill, strong_ink) = palette;
        let (initial_size, initial_radius, initial_icon) = match self.size {
            FabSize::Small => (56., 16., 24.),
            FabSize::Medium => (80., 20., 28.),
            FabSize::Large => (96., 28., 36.),
        };
        let collapsed = TriggerVisual {
            size: initial_size,
            corners: [initial_radius; 4],
            icon: initial_icon,
            fill,
            ink,
            elevation: Elevation::Level3.dp(),
        }
        .at(self.trigger_style.collapsed, cap, direction);
        let expanded = TriggerVisual {
            size: 56.,
            corners: [28.; 4],
            icon: 20.,
            fill: strong_fill,
            ink: strong_ink,
            elevation: Elevation::Level3.dp(),
        }
        .at(self.trigger_style.expanded, cap, direction);
        collapsed.interpolate(expanded, reveal)
    }

    pub(super) fn render_trigger_layer(
        &mut self,
        ctx: &TriggerRenderContext<'_>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let content = if ctx.open {
            self.close_content.take()
        } else {
            self.open_content.take()
        };
        let icon_size = px(ctx.visual.icon * ctx.scale);
        let content = match content {
            Some(TriggerContent::Custom(content)) => content,
            Some(TriggerContent::Icon(icon)) => icon_content(icon, icon_size).into_any_element(),
            None => Icon::new(if ctx.open { IconName::Close } else { self.icon })
                .size(icon_size)
                .into_any_element(),
        };
        let content = decorative_content((ctx.id.clone(), "trigger-content").into(), content)
            .into_any_element();
        if !ctx.interactive {
            return ctx
                .visual
                .decorate(
                    div().bg(ctx.visual.fill).text_color(ctx.visual.ink),
                    ctx.scale,
                    ctx.shadow,
                    content,
                )
                .opacity(ctx.visibility_alpha)
                .into_any_element();
        }
        let activate = {
            let state = ctx.state.clone();
            let request = ctx.request.clone();
            move |window: &mut Window, cx: &mut App| request(!state.open.get(), window, cx)
        };
        let accessibility = activate.clone();
        let request_accessibility = ctx.request.clone();
        let keyboard = activate.clone();
        let open = ctx.open;
        let trigger = Button::new((ctx.id.clone(), "trigger"))
            .track_focus(&ctx.focus.trigger)
            .accessibility_label(if open {
                self.close_label.clone()
            } else {
                self.open_label.clone()
            })
            .aria_expanded(open)
            .disabled(!ctx.has_items)
            .when(ctx.has_items, |trigger| {
                trigger
                    .on_click(move |_, window, cx| activate(window, cx))
                    .on_a11y_action(gpui::AccessibleAction::Click, move |_, window, cx| {
                        accessibility(window, cx)
                    })
                    .on_a11y_action(
                        if open {
                            gpui::AccessibleAction::Collapse
                        } else {
                            gpui::AccessibleAction::Expand
                        },
                        move |_, window, cx| request_accessibility(!open, window, cx),
                    )
                    .capture_key_down(move |event, window, cx| {
                        activate_key(event, window, cx, &keyboard)
                    })
            });
        let trigger = ctx
            .visual
            .decorate(
                style_button(
                    trigger,
                    ctx.visual.fill,
                    ctx.visual.ink,
                    ctx.style.ring,
                    ctx.has_items,
                ),
                ctx.scale,
                ctx.shadow,
                content,
            )
            .opacity(ctx.visibility_alpha);
        let measure_trigger = ctx.state.clone();
        let trigger = div().relative().child(trigger).child(
            canvas(
                move |bounds, _, _| measure_trigger.trigger_bounds.set(bounds),
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        );
        let mut popup = Popup::new((ctx.id.clone(), "popup"), trigger)
            .anchor(ctx.style.anchor)
            .offset(px(8.));
        if open {
            let outside = ctx.request.clone();
            let exclude_trigger = ctx.state.clone();
            let record_bounds = ctx.state.clone();
            let panel = div()
                .id((ctx.id.clone(), "panel"))
                .occlude()
                .on_mouse_down_out(move |event, window, cx| {
                    if !exclude_trigger
                        .trigger_bounds
                        .get()
                        .contains(&event.position)
                    {
                        outside(false, window, cx);
                    }
                })
                .child(self.render_actions(
                    &ctx.focus.handles,
                    ctx.state,
                    ctx.request,
                    ctx.style,
                    window,
                ));
            popup = popup
                .on_position(move |position, _| record_bounds.bounds.set(position.bounds))
                .content(ThemeScope::inherit(panel, window, cx));
        }
        popup.into_any_element()
    }
}

use std::{cell::Cell, rc::Rc};

use gpui::{
    Anchor, AnyElement, App, Axis, Bounds, ElementId, FocusHandle, Hsla, InteractiveElement,
    IntoElement, ParentElement, Pixels, RenderOnce, Rgba, Role, SharedString,
    StatefulInteractiveElement, Styled, Window, canvas, deferred, div, point,
    prelude::FluentBuilder, px,
};
use gpui_base::{Button, FocusTrapElement, Popup, Positioner, TestSupportExt};
use gpui_material_components::{FabColor, FabSize, FabVisibilityAnimation, Icon, IconName, Text};
use gpui_material_foundation::{
    ThemeScope, animate, color, current_material_theme, elevation_shadows, logical_anchor,
    move_roving_focus,
};
use gpui_material_theme::{
    ColorRole, CornerShape, Direction, Elevation, MotionKind, MotionSpeed, TypeRole,
};

use super::{Invoke, activate_key, focus_handle, style_button};

type Expand = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// An individually labeled FAB menu action. Supply a stable id for each item.
pub struct FabMenuItem {
    id: SharedString,
    label: SharedString,
    icon: Option<AnyElement>,
    content: Option<AnyElement>,
    colors: FabMenuItemColors,
    disabled: bool,
    invoke: Option<Invoke>,
}

impl FabMenuItem {
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: IconName,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: Some(Icon::new(icon).into_any_element()),
            content: None,
            colors: FabMenuItemColors::default(),
            disabled: false,
            invoke: None,
        }
    }

    /// Uses arbitrary decorative content while retaining the supplied action name.
    pub fn custom(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        content: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            content: Some(content.into_any_element()),
            colors: FabMenuItemColors::default(),
            disabled: false,
            invoke: None,
        }
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    /// Replaces the item's visible label. Its accessible name remains the same.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn colors(mut self, colors: FabMenuItemColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_activate(mut self, invoke: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.invoke = Some(Rc::new(invoke));
        self
    }
}

/// Optional colors for an individual FAB menu action.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FabMenuItemColors {
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub disabled_container: Option<Hsla>,
    pub disabled_content: Option<Hsla>,
}

/// Horizontal alignment of a FAB menu relative to its trigger.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FabMenuAlignment {
    Start,
    Center,
    #[default]
    End,
}

impl FabMenuAlignment {
    fn anchor(self, direction: Direction) -> Anchor {
        let anchor = match self {
            Self::Start => Anchor::BottomLeft,
            Self::Center => Anchor::BottomCenter,
            Self::End => Anchor::BottomRight,
        };
        logical_anchor(anchor, direction)
    }
}

/// Optional appearance of one endpoint of the FAB menu trigger transition.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FabMenuTriggerAppearance {
    pub size: Option<Pixels>,
    pub shape: Option<CornerShape>,
    pub icon_size: Option<Pixels>,
    pub container_color: Option<Hsla>,
    pub content_color: Option<Hsla>,
    pub elevation: Option<Elevation>,
}

/// Interpolated collapsed and expanded trigger appearances.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FabMenuTriggerStyle {
    pub collapsed: FabMenuTriggerAppearance,
    pub expanded: FabMenuTriggerAppearance,
}

enum TriggerContent {
    Icon(AnyElement),
    Custom(AnyElement),
}

/// A floating action button that opens a stack of commands above it.
/// Provide two to six actions with stable IDs.
#[derive(IntoElement)]
pub struct FabMenu {
    id: ElementId,
    open_label: SharedString,
    close_label: SharedString,
    icon: IconName,
    open_content: Option<TriggerContent>,
    close_content: Option<TriggerContent>,
    items: Vec<FabMenuItem>,
    size: FabSize,
    color: FabColor,
    focus: Option<FocusHandle>,
    expanded: Option<bool>,
    on_expanded_change: Option<Expand>,
    alignment: FabMenuAlignment,
    trigger_style: FabMenuTriggerStyle,
    visible: Option<bool>,
    visibility_animation: FabVisibilityAnimation,
}

impl FabMenu {
    pub fn new(
        id: impl Into<ElementId>,
        open_label: impl Into<SharedString>,
        icon: IconName,
        items: impl IntoIterator<Item = FabMenuItem>,
    ) -> Self {
        Self {
            id: id.into(),
            open_label: open_label.into(),
            close_label: "Close actions".into(),
            icon,
            open_content: None,
            close_content: None,
            items: items.into_iter().collect(),
            size: FabSize::Small,
            color: FabColor::Primary,
            focus: None,
            expanded: None,
            on_expanded_change: None,
            alignment: FabMenuAlignment::default(),
            trigger_style: FabMenuTriggerStyle::default(),
            visible: None,
            visibility_animation: FabVisibilityAnimation::default(),
        }
    }

    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = label.into();
        self
    }

    pub fn open_icon(mut self, icon: impl IntoElement) -> Self {
        self.open_content = Some(TriggerContent::Icon(icon.into_any_element()));
        self
    }

    pub fn close_icon(mut self, icon: impl IntoElement) -> Self {
        self.close_content = Some(TriggerContent::Icon(icon.into_any_element()));
        self
    }

    pub fn open_content(mut self, content: impl IntoElement) -> Self {
        self.open_content = Some(TriggerContent::Custom(content.into_any_element()));
        self
    }

    pub fn close_content(mut self, content: impl IntoElement) -> Self {
        self.close_content = Some(TriggerContent::Custom(content.into_any_element()));
        self
    }

    /// Renders caller-owned expansion. Without this setter, the menu retains its value.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    /// Reports a requested expansion value from the trigger, dismissal, or selection.
    pub fn on_expanded_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_expanded_change = Some(Rc::new(handler));
        self
    }

    pub fn alignment(mut self, alignment: FabMenuAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn trigger_style(mut self, style: FabMenuTriggerStyle) -> Self {
        self.trigger_style = style;
        self
    }

    /// Controls the presence of the trigger independently of expansion.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = Some(visible);
        self
    }

    pub fn visibility_animation(mut self, animation: FabVisibilityAnimation) -> Self {
        self.visibility_animation = animation;
        self
    }

    pub fn size(mut self, size: FabSize) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: FabColor) -> Self {
        self.color = color;
        self
    }

    /// Uses a caller-owned focus handle for the toggle button.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }
}

#[derive(Default)]
struct MenuState {
    open: Cell<bool>,
    interactive: Cell<bool>,
    bounds: Cell<Bounds<Pixels>>,
    trigger_bounds: Cell<Bounds<Pixels>>,
    scroll: gpui::ScrollHandle,
}

#[derive(Clone, Copy)]
struct MenuStyle {
    ink: Hsla,
    fill: Hsla,
    ring: Hsla,
    rtl: bool,
    anchor: Anchor,
    max_width: Pixels,
    alpha: f32,
    reveal: f32,
}

impl MenuStyle {
    fn colors(self, item: &FabMenuItem) -> (Hsla, Hsla) {
        if item.disabled {
            (
                item.colors
                    .disabled_container
                    .unwrap_or_else(|| self.ink.opacity(0.12)),
                item.colors
                    .disabled_content
                    .unwrap_or_else(|| self.ink.opacity(0.38)),
            )
        } else {
            (
                item.colors.container.unwrap_or(self.fill),
                item.colors.content.unwrap_or(self.ink),
            )
        }
    }
}

struct MenuFocus {
    trigger: FocusHandle,
    scope: FocusHandle,
    open_focus: FocusHandle,
    handles: Vec<(FocusHandle, bool)>,
}

struct TriggerRenderContext<'a> {
    id: &'a ElementId,
    state: &'a Rc<MenuState>,
    focus: &'a MenuFocus,
    request: &'a Expand,
    visual: TriggerVisual,
    style: MenuStyle,
    shadow: Hsla,
    scale: f32,
    visibility_alpha: f32,
    open: bool,
    interactive: bool,
    has_items: bool,
}

impl FabMenu {
    fn prepare_focus(
        &self,
        open: bool,
        was_open: bool,
        state: &Rc<MenuState>,
        window: &mut Window,
        cx: &mut App,
    ) -> MenuFocus {
        let interactive = state.interactive.get();
        let has_items = !self.items.is_empty();
        let trigger = self
            .focus
            .clone()
            .unwrap_or_else(|| focus_handle(&self.id, "trigger-focus", window, cx));
        let scope = focus_handle(&self.id, "menu-scope", window, cx);
        let handles = self
            .items
            .iter()
            .map(|item| {
                let item_id = ElementId::from((self.id.clone(), item.id.clone()));
                (focus_handle(&item_id, "focus", window, cx), !item.disabled)
            })
            .collect::<Vec<_>>();
        let first = handles
            .iter()
            .find(|(_, enabled)| *enabled)
            .map(|(handle, _)| handle.clone());
        let open_focus = first.unwrap_or_else(|| scope.clone());
        if open && !was_open {
            let state = state.clone();
            let open_focus = open_focus.clone();
            window.defer(cx, move |window, cx| {
                if state.open.get() {
                    open_focus.focus(window, cx);
                }
            });
        } else if !open
            && ((was_open && scope.contains_focused(window, cx))
                || (!interactive && trigger.is_focused(window)))
        {
            let state = state.clone();
            let trigger_focus = trigger.clone();
            let previous_focus = window.focused(cx);
            window.defer(cx, move |window, cx| {
                let current = window.focused(cx);
                if state.open.get()
                    || interactive != state.interactive.get()
                    || current.is_some() && current != previous_focus
                {
                    return;
                }
                if interactive && has_items {
                    trigger_focus.focus(window, cx);
                } else if current.is_some() {
                    window.blur(cx);
                }
            });
        }
        MenuFocus {
            trigger,
            scope,
            open_focus,
            handles,
        }
    }

    fn expansion_request(
        &self,
        state: &Rc<MenuState>,
        focus: &MenuFocus,
        has_items: bool,
    ) -> Expand {
        let state = state.clone();
        let trigger_focus = focus.trigger.clone();
        let scope = focus.scope.clone();
        let open_focus = focus.open_focus.clone();
        let controlled = self.expanded.is_some();
        let on_change = self.on_expanded_change.clone();
        Rc::new(move |expanded, window, cx| {
            if !state.interactive.get() || !has_items || state.open.get() == expanded {
                return;
            }
            if !controlled {
                state.open.set(expanded);
                if expanded {
                    open_focus.focus(window, cx);
                } else if scope.contains_focused(window, cx) {
                    trigger_focus.focus(window, cx);
                }
                window.refresh();
            }
            if let Some(on_change) = &on_change {
                on_change(expanded, window, cx);
            }
        })
    }

    fn trigger_visual(
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

    fn render_trigger_layer(
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

    fn render_actions(
        &mut self,
        handles: &[(FocusHandle, bool)],
        state: &Rc<MenuState>,
        request: &Expand,
        style: MenuStyle,
        window: &Window,
    ) -> AnyElement {
        let available_height = (window.viewport_size().height - px(24.)).max(px(0.));
        let items = std::mem::take(&mut self.items).into_iter().zip(handles);
        if !state.open.get() {
            return div()
                .id((self.id.clone(), "exit"))
                .test_support()
                .role(Role::Group)
                .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
                .max_h(available_height)
                .overflow_y_hidden()
                .track_scroll(&state.scroll)
                .children(items.map(|(item, _)| {
                    let colors = style.colors(&item);
                    menu_row(menu_pill(div(), item, style, colors), style.anchor)
                }))
                .into_any_element();
        }

        let key_handles = handles.to_vec();
        let key_scroll = state.scroll.clone();
        div()
            .id((self.id.clone(), "actions"))
            .test_support()
            .role(Role::Menu)
            .aria_label(self.open_label.clone())
            .max_h(available_height)
            .overflow_y_scroll()
            .track_scroll(&state.scroll)
            .on_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let focus = key_handles
                    .iter()
                    .map(|(handle, enabled)| (handle, *enabled))
                    .collect::<Vec<_>>();
                if let Some(index) = move_roving_focus(
                    &focus,
                    &event.keystroke.key,
                    Axis::Vertical,
                    Direction::Ltr,
                    window,
                    cx,
                ) {
                    key_scroll.scroll_to_item(index);
                    cx.stop_propagation();
                }
            })
            .children(items.map(|(mut item, (focus, enabled))| {
                let invoke = item.invoke.take();
                let request = request.clone();
                let open = state.clone();
                let activate = move |window: &mut Window, cx: &mut App| {
                    if open.open.get() {
                        request(false, window, cx);
                        if let Some(invoke) = &invoke {
                            invoke(window, cx);
                        }
                    }
                };
                let keyboard = activate.clone();
                let accessibility = activate.clone();
                let colors = style.colors(&item);
                let button = Button::new((self.id.clone(), item.id.clone()))
                    .role(Role::MenuItem)
                    .accessibility_label(item.label.clone())
                    .track_focus(focus)
                    .disabled(!enabled)
                    .when(*enabled, |button| {
                        button
                            .on_click(move |_, window, cx| activate(window, cx))
                            .on_a11y_action(gpui::AccessibleAction::Click, move |_, window, cx| {
                                accessibility(window, cx)
                            })
                            .capture_key_down(move |event, window, cx| {
                                activate_key(event, window, cx, &keyboard)
                            })
                    });
                menu_row(
                    menu_pill(
                        style_button(button, colors.0, colors.1, style.ring, *enabled),
                        item,
                        style,
                        colors,
                    ),
                    style.anchor,
                )
            }))
            .into_any_element()
    }
}

fn menu_row(child: impl IntoElement, anchor: Anchor) -> impl IntoElement {
    let row = div().flex().px(px(4.)).py(px(2.));
    match anchor {
        Anchor::BottomLeft => row.justify_start(),
        Anchor::BottomCenter => row.justify_center(),
        _ => row.justify_end(),
    }
    .child(child)
}

fn icon_content(icon: AnyElement, side: Pixels) -> gpui::Div {
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

fn decorative_content(id: ElementId, content: impl IntoElement) -> impl IntoElement {
    div()
        .id(id)
        .role(Role::Group)
        .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
        .min_w_0()
        .max_w_full()
        .overflow_hidden()
        .child(content)
}

fn menu_pill<E>(element: E, item: FabMenuItem, style: MenuStyle, (fill, ink): (Hsla, Hsla)) -> E
where
    E: Styled + ParentElement + FluentBuilder,
{
    let width = style.max_width * style.reveal;
    let label = item.content.unwrap_or_else(|| {
        Text::new(item.label)
            .role(TypeRole::TitleMedium)
            .into_any_element()
    });
    element
        .flex()
        .flex_row()
        .flex_none()
        .when(style.rtl, Styled::flex_row_reverse)
        .items_center()
        .justify_end()
        .overflow_hidden()
        .min_h(px(56.))
        .max_w(width)
        .rounded_full()
        .bg(fill)
        .text_color(ink)
        .opacity(style.alpha)
        .px(px(24.).min(width / 3.))
        .gap(px(8.))
        .children(item.icon.map(|icon| {
            icon_content(icon, px(24.))
                .id((ElementId::from("fab-menu-icon"), item.id.clone()))
                .role(Role::Group)
                .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
        }))
        .child(decorative_content(
            (ElementId::from("fab-menu-label"), item.id).into(),
            label,
        ))
}

#[derive(Clone, Copy)]
struct TriggerVisual {
    size: f32,
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

impl RenderOnce for FabMenu {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let retained = window.use_keyed_state((self.id.clone(), "menu-state"), cx, |_, _| {
            Rc::new(MenuState::default())
        });
        let state = retained.read(cx).clone();
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let motion = theme.motion;
        let tone = |role| color(theme.colors.color(role));
        let (background, foreground, strong, strong_ink) = fab_roles(self.color);
        let fill = tone(background);
        let ink = tone(foreground);
        let strong_fill = tone(strong);
        let strong_ink = tone(strong_ink);
        let ring = tone(ColorRole::OnSurface);
        let shadow = tone(ColorRole::Shadow);
        let viewport = window.viewport_size();
        let cap = f32::from(viewport.width.min(viewport.height)).max(0.);
        let show = self.visible.unwrap_or(true);
        let (scale, visibility_alpha, pivot) =
            self.visible.map_or((1., 1., point(0.5, 0.5)), |visible| {
                self.visibility_animation
                    .frame(&self.id, visible, cap.max(1.), window, cx)
            });
        let interactive = show && cap > 0. && scale > 0. && visibility_alpha > 0.;
        state.interactive.set(interactive);
        let has_items = !self.items.is_empty();
        let open = interactive && has_items && self.expanded.unwrap_or_else(|| state.open.get());
        let was_open = state.open.replace(open);
        let alpha = animate(
            (self.id.clone(), "menu-alpha"),
            f32::from(open),
            motion.spec(MotionKind::Effects, MotionSpeed::Fast, cx.reduce_motion()),
            window,
            cx,
        )
        .clamp(0., 1.);
        let reveal = animate(
            (self.id.clone(), "menu-reveal"),
            f32::from(open),
            motion.spec(MotionKind::Spatial, MotionSpeed::Fast, cx.reduce_motion()),
            window,
            cx,
        )
        .clamp(0., 1.);

        let id = self.id.clone();
        let focus = self.prepare_focus(open, was_open, &state, window, cx);
        let request = self.expansion_request(&state, &focus, has_items);
        let visual =
            self.trigger_visual((fill, ink, strong_fill, strong_ink), cap, reveal, direction);
        let scale = if visual.size > 0. {
            scale.min(cap / visual.size)
        } else {
            0.
        };
        let anchor = self.alignment.anchor(direction);
        let style = MenuStyle {
            fill,
            ink,
            ring,
            rtl: direction == Direction::Rtl,
            anchor,
            max_width: (viewport.width - px(24.)).clamp(px(0.), px(360.)),
            alpha,
            reveal,
        };
        let exit_bounds = state.bounds.get();
        let mut root = div().id(id.clone()).flex_none();
        if visibility_alpha > 0. {
            let side = px(visual.size);
            let mut footprint = div().relative().size(side).flex_none();
            if scale > 0. {
                let trigger_ctx = TriggerRenderContext {
                    id: &id,
                    state: &state,
                    focus: &focus,
                    request: &request,
                    visual,
                    style,
                    shadow,
                    scale,
                    visibility_alpha,
                    open,
                    interactive,
                    has_items,
                };
                let layer = self.render_trigger_layer(&trigger_ctx, window, cx);
                if self.visible.is_none() {
                    root = root.child(layer);
                } else {
                    let distance = side * (1. - scale);
                    footprint = footprint.child(
                        div()
                            .absolute()
                            .left(distance * pivot.x)
                            .top(distance * pivot.y)
                            .child(layer),
                    );
                    root = root.child(footprint);
                }
            } else {
                root = root.child(footprint);
            }
        }
        if !open && alpha > 0. && exit_bounds.size.width > px(0.) {
            let x = match anchor {
                Anchor::BottomLeft => exit_bounds.left(),
                Anchor::BottomCenter => exit_bounds.center().x,
                _ => exit_bounds.right(),
            };
            root = root.child(
                deferred(
                    Positioner::corner(anchor, point(x, exit_bounds.bottom())).child(
                        ThemeScope::inherit(
                            self.render_actions(&focus.handles, &state, &request, style, window),
                            window,
                            cx,
                        ),
                    ),
                )
                .with_priority(100),
            );
        }
        if open {
            root.on_key_down(move |event, window, cx| {
                if event.keystroke.key == "escape" {
                    request(false, window, cx);
                    cx.stop_propagation();
                }
            })
            .focus_trap((id, "focus-trap"), &focus.scope)
            .into_any_element()
        } else {
            root.into_any_element()
        }
    }
}

fn fab_roles(value: FabColor) -> (ColorRole, ColorRole, ColorRole, ColorRole) {
    use ColorRole::*;
    match value {
        FabColor::Primary => (PrimaryContainer, OnPrimaryContainer, Primary, OnPrimary),
        FabColor::Secondary => (
            SecondaryContainer,
            OnSecondaryContainer,
            Secondary,
            OnSecondary,
        ),
        FabColor::Tertiary => (TertiaryContainer, OnTertiaryContainer, Tertiary, OnTertiary),
        FabColor::PrimaryStrong => (Primary, OnPrimary, PrimaryContainer, OnPrimaryContainer),
        FabColor::SecondaryStrong => (
            Secondary,
            OnSecondary,
            SecondaryContainer,
            OnSecondaryContainer,
        ),
        FabColor::TertiaryStrong => (Tertiary, OnTertiary, TertiaryContainer, OnTertiaryContainer),
    }
}

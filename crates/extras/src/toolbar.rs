use std::{cell::Cell, rc::Rc};

use gpui::{
    Anchor, AnyElement, App, Axis, Bounds, ElementId, FocusHandle, Hsla, InteractiveElement,
    IntoElement, ParentElement, Pixels, RenderOnce, Role, SharedString, StatefulInteractiveElement,
    Styled, Window, canvas, deferred, div, point, prelude::FluentBuilder, px,
};
use gpui_base::{Button, FocusTrapElement, Popup, Positioner, TestSupportExt};
use gpui_material_components::{FabColor, FabSize, Icon, IconName, Text};
use gpui_material_foundation::{
    animate, color, elevation_shadows, logical_anchor, move_roving_focus, window_material_theme,
};
use gpui_material_theme::{
    ColorRole, Direction, Elevation, MotionKind, MotionSpeed, StateLayer, TypeRole,
};

type Invoke = Rc<dyn Fn(&mut Window, &mut App)>;

mod bar;
pub use bar::{
    Toolbar, ToolbarAction, ToolbarColor, ToolbarFabPosition, ToolbarOrientation, ToolbarPlacement,
};

/// An individually labeled FAB menu action. Supply a stable id for each item.
pub struct FabMenuItem {
    id: SharedString,
    label: SharedString,
    icon: IconName,
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
            icon,
            disabled: false,
            invoke: None,
        }
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

/// An end-aligned floating action button that opens a stack of actions above it.
/// Provide two to six actions. Each activation closes the menu and returns
/// focus to the trigger before invoking the application command.
#[derive(IntoElement)]
pub struct FabMenu {
    id: ElementId,
    open_label: SharedString,
    close_label: SharedString,
    icon: IconName,
    items: Vec<FabMenuItem>,
    size: FabSize,
    color: FabColor,
    focus: Option<FocusHandle>,
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
            items: items.into_iter().collect(),
            size: FabSize::Small,
            color: FabColor::Primary,
            focus: None,
        }
    }

    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = label.into();
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
    max_width: Pixels,
    alpha: f32,
    reveal: f32,
}

impl FabMenu {
    fn render_actions(
        &self,
        handles: &[(FocusHandle, bool)],
        state: &MenuState,
        dismiss: &Invoke,
        style: MenuStyle,
        window: &Window,
    ) -> AnyElement {
        let available_height = (window.viewport_size().height - px(24.)).max(px(0.));
        let items = self.items.iter().zip(handles);
        if !state.open.get() {
            return div()
                .id((self.id.clone(), "exit"))
                .test_support()
                .max_h(available_height)
                .overflow_y_scroll()
                .track_scroll(&state.scroll)
                .children(items.map(|(item, _)| menu_row(menu_pill(div(), item, style), style.rtl)))
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
            .children(items.map(|(item, (focus, enabled))| {
                let invoke = item.invoke.clone();
                let dismiss = dismiss.clone();
                let activate = move |window: &mut Window, cx: &mut App| {
                    dismiss(window, cx);
                    if let Some(invoke) = &invoke {
                        invoke(window, cx);
                    }
                };
                let keyboard = activate.clone();
                let accessibility = activate.clone();
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
                        style_button(button, style.fill, style.ink, style.ring, *enabled),
                        item,
                        style,
                    ),
                    style.rtl,
                )
            }))
            .into_any_element()
    }
}

fn menu_row(child: impl IntoElement, rtl: bool) -> impl IntoElement {
    div()
        .flex()
        .px(px(4.))
        .py(px(2.))
        .when_else(rtl, Styled::justify_start, Styled::justify_end)
        .child(child)
}

fn menu_pill<E>(element: E, item: &FabMenuItem, style: MenuStyle) -> E
where
    E: Styled + ParentElement + FluentBuilder,
{
    let ink = style.ink.opacity(if item.disabled { 0.38 } else { 1. });
    let fill = if item.disabled {
        style.ink.opacity(0.12)
    } else {
        style.fill
    };
    element
        .flex()
        .flex_row()
        .flex_none()
        .when(style.rtl, Styled::flex_row_reverse)
        .items_center()
        .justify_end()
        .overflow_hidden()
        .min_h(px(56.))
        .max_w(style.max_width * style.reveal)
        .rounded_full()
        .bg(fill)
        .text_color(ink)
        .opacity(style.alpha)
        .px(px(24.))
        .gap(px(8.))
        .child(Icon::new(item.icon).size(px(24.)))
        .child(
            Text::new(item.label.clone())
                .id((ElementId::from("fab-menu-label"), item.id.clone()))
                .role(TypeRole::TitleMedium),
        )
}

impl RenderOnce for FabMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let retained = window.use_keyed_state((self.id.clone(), "menu-state"), cx, |_, _| {
            Rc::new(MenuState::default())
        });
        let state = retained.read(cx).clone();
        let open = state.open.get();
        let theme = window_material_theme(window, cx);
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
        let trigger_focus = self
            .focus
            .clone()
            .unwrap_or_else(|| focus_handle(&id, "trigger-focus", window, cx));
        let scope = focus_handle(&id, "menu-scope", window, cx);
        let handles = self
            .items
            .iter()
            .map(|item| {
                let item_id = ElementId::from((id.clone(), item.id.clone()));
                (focus_handle(&item_id, "focus", window, cx), !item.disabled)
            })
            .collect::<Vec<_>>();

        let dismiss: Invoke = Rc::new({
            let state = state.clone();
            let trigger_focus = trigger_focus.clone();
            let scope = scope.clone();
            move |window, cx| {
                if state.open.replace(false) {
                    if scope.contains_focused(window, cx) {
                        trigger_focus.focus(window, cx);
                    }
                    window.refresh();
                }
            }
        });

        let activate = {
            let state = state.clone();
            let dismiss = dismiss.clone();
            let scope = scope.clone();
            let first = handles
                .iter()
                .find(|(_, enabled)| *enabled)
                .map(|(handle, _)| handle.clone());
            move |window: &mut Window, cx: &mut App| {
                if state.open.get() {
                    dismiss(window, cx);
                    return;
                }
                state.open.set(true);
                // Focus before paint so the first opened frame has a valid tab target.
                if let Some(first) = &first {
                    first.focus(window, cx);
                } else {
                    scope.focus(window, cx);
                }
                window.refresh();
            }
        };
        let (initial_size, initial_radius, initial_icon) = match self.size {
            FabSize::Small => (56., 16., 24.),
            FabSize::Medium => (80., 20., 28.),
            FabSize::Large => (96., 28., 36.),
        };
        let size = initial_size + (56. - initial_size) * reveal;
        let radius = initial_radius + (28. - initial_radius) * reveal;
        let icon_size = initial_icon + (20. - initial_icon) * reveal;
        let background = fill.blend(strong_fill.opacity(reveal));
        let foreground = ink.blend(strong_ink.opacity(reveal));
        let accessibility = activate.clone();
        let keyboard = activate.clone();
        let trigger = Button::new((id.clone(), "trigger"))
            .track_focus(&trigger_focus)
            .accessibility_label(if open {
                self.close_label.clone()
            } else {
                self.open_label.clone()
            })
            .aria_expanded(open)
            .disabled(self.items.is_empty())
            .size(px(size))
            .rounded(px(radius))
            .shadow(elevation_shadows(Elevation::Level3.dp(), 1., shadow))
            .when(!self.items.is_empty(), |trigger| {
                trigger
                    .on_click(move |_, window, cx| activate(window, cx))
                    .on_a11y_action(gpui::AccessibleAction::Click, move |_, window, cx| {
                        accessibility(window, cx)
                    })
                    .capture_key_down(move |event, window, cx| {
                        activate_key(event, window, cx, &keyboard)
                    })
            })
            .child(Icon::new(if open { IconName::Close } else { self.icon }).size(px(icon_size)));
        let trigger = style_button(
            trigger,
            background,
            foreground,
            ring,
            !self.items.is_empty(),
        );
        let measure_trigger = state.clone();
        let trigger = div().relative().child(trigger).child(
            canvas(
                move |bounds, _, _| measure_trigger.trigger_bounds.set(bounds),
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        );

        let style = MenuStyle {
            fill,
            ink,
            ring,
            rtl: direction == Direction::Rtl,
            max_width: (window.viewport_size().width - px(24.)).clamp(px(0.), px(360.)),
            alpha,
            reveal,
        };
        let exit_bounds = state.bounds.get();
        let mut popup = Popup::new((id.clone(), "popup"), trigger)
            .anchor(logical_anchor(Anchor::BottomRight, direction))
            .offset(px(8.));
        if open {
            let outside = dismiss.clone();
            let exclude_trigger = state.clone();
            let record_bounds = state.clone();
            let panel = div()
                .id((id.clone(), "panel"))
                .occlude()
                .on_mouse_down_out(move |event, window, cx| {
                    if !exclude_trigger
                        .trigger_bounds
                        .get()
                        .contains(&event.position)
                    {
                        outside(window, cx);
                    }
                })
                .child(self.render_actions(&handles, &state, &dismiss, style, window));
            popup = popup
                .on_position(move |position, _| record_bounds.bounds.set(position.bounds))
                .content(panel);
        }
        let root = div()
            .id(id.clone())
            .flex_none()
            .when(open, |root| {
                root.on_key_down({
                    let dismiss = dismiss.clone();
                    move |event, window, cx| {
                        if event.keystroke.key == "escape" {
                            dismiss(window, cx);
                            cx.stop_propagation();
                        }
                    }
                })
            })
            .child(popup)
            .children(
                (!open && alpha > 0. && exit_bounds.size.width > px(0.)).then(|| {
                    let anchor = if style.rtl {
                        Anchor::BottomLeft
                    } else {
                        Anchor::BottomRight
                    };
                    let x = if style.rtl {
                        exit_bounds.left()
                    } else {
                        exit_bounds.right()
                    };
                    deferred(
                        Positioner::corner(anchor, point(x, exit_bounds.bottom()))
                            .child(self.render_actions(&handles, &state, &dismiss, style, window)),
                    )
                    .with_priority(100)
                }),
            );
        if open {
            root.focus_trap((id, "focus-trap"), &scope)
                .into_any_element()
        } else {
            root.into_any_element()
        }
    }
}

fn focus_handle(
    id: &ElementId,
    part: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> FocusHandle {
    window
        .use_keyed_state((id.clone(), part), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone()
}

fn activate_key(
    event: &gpui::KeyDownEvent,
    window: &mut Window,
    cx: &mut App,
    command: &dyn Fn(&mut Window, &mut App),
) {
    if !event.keystroke.modifiers.modified()
        && matches!(event.keystroke.key.as_str(), "enter" | "space")
    {
        if !event.is_held {
            command(window, cx);
        }
        cx.stop_propagation();
    }
}

fn style_button(
    button: Button,
    background: Hsla,
    foreground: Hsla,
    ring: Hsla,
    enabled: bool,
) -> Button {
    button
        .bg(background)
        .text_color(foreground)
        .border_2()
        .border_color(gpui::transparent_black())
        .when(enabled, |button| {
            button
                .hover(move |style| {
                    style.bg(background.blend(foreground.opacity(StateLayer::Hover.opacity())))
                })
                .active(move |style| {
                    style.bg(background.blend(foreground.opacity(StateLayer::Pressed.opacity())))
                })
                .focus_visible(move |style| style.border_color(ring))
        })
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

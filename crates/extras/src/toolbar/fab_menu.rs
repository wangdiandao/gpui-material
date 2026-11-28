use std::{cell::Cell, rc::Rc};

use gpui::{
    Anchor, AnyElement, App, Bounds, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, SharedString, Styled, Window, deferred, div, point, px,
};
use gpui_base::{FocusTrapElement, Positioner};
use gpui_material_components::{FabColor, FabSize, FabVisibilityAnimation, Icon, IconName};
use gpui_material_foundation::{
    ThemeScope, animate, color, current_material_theme, logical_anchor,
};
use gpui_material_theme::{ColorRole, CornerShape, Direction, Elevation, MotionKind, MotionSpeed};

use super::{Invoke, activate_key, focus_handle, style_button};

mod menu;
mod trigger;

use trigger::{TriggerContent, TriggerRenderContext};

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
    /// Creates a FAB menu item with a stable `id`, accessible `label`, and Material `icon`.
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

    /// Sets a custom icon element for this menu item.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    /// Replaces the item's visible label. Its accessible name remains the same.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    /// Overrides the container and content colors for this menu item.
    pub fn colors(mut self, colors: FabMenuItemColors) -> Self {
        self.colors = colors;
        self
    }

    /// Disables activation and keyboard focus navigation on this menu item when `true`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Registers a callback invoked when this menu item is activated.
    pub fn on_activate(mut self, invoke: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.invoke = Some(Rc::new(invoke));
        self
    }
}

/// Optional colors for an individual FAB menu action.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FabMenuItemColors {
    /// Background fill color when enabled.
    pub container: Option<Hsla>,
    /// Icon and label color when enabled.
    pub content: Option<Hsla>,
    /// Background fill color when disabled.
    pub disabled_container: Option<Hsla>,
    /// Icon and label color when disabled.
    pub disabled_content: Option<Hsla>,
}

/// Horizontal alignment of a FAB menu relative to its trigger.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FabMenuAlignment {
    /// Aligns the menu stack with the leading edge of the trigger (mirrored in RTL).
    Start,
    /// Centers the menu stack above the trigger.
    Center,
    /// Aligns the menu stack with the trailing edge of the trigger (mirrored in RTL).
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
    /// Trigger button side length in logical pixels.
    pub size: Option<Pixels>,
    /// Corner shape of the trigger button.
    pub shape: Option<CornerShape>,
    /// Icon size in logical pixels.
    pub icon_size: Option<Pixels>,
    /// Background fill color of the trigger button.
    pub container_color: Option<Hsla>,
    /// Foreground icon/content color of the trigger button.
    pub content_color: Option<Hsla>,
    /// Shadow elevation of the trigger button.
    pub elevation: Option<Elevation>,
}

/// Interpolated collapsed and expanded trigger appearances.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FabMenuTriggerStyle {
    /// Trigger appearance when the menu is collapsed.
    pub collapsed: FabMenuTriggerAppearance,
    /// Trigger appearance when the menu is expanded.
    pub expanded: FabMenuTriggerAppearance,
}

/// A floating action button that opens a stack of commands above it.
/// Provide two to six actions with stable IDs.
#[derive(IntoElement)]
pub struct FabMenu {
    id: ElementId,
    open_label: SharedString,
    close_label: Option<SharedString>,
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
    /// Creates a FAB menu with an element `id`, collapsed accessible `open_label`, `icon`, and `items`.
    pub fn new(
        id: impl Into<ElementId>,
        open_label: impl Into<SharedString>,
        icon: IconName,
        items: impl IntoIterator<Item = FabMenuItem>,
    ) -> Self {
        Self {
            id: id.into(),
            open_label: open_label.into(),
            close_label: None,
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

    /// Sets the accessible label used on the trigger button while the menu is expanded.
    /// Defaults to [`gpui_material_theme::MaterialStrings::close_actions`] (`"Close actions"`).
    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = Some(label.into());
        self
    }

    /// Replaces the collapsed trigger icon with a custom icon element scaled to the trigger's icon size.
    pub fn open_icon(mut self, icon: impl IntoElement) -> Self {
        self.open_content = Some(TriggerContent::Icon(icon.into_any_element()));
        self
    }

    /// Replaces the expanded close icon with a custom icon element scaled to the trigger's icon size.
    pub fn close_icon(mut self, icon: impl IntoElement) -> Self {
        self.close_content = Some(TriggerContent::Icon(icon.into_any_element()));
        self
    }

    /// Replaces the collapsed trigger's inner content with arbitrary decorative artwork.
    pub fn open_content(mut self, content: impl IntoElement) -> Self {
        self.open_content = Some(TriggerContent::Custom(content.into_any_element()));
        self
    }

    /// Replaces the expanded trigger's inner content with arbitrary decorative artwork.
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

    /// Sets horizontal alignment of the expanded menu stack relative to the trigger.
    pub fn alignment(mut self, alignment: FabMenuAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Overrides the collapsed and expanded appearance endpoints of the trigger button.
    pub fn trigger_style(mut self, style: FabMenuTriggerStyle) -> Self {
        self.trigger_style = style;
        self
    }

    /// Controls the presence of the trigger independently of expansion.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = Some(visible);
        self
    }

    /// Sets the animation style used when toggling [`Self::visible`].
    pub fn visibility_animation(mut self, animation: FabVisibilityAnimation) -> Self {
        self.visibility_animation = animation;
        self
    }

    /// Sets the collapsed [`FabSize`] preset (`Small`, `Medium`, or `Large`). Defaults to `Small`.
    pub fn size(mut self, size: FabSize) -> Self {
        self.size = size;
        self
    }

    /// Sets the [`FabColor`] role preset for the trigger and default menu items. Defaults to `Primary`.
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

#[cfg(test)]
mod tests {
    use gpui::{ElementId, SharedString};

    use super::menu::item_part_id;

    #[test]
    fn fab_menu_item_part_ids_are_scoped_by_menu_id() {
        let first_icon = item_part_id(
            &ElementId::from("menu-a"),
            "fab-menu-icon",
            SharedString::from("copy"),
        );
        let second_icon = item_part_id(
            &ElementId::from("menu-b"),
            "fab-menu-icon",
            SharedString::from("copy"),
        );
        let first_label = item_part_id(
            &ElementId::from("menu-a"),
            "fab-menu-label",
            SharedString::from("copy"),
        );
        assert_ne!(first_icon, second_icon);
        assert_ne!(first_icon, first_label);
    }
}

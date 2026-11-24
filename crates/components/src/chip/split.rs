use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, Stateful, Styled, Window, canvas, div, point, prelude::FluentBuilder, px,
};
use gpui_material_foundation::{
    InteractionState, InteractionTracker, ThemeScope, animate, color, current_material_theme,
};
use gpui_material_theme::{
    ColorRole, CornerRole, Direction, Elevation, LogicalEdges, MotionKind, MotionSpeed,
};

use crate::action::{
    Action, ActionControl, ActionLayout, ActionStyle, Activate, InvokeAction, SegmentShape,
    icon_slot, interpolated_corners, paint_container_surface,
};
use crate::control_style::{ButtonStyle, ControlBorder, ControlElevations};
use crate::{Icon, IconName};

use super::palette::{ChipPalette, action_colors};
use super::render::logical_edges;

pub(super) fn split_action_style(style: ActionStyle, min_width: f32) -> ActionStyle {
    ActionStyle {
        min_width,
        elevation: Elevation::Level0,
        hover_elevation: Elevation::Level0,
        ..style
    }
}

pub(super) fn measure_height(height: Rc<Cell<Pixels>>) -> gpui::Canvas<()> {
    canvas(
        move |bounds, _, _| height.set(height.get().max(bounds.size.height)),
        |_, (), _, _| {},
    )
    .absolute()
    .inset_0()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_split_chip(
    id: ElementId,
    row: Stateful<Div>,
    primary: ActionControl,
    remove_label: SharedString,
    on_remove: Activate,
    remove_content: Option<AnyElement>,
    disabled: bool,
    dragged: bool,
    selected: bool,
    direction: Direction,
    padding: LogicalEdges,
    explicit_padding: Option<LogicalEdges>,
    style: ActionStyle,
    layout: ActionLayout,
    palette: ChipPalette,
    elevations: ControlElevations,
    visual_height: Rc<Cell<Pixels>>,
    tracker: Option<InteractionTracker>,
    window: &mut Window,
    cx: &mut App,
) -> Stateful<Div> {
    let primary_focus = primary.focus_handle(window, cx);
    let key_remove = on_remove.clone();
    let remove = ActionControl {
        id: ElementId::from((id.clone(), "remove")),
        label: remove_label,
        focus: None,
        disabled,
        action: Action::Invoke(Some(InvokeAction::Full(on_remove))),
        style: split_action_style(style, 32.),
        content: logical_edges(
            div().relative().pt(px(padding.top)).pb(px(padding.bottom)),
            px(0.),
            explicit_padding.map_or(px(0.), |padding| px(padding.end)),
            direction,
        )
        .child(remove_content.unwrap_or_else(|| {
            icon_slot(Icon::new(IconName::Close).into_any_element(), 18.).into_any_element()
        }))
        .child(measure_height(visual_height.clone()))
        .into_any_element(),
    };
    let primary_state = primary.interaction_snapshot(dragged, window, cx);
    let remove_state = remove.interaction_snapshot(dragged, window, cx);
    let mut interaction = primary_state;
    interaction.hovered |= remove_state.hovered;
    interaction.pressed |= remove_state.pressed;
    interaction.focus_visible |= remove_state.focus_visible;
    let surface = SplitSurface {
        id,
        style,
        layout,
        direction,
        selected,
        container: palette.container,
        border: ControlBorder {
            width: palette.border_width,
            color: palette.border_color,
        },
        elevations: ControlElevations {
            dragged: elevations.dragged.or(Some(Elevation::Level4)),
            ..elevations
        },
        visual_height,
    }
    .render(interaction, window, cx);
    let primary_layout = ActionLayout {
        segment: Some(SegmentShape {
            leading: true,
            trailing: false,
            outer: style.corner,
            inner: CornerRole::None,
            pressed: CornerRole::None,
        }),
        ..layout
    };
    let remove_layout = ActionLayout {
        segment: Some(SegmentShape {
            leading: false,
            trailing: true,
            outer: style.corner,
            inner: CornerRole::None,
            pressed: CornerRole::None,
        }),
        presentation_selected: Some(selected),
        appearance: ButtonStyle {
            shapes: layout.appearance.shapes,
            colors: action_colors(palette.trailing, gpui::transparent_black()),
            ..Default::default()
        },
        ..layout
    };
    row.when(!disabled, |row| {
        let on_remove =
            ThemeScope::bind_action(move |window, cx| key_remove(window, cx), window, cx);
        row.on_key_down(move |event, window, cx| {
            if !event.keystroke.modifiers.modified()
                && matches!(event.keystroke.key.as_str(), "backspace" | "delete")
                && primary_focus.is_focused(window)
            {
                cx.stop_propagation();
                on_remove(window, cx);
            }
        })
    })
    .child(surface)
    .child(primary.render_with_tracker(1., primary_layout, tracker, window, cx))
    .child(remove.render_with_tracker(1., remove_layout, None, window, cx))
}

/// A split chip shares one container around its two independent actions.
struct SplitSurface {
    id: ElementId,
    style: ActionStyle,
    layout: ActionLayout,
    direction: Direction,
    selected: bool,
    container: Hsla,
    border: ControlBorder,
    elevations: ControlElevations,
    visual_height: Rc<Cell<Pixels>>,
}

impl SplitSurface {
    fn render(
        self,
        state: InteractionState,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement + use<> {
        let theme = current_material_theme(window, cx);
        let shadow = color(theme.colors.color(ColorRole::Shadow));
        let instant = cx.reduce_motion() || state.disabled;
        let effects = theme
            .motion
            .spec(MotionKind::Effects, MotionSpeed::Fast, instant);
        let spatial = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Fast, instant);
        let pressed = animate(
            (self.id.clone(), "chip-press"),
            f32::from(state.pressed),
            spatial,
            window,
            cx,
        )
        .clamp(0., 1.);
        let selected = if self.style.selected_corner.is_some() {
            animate(
                (self.id.clone(), "chip-selection"),
                f32::from(self.selected),
                spatial,
                window,
                cx,
            )
            .clamp(0., 1.)
        } else {
            0.
        };
        let default_elevation = if state.disabled {
            Elevation::Level0
        } else if state.hovered && !state.pressed {
            self.style.hover_elevation
        } else {
            self.style.elevation
        };
        let elevation = self
            .elevations
            .for_state(state)
            .unwrap_or(default_elevation);
        let elevation = animate(
            (self.id.clone(), "chip-elevation"),
            elevation.dp(),
            effects,
            window,
            cx,
        )
        .max(0.);
        let outline = animate(
            (self.id.clone(), "chip-outline"),
            f32::from(self.border.width),
            effects,
            window,
            cx,
        )
        .max(0.);
        canvas(
            |_, _, _| (),
            move |mut bounds, (), window, _| {
                let height = self.visual_height.get().min(bounds.size.height);
                bounds.origin += point(px(0.), (bounds.size.height - height) / 2.);
                bounds.size.height = height;
                let shape =
                    interpolated_corners(bounds, self.style, selected, pressed, 1., self.layout);
                let radii = shape
                    .radii(
                        f32::from(bounds.size.width),
                        f32::from(bounds.size.height),
                        self.direction,
                    )
                    .map(px);
                paint_container_surface(
                    bounds,
                    radii,
                    elevation,
                    1.,
                    shadow,
                    self.container,
                    outline,
                    self.border.color,
                    window,
                );
            },
        )
        .absolute()
        .inset_0()
    }
}

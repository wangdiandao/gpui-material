use std::{cell::Cell, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, Axis, BorderStyle, ClickEvent, Corners, ElementId, Entity,
    FocusHandle, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role,
    SharedString, StatefulInteractiveElement, Styled, Window, accesskit::Toggled, canvas, div,
    prelude::FluentBuilder, px, quad,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{
    InteractionFeedback, InteractionSource, Shape, animate, color, elevation_shadows,
    move_roving_focus, window_material_theme,
};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Direction, MotionKind, MotionSpec, MotionSpeed, TypeRole,
};

use crate::{
    Button, ButtonVariant, ControlSize, SwipeToDismiss, SwipeToDismissState, Text,
    action::{Activate, activate_feedback, feedback_target, ripple_state},
    button_style::finite_length,
};

mod long_press;
mod style;

use long_press::LongPress;
pub use style::{
    ListItemAlignment, ListItemColors, ListItemElevations, ListItemPadding, ListItemShapes,
    ListItemStateColors,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ListVariant {
    #[default]
    Standard,
    Segmented,
}

type ValueChange = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type SwipeConfig = Box<dyn FnOnce(SwipeToDismiss) -> SwipeToDismiss>;

enum Slot {
    Text(SharedString),
    Custom(AnyElement),
}

impl Slot {
    fn render(self, id: &'static str, role: TypeRole, tint: Hsla, wrap: bool) -> AnyElement {
        let content = match self {
            Self::Text(value) => Text::new(value)
                .id(id)
                .role(role)
                .when(wrap, Text::wrap)
                .into_any_element(),
            Self::Custom(element) => element,
        };
        inherit_color((ElementId::from(id), "slot"), content, Some(tint))
    }

    fn has_line_break(&self) -> bool {
        matches!(self, Self::Text(value) if value.contains('\n'))
    }
}

struct Expansion {
    expanded: bool,
    content: AnyElement,
}

struct SecondaryAction {
    label: SharedString,
    handler: Activate,
}

struct RowSwipe {
    state: Entity<SwipeToDismissState>,
    configure: SwipeConfig,
}

/// A Standard or Segmented list row. Values and data changes belong to the caller.
#[derive(IntoElement)]
pub struct ListItem {
    id: ElementId,
    label: SharedString,
    content: Option<AnyElement>,
    supporting: Option<Slot>,
    overline: Option<Slot>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    activation: Option<Activate>,
    checked_change: Option<ValueChange>,
    expanded_change: Option<ValueChange>,
    long_press: Option<SecondaryAction>,
    long_press_button: Option<ElementId>,
    expansion: Option<Expansion>,
    swipe: Option<RowSwipe>,
    checked: Option<bool>,
    selected: bool,
    disabled: bool,
    dragged: bool,
    focus: Option<(FocusHandle, bool)>,
    shapes: ListItemShapes,
    colors: ListItemColors,
    elevations: ListItemElevations,
    alignment: ListItemAlignment,
    padding: Option<ListItemPadding>,
    source: Option<Entity<InteractionSource>>,
}

impl ListItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            content: None,
            supporting: None,
            overline: None,
            leading: None,
            trailing: None,
            activation: None,
            checked_change: None,
            expanded_change: None,
            long_press: None,
            long_press_button: None,
            expansion: None,
            swipe: None,
            checked: None,
            selected: false,
            disabled: false,
            dragged: false,
            focus: None,
            shapes: ListItemShapes::default(),
            colors: ListItemColors::default(),
            elevations: ListItemElevations::default(),
            alignment: ListItemAlignment::Auto,
            padding: None,
            source: None,
        }
    }

    /// Replaces the visual headline. The constructor's label still names the row action.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(Slot::Text(text.into()));
        self
    }

    pub fn supporting_content(mut self, content: impl IntoElement) -> Self {
        self.supporting = Some(Slot::Custom(content.into_any_element()));
        self
    }

    pub fn overline(mut self, text: impl Into<SharedString>) -> Self {
        self.overline = Some(Slot::Text(text.into()));
        self
    }

    pub fn overline_content(mut self, content: impl IntoElement) -> Self {
        self.overline = Some(Slot::Custom(content.into_any_element()));
        self
    }

    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self
    }

    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// The controlled checked value, separate from the row's selection state.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Requests the opposite checked value on activation. The current value defaults to false.
    pub fn on_checked_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.checked.get_or_insert(false);
        self.checked_change = Some(Rc::new(handler));
        self
    }

    /// Supplies controlled disclosure content. It is mounted below the row while expanded.
    pub fn expanded_content(mut self, expanded: bool, content: impl IntoElement) -> Self {
        self.expansion = Some(Expansion {
            expanded,
            content: content.into_any_element(),
        });
        self
    }

    pub fn on_expanded_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.expanded_change = Some(Rc::new(handler));
        self
    }

    /// Names the secondary command invoked by a pointer hold or Shift+F10.
    pub fn on_long_press(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.long_press = Some(SecondaryAction {
            label: label.into(),
            handler: Rc::new(handler),
        });
        self
    }

    /// Adds a visible button for the named secondary command in the trailing area.
    pub fn long_press_button(mut self, id: impl Into<ElementId>) -> Self {
        self.long_press_button = Some(id.into());
        self
    }

    /// Composes the row with a retained SwipeToDismiss instance. Use its configuration
    /// callback for direction, background and dismissal, keeping the state's owner unchanged.
    pub fn swipe_to_dismiss(
        mut self,
        state: &Entity<SwipeToDismissState>,
        configure: impl FnOnce(SwipeToDismiss) -> SwipeToDismiss + 'static,
    ) -> Self {
        self.swipe = Some(RowSwipe {
            state: state.clone(),
            configure: Box::new(configure),
        });
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.activation = Some(Rc::new(handler));
        self
    }

    pub fn shapes(mut self, shapes: ListItemShapes) -> Self {
        self.shapes = shapes;
        self
    }

    pub fn colors(mut self, colors: ListItemColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn elevations(mut self, elevations: ListItemElevations) -> Self {
        self.elevations = elevations;
        self
    }

    pub fn vertical_alignment(mut self, alignment: ListItemAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn content_padding(mut self, padding: ListItemPadding) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Reports a caller-owned drag. Clear the flag after completion or cancellation.
    pub fn dragged(mut self, dragged: bool) -> Self {
        self.dragged = dragged;
        self
    }

    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.source = Some(source.clone());
        self
    }

    /// Uses a retained focus handle and roving tab stop supplied by a virtual
    /// collection. The collection owns navigation across unrendered rows.
    pub fn track_focus(mut self, focus: &FocusHandle, tab_stop: bool) -> Self {
        self.focus = Some((focus.clone(), tab_stop));
        self
    }

    fn has_primary_action(&self) -> bool {
        self.activation.is_some()
            || self.checked_change.is_some()
            || self.expansion.is_some() && self.expanded_change.is_some()
    }

    fn takes_focus(&self, cx: &App) -> bool {
        !self.disabled
            && (self.has_primary_action() || self.long_press.is_some())
            && self
                .swipe
                .as_ref()
                .is_none_or(|swipe| swipe.state.read(cx).shows_foreground())
    }

    fn primary_action(&self) -> Option<Activate> {
        if !self.has_primary_action() {
            return None;
        }
        let action = self.activation.clone();
        let check = self.checked.zip(self.checked_change.clone());
        let expansion = self
            .expansion
            .as_ref()
            .map(|expansion| expansion.expanded)
            .zip(self.expanded_change.clone());
        Some(Rc::new(move |window, cx| {
            if let Some((checked, change)) = &check {
                change(!checked, window, cx);
            }
            if let Some((expanded, change)) = &expansion {
                change(!expanded, window, cx);
            }
            if let Some(action) = &action {
                action(window, cx);
            }
        }))
    }

    fn row(
        self,
        resting: CornerShape,
        selected_shape: CornerShape,
        focus: Option<(FocusHandle, bool)>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        window.with_element_namespace((self.id.clone(), "material-list-row"), |window| {
            self.render_row(resting, selected_shape, focus, window, cx)
        })
    }

    fn render_row(
        self,
        resting: CornerShape,
        selected_shape: CornerShape,
        focus: Option<(FocusHandle, bool)>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let (focus, tab_stop) = focus.unwrap_or_else(|| {
            let focus = window
                .use_keyed_state("focus", cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone();
            (focus, true)
        });
        let primary = self.primary_action();
        let interactive = primary.is_some() || self.long_press.is_some();
        let visible = self
            .swipe
            .as_ref()
            .is_none_or(|swipe| swipe.state.read(cx).shows_foreground());
        let enabled = !self.disabled && visible;
        let dragged = enabled
            && (self.dragged
                || self
                    .swipe
                    .as_ref()
                    .is_some_and(|swipe| swipe.state.read(cx).is_dragging()));
        let feedback = ripple_state(&self.id, interactive && enabled, window, cx);
        let tracker = self
            .source
            .as_ref()
            .filter(|_| visible)
            .map(|source| InteractionSource::track(source, &self.id, window, cx));
        let interaction = {
            let mut feedback = feedback.borrow_mut();
            feedback.set_dragged(dragged);
            feedback.track(enabled, &focus, tracker, window, cx)
        };

        let long_press = self.long_press.as_ref().map(|long_press| {
            LongPress::mount(
                (enabled && !dragged).then(|| long_press.handler.clone()),
                window,
                cx,
            )
        });
        let theme = window_material_theme(window, cx);
        let layout = theme.layout;
        let direction = layout.direction;
        let motion = theme.motion;
        let reduced = cx.reduce_motion();
        let selected = self.selected || self.checked == Some(true);
        let shape = self
            .shapes
            .resolve(resting, selected_shape, selected, interaction);
        let effects = motion.spec(MotionKind::Effects, MotionSpeed::Default, reduced);
        let spatial = motion.spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let selection =
            animate("selection", f32::from(selected), effects, window, cx).clamp(0., 1.);
        let scheme = &window_material_theme(window, cx).colors;
        let tone = |role| color(scheme.color(role));
        let ring = tone(ColorRole::Secondary);
        let shadow = tone(ColorRole::Shadow);
        let colors = self.colors.resolve(selected, selection, interaction, tone);
        let layer = animate(
            "opacity",
            interaction
                .layer_with_ripple(reduced)
                .map_or(0., |layer| layer.opacity()),
            if enabled {
                effects
            } else {
                MotionSpec::Instant
            },
            window,
            cx,
        )
        .clamp(0., 1.);
        let elevation = animate(
            "elevation",
            self.elevations.resolve(interaction).dp(),
            if enabled {
                spatial
            } else {
                MotionSpec::Instant
            },
            window,
            cx,
        )
        .max(0.);
        let text_height = window
            .use_keyed_state("text-height", cx, |_, _| Rc::new(Cell::new(px(0.))))
            .read(cx)
            .clone();
        let three_lines = self.overline.is_some() && self.supporting.is_some()
            || self.supporting.as_ref().is_some_and(Slot::has_line_break)
            || text_height.get() > px(44.);
        let height = match (self.overline.is_some(), self.supporting.is_some()) {
            _ if three_lines => 88.,
            (false, false) => 56.,
            _ => 72.,
        };
        let radii = match shape {
            CornerShape::Radii(radii) => radii,
            CornerShape::Full => [height * 0.5; 4],
        };
        // Include the measured pill extent separately so the final shape follows
        // custom content that raises the row above its minimum height.
        let full = animate(
            "full",
            f32::from(shape == CornerShape::Full),
            spatial,
            window,
            cx,
        )
        .clamp(0., 1.);
        let radii = std::array::from_fn(|ix| {
            animate(["ts", "te", "be", "bs"][ix], radii[ix], spatial, window, cx).max(0.)
        });
        let geometry = feedback.clone();
        let ink = feedback.clone();
        let visual = canvas(
            move |bounds, window, _| {
                let extent = f32::from(bounds.size.width.min(bounds.size.height)) * 0.5;
                let shape =
                    CornerShape::Radii(radii.map(|radius| radius + (extent - radius) * full));
                let [top_left, top_right, bottom_right, bottom_left] = shape
                    .radii(
                        bounds.size.width.into(),
                        bounds.size.height.into(),
                        direction,
                    )
                    .map(px);
                let feedback_shape = enabled
                    .then(|| Shape::rounded(bounds, shape, direction))
                    .flatten();
                geometry.borrow_mut().set_shape(feedback_shape, window);
                Corners {
                    top_left,
                    top_right,
                    bottom_right,
                    bottom_left,
                }
            },
            move |bounds, corners, window, cx| {
                if elevation > 0. {
                    window.paint_drop_shadows(
                        bounds,
                        corners,
                        &elevation_shadows(elevation, 1., shadow),
                    );
                }
                window.paint_quad(quad(
                    bounds,
                    corners,
                    colors.container,
                    px(0.),
                    gpui::transparent_black(),
                    BorderStyle::Solid,
                ));
                let feedback = ink.borrow();
                if let Some(shape) = &feedback.shape {
                    // Base's outer ring keeps the existing list focus geometry.
                    let mut visual_interaction = interaction;
                    visual_interaction.focus_visible = false;
                    InteractionFeedback::new(visual_interaction, colors.content, ring)
                        .paint_with_opacity(
                            shape,
                            feedback.ripple.as_ref(),
                            layer,
                            cx.background_executor().now(),
                            reduced,
                            window,
                        );
                }
            },
        )
        .absolute()
        .inset_0();

        let content = div()
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .text_color(colors.content)
            .children(
                self.overline.map(|slot| {
                    slot.render("overline", TypeRole::LabelSmall, colors.overline, false)
                }),
            )
            .child(div().id("headline-slot").child(
                self.content.unwrap_or_else(|| {
                    Text::new(self.label.clone()).id("label").into_any_element()
                }),
            ))
            .children(self.supporting.map(|slot| {
                slot.render("supporting", TypeRole::BodyMedium, colors.supporting, true)
            }))
            .children(
                long_press
                    .as_ref()
                    .filter(|_| enabled && !dragged)
                    .map(|press| press.target()),
            )
            .child(
                canvas(
                    move |bounds, window, _| {
                        if text_height.replace(bounds.size.height) != bounds.size.height {
                            window.refresh();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            );

        let padding = self.padding.unwrap_or_else(|| {
            let vertical = px(if three_lines { 12. } else { 10. });
            ListItemPadding {
                top: vertical,
                bottom: vertical,
                ..ListItemPadding::default()
            }
        });
        let (left, right) =
            layout.horizontal(finite_length(padding.start), finite_length(padding.end));
        let press_button =
            self.long_press
                .as_ref()
                .zip(self.long_press_button)
                .map(|(press, id)| {
                    let invoke = press.handler.clone();
                    Button::new(id, press.label.clone())
                        .variant(ButtonVariant::Text)
                        .size(ControlSize::ExtraSmall)
                        .disabled(!enabled || dragged)
                        .on_activate(move |window, cx| invoke(window, cx))
                });
        let alignment = if self.alignment == ListItemAlignment::Auto {
            if three_lines {
                ListItemAlignment::Start
            } else {
                ListItemAlignment::Center
            }
        } else {
            self.alignment
        };
        let surface = div()
            .id("list-surface")
            .relative()
            .min_h(px(height))
            .w_full()
            .flex()
            .map(|row| match alignment {
                ListItemAlignment::Start => row.items_start(),
                ListItemAlignment::End => row.items_end(),
                _ => row.items_center(),
            })
            .gap(px(12.))
            .pl(left)
            .pr(right)
            .pt(finite_length(padding.top))
            .pb(finite_length(padding.bottom))
            .when(direction == Direction::Rtl, Styled::flex_row_reverse)
            .child(visual)
            .children(
                self.leading
                    .map(|element| inherit_color("leading-slot", element, colors.leading)),
            )
            .child(content)
            .children(
                self.trailing
                    .map(|element| inherit_color("trailing-slot", element, colors.trailing)),
            )
            .children(press_button.map(|button| div().id("long-press-slot").child(button)));

        let expanded = self.expansion.as_ref().map(|expansion| expansion.expanded);
        let checked = self.checked.map(|checked| {
            if checked {
                Toggled::True
            } else {
                Toggled::False
            }
        });
        let expanded_content = self.expansion.and_then(|expansion| {
            let scope = window
                .use_keyed_state("disclosure-focus", cx, |_, cx| {
                    cx.focus_handle().tab_stop(false)
                })
                .read(cx)
                .clone();
            if !expansion.expanded && interactive && enabled && scope.contains_focused(window, cx) {
                focus.focus(window, cx);
            }
            expansion.expanded.then(|| {
                div()
                    .id("list-expanded-content")
                    .track_focus(&scope)
                    .child(expansion.content)
            })
        });
        let row = if interactive {
            let activate = feedback.clone();
            let mut control = gpui_base::Button::new((self.id.clone(), "activate"))
                .track_focus(&focus)
                .tab_stop(tab_stop)
                .w_full()
                .accessibility_label(self.label.clone())
                .disabled(!enabled)
                .selected(self.selected)
                .when_some(checked, |control, checked| {
                    control.role(Role::CheckBox).aria_toggled(checked)
                })
                .when_some(expanded, |control, expanded| {
                    control.aria_expanded(expanded)
                })
                .border_2()
                .border_color(gpui::transparent_black())
                .rounded_md()
                .focus_visible(move |style| style.border_color(ring))
                .when(enabled && !dragged, |control| {
                    feedback_target(control, feedback, true, Some(focus.clone()))
                });
            if let Some(primary) = primary.filter(|_| enabled && !dragged) {
                let invoke = primary.clone();
                let hold = long_press.clone();
                control = control
                    .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                        invoke(window, cx)
                    })
                    .on_click(move |event, window, cx| {
                        cx.stop_propagation();
                        let native_drag =
                            matches!(event, ClickEvent::Mouse(_)) && cx.has_active_drag();
                        let allowed = !native_drag
                            && !activate.borrow().blocks_activation(event)
                            && !hold
                                .as_ref()
                                .is_some_and(|hold| hold.blocks_activation(event));
                        if allowed || matches!(event, ClickEvent::Mouse(_)) {
                            activate_feedback(event, &activate, window, cx);
                        }
                        if allowed {
                            primary(window, cx);
                        }
                    });
            }
            if let Some((expanded, change)) = expanded
                .zip(self.expanded_change)
                .filter(|_| enabled && !dragged)
            {
                control = control.on_a11y_action(
                    if expanded {
                        AccessibleAction::Collapse
                    } else {
                        AccessibleAction::Expand
                    },
                    move |_, window, cx| change(!expanded, window, cx),
                );
            }
            if let Some((secondary, hold)) = self
                .long_press
                .zip(long_press)
                .filter(|_| enabled && !dragged)
            {
                let invoke = secondary.handler.clone();
                control = control
                    .aria_description(secondary.label)
                    .on_a11y_action(AccessibleAction::ShowContextMenu, move |_, window, cx| {
                        invoke(window, cx)
                    })
                    .on_key_down(move |event, window, cx| {
                        if !focus.is_focused(window) {
                            return;
                        }
                        if event.keystroke.key == "escape" {
                            hold.cancel();
                        }
                        if is_long_press_shortcut(&event.keystroke) {
                            cx.stop_propagation();
                            if !event.is_held {
                                (secondary.handler)(window, cx);
                            }
                        }
                    });
            }
            control.child(surface).into_any_element()
        } else {
            surface
                .when_some(checked, |row, checked| {
                    row.role(Role::CheckBox).aria_toggled(checked)
                })
                .when_some(expanded, |row, expanded| {
                    row.when(checked.is_none(), |row| row.role(Role::Group))
                        .aria_expanded(expanded)
                })
                .when(
                    self.disabled && (checked.is_some() || expanded.is_some()),
                    |row| row.a11y_synthetic_children(|tree| tree.parent_node().set_disabled()),
                )
                .into_any_element()
        };
        let id = self.id;
        let row = div()
            .id(id.clone())
            .role(Role::ListItem)
            .test_support()
            .aria_selected(self.selected)
            .when(!interactive, |row| row.aria_label(self.label))
            .flex()
            .flex_col()
            .w_full()
            .child(row)
            .children(expanded_content);
        match self.swipe {
            Some(swipe) => (swipe.configure)(SwipeToDismiss::new((id, "swipe"), &swipe.state, row))
                .when(self.disabled, |swipe| swipe.gestures_enabled(false))
                .into_any_element(),
            None => row.into_any_element(),
        }
    }
}

fn inherit_color(id: impl Into<ElementId>, element: AnyElement, color: Option<Hsla>) -> AnyElement {
    // Each slot owns a separate identity scope, including elements that use
    // implicit IDs such as Text and Icon.
    div()
        .id(id)
        .when_some(color, Styled::text_color)
        .child(element)
        .into_any_element()
}

fn is_long_press_shortcut(keystroke: &gpui::Keystroke) -> bool {
    let modifiers = keystroke.modifiers;
    keystroke.key == "f10"
        && modifiers.shift
        && !modifiers.control
        && !modifiers.alt
        && !modifiers.platform
}

impl RenderOnce for ListItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus = self.focus.clone();
        self.row(
            CornerRole::None.shape(),
            CornerRole::None.shape(),
            focus,
            window,
            cx,
        )
    }
}

/// A list of identified rows. Applications can supply a visible window of rows
/// from their own virtualized collection.
#[derive(IntoElement)]
pub struct List {
    id: ElementId,
    items: Vec<ListItem>,
    variant: ListVariant,
}

impl List {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            variant: ListVariant::default(),
        }
    }

    pub fn variant(mut self, variant: ListVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ListItem>) -> Self {
        self.items.extend(items);
        self
    }
}

impl RenderOnce for List {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        window.with_element_namespace((self.id.clone(), "material-list"), |window| {
            let last = self.items.len().saturating_sub(1);
            let direction = window_material_theme(window, cx).layout.direction;
            let focus: Vec<_> = self
                .items
                .iter()
                .map(|item| {
                    window
                        .use_keyed_state((item.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                        .read(cx)
                        .clone()
                })
                .collect();
            let enabled: Vec<_> = self.items.iter().map(|item| item.takes_focus(cx)).collect();
            let tab_stop = focus
                .iter()
                .enumerate()
                .position(|(ix, focus)| enabled[ix] && focus.is_focused(window))
                .or_else(|| {
                    self.items
                        .iter()
                        .enumerate()
                        .position(|(ix, item)| enabled[ix] && item.selected)
                })
                .or_else(|| enabled.iter().position(|enabled| *enabled));
            let navigation = focus.clone();
            div()
                .id(self.id)
                .role(Role::List)
                .test_support()
                .flex()
                .flex_col()
                .w_full()
                .on_key_down(move |event, window, cx| {
                    // Nested controls keep their own arrow-key behavior.
                    if !navigation.iter().any(|focus| focus.is_focused(window)) {
                        return;
                    }
                    let items: Vec<_> = navigation
                        .iter()
                        .zip(&enabled)
                        .map(|(focus, enabled)| (focus, *enabled))
                        .collect();
                    if move_roving_focus(
                        &items,
                        &event.keystroke.key,
                        Axis::Vertical,
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
                .when(self.variant == ListVariant::Segmented, |list| {
                    list.gap(px(2.))
                })
                .children(self.items.into_iter().enumerate().map(|(ix, item)| {
                    let (resting, selected) = match self.variant {
                        ListVariant::Standard => {
                            (CornerRole::None.shape(), CornerRole::None.shape())
                        }
                        ListVariant::Segmented => {
                            let top = if ix == 0 { 16. } else { 4. };
                            let bottom = if ix == last { 16. } else { 4. };
                            (
                                CornerShape::Radii([top, top, bottom, bottom]),
                                CornerRole::Large.shape(),
                            )
                        }
                    };
                    item.row(
                        resting,
                        selected,
                        Some((focus[ix].clone(), Some(ix) == tab_stop)),
                        window,
                        cx,
                    )
                }))
        })
    }
}

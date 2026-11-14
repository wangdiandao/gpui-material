use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_material_foundation::{animate, color, window_material_theme};
use gpui_material_theme::{
    ColorRole, CornerRole, Direction, Elevation, MotionKind, MotionSpeed, TypeRole,
    TypographyEmphasis,
};

use crate::action::{
    Action, ActionControl, ActionStyle, Activate, Change, DisabledStyle, icon_slot,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ChipVariant {
    #[default]
    Assist,
    Filter,
    Input,
    Suggestion,
}

/// A Material chip. Filter and Input chips request controlled selection changes.
/// Filter and Input chips can expose a separate removal action with its own focus.
#[derive(IntoElement)]
pub struct Chip {
    id: ElementId,
    label: SharedString,
    variant: ChipVariant,
    selected: bool,
    elevated: bool,
    disabled: bool,
    leading: Option<AnyElement>,
    on_activate: Option<Activate>,
    on_change: Option<Change>,
    remove: Option<(SharedString, Activate)>,
}

impl Chip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: ChipVariant::default(),
            selected: false,
            elevated: false,
            disabled: false,
            leading: None,
            on_activate: None,
            on_change: None,
            remove: None,
        }
    }

    pub fn variant(mut self, variant: ChipVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn elevated(mut self, elevated: bool) -> Self {
        self.elevated = elevated;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn leading_icon(mut self, icon: impl IntoElement) -> Self {
        self.leading = Some(icon.into_any_element());
        self
    }

    pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }

    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Adds an independent removal action to a Filter or Input chip.
    /// The optional restoration target is focused before calling the owner.
    /// The owner can pass the next chip or a stable control outside the group.
    pub fn on_remove(
        mut self,
        name: impl Into<SharedString>,
        restore: Option<FocusHandle>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.remove = Some((
            name.into(),
            Rc::new(move |window, cx| {
                if let Some(restore) = &restore {
                    restore.focus(window, cx);
                }
                handler(window, cx);
            }),
        ));
        self
    }
}

impl RenderOnce for Chip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use ColorRole::*;
        let selectable = matches!(self.variant, ChipVariant::Filter | ChipVariant::Input);
        let selected = selectable && self.selected;
        let elevated = self.elevated && self.variant != ChipVariant::Input;
        let background = if selected {
            Some(SecondaryContainer)
        } else if elevated {
            Some(SurfaceContainerLow)
        } else {
            None
        };
        let foreground = if selected {
            OnSecondaryContainer
        } else if self.variant == ChipVariant::Assist {
            OnSurface
        } else {
            OnSurfaceVariant
        };
        let theme = window_material_theme(window, cx).clone();
        let direction = theme.layout.direction;
        let split = selectable && self.remove.is_some();
        let on_surface = color(theme.colors.color(OnSurface));
        let theme_color = |role| color(theme.colors.color(role));
        let idle = if elevated {
            theme_color(SurfaceContainerLow)
        } else {
            theme_color(SecondaryContainer).opacity(0.)
        };
        let effects = theme.motion.spec(
            MotionKind::Effects,
            MotionSpeed::Fast,
            cx.reduce_motion() || self.disabled,
        );
        let selection = if selectable {
            animate(
                (self.id.clone(), "chip-tint"),
                f32::from(selected),
                effects,
                window,
                cx,
            )
            .clamp(0., 1.)
        } else {
            0.
        };
        let container = if self.disabled {
            on_surface.opacity(if background.is_some() { 0.12 } else { 0. })
        } else {
            crate::motion::mix(idle, theme_color(SecondaryContainer), selection)
        };
        let border = if self.disabled {
            on_surface.opacity(0.12)
        } else {
            theme_color(OutlineVariant)
        };
        let outline_opacity = if elevated { 0. } else { 1. - selection };
        let height = px(theme.layout.density.visual_size(32.));
        let radius = px(CornerRole::Small.shape().radii(32., 32., direction)[0]);
        let style = ActionStyle {
            background: background.filter(|_| !split),
            background_color: (!split).then_some(container),
            foreground,
            disabled: DisabledStyle {
                foreground: OnSurface,
                background: background.filter(|_| !split).map(|_| (OnSurface, 0.12)),
                outline: (OnSurface, 0.12),
            },
            height: 32.,
            min_width: 0.,
            corner: CornerRole::Small,
            pressed_corner: CornerRole::Small,
            selected_corner: None,
            corner_motion: (MotionKind::Spatial, MotionSpeed::Fast),
            outline: f32::from(!split && !elevated && !selected),
            elevation: if elevated {
                Elevation::Level1
            } else {
                Elevation::Level0
            },
            hover_elevation: if elevated {
                Elevation::Level2
            } else if selected && self.variant == ChipVariant::Filter {
                Elevation::Level1
            } else {
                Elevation::Level0
            },
        };
        let filter_mark = if self.variant == ChipVariant::Filter && self.leading.is_none() {
            selection
        } else {
            0.
        };
        let leading_tint = if self.disabled {
            on_surface.opacity(0.38)
        } else {
            theme_color(match (self.variant, selected) {
                (ChipVariant::Input, false) => OnSurfaceVariant,
                (ChipVariant::Filter, true) => OnSecondaryContainer,
                _ => Primary,
            })
        };
        let leading = self
            .leading
            .map(|icon| {
                div()
                    .text_color(leading_tint)
                    .child(icon_slot(icon, 18.))
                    .into_any_element()
            })
            .or_else(|| {
                (filter_mark > 0.).then(|| {
                    div()
                        .w(px(26. * filter_mark))
                        .flex_none()
                        .overflow_hidden()
                        .opacity(filter_mark)
                        .child(icon_slot("\u{2713}".into_any_element(), 18.))
                        .into_any_element()
                })
            });
        // Label edges have 16 dp; an icon consumes the outer 8 dp. The
        // generated filter check animates both its slot and its leading inset.
        let start_padding = if filter_mark > 0. {
            16. - 8. * filter_mark
        } else if leading.is_some() {
            8.
        } else {
            16.
        };
        let end_padding = if split { 8. } else { 16. };
        let content = div()
            .flex()
            .items_center()
            .min_w_0()
            .when(
                self.variant != ChipVariant::Filter || filter_mark == 0.,
                |row| row.gap(px(8.)),
            )
            .pl(px(if direction == Direction::Rtl {
                end_padding
            } else {
                start_padding
            }))
            .pr(px(if direction == Direction::Rtl {
                start_padding
            } else {
                end_padding
            }))
            .text_size(px(14.))
            .when(direction == Direction::Rtl, Styled::flex_row_reverse)
            .children(leading)
            .child(crate::text::single_line(
                self.label.clone(),
                TypeRole::LabelLarge.style(TypographyEmphasis::Baseline),
                1.,
                style.foreground(self.disabled, window, cx),
                (None, 1., 1.),
                window,
                cx,
            ));
        let action = if selectable {
            Action::Toggle {
                selected,
                on_change: self.on_change,
            }
        } else {
            Action::Invoke(self.on_activate)
        };
        let row = div()
            .id(self.id.clone())
            .relative()
            .flex()
            .min_w_0()
            .items_center()
            .when(direction == Direction::Rtl, Styled::flex_row_reverse)
            .when(split, |row| {
                row.child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .top(gpui::relative(0.5))
                        .mt(-height / 2.)
                        .h(height)
                        .rounded(radius)
                        .bg(container)
                        .when(outline_opacity > 0., |surface| {
                            surface
                                .border_1()
                                .border_color(border.opacity(outline_opacity))
                        }),
                )
            })
            .child(
                ActionControl {
                    id: ElementId::from((self.id.clone(), "activate")),
                    label: self.label,
                    focus: None,
                    disabled: self.disabled,
                    action,
                    style,
                    content: content.into_any_element(),
                }
                .render(window, cx),
            );
        if let (true, Some((label, on_remove))) = (selectable, self.remove) {
            row.child(
                ActionControl {
                    id: ElementId::from((self.id, "remove")),
                    label,
                    focus: None,
                    disabled: self.disabled,
                    action: Action::Invoke(Some(on_remove)),
                    style: ActionStyle {
                        background: None,
                        background_color: None,
                        foreground,
                        disabled: DisabledStyle {
                            foreground: OnSurface,
                            background: None,
                            outline: (OnSurface, 0.12),
                        },
                        height: 32.,
                        min_width: 32.,
                        corner: CornerRole::Small,
                        pressed_corner: CornerRole::Small,
                        selected_corner: None,
                        corner_motion: (
                            gpui_material_theme::MotionKind::Spatial,
                            gpui_material_theme::MotionSpeed::Fast,
                        ),
                        outline: 0.,
                        elevation: Elevation::Level0,
                        hover_elevation: Elevation::Level0,
                    },
                    content: icon_slot("×".into_any_element(), 18.).into_any_element(),
                }
                .render(window, cx),
            )
        } else {
            row
        }
    }
}

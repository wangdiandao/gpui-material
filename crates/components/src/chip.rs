use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_material_foundation::material_theme;
use gpui_material_theme::{ColorRole, CornerRole, Direction, Elevation};

use crate::action::{Action, ActionControl, ActionStyle, Activate, Change, icon_slot};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ChipVariant {
    #[default]
    Assist,
    Filter,
    Input,
    Suggestion,
}

/// A Material chip. Filter and Input chips request controlled selection changes.
/// An Input chip can expose a separate removal action with its own focus.
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
        let direction = material_theme(cx).layout.direction;
        let style = ActionStyle {
            background,
            foreground,
            height: 32.,
            min_width: 0.,
            corner: CornerRole::Small,
            pressed_corner: CornerRole::Small,
            outline: f32::from(background.is_none()),
            elevation: if elevated {
                Elevation::Level1
            } else {
                Elevation::Level0
            },
            hover_elevation: if elevated {
                Elevation::Level2
            } else {
                Elevation::Level0
            },
        };
        let leading = self.leading.or_else(|| {
            (self.variant == ChipVariant::Filter && selected).then(|| "✓".into_any_element())
        });
        let content = div()
            .flex()
            .items_center()
            .min_w_0()
            .gap_2()
            .px_3()
            .text_size(px(14.))
            .when(direction == Direction::Rtl, Styled::flex_row_reverse)
            .children(leading.map(|icon| icon_slot(icon, 18.)))
            .child(div().min_w_0().truncate().child(self.label.clone()));
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
            .flex()
            .items_center()
            .when(direction == Direction::Rtl, Styled::flex_row_reverse)
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
        if let (ChipVariant::Input, Some((label, on_remove))) = (self.variant, self.remove) {
            row.child(
                ActionControl {
                    id: ElementId::from((self.id, "remove")),
                    label,
                    focus: None,
                    disabled: self.disabled,
                    action: Action::Invoke(Some(on_remove)),
                    style: ActionStyle {
                        background,
                        foreground,
                        height: 32.,
                        min_width: 32.,
                        corner: CornerRole::Small,
                        pressed_corner: CornerRole::Small,
                        outline: f32::from(background.is_none()),
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

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::motion::{OverlayMotion, ScaledOverlay, menu_pivot};
use gpui::{
    Anchor, App, AppContext, Bounds, Context, ElementId, Entity, EntityInputHandler, Focusable,
    InteractiveElement, IntoElement, ParentElement, RenderOnce, Role, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, Subscription, Window, canvas, div, prelude::FluentBuilder,
    px,
};
use gpui_base::{Popup, TestSupportExt};
use gpui_material_foundation::{Surface, animate, color, logical_anchor, window_material_theme};
use gpui_material_theme::{ColorRole, Elevation, MotionKind, MotionSpeed};

use super::{TextField, TextFieldEvent, TextFieldState, TextFieldVariant};

/// A selectable dropdown value. IDs remain stable when options are reordered.
#[derive(Clone)]
pub struct DropdownOption {
    pub id: ElementId,
    pub label: SharedString,
}

impl DropdownOption {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// Keeps the editable query and pending keyboard choice across renders.
pub struct ExposedDropdownState {
    input: Entity<TextFieldState>,
    committed: SharedString,
    editable: bool,
    open: bool,
    active: Option<ElementId>,
    _input: Subscription,
}

impl ExposedDropdownState {
    pub fn new(
        value: impl Into<SharedString>,
        editable: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let committed = value.into();
        let input = cx.new(|cx| {
            let mut state = TextFieldState::new(window, cx).default_value(committed.clone());
            state.set_readonly(!editable, cx);
            state
        });
        let subscription = cx.subscribe_in(&input, window, |state, input, event, window, cx| {
            if !state.editable || !matches!(event, TextFieldEvent::Change) {
                return;
            }
            if input.update(cx, |input, cx| {
                input.marked_text_range(window, cx).is_some()
            }) {
                return;
            }
            if input.read(cx).value() != state.committed {
                state.open = true;
                state.active = None;
            }
            cx.notify();
        });
        Self {
            input,
            committed,
            editable,
            open: false,
            active: None,
            _input: subscription,
        }
    }

    /// Synchronizes an externally selected label and dismisses the pending list.
    pub fn set_value(
        &mut self,
        value: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.committed = value.into();
        self.open = false;
        self.active = None;
        self.input.update(cx, |input, cx| {
            if input.value() != self.committed {
                input.set_value(self.committed.clone(), window, cx);
            }
        });
        cx.notify();
    }

    fn cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_value(self.committed.clone(), window, cx);
    }
}

type Select = Rc<dyn Fn(&DropdownOption, &mut Window, &mut App)>;

/// A Material field and anchored list in editable and selection-only modes.
/// Typing filters the list, while only selecting an option calls the owner.
#[derive(IntoElement)]
pub struct ExposedDropdown {
    id: ElementId,
    label: SharedString,
    state: Entity<ExposedDropdownState>,
    options: Vec<DropdownOption>,
    on_select: Option<Select>,
}

impl ExposedDropdown {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<ExposedDropdownState>,
        options: impl IntoIterator<Item = DropdownOption>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: state.clone(),
            options: options.into_iter().collect(),
            on_select: None,
        }
    }

    pub fn on_select(
        mut self,
        on_select: impl Fn(&DropdownOption, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(on_select));
        self
    }
}

fn choose(
    state: &Entity<ExposedDropdownState>,
    choice: &DropdownOption,
    callback: &Option<Select>,
    window: &mut Window,
    cx: &mut App,
) {
    state.update(cx, |state, cx| {
        state.set_value(choice.label.clone(), window, cx)
    });
    if let Some(callback) = callback {
        callback(choice, window, cx);
    }
}

impl RenderOnce for ExposedDropdown {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let model = self.state.read(cx);
        let input = model.input.clone();
        let focus = input.read(cx).focus_handle(cx);
        let open = model.open;
        let editable = model.editable;
        let active = model.active.clone();
        let query = input.read(cx).value();
        let committed = model.committed.clone();
        let composing = input.update(cx, |input, cx| {
            input.marked_text_range(window, cx).is_some()
        });
        let filter = editable && !composing && query != committed;
        let filter_text = query.to_lowercase();
        let options: Vec<_> = self
            .options
            .into_iter()
            .filter(|option| !filter || option.label.to_lowercase().contains(&filter_text))
            .collect();
        let last_visible_options = window
            .use_keyed_state((self.id.clone(), "presented-options"), cx, |_, _| {
                Rc::new(RefCell::new((
                    Vec::<DropdownOption>::new(),
                    None::<ElementId>,
                )))
            })
            .read(cx)
            .clone();
        if open {
            let mut visible = last_visible_options.borrow_mut();
            visible.0.clone_from(&options);
            visible.1.clone_from(&active);
        }
        let theme = window_material_theme(window, cx);
        let anchor = logical_anchor(Anchor::TopLeft, theme.layout.direction);
        let ink = color(theme.colors.color(ColorRole::OnSurface));
        let selected = color(theme.colors.color(ColorRole::SecondaryContainer));
        let arrow = animate(
            (self.id.clone(), "arrow"),
            f32::from(open),
            theme
                .motion
                .spec(MotionKind::Effects, MotionSpeed::Fast, cx.reduce_motion()),
            window,
            cx,
        )
        .clamp(0., 1.);
        let parent = window.current_view();
        let close = self.state.clone();
        let keys = close.clone();
        let toggle = close.clone();
        let callback = self.on_select.clone();
        let choices = options.clone();
        let key_input = input.clone();
        let toggle_focus = focus.clone();
        let bounds = window
            .use_keyed_state((self.id.clone(), "bounds"), cx, |_, _| {
                Rc::new(Cell::new(Bounds::default()))
            })
            .read(cx)
            .clone();
        let measure = bounds.clone();
        let popup_width = bounds.get().size.width;
        let trigger = div()
            .id((self.id.clone(), "combobox"))
            .capture_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if !matches!(key, "up" | "down" | "enter" | "escape") {
                    return;
                }
                if key_input.update(cx, |input, cx| {
                    input.marked_text_range(window, cx).is_some()
                }) {
                    return;
                }
                let model = keys.read(cx);
                let index = model
                    .active
                    .as_ref()
                    .and_then(|id| choices.iter().position(|item| &item.id == id));
                match key {
                    "up" | "down" => {
                        let next = index.map_or_else(
                            || {
                                if key == "up" {
                                    choices.len().saturating_sub(1)
                                } else {
                                    0
                                }
                            },
                            |index| {
                                if key == "up" {
                                    index
                                        .checked_sub(1)
                                        .unwrap_or(choices.len().saturating_sub(1))
                                } else {
                                    (index + 1) % choices.len()
                                }
                            },
                        );
                        keys.update(cx, |state, cx| {
                            state.active = choices.get(next).map(|choice| choice.id.clone());
                            state.open = true;
                            cx.notify();
                        });
                    }
                    "escape" if model.open => keys.update(cx, |state, cx| state.cancel(window, cx)),
                    "enter" if model.open => {
                        if let Some(choice) = choices.get(index.unwrap_or(0)) {
                            choose(&keys, choice, &callback, window, cx);
                        }
                    }
                    "enter" if !editable => keys.update(cx, |state, cx| {
                        state.open = true;
                        cx.notify();
                    }),
                    _ => return,
                }
                cx.notify(parent);
                cx.stop_propagation();
            })
            .child(
                TextField::new("value", self.label, &input)
                    .variant(TextFieldVariant::Outlined)
                    .expanded(open)
                    .trailing(
                        gpui_base::Button::new("toggle-options")
                            .accessibility_label(if open {
                                "Close options"
                            } else {
                                "Open options"
                            })
                            .aria_expanded(open)
                            .min_w(px(36.))
                            .min_h(px(36.))
                            .focus_visible(move |style| style.border_1().border_color(ink))
                            .on_click(move |_, window, cx| {
                                toggle_focus.focus(window, cx);
                                toggle.update(cx, |state, cx| {
                                    if state.open {
                                        state.cancel(window, cx);
                                    } else {
                                        state.open = true;
                                        cx.notify();
                                    }
                                });
                                cx.notify(parent);
                            })
                            .child(dropdown_arrow(arrow, ink)),
                    ),
            );
        let height = (window.viewport_size().height * 0.55).min(px(280.));
        let trigger = trigger.relative().child(
            canvas(
                move |bounds, window, _| {
                    if measure.replace(bounds).size != bounds.size && open {
                        window.refresh();
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        );
        let motion = OverlayMotion::sample(&self.id, open, window, cx);
        let scroll = window
            .use_keyed_state((self.id.clone(), "options-scroll"), cx, |_, _| {
                ScrollHandle::new()
            })
            .read(cx)
            .clone();
        let placement = window
            .use_keyed_state((self.id.clone(), "last-popup-bounds"), cx, |_, _| {
                Rc::new(Cell::new((Bounds::default(), Bounds::default())))
            })
            .read(cx)
            .clone();
        let (last_bounds, last_trigger) = placement.get();
        let trigger_bounds = placement.clone();
        let popup = Popup::new(self.id.clone(), trigger)
            .anchor(anchor)
            .offset(px(4.))
            .on_position(move |resolved, trigger| trigger_bounds.set((resolved.bounds, trigger)));
        let content = motion.map(|motion| {
            let dp = |value| motion.px(value);
            let spacing = window.rem_size() * motion.scale;
            let (options, active) = if open {
                (options, active)
            } else {
                last_visible_options.borrow().clone()
            };
            let width = if open {
                popup_width.max(px(120.)).min(window.viewport_size().width)
            } else {
                last_bounds.size.width
            };
            let content = div()
                .id(if open { "options" } else { "options-exit" })
                .test_support()
                .opacity(motion.alpha)
                .w(width * motion.scale)
                .max_h(height * motion.scale)
                .overflow_y_scroll()
                .track_scroll(&scroll)
                .when(open, |list| {
                    list.role(Role::ListBox)
                        .on_mouse_down_out(move |event, window, cx| {
                            if !bounds.get().contains(&event.position) {
                                close.update(cx, |state, cx| state.cancel(window, cx));
                                cx.notify(parent);
                            }
                        })
                })
                .child(
                    Surface::new(ColorRole::SurfaceContainer)
                        .shadow_elevation(Elevation::Level2)
                        .shadow_scale(motion.scale)
                        .rounded(spacing * 0.25)
                        .w_full()
                        .py(spacing * 0.5)
                        .when(options.is_empty(), |list| {
                            list.child(
                                div()
                                    .px(spacing * 0.75)
                                    .py(spacing * 0.5)
                                    .child("No matching options"),
                            )
                        })
                        .children(options.into_iter().map(|choice| {
                            let current = active.as_ref() == Some(&choice.id);
                            let state = self.state.clone();
                            let callback = self.on_select.clone();
                            let accessible = choice.clone();
                            let accessible_state = state.clone();
                            let accessible_callback = callback.clone();
                            div()
                                .id(choice.id.clone())
                                .when(open, |row| {
                                    row.role(Role::ListBoxOption)
                                        .aria_label(choice.label.clone())
                                        .aria_selected(current)
                                })
                                .when(current, |row| row.bg(selected))
                                .when(open && current, |row| row.aria_active_descendant())
                                .when(open, |row| {
                                    row.hover(move |style| style.bg(ink.opacity(0.08)))
                                })
                                .min_h(dp(40.))
                                .px(spacing * 0.75)
                                .flex()
                                .items_center()
                                .child(choice.label.clone())
                                .when(open, |row| {
                                    row.on_a11y_action(
                                        gpui::AccessibleAction::Click,
                                        move |_, window, cx| {
                                            choose(
                                                &accessible_state,
                                                &accessible,
                                                &accessible_callback,
                                                window,
                                                cx,
                                            );
                                            cx.notify(parent);
                                        },
                                    )
                                    .on_click(
                                        move |_, window, cx| {
                                            choose(&state, &choice, &callback, window, cx);
                                            cx.notify(parent);
                                        },
                                    )
                                })
                        })),
                )
                .when(!open, |list| {
                    list.child(div().absolute().inset_0().occlude())
                });
            ScaledOverlay::new(motion.text(content, window), motion, move |menu| {
                menu_pivot(
                    menu,
                    if open {
                        placement.get().1
                    } else {
                        last_trigger
                    },
                )
            })
            .width(width)
        });
        if open {
            return div().child(popup.content(content.expect("open dropdown")));
        }
        let exit = content
            .filter(|_| last_bounds.size.width > px(0.))
            .map(|content| {
                gpui::deferred(
                    gpui_base::Positioner::corner(Anchor::TopLeft, last_bounds.origin)
                        .child(content),
                )
                .with_priority(100)
            });
        div().child(popup).children(exit)
    }
}

// Match the Compose dropdown glyph's two endpoints while keeping the Base
// trigger's geometry and semantics independent of the rotating indicator.
fn dropdown_arrow(progress: f32, ink: gpui::Hsla) -> impl IntoElement {
    let (sin, cos) = (std::f32::consts::PI * progress).sin_cos();
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let center = bounds.center();
            let rotate = |x, y| center + gpui::point(px(x * cos - y * sin), px(x * sin + y * cos));
            let mut path = gpui::PathBuilder::fill();
            path.move_to(rotate(-5., -2.));
            path.line_to(rotate(5., -2.));
            path.line_to(rotate(0., 3.));
            path.close();
            if let Ok(path) = path.build() {
                window.paint_path(path, ink);
            }
        },
    )
    .size(px(16.))
    .mx_auto()
}

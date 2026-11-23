use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{
    MenuSurfaceStyle,
    motion::{OverlayMotion, ScaledOverlay, menu_pivot},
};
use gpui::{
    AccessibleAction, Anchor, AnyElement, App, Bounds, ElementId, Entity, EntityInputHandler,
    FocusHandle, Focusable, Hsla, InteractiveElement, IntoElement, KeyDownEvent,
    ListSizingBehavior, ListState, ParentElement, Pixels, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, Styled, Window, canvas, div, list, prelude::FluentBuilder, px,
};
use gpui_base::{Popup, TestSupportExt};
use gpui_material_foundation::{
    ThemeScope, animate, color, current_material_theme, logical_anchor,
};
use gpui_material_theme::{
    ColorRole, Direction, MaterialTheme, MotionKind, MotionSpeed, TypeRole, TypographyEmphasis,
};

use super::{
    super::{TextField, TextFieldVariant},
    AnchorContent, DropdownAnchor, DropdownOption, ExposedDropdown, OpenChange, OptionContent,
    Select,
    state::{ExposedDropdownState, Opening, Visibility},
};

fn choose(
    state: &Entity<ExposedDropdownState>,
    choice: &DropdownOption,
    callback: &Option<Select>,
    window: &mut Window,
    cx: &mut App,
) {
    state.update(cx, |state, cx| {
        state.committed = choice.label.clone();
        state.selected = Some(choice.id.clone());
        state.reset_draft(window, cx);
        state.request_open(false, None, window, cx);
    });
    if let Some(callback) = callback {
        callback(choice, window, cx);
    }
}

#[derive(Clone)]
struct Interaction {
    state: Entity<ExposedDropdownState>,
    // The mounted handlers own the policy. The editor subscription observes it weakly.
    _visibility: Rc<Visibility>,
    select: Option<Select>,
    primary: Opening,
    primary_enabled: bool,
    editable: bool,
    parent: gpui::EntityId,
}

impl Interaction {
    fn request(&self, open: Option<bool>, opening: Opening, window: &mut Window, cx: &mut App) {
        self.state.update(cx, |state, cx| {
            let open = open.unwrap_or(!state.open);
            if open && matches!(opening, Opening::Editor) {
                state.input.read(cx).focus_handle(cx).focus(window, cx);
            }
            state.request_open(open, open.then_some(opening), window, cx);
        });
        cx.notify(self.parent);
    }

    fn dismiss(&self, window: &mut Window, cx: &mut App) {
        self.request(Some(false), self.primary, window, cx);
    }

    fn dismiss_outside(&self, window: &mut Window, cx: &mut App) {
        let editor_focused = self
            .state
            .read(cx)
            .input
            .read(cx)
            .focus_handle(cx)
            .is_focused(window);
        if editor_focused {
            window.blur(cx);
        }
        self.dismiss(window, cx);
    }

    fn keys(&self, event: &KeyDownEvent, window: &mut Window, cx: &mut App) {
        let key = event.keystroke.key.as_str();
        if event.keystroke.modifiers.modified()
            || !(matches!(key, "up" | "down" | "enter" | "escape")
                || key == "space" && !self.editable)
        {
            return;
        }
        let model = self.state.read(cx);
        let open = model.open;
        let input = model.input.clone();
        if !open && !self.primary_enabled
            || input.update(cx, |input, cx| {
                input.marked_text_range(window, cx).is_some()
            })
        {
            return;
        }
        match key {
            "up" | "down" => self.state.update(cx, |state, cx| {
                state.navigate(key == "up", cx);
                if state.open {
                    return;
                }
                state.request_open(true, Some(self.primary), window, cx);
            }),
            "escape" if open => self.dismiss(window, cx),
            "enter" if open => {
                if let Some(choice) = self.state.read(cx).pending() {
                    choose(&self.state, &choice, &self.select, window, cx);
                }
            }
            "enter" | "space" if !self.editable => self.request(None, self.primary, window, cx),
            _ => return,
        }
        cx.notify(self.parent);
        cx.stop_propagation();
    }
}

#[allow(clippy::too_many_arguments)]
fn build_secondary_trigger(
    secondary: Option<(SharedString, AnchorContent)>,
    secondary_enabled: bool,
    secondary_focus: &FocusHandle,
    open: bool,
    editable: bool,
    primary: Opening,
    arrow: f32,
    ink: Hsla,
    interaction: Interaction,
) -> AnyElement {
    let (secondary_label, secondary_content) = secondary.map_or_else(
        || {
            let label = if open {
                "Close options"
            } else {
                "Open options"
            };
            (
                SharedString::from(label),
                dropdown_arrow(arrow, ink).into_any_element(),
            )
        },
        |(label, content)| (label, content(open)),
    );
    if secondary_enabled {
        let pointer = interaction.clone();
        let a11y = interaction.clone();
        let expansion = interaction;
        let accessible_opening = if editable {
            Opening::PopupFromSecondary
        } else {
            primary
        };
        gpui_base::Button::new("toggle-options")
            .track_focus(secondary_focus)
            .accessibility_label(secondary_label)
            .aria_expanded(open)
            .min_w(px(36.))
            .min_h(px(36.))
            .focus_visible(move |style| style.border_1().border_color(ink))
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                pointer.request(None, primary, window, cx);
            })
            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                a11y.request(None, accessible_opening, window, cx);
            })
            .on_a11y_action(
                if open {
                    AccessibleAction::Collapse
                } else {
                    AccessibleAction::Expand
                },
                move |_, window, cx| {
                    expansion.request(Some(!open), accessible_opening, window, cx);
                },
            )
            .child(secondary_content)
            .into_any_element()
    } else {
        div()
            .id("toggle-options-disabled")
            .role(Role::Group)
            .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
            .min_w(px(36.))
            .min_h(px(36.))
            .flex()
            .items_center()
            .justify_center()
            .opacity(0.38)
            .child(secondary_content)
            .into_any_element()
    }
}

struct PopupGeometry {
    anchor: Anchor,
    height: Pixels,
    width: Pixels,
    gap: Pixels,
}

fn resolve_popup_geometry(
    measured: Bounds<Pixels>,
    viewport: gpui::Size<Pixels>,
    direction: Direction,
    match_anchor_width: bool,
    popup_width: Pixels,
) -> PopupGeometry {
    let default_height = (viewport.height * 0.55).min(px(280.));
    let gap = px(4.);
    let margin = px(8.);
    let (anchor, height) = if measured.size.height > px(0.) {
        let below = (viewport.height - measured.bottom() - gap - margin).max(px(0.));
        let above = (measured.top() - gap - margin).max(px(0.));
        if below < default_height && above > below {
            (Anchor::BottomLeft, default_height.min(above))
        } else {
            (Anchor::TopLeft, default_height.min(below))
        }
    } else {
        (Anchor::TopLeft, default_height)
    };
    let anchor = logical_anchor(anchor, direction);
    let width = if match_anchor_width {
        if measured.size.width > px(0.) {
            measured.size.width
        } else {
            px(120.)
        }
    } else {
        popup_width
    };
    let width = width.min((viewport.width - margin * 2.).max(px(0.)));
    PopupGeometry {
        anchor,
        height,
        width,
        gap,
    }
}

struct OptionRowContext {
    state: Entity<ExposedDropdownState>,
    on_select: Option<Select>,
    option_content: Option<OptionContent>,
    open: bool,
    parent: gpui::EntityId,
    ink: Hsla,
    selected_bg: Hsla,
    selected_ink: Hsla,
    min_height: Pixels,
    horizontal_padding: Pixels,
}

fn render_option_row(
    choice: DropdownOption,
    index: usize,
    total: usize,
    current: bool,
    ctx: &OptionRowContext,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let content = ctx.option_content.as_ref().map_or_else(
        || choice.label.clone().into_any_element(),
        |content| content(&choice, current, window, cx),
    );
    let ink = ctx.ink;
    let selected = ctx.selected_bg;
    let selected_ink = ctx.selected_ink;
    div()
        .id(choice.id.clone())
        .test_support()
        .when(ctx.open, |row| {
            row.role(Role::ListBoxOption)
                .aria_label(choice.label.clone())
                .aria_selected(current)
                .aria_position_in_set(index + 1)
                .aria_size_of_set(total)
        })
        .when(current, |row| row.bg(selected).text_color(selected_ink))
        .when(ctx.open && current, |row| row.aria_active_descendant())
        .when(ctx.open, |row| {
            row.hover(move |style| {
                style.bg(if current {
                    selected.blend(selected_ink.opacity(0.08))
                } else {
                    ink.opacity(0.08)
                })
            })
        })
        .min_h(ctx.min_height)
        .w_full()
        .px(ctx.horizontal_padding)
        .flex()
        .items_center()
        .child(content)
        .when(ctx.open, |row| {
            let state = ctx.state.clone();
            let callback = ctx.on_select.clone();
            let parent = ctx.parent;
            let select_row = Rc::new(move |window: &mut Window, cx: &mut App| {
                choose(&state, &choice, &callback, window, cx);
                cx.notify(parent);
            });
            let accessible_select = select_row.clone();
            row.on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                accessible_select(window, cx);
            })
            .on_click(move |_, window, cx| {
                select_row(window, cx);
            })
        })
        .into_any_element()
}

struct PopupContentConfig {
    state: Entity<ExposedDropdownState>,
    scroll: ListState,
    open: bool,
    options: Rc<[DropdownOption]>,
    active: Option<ElementId>,
    selected_option: Option<ElementId>,
    options_label: String,
    transfer_focus: bool,
    popup_focus: FocusHandle,
    popup_keys: Interaction,
    outside: Interaction,
    bounds: Rc<Cell<Bounds<Pixels>>>,
    placement: Rc<Cell<(Bounds<Pixels>, Bounds<Pixels>)>>,
    last_trigger: Bounds<Pixels>,
    width: Pixels,
    height: Pixels,
    appearance: MenuSurfaceStyle,
    container_color: Option<Hsla>,
    on_select: Option<Select>,
    option_content: Option<OptionContent>,
    parent: gpui::EntityId,
    theme: MaterialTheme,
    ink: Hsla,
    selected_bg: Hsla,
    selected_ink: Hsla,
}

fn build_popup_content(
    config: PopupContentConfig,
    motion: OverlayMotion,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let dp = move |value| motion.px(value);
    let spacing = dp(16.);
    let scroll_height =
        (config.height - config.appearance.popup_vertical_insets()).max(px(0.)) * motion.scale;
    let font_revision = gpui_material_foundation::font_cache_revision(cx);
    config.state.update(cx, |state, _| {
        state.sync_presented_list(
            &config.options,
            config.active.as_ref(),
            (
                config.width,
                motion.scale,
                window.rem_size(),
                config.theme.typography.clone(),
                font_revision,
            ),
        );
    });
    let options = config.options;
    let active = config.active;
    let selected_option = config.selected_option;
    let open = config.open;
    let list_body = if options.is_empty() {
        div()
            .id("options-scroll-region")
            .test_support()
            .max_h(scroll_height)
            .overflow_hidden()
            .child(
                div()
                    .px(spacing * 0.75)
                    .py(spacing * 0.5)
                    .child("No matching options"),
            )
            .into_any_element()
    } else {
        let row_ctx = Rc::new(OptionRowContext {
            state: config.state,
            on_select: config.on_select,
            option_content: config.option_content,
            open,
            parent: config.parent,
            ink: config.ink,
            selected_bg: config.selected_bg,
            selected_ink: config.selected_ink,
            min_height: dp(48.),
            horizontal_padding: spacing * 0.75,
        });
        div()
            .id("options-scroll-region")
            .test_support()
            .max_h(scroll_height)
            .overflow_hidden()
            .child(
                list(config.scroll, move |index, window, cx| {
                    let choice = options[index].clone();
                    let current = active.as_ref().or(selected_option.as_ref()) == Some(&choice.id);
                    render_option_row(choice, index, options.len(), current, &row_ctx, window, cx)
                })
                .with_sizing_behavior(ListSizingBehavior::Infer)
                .w_full()
                .max_h(scroll_height),
            )
            .into_any_element()
    };
    let bounds = config.bounds;
    let popup_focus = config.popup_focus;
    let popup_keys = config.popup_keys;
    let outside = config.outside;
    let content = div()
        .id(if open { "options" } else { "options-exit" })
        .test_support()
        .opacity(motion.alpha)
        .w(config.width * motion.scale)
        .text_color(config.ink)
        .max_h(config.height * motion.scale)
        .when(open, |list| {
            list.role(Role::ListBox)
                .aria_label(config.options_label)
                .when(config.transfer_focus, |list| {
                    list.track_focus(&popup_focus).tab_stop(false)
                })
                .capture_key_down(move |event, window, cx| popup_keys.keys(event, window, cx))
                .on_mouse_down_out(move |event, window, cx| {
                    if !bounds.get().contains(&event.position) {
                        outside.dismiss_outside(window, cx);
                    }
                })
        })
        .child(
            config
                .appearance
                .dropdown_surface(&config.theme, motion)
                .when_some(config.container_color, |surface, color| surface.bg(color))
                .w_full()
                .child(list_body),
        )
        .when(!open, |list| {
            list.child(div().absolute().inset_0().occlude())
        });
    let text_style = TypeRole::LabelLarge.style(TypographyEmphasis::Baseline);
    let content =
        gpui_material_foundation::text_style(content, text_style, motion.scale, window, cx);
    let placement = config.placement;
    let last_trigger = config.last_trigger;
    let scaled = ScaledOverlay::new(content, motion, move |menu| {
        menu_pivot(
            menu,
            if open {
                placement.get().1
            } else {
                last_trigger
            },
        )
    })
    .width(config.width);
    ThemeScope::inherit(scaled, window, cx)
}

impl RenderOnce for ExposedDropdown {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let primary = if self.primary == Some(DropdownAnchor::PrimaryNotEditable) {
            Opening::PopupFromPrimary
        } else {
            Opening::Editor
        };
        let visibility = Rc::new(self.visibility);
        let (
            input,
            open,
            editable,
            popup_focus,
            transfer_focus,
            secondary_focus,
            scroll,
            options,
            active,
            selected_option,
        ) = self.state.update(cx, |state, cx| {
            state.present(
                &visibility,
                self.primary_enabled,
                self.secondary_enabled,
                primary,
                window,
                cx,
            );
            let input = state.input.clone();
            let open = state.open;
            let editable = state.editable;
            let popup_focus = state.popup_focus.clone();
            let transfer_focus = state.restore_focus.is_some();
            let secondary_focus = state.secondary_focus.clone();
            let scroll = state.scroll.clone();
            let (query, composing) = input.update(cx, |input, cx| {
                (
                    input.value(),
                    editable && input.marked_text_range(window, cx).is_some(),
                )
            });
            let filter = editable && !composing && query != state.committed;
            let (options, active, selected_option) =
                state.reconcile_options(&self.options, filter.then_some(query));
            (
                input,
                open,
                editable,
                popup_focus,
                transfer_focus,
                secondary_focus,
                scroll,
                options,
                active,
                selected_option,
            )
        });
        let last_visible_options = window
            .use_keyed_state((self.id.clone(), "presented-options"), cx, |_, _| {
                Rc::new(RefCell::new((
                    Rc::<[DropdownOption]>::default(),
                    None::<ElementId>,
                    None::<ElementId>,
                )))
            })
            .read(cx)
            .clone();
        if open {
            let mut visible = last_visible_options.borrow_mut();
            visible.0 = options.clone();
            visible.1.clone_from(&active);
            visible.2.clone_from(&selected_option);
        }
        let theme = current_material_theme(window, cx);
        let ink = color(theme.colors.color(ColorRole::OnSurface));
        let selected_bg = color(theme.colors.color(ColorRole::SecondaryContainer));
        let selected_ink = color(theme.colors.color(ColorRole::OnSecondaryContainer));
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
        let interaction = Interaction {
            state: self.state.clone(),
            _visibility: visibility,
            select: self.on_select.clone(),
            primary,
            primary_enabled: self.primary_enabled,
            editable,
            parent,
        };
        let popup_keys = interaction.clone();
        let outside = interaction.clone();
        let keys = interaction.clone();
        let primary_click = interaction.clone();
        let request: Option<OpenChange> = self.primary_enabled.then(|| {
            let interaction = interaction.clone();
            Rc::new(move |open, window: &mut Window, cx: &mut App| {
                interaction.request(Some(open), primary, window, cx);
            }) as OpenChange
        });
        let secondary = build_secondary_trigger(
            self.secondary,
            self.secondary_enabled,
            &secondary_focus,
            open,
            editable,
            primary,
            arrow,
            ink,
            interaction,
        );
        let options_label = format!("{} options", self.label);
        let field = TextField::new("value", self.label, &input)
            .variant(TextFieldVariant::Outlined)
            .dropdown(open, request)
            .trailing(secondary);
        let editor = if let Some(editor) = self.editor {
            editor(field)
        } else {
            field.into_any_element()
        };

        let bounds = window
            .use_keyed_state((self.id.clone(), "bounds"), cx, |_, _| {
                Rc::new(Cell::new(Bounds::default()))
            })
            .read(cx)
            .clone();
        let measure = bounds.clone();
        let geometry = resolve_popup_geometry(
            bounds.get(),
            window.viewport_size(),
            theme.layout.direction,
            self.match_anchor_width,
            self.popup_width,
        );
        let trigger = div()
            .id((self.id.clone(), "combobox"))
            .test_support()
            .when(
                self.primary_enabled && (!editable || self.primary.is_some()),
                |trigger| {
                    trigger.on_click(move |_, window, cx| {
                        primary_click.request(None, primary, window, cx);
                    })
                },
            )
            // TextField owns the single combo box node and delegates IME to its editor.
            .capture_key_down(move |event, window, cx| keys.keys(event, window, cx))
            .child(editor)
            .relative()
            .child(
                canvas(
                    move |bounds, window, _| {
                        if measure.replace(bounds) != bounds && open {
                            window.refresh();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            )
            // Keep the default cursor over the read-only editor's I-beam.
            .when(!editable, |trigger| {
                trigger.child(div().absolute().inset_0().cursor_default())
            });
        let motion = OverlayMotion::sample(&self.id, open, window, cx);
        let placement = window
            .use_keyed_state((self.id.clone(), "last-popup-bounds"), cx, |_, _| {
                Rc::new(Cell::new((Bounds::default(), Bounds::default())))
            })
            .read(cx)
            .clone();
        let (last_bounds, last_trigger) = placement.get();
        let trigger_bounds = placement.clone();
        let popup = Popup::new(self.id.clone(), trigger)
            .anchor(geometry.anchor)
            .offset(geometry.gap)
            .on_position(move |resolved, trigger| trigger_bounds.set((resolved.bounds, trigger)));
        let content = motion.map(|motion| {
            let (options, active, selected_option) = if open {
                (options, active, selected_option)
            } else {
                last_visible_options.borrow().clone()
            };
            let width = if open {
                geometry.width
            } else {
                last_bounds.size.width
            };
            build_popup_content(
                PopupContentConfig {
                    state: self.state,
                    scroll,
                    open,
                    options,
                    active,
                    selected_option,
                    options_label,
                    transfer_focus,
                    popup_focus,
                    popup_keys,
                    outside,
                    bounds,
                    placement,
                    last_trigger,
                    width,
                    height: geometry.height,
                    appearance: self.appearance,
                    container_color: self.container_color,
                    on_select: self.on_select,
                    option_content: self.option_content,
                    parent,
                    theme: (*theme).clone(),
                    ink,
                    selected_bg,
                    selected_ink,
                },
                motion,
                window,
                cx,
            )
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

// The official dropdown symbol rotates with the existing expansion animation.
// Keep this decorative element separate from the Base trigger's semantics.
fn dropdown_arrow(progress: f32, ink: Hsla) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .text_color(ink)
        .child(
            crate::Icon::new(crate::IconName::ArrowDropDown)
                .size(px(24.))
                .rotate(gpui::radians(std::f32::consts::PI * progress)),
        )
}

#[cfg(test)]
mod tests {
    use super::super::state::{OptionSnapshot, contains_ignore_case};
    use super::*;

    #[test]
    fn option_snapshot_reuses_results_and_invalidates_labels_and_query() {
        let mut snapshot = OptionSnapshot::default();
        let options = || -> Rc<[DropdownOption]> {
            vec![
                DropdownOption::new("a", "Alpha"),
                DropdownOption::new("b", "Beta"),
            ]
            .into()
        };
        let shared = options();
        assert!(snapshot.reconcile(&shared, None));
        let unfiltered = snapshot.visible.clone();
        assert!(Rc::ptr_eq(&unfiltered, &snapshot.source));
        assert!(Rc::ptr_eq(&unfiltered, &shared));
        assert!(!snapshot.reconcile(&shared, None));
        assert!(Rc::ptr_eq(&unfiltered, &snapshot.visible));
        let cloned_equal = options();
        assert!(!snapshot.reconcile(&cloned_equal, None));
        assert!(Rc::ptr_eq(&snapshot.source, &cloned_equal));
        assert!(Rc::ptr_eq(&unfiltered, &snapshot.visible));
        assert!(snapshot.reconcile(&shared, Some("ALP".into())));
        assert_eq!(snapshot.visible.len(), 1);
        assert_eq!(snapshot.visible[0].id, ElementId::from("a"));
        let renamed: Rc<[DropdownOption]> = vec![
            DropdownOption::new("a", "Renamed"),
            DropdownOption::new("b", "Beta"),
        ]
        .into();
        assert!(snapshot.reconcile(&renamed, Some("ALP".into())));
        assert!(snapshot.visible.is_empty());
    }

    #[test]
    fn contains_ignore_case_handles_ascii_and_unicode() {
        assert!(contains_ignore_case("Alpha Beta", "", ""));
        assert!(contains_ignore_case("Alpha Beta", "ALP", "alp"));
        assert!(contains_ignore_case("Alpha Beta", "beta", "beta"));
        assert!(!contains_ignore_case("Alpha", "Alpha Beta", "alpha beta"));
        assert!(contains_ignore_case("北京市", "北京", "北京"));
        assert!(contains_ignore_case("Café Crème", "CAFÉ", "café"));
    }
}

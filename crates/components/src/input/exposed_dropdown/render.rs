use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    AccessibleAction, Anchor, AnyElement, App, Bounds, ElementId, EntityInputHandler, FocusHandle,
    Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, Styled, Window, canvas, div, prelude::FluentBuilder, px,
};
use gpui_base::{Popup, TestSupportExt};
use gpui_material_foundation::{
    OverlayMotion, animate, color, current_material_strings, current_material_theme,
};
use gpui_material_theme::{ColorRole, MotionKind, MotionSpeed};

use super::{
    super::{TextField, TextFieldVariant},
    AnchorContent, DropdownAnchor, DropdownOption, ExposedDropdown, OpenChange,
    interaction::Interaction,
    popup::{PopupContentConfig, build_popup_content, resolve_popup_geometry},
    state::Opening,
};

#[allow(clippy::too_many_arguments)]
fn build_secondary_trigger(
    id: &ElementId,
    secondary: Option<(SharedString, AnchorContent)>,
    default_label: SharedString,
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
        || (default_label, dropdown_arrow(arrow, ink).into_any_element()),
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
        gpui_base::Button::new((id.clone(), "toggle-options"))
            .track_focus(secondary_focus)
            .accessibility_label(secondary_label)
            .aria_expanded(open)
            .min_w(px(36.))
            .min_h(px(36.))
            .cursor_pointer()
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
            .id((id.clone(), "toggle-options-disabled"))
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

impl RenderOnce for ExposedDropdown {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let component_disabled = window
            .use_keyed_state((self.id.clone(), "component-disabled"), cx, |_, _| {
                Rc::new(Cell::new(false))
            })
            .read(cx)
            .clone();
        let input_entity = self.state.read(cx).input.clone();
        let state_disabled = input_entity.read(cx).presentation().is_disabled();
        if self.disabled {
            if !state_disabled {
                component_disabled.set(true);
                input_entity.update(cx, |input, cx| input.set_disabled(true, cx));
            }
        } else if component_disabled.replace(false) {
            input_entity.update(cx, |input, cx| input.set_disabled(false, cx));
        }
        let disabled = self.disabled || input_entity.read(cx).presentation().is_disabled();
        let primary_enabled = !disabled && self.primary_enabled;
        let secondary_enabled = !disabled && self.secondary_enabled;
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
                disabled,
                primary_enabled,
                secondary_enabled,
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
            primary_enabled,
            editable,
            disabled,
            parent,
        };
        let popup_keys = interaction.clone();
        let outside = interaction.clone();
        let keys = interaction.clone();
        let primary_click = interaction.clone();
        let request: Option<OpenChange> = primary_enabled.then(|| {
            let interaction = interaction.clone();
            Rc::new(move |open, window: &mut Window, cx: &mut App| {
                interaction.request(Some(open), primary, window, cx);
            }) as OpenChange
        });
        let default_secondary_label = {
            let strings = current_material_strings(window, cx);
            SharedString::from(if open {
                strings.close_options.clone()
            } else {
                strings.open_options.clone()
            })
        };
        let secondary = build_secondary_trigger(
            &self.id,
            self.secondary,
            default_secondary_label,
            secondary_enabled,
            &secondary_focus,
            open,
            editable,
            primary,
            arrow,
            ink,
            interaction,
        );
        let options_label = format!("{} options", self.label);
        let field = TextField::new((self.id.clone(), "value"), self.label, &input)
            .variant(TextFieldVariant::Outlined)
            .disabled(disabled)
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
                primary_enabled && (!editable || self.primary.is_some()),
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
            );
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
                    id: self.id,
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

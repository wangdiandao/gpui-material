use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, FocusHandle, InteractiveElement, IntoElement, ParentElement, Role, Styled,
    Window, accesskit::Toggled, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{
    InteractionSource, ThemeScope, animate, color, current_material_theme,
};
use gpui_material_theme::{ColorRole, CornerShape, MotionKind, MotionSpec, MotionSpeed};

use crate::{
    SwipeToDismiss,
    action::{Activate, ripple_state},
};

use super::long_press::LongPress;
use super::row_wrapper::{render_expanded_content, wrap_interactive_row, wrap_static_row};
use super::style::{ListItemShapes, resolve_list_elevation};
use super::surface::{
    list_item_min_height, render_content_slot, render_surface, render_visual_canvas,
};
use super::{ListItem, Slot};

impl ListItem {
    pub(super) fn render_row(
        mut self,
        resting: CornerShape,
        selected_shape: CornerShape,
        focus: Option<(FocusHandle, bool)>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let (focus, tab_stop) = focus.unwrap_or_else(|| {
            let focus = self.focus.take().unwrap_or_else(|| {
                window
                    .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                    .read(cx)
                    .clone()
            });
            (focus, self.tab_stop.unwrap_or(true))
        });
        let primary = self.primary_action().map(|primary| {
            let bound: Activate = Rc::new(ThemeScope::bind_action(
                move |window, cx| primary(window, cx),
                window,
                cx,
            ));
            bound
        });
        let has_menu_trigger = self.has_menu_trigger();
        let secondary_action = self.long_press.map(|mut action| {
            let handler = action.handler;
            action.handler = Rc::new(ThemeScope::bind_action(
                move |window, cx| handler(window, cx),
                window,
                cx,
            ));
            action
        });
        let interactive = primary.is_some() || has_menu_trigger || secondary_action.is_some();
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

        let long_press = secondary_action.as_ref().map(|long_press| {
            LongPress::mount(
                (enabled && !dragged).then(|| long_press.handler.clone()),
                window,
                cx,
            )
        });
        let (layout, motion) = {
            let theme = current_material_theme(window, cx);
            (theme.layout, theme.motion)
        };
        let reduced = cx.reduce_motion();
        let selected = self.selected
            || (self.checked == Some(Toggled::True)
                && self.checked_selects_row
                && self.checked_role != Role::Switch);
        let shape = self
            .shapes
            .resolve(resting, selected_shape, selected, interaction);
        let effects = motion.spec(MotionKind::Effects, MotionSpeed::Default, reduced);
        let spatial = motion.spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let selection =
            animate("selection", f32::from(selected), effects, window, cx).clamp(0., 1.);
        let (ring, shadow, colors) = {
            let scheme = &current_material_theme(window, cx).colors;
            let tone = |role| color(scheme.color(role));
            (
                tone(ColorRole::Secondary),
                tone(ColorRole::Shadow),
                self.colors
                    .resolve(selected, selection, interaction, self.tone, tone),
            )
        };
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
            resolve_list_elevation(self.elevations, interaction).dp(),
            if enabled {
                spatial
            } else {
                MotionSpec::Instant
            },
            window,
            cx,
        )
        .max(0.);
        let static_single_line =
            self.overline.is_none() && self.supporting.is_none() && self.content.is_none();
        let static_three_lines = (self.overline.is_some() && self.supporting.is_some())
            || self.supporting.as_ref().is_some_and(Slot::has_line_break);
        let text_height = (!static_single_line && !static_three_lines).then(|| {
            window
                .use_keyed_state("text-height", cx, |_, _| Rc::new(Cell::new(px(0.))))
                .read(cx)
                .clone()
        });
        let three_lines = static_three_lines
            || text_height
                .as_ref()
                .is_some_and(|text_height| text_height.get() > px(44.));
        let height = list_item_min_height(
            self.overline.is_some(),
            self.supporting.is_some(),
            three_lines,
            interactive,
            layout.density,
        );
        let radii = match shape {
            CornerShape::Radii(radii) => radii,
            CornerShape::Full => [height * 0.5; 4],
        };
        let (full, radii) = if resting == selected_shape && self.shapes == ListItemShapes::default()
        {
            (f32::from(shape == CornerShape::Full), radii)
        } else {
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
            (full, radii)
        };
        let visual = render_visual_canvas(
            radii,
            full,
            layout.direction,
            enabled,
            elevation,
            shadow,
            colors.container,
            colors.content,
            ring,
            interaction,
            layer,
            reduced,
            &feedback,
        );
        let content = render_content_slot(
            self.label.clone(),
            self.content,
            self.overline,
            self.supporting,
            &colors,
            text_height,
        );
        let surface = render_surface(
            height,
            three_lines,
            self.alignment,
            self.padding,
            layout,
            visual,
            self.leading,
            content,
            self.trailing,
            self.footer,
            secondary_action.as_ref(),
            long_press.as_ref(),
            self.long_press_button,
            &colors,
            enabled,
            dragged,
        );
        #[cfg(feature = "settings-tiles")]
        let (menu_open, menu_a11y_toggle) = match self.menu_trigger {
            Some((open, toggle)) => (Some(open), Some(toggle)),
            None => (None, None),
        };
        #[cfg(not(feature = "settings-tiles"))]
        let (menu_open, menu_a11y_toggle): (Option<bool>, Option<Activate>) = (None, None);
        let expanded =
            menu_open.or_else(|| self.expansion.as_ref().map(|expansion| expansion.expanded));
        let checked = self.checked;
        let expanded_content =
            render_expanded_content(self.expansion, interactive, enabled, &focus, window, cx);
        let id = self.id;
        let row = if interactive {
            let interactive_row = wrap_interactive_row(
                &id,
                self.label,
                self.aria_description,
                surface,
                focus,
                tab_stop,
                self.selected,
                self.selectable,
                enabled,
                dragged,
                checked,
                self.checked_role,
                self.invalid,
                expanded,
                menu_a11y_toggle,
                ring,
                feedback,
                primary,
                self.expanded_change,
                secondary_action,
                long_press,
                window,
                cx,
            );
            // Keep the outer layout container free of duplicate ARIA role/selected
            // attributes so interactive rows expose a single unified accessible node.
            div()
                .id(id.clone())
                .test_support()
                .flex()
                .flex_col()
                .w_full()
                .child(interactive_row)
                .children(expanded_content)
                .into_any_element()
        } else {
            wrap_static_row(
                id.clone(),
                self.label,
                self.aria_description,
                self.selected,
                self.selectable,
                self.disabled,
                checked,
                self.checked_role,
                self.invalid,
                expanded,
                surface,
                expanded_content,
            )
        };
        match self.swipe {
            Some(swipe) => (swipe.configure)(SwipeToDismiss::new((id, "swipe"), &swipe.state, row))
                .when(self.disabled, |swipe| swipe.gestures_enabled(false))
                .into_any_element(),
            None => row,
        }
    }
}

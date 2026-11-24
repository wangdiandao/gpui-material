use std::{cell::RefCell, rc::Rc};

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window,
    div, prelude::FluentBuilder, px,
};
use gpui_base::{
    AxisExt,
    slider::{Slider as BaseSlider, SliderIndicator},
};
use gpui_material_foundation::{RangeThumb, current_material_theme};
use gpui_material_theme::{MotionKind, MotionSpeed};

use super::geometry::Geometry;
use super::gesture::{FinishMode, Gesture, attach_pointer_handlers, finish, sync_gesture_feedback};
use super::layout::{label_opacity, resolve_track_layout};
use super::palette::resolve_palette;
use super::thumb::render_thumb;
use super::track::render_track_artwork;
use super::{Slider, SliderKind, thumb_index};

impl RenderOnce for Slider {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let ([default_start_focus, default_end_focus], gesture) = window
            .use_keyed_state(
                ElementId::from((self.id.clone(), "slider-state")),
                cx,
                |_, cx| {
                    (
                        [cx.focus_handle(), cx.focus_handle()],
                        Rc::new(RefCell::new(Gesture::default())),
                    )
                },
            )
            .read(cx)
            .clone();
        let start_focus = self.thumbs[thumb_index(RangeThumb::Start)]
            .focus
            .take()
            .unwrap_or(default_start_focus);
        let end_focus = self.thumbs[thumb_index(RangeThumb::End)]
            .focus
            .take()
            .unwrap_or(default_end_focus);
        let theme = current_material_theme(window, cx);
        let geometry = Geometry {
            axis: self.axis,
            direction: theme.layout.direction,
            reverse_vertical: self.reverse_vertical,
        };
        let density = theme.layout.density;
        let horizontal = self.axis.is_horizontal();
        let state = self.state.read(cx);
        let value = state.value();
        let percentage = state.percentage();
        let start = percentage.start;
        let end = percentage.end;
        let range = value.is_range();
        let centered = self.kind == SliderKind::Centered && !range;
        let center = if centered { 0.5 } else { start };
        sync_gesture_feedback(
            &self,
            [&start_focus, &end_focus],
            &gesture,
            range,
            window,
            cx,
        );

        let [start_content, end_content] = self.thumbs.each_mut().map(|thumb| thumb.content.take());
        let thumb_styles = [self.thumbs[0].style, self.thumbs[1].style];
        let palette = resolve_palette(self.colors, self.disabled, window, cx);
        let dragging = gesture
            .borrow()
            .thumb
            .filter(|_| !self.disabled && window.is_window_active());
        let reduced = cx.reduce_motion();
        let scheme = theme.motion;
        let resize = scheme.spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let reveal = scheme.spec(MotionKind::Effects, MotionSpeed::Fast, reduced);
        let layout = resolve_track_layout(
            &self.id,
            self.size,
            density,
            self.track_style,
            thumb_styles,
            range,
            dragging,
            resize,
            window,
            cx,
        );

        let start_label = if range {
            label_opacity(
                &self.id,
                RangeThumb::Start,
                &start_focus,
                self.value_indicator,
                self.disabled,
                dragging,
                reveal,
                window,
                cx,
            )
        } else {
            0.
        };
        let end_label = label_opacity(
            &self.id,
            RangeThumb::End,
            &end_focus,
            self.value_indicator,
            self.disabled,
            dragging,
            reveal,
            window,
            cx,
        );
        // Expressive tracks leave 6 dp between a handle's edge and each adjacent segment.
        let end_gap = layout.thumb_gap + layout.handle_sizes[1].width / 2.;
        let center_gap = if range {
            layout.thumb_gap + layout.handle_sizes[0].width / 2.
        } else if centered {
            layout.thumb_gap
        } else {
            px(0.)
        };
        let (first, last) = if center <= end {
            ((center, center_gap), (end, end_gap))
        } else {
            ((end, end_gap), (center, center_gap))
        };
        let track_artwork = render_track_artwork(
            self.track.take(),
            self.icon.take(),
            geometry,
            first,
            last,
            layout,
            palette,
            self.disabled,
            self.tick_intervals,
            self.show_stop_indicators,
            self.draw_tick.take(),
            self.draw_stop.take(),
        );
        let track = SliderIndicator::new(&self.state)
            .absolute()
            .inset_0()
            .size_full()
            .child(track_artwork);
        let disabled_name = self.disabled.then(|| {
            self.range_end_name.as_ref().map_or_else(
                || self.name.clone(),
                |end| format!("{} – {end}", self.name).into(),
            )
        });
        // The indicator records these bounds for Base input. Absolute thumbs are children
        // of the same unpadded coordinate space, so percentages use those bounds as well.
        let format_value = self.format_value.as_ref();
        let track_and_handles = div()
            .relative()
            .when(horizontal, |slot| slot.size_full())
            .when(!horizontal, |slot| slot.w_full().flex_1())
            .child(track)
            .when(range, |root| {
                root.child(render_thumb(
                    &self.id,
                    RangeThumb::Start,
                    start,
                    self.name.clone(),
                    &start_focus,
                    start_content,
                    thumb_styles[0],
                    start_label,
                    value,
                    format_value,
                    &self.state,
                    &gesture,
                    geometry,
                    layout,
                    palette,
                    self.disabled,
                    window,
                    cx,
                ))
            })
            .child(render_thumb(
                &self.id,
                RangeThumb::End,
                end,
                self.range_end_name.unwrap_or(self.name),
                &end_focus,
                end_content,
                thumb_styles[1],
                end_label,
                value,
                format_value,
                &self.state,
                &gesture,
                geometry,
                layout,
                palette,
                self.disabled,
                window,
                cx,
            ));
        let root = div()
            .id(self.id.clone())
            .relative()
            .flex()
            .when(horizontal, |root| {
                root.w_full().h(layout.target).px(layout.padding)
            })
            .when(!horizontal, |root| {
                root.flex_col()
                    .w(layout.target)
                    .h_full()
                    .min_h(px(200.))
                    .py(layout.padding)
            })
            .child(track_and_handles);
        let active_thumb = gesture.borrow().thumb;
        let inactive = !window.is_window_active();
        if active_thumb
            .is_some_and(|which| self.disabled || inactive || !range && which == RangeThumb::Start)
        {
            let state = self.state.clone();
            let gesture = gesture.clone();
            window.defer(cx, move |window, cx| {
                finish(&state, &gesture, window, cx, FinishMode::CancelCapture)
            });
        }
        if self.disabled || inactive && active_thumb.is_some() {
            return if let Some(disabled_name) = disabled_name {
                BaseSlider::new(&self.state)
                    .axis(self.axis)
                    .disabled(true)
                    .accessibility_label(disabled_name)
                    .child(root)
                    .into_any_element()
            } else {
                root.into_any_element()
            };
        }

        attach_pointer_handlers(
            root,
            self.state,
            gesture,
            [start_focus, end_focus],
            geometry,
            range,
            start,
            end,
            layout.handle_sizes,
        )
    }
}

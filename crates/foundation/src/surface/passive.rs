use std::{cell::RefCell, rc::Rc};

use gpui::{AnyElement, IntoElement, ParentElement, Styled, canvas, div, prelude::FluentBuilder};
use gpui_base::StyledExt;

use super::{PolygonPaint, ResolvedShape, Surface};
use crate::Shape;

impl Surface {
    pub(super) fn render_passive(
        self,
        resolved_shape: ResolvedShape,
        polygon_paint: Option<PolygonPaint>,
    ) -> AnyElement {
        let clip_backdrop = self.clip_backdrop;
        if polygon_paint.is_none() && clip_backdrop.is_none() {
            return div()
                .refine_style(&self.style)
                .children(self.children)
                .into_any_element();
        }
        let prepared: Rc<RefCell<Option<Shape>>> = Rc::new(RefCell::new(None));
        let border = polygon_paint.as_ref().and_then(|paint| paint.border);
        let prepaint_shape = prepared.clone();
        let bg_shape = prepared.clone();
        let background_layer = polygon_paint.map(|paint| {
            let spec = resolved_shape.clone();
            canvas(
                move |bounds, _, _| {
                    *prepaint_shape.borrow_mut() = spec.build(bounds);
                },
                move |_, _, window, _| {
                    if let Some(shape) = bg_shape.borrow().as_ref() {
                        shape.paint_shadow(
                            paint.shadow_elevation,
                            paint.shadow_scale,
                            paint.shadow_color,
                            window,
                        );
                        shape.paint(paint.background, window);
                    }
                },
            )
            .absolute()
            .inset_0()
        });
        let needs_fg = clip_backdrop.is_some() || border.is_some();
        let fg_prepaint = prepared.clone();
        let fg_paint = prepared;
        let has_bg = background_layer.is_some();
        let foreground_layer = needs_fg.then(|| {
            canvas(
                move |bounds, _, _| {
                    if !has_bg {
                        *fg_prepaint.borrow_mut() = resolved_shape.build(bounds);
                    }
                },
                move |bounds, _, window, _| {
                    if let Some(shape) = fg_paint.borrow().as_ref() {
                        if let Some(backdrop) = clip_backdrop {
                            shape.paint_backdrop_mask(bounds, backdrop, window);
                        }
                        if let Some((width, color)) = border {
                            shape.paint_stroke(width, color, window);
                        }
                    }
                },
            )
            .absolute()
            .inset_0()
        });
        div()
            .relative()
            .when(clip_backdrop.is_some(), Styled::overflow_hidden)
            .refine_style(&self.style)
            .children(background_layer)
            .children(self.children)
            .children(foreground_layer)
            .into_any_element()
    }
}

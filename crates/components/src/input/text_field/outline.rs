use std::{cell::Cell, rc::Rc};

use gpui::{
    BorderStyle, Bounds, ContentMask, Corners, Edges, Hsla, Pixels, Styled, canvas, point, px, quad,
};
use gpui_material_theme::{CornerShape, Direction};

#[allow(clippy::too_many_arguments)]
pub(super) fn build_outline_canvas(
    shape: CornerShape,
    direction: Direction,
    background: Hsla,
    filled: bool,
    outline_width: Pixels,
    outline_tint: Hsla,
    cutout: Option<Rc<Cell<Option<Bounds<Pixels>>>>>,
) -> gpui::Canvas<()> {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let [top_left, top_right, bottom_right, bottom_left] = shape
                .radii(
                    bounds.size.width.into(),
                    bounds.size.height.into(),
                    direction,
                )
                .map(px);
            let corners = Corners {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            };
            if background.a > 0. {
                window.paint_quad(quad(
                    bounds,
                    corners,
                    background,
                    px(0.),
                    gpui::transparent_black(),
                    BorderStyle::Solid,
                ));
            }
            if outline_width <= px(0.) {
                return;
            }
            let border = quad(
                bounds,
                corners,
                gpui::transparent_black(),
                if filled {
                    Edges {
                        bottom: outline_width,
                        ..Default::default()
                    }
                } else {
                    Edges::all(outline_width)
                },
                outline_tint,
                BorderStyle::Solid,
            );
            // The outer edge and stroke width each snap independently.
            // Account for their combined rounding when opening the notch.
            let Some(label_bounds) =
                cutout
                    .as_ref()
                    .and_then(|cutout| cutout.get())
                    .filter(|label| {
                        let device_pixel = px(1. / window.scale_factor());
                        label.top() < bounds.top() + outline_width + device_pixel
                            && label.bottom() > bounds.top() - device_pixel
                    })
            else {
                window.paint_quad(border);
                return;
            };
            // The missing stroke must clear the complete label in each
            // frame that overlaps it, including when focus is leaving.
            let left = label_bounds.left().max(bounds.left());
            let right = label_bounds.right().min(bounds.right());
            for (min, max) in [
                (bounds.origin, point(left, bounds.bottom())),
                (point(right, bounds.top()), bounds.bottom_right()),
                // Only the bottom edge must be restored between the two
                // side masks. Start in the interior, so outward clip
                // rounding cannot expose a snapped top-stroke pixel.
                (
                    point(left, bounds.center().y),
                    point(right, bounds.bottom()),
                ),
            ] {
                window.with_content_mask(
                    Some(ContentMask {
                        bounds: Bounds::from_corners(min, max),
                    }),
                    |window| window.paint_quad(border.clone()),
                );
            }
        },
    )
    .absolute()
    .inset_0()
}

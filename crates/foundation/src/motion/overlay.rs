use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, Point, Size, Style, Styled, Window, point,
    px,
};
use gpui_material_theme::{MotionKind, MotionSpec, MotionSpeed};

use crate::{animate, current_material_theme};

const MIN_OVERLAY_SCALE: f32 = 1e-3;

/// Sanitizes an overlay scale factor so zero, negative, or non-finite scales cannot
/// produce division-by-zero layout sizes or degenerate transforms.
pub const fn sanitize_overlay_scale(scale: f32) -> f32 {
    if !scale.is_finite() || scale <= MIN_OVERLAY_SCALE {
        MIN_OVERLAY_SCALE
    } else {
        scale
    }
}

/// Sampled scale and alpha values for Material overlay enter/exit transitions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayMotion {
    /// Current overlay scale factor (typically `0.8..=1.0`, guarded against zero).
    pub scale: f32,
    /// Current overlay opacity in `0.0..=1.0`.
    pub alpha: f32,
}

impl OverlayMotion {
    /// Samples keyed spatial (scale) and effects (alpha) channels for an overlay.
    ///
    /// Returns `None` once closed and fully faded out.
    pub fn sample(id: &ElementId, open: bool, window: &mut Window, cx: &mut App) -> Option<Self> {
        let id = (id.clone(), "overlay-motion");
        let mounted = window.use_keyed_state(id.clone(), cx, |_, _| false);
        let initial = !*mounted.read(cx);
        if initial && !open {
            return None;
        }
        let scheme = current_material_theme(window, cx).motion;
        let reduced = cx.reduce_motion();
        let mut sample = |channel, kind, closed| {
            let id = (ElementId::from(id.clone()), channel);
            if initial {
                animate(id.clone(), closed, MotionSpec::Instant, window, cx);
            }
            animate(
                id,
                if open { 1. } else { closed },
                scheme.spec(kind, MotionSpeed::Fast, reduced),
                window,
                cx,
            )
        };
        let scale = sanitize_overlay_scale(sample("scale", MotionKind::Spatial, 0.8));
        let alpha = sample("alpha", MotionKind::Effects, 0.).clamp(0., 1.);
        let render = open || alpha > 0.;
        if initial == render {
            mounted.update(cx, |mounted, _| *mounted = render);
        }
        render.then_some(Self { scale, alpha })
    }

    /// Returns [`Self::scale`] clamped safely above zero for division and sizing.
    pub const fn safe_scale(self) -> f32 {
        sanitize_overlay_scale(self.scale)
    }

    /// Multiplies a logical pixel scalar by [`Self::safe_scale`].
    pub fn px(self, value: f32) -> Pixels {
        px(value * self.safe_scale())
    }

    /// Computes the prepaint origin for scaling `bounds` around normalized `pivot` (`0.0..=1.0`).
    pub fn origin(self, bounds: Bounds<Pixels>, pivot: Point<f32>) -> Point<Pixels> {
        let scale = self.safe_scale();
        bounds.origin
            + point(
                bounds.size.width * (1. - scale) * pivot.x,
                bounds.size.height * (1. - scale) * pivot.y,
            )
    }

    /// Scales an element's text size and line height proportionally to [`Self::safe_scale`].
    pub fn text<E: Styled>(self, element: E, window: &Window) -> E {
        let scale = self.safe_scale();
        let style = window.text_style();
        let rem = window.rem_size();
        element
            .text_size(style.font_size.to_pixels(rem) * scale)
            .line_height(style.line_height_in_pixels(rem) * scale)
    }
}

/// Preserves the full-size footprint for popup positioning while its scaled
/// child is prepainted around the resolved transform origin, with zero-scale protection.
pub struct ScaledOverlay<F> {
    child: AnyElement,
    motion: OverlayMotion,
    pivot: F,
    width: Option<Pixels>,
}

impl<F: Fn(Bounds<Pixels>) -> Point<f32> + 'static> ScaledOverlay<F> {
    /// Wraps `child` so it lays out at `motion.safe_scale()` while reserving full popup bounds.
    pub fn new(child: impl IntoElement, motion: OverlayMotion, pivot: F) -> Self {
        Self {
            child: child.into_any_element(),
            motion,
            pivot,
            width: None,
        }
    }

    /// Constrains the unscaled overlay footprint width.
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Returns the sanitized scale factor used during layout and prepaint.
    pub const fn safe_scale(&self) -> f32 {
        self.motion.safe_scale()
    }
}

impl<F: Fn(Bounds<Pixels>) -> Point<f32> + 'static> Element for ScaledOverlay<F> {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let scale = self.motion.safe_scale();
        let measured = self.child.layout_as_root(
            Size {
                width: self.width.map_or(AvailableSpace::MaxContent, |width| {
                    AvailableSpace::Definite(width * scale)
                }),
                height: AvailableSpace::MaxContent,
            },
            window,
            cx,
        );
        let mut style = Style::default();
        style.size.width = self.width.unwrap_or(measured.width / scale).into();
        style.size.height = (measured.height / scale).into();
        (window.request_layout(style, None, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child
            .prepaint_at(self.motion.origin(bounds, (self.pivot)(bounds)), window, cx);
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}

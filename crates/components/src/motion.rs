use std::time::Duration;

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Element, ElementId, GlobalElementId, Hsla,
    InspectorElementId, IntoElement, LayoutId, Pixels, Point, Rgba, Size, Style, Styled, Window,
    point, px,
};
use gpui_base::motion::{
    Easing, MotionStatus, Presence, PresencePhase, PresenceSample, Transition,
};
use gpui_material_foundation::{animate, window_material_theme};
use gpui_material_theme::{MotionKind, MotionSpec, MotionSpeed};

#[derive(Clone, Copy)]
pub(crate) struct OverlayMotion {
    pub(crate) scale: f32,
    pub(crate) alpha: f32,
}

impl OverlayMotion {
    pub(crate) fn sample(
        id: &ElementId,
        open: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<Self> {
        let id = (id.clone(), "overlay-motion");
        let mounted = window.use_keyed_state(id.clone(), cx, |_, _| false);
        let initial = !*mounted.read(cx);
        if initial && !open {
            return None;
        }
        let scheme = window_material_theme(window, cx).motion;
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
        let scale = sample("scale", MotionKind::Spatial, 0.8);
        let alpha = sample("alpha", MotionKind::Effects, 0.).clamp(0., 1.);
        let render = open || alpha > 0.;
        if initial == render {
            mounted.update(cx, |mounted, _| *mounted = render);
        }
        render.then_some(Self { scale, alpha })
    }

    pub(crate) fn px(self, value: f32) -> Pixels {
        px(value * self.scale)
    }

    pub(crate) fn origin(self, bounds: Bounds<Pixels>, pivot: Point<f32>) -> Point<Pixels> {
        bounds.origin
            + point(
                bounds.size.width * (1. - self.scale) * pivot.x,
                bounds.size.height * (1. - self.scale) * pivot.y,
            )
    }

    pub(crate) fn text<E: Styled>(self, element: E, window: &Window) -> E {
        let style = window.text_style();
        let rem = window.rem_size();
        element
            .text_size(style.font_size.to_pixels(rem) * self.scale)
            .line_height(style.line_height_in_pixels(rem) * self.scale)
    }
}

/// Preserves the full-size footprint for popup positioning while its scaled
/// child is prepainted around the resolved transform origin.
pub(crate) struct ScaledOverlay<F> {
    child: AnyElement,
    motion: OverlayMotion,
    pivot: F,
    width: Option<Pixels>,
}

impl<F: Fn(Bounds<Pixels>) -> Point<f32> + 'static> ScaledOverlay<F> {
    pub(crate) fn new(child: impl IntoElement, motion: OverlayMotion, pivot: F) -> Self {
        Self {
            child: child.into_any_element(),
            motion,
            pivot,
            width: None,
        }
    }

    pub(crate) fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }
}

impl<F: Fn(Bounds<Pixels>) -> Point<f32> + 'static> IntoElement for ScaledOverlay<F> {
    type Element = Self;

    fn into_element(self) -> Self {
        self
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
        let measured = self.child.layout_as_root(
            Size {
                width: self.width.map_or(AvailableSpace::MaxContent, |width| {
                    AvailableSpace::Definite(width * self.motion.scale)
                }),
                height: AvailableSpace::MaxContent,
            },
            window,
            cx,
        );
        let mut style = Style::default();
        style.size.width = self
            .width
            .unwrap_or(measured.width / self.motion.scale)
            .into();
        style.size.height = (measured.height / self.motion.scale).into();
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

pub(crate) fn menu_pivot(menu: Bounds<Pixels>, anchor: Bounds<Pixels>) -> Point<f32> {
    fn axis(start: Pixels, end: Pixels, anchor_start: Pixels, anchor_end: Pixels) -> f32 {
        if start >= anchor_end || start == end {
            0.
        } else if end <= anchor_start {
            1.
        } else {
            ((start.max(anchor_start) + end.min(anchor_end)) / 2. - start) / (end - start)
        }
    }
    point(
        axis(menu.left(), menu.right(), anchor.left(), anchor.right()),
        axis(menu.top(), menu.bottom(), anchor.top(), anchor.bottom()),
    )
}

pub(crate) fn presence(
    id: &ElementId,
    visible: bool,
    duration_ms: u64,
    window: &mut Window,
    cx: &mut App,
) -> PresenceSample {
    const ABSENT: PresenceSample = PresenceSample {
        phase: PresencePhase::Absent,
        progress: 0.,
        status: MotionStatus::Finished,
    };
    let mounted =
        window.use_keyed_state((id.clone(), "material-presence-mounted"), cx, |_, _| false);
    let was_mounted = *mounted.read(cx);
    if !visible && !was_mounted {
        return ABSENT;
    }
    let sample = Presence::new((id.clone(), "material-presence"), visible)
        .transition(Transition::new(Duration::from_millis(duration_ms)).easing(Easing::EaseOut))
        .sample(window, cx);
    let render = visible || sample.progress > 0.;
    if was_mounted != render {
        mounted.update(cx, |mounted, _| *mounted = render);
    }
    if render { sample } else { ABSENT }
}

pub(crate) fn mix(from: Hsla, to: Hsla, fraction: f32) -> Hsla {
    let from = Rgba::from(from);
    let to = Rgba::from(to);
    let mix = |a, b| a + (b - a) * fraction;
    Rgba {
        r: mix(from.r, to.r),
        g: mix(from.g, to.g),
        b: mix(from.b, to.b),
        a: mix(from.a, to.a),
    }
    .into()
}

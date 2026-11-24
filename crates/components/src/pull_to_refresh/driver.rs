use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, ElementId, GlobalElementId, Hitbox, HitboxBehavior,
    InspectorElementId, LayoutId, Pixels, ScrollEvent, TouchPhase, WeakEntity, Window, px,
};
use gpui_material_foundation::is_bounds_visible;

use super::state::{
    PullGesture, PullToRefreshState, ScrollBinding, ViewportKey, meets_refresh_threshold,
};

pub(super) type Refresh = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Clone)]
pub(super) struct RefreshDriver {
    pub(super) state: WeakEntity<PullToRefreshState>,
    pub(super) viewport: ViewportKey,
    pub(super) refresh: Refresh,
    pub(super) threshold: Pixels,
    pub(super) binding: ScrollBinding,
}

pub(super) fn is_pull_scroll_event(event: &ScrollEvent) -> bool {
    if !event.delta.precise() {
        return false;
    }
    if event.modifiers.control || event.modifiers.platform || event.modifiers.alt {
        return false;
    }
    let delta = event.delta.pixel_delta(px(1.));
    f32::from(delta.x).is_finite() && f32::from(delta.y).is_finite()
}

impl RefreshDriver {
    pub(super) fn request(&self, window: &mut Window, cx: &mut App) {
        let accepted = self.state.update(cx, |state, cx| {
            if state.viewport.as_ref() != Some(&self.viewport) || !state.can_request() {
                return false;
            }
            state.pending = true;
            state.gesture = None;
            state.settle_to(1., cx);
            cx.notify();
            true
        });
        if accepted.unwrap_or(false) {
            window.refresh();
            (self.refresh)(window, cx);
        }
    }

    fn listen(self, hitbox: Hitbox, window: &mut Window) {
        window.on_mouse_event(move |event: &ScrollEvent, phase, window, cx| {
            if !is_pull_scroll_event(event) {
                return;
            }
            let hits = hitbox.should_handle_scroll(window);
            let delta = event.delta.pixel_delta(px(1.));
            let vertical = delta.y != px(0.) && delta.y.abs() >= delta.x.abs();
            let horizontal = delta.x != px(0.) && delta.x.abs() > delta.y.abs();
            let before = self.binding.distance_from_top(window, cx);
            let (consumed, request) = self
                .state
                .update(cx, |state, cx| {
                    if state.viewport.as_ref() != Some(&self.viewport) || !state.can_request() {
                        return (false, false);
                    }
                    if phase.capture() {
                        if event.touch_phase == TouchPhase::Started && hits {
                            if state
                                .gesture
                                .as_ref()
                                .is_some_and(|gesture| gesture.distance > px(0.))
                            {
                                state.settle_to(0., cx);
                                cx.notify();
                            }
                            state.gesture = Some(PullGesture {
                                source: event.source,
                                distance: px(0.),
                                before_scroll: before,
                            });
                        }
                        let Some(gesture) = state
                            .gesture
                            .as_mut()
                            .filter(|gesture| gesture.source == event.source)
                        else {
                            return (false, false);
                        };
                        let pulling = gesture.distance > px(0.);
                        if !pulling && horizontal {
                            return (false, false);
                        }
                        if matches!(event.touch_phase, TouchPhase::Ended | TouchPhase::Cancelled)
                            || !hits
                        {
                            let request = hits
                                && event.touch_phase == TouchPhase::Ended
                                && pulling
                                && meets_refresh_threshold(state.fraction);
                            state.gesture = None;
                            if pulling && !request {
                                state.settle_to(0., cx);
                                cx.notify();
                            }
                            return (pulling && hits, request);
                        }
                        gesture.before_scroll = before;
                        if pulling && vertical {
                            state.pull_by(delta.y, self.threshold, cx);
                            return (true, false);
                        }
                    } else if hits
                        && vertical
                        && delta.y > px(0.)
                        && let Some(gesture) = state
                            .gesture
                            .as_ref()
                            .filter(|gesture| gesture.source == event.source)
                    {
                        // The scroll owner has handled as much of the delta as fits above its top.
                        let remainder = delta.y - gesture.before_scroll;
                        if remainder > px(0.) && before <= px(0.) {
                            self.binding.remove_overscroll();
                            state.pull_by(remainder, self.threshold, cx);
                            return (true, false);
                        }
                    }
                    (false, false)
                })
                .unwrap_or_default();
            if consumed {
                window.refresh();
                cx.stop_propagation();
            }
            if request {
                self.request(window, cx);
            }
        });
    }
}

/// Registers before the scroll child in paint order so the child receives the bubble first.
pub(super) struct RefreshViewport {
    pub(super) driver: RefreshDriver,
    pub(super) body: AnyElement,
}

impl gpui::Element for RefreshViewport {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.body.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Hitbox {
        self.body.prepaint(window, cx);
        window.insert_hitbox(bounds, HitboxBehavior::Normal)
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        hitbox: &mut Hitbox,
        window: &mut Window,
        cx: &mut App,
    ) {
        let visible = is_bounds_visible(bounds, window);
        let _ = self.driver.state.update(cx, |state, _| {
            state.set_visible(visible, window);
        });
        self.driver.clone().listen(hitbox.clone(), window);
        self.body.paint(window, cx);
    }
}

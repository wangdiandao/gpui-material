use std::{cell::RefCell, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, Bounds, ContentMask, Element, ElementId, Entity,
    GlobalElementId, Hsla, InspectorElementId, InteractiveElement, IntoElement, LayoutId,
    ObjectFit, ParentElement, Pixels, StatefulInteractiveElement, Style, Styled, StyledImage,
    Window, div, img, point, prelude::FluentBuilder, px, relative, size,
};
use gpui_base::Button;
use gpui_material_theme::{CornerRole, Direction};

use super::{
    Activate, CarouselItem, CarouselState, CarouselVariant, Content,
    geometry::{Layout, LayoutOptions},
    state::ItemFocus,
};

pub(super) struct Viewport {
    pub(super) id: ElementId,
    pub(super) variant: CarouselVariant,
    pub(super) height: Pixels,
    pub(super) preferred_width: Pixels,
    pub(super) layout_options: LayoutOptions,
    pub(super) user_scroll_enabled: bool,
    pub(super) items: Vec<Option<CarouselItem>>,
    pub(super) focus: Rc<[ItemFocus]>,
    pub(super) focused: Option<usize>,
    pub(super) tab_stop: Option<usize>,
    pub(super) state: Entity<CarouselState>,
    pub(super) sample: f32,
    pub(super) direction: Direction,
    pub(super) ring: Hsla,
    pub(super) hover: Hsla,
    pub(super) surface: Hsla,
    pub(super) activate: Option<Activate>,
    pub(super) layout: Rc<RefCell<Option<Layout>>>,
}

pub(super) struct Prepaint {
    children: Vec<AnyElement>,
    mask: Bounds<Pixels>,
}

impl Element for Viewport {
    type RequestLayoutState = ();
    type PrepaintState = Prepaint;

    fn id(&self) -> Option<ElementId> {
        Some((self.id.clone(), "viewport").into())
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
        let style = Style {
            size: size(relative(1.).into(), self.height.into()),
            ..Style::default()
        };
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let vertical = self.variant.is_vertical();
        let layout = layout(
            self.variant,
            bounds.size.width,
            bounds.size.height,
            self.preferred_width,
            self.layout_options,
            self.items.iter().flatten(),
        );
        let (inset, cross) = layout.cross_bounds(self.direction);
        let (inset, cross) = (px(inset), px(cross));
        let mut visible = layout.visible(self.sample, self.direction);
        // When keyboard or UI Automation focuses an item outside the current
        // mask, reveal it in this frame rather than leaving an invisible focus
        // target during a long animation.
        if let Some(focused) = self.focused
            && !self.state.read(cx).direct
            && self.state.read(cx).requested.round() as usize == focused
            && !visible.iter().any(|(index, item)| {
                *index == focused
                    && item.size >= 40.
                    && item.start >= -0.5
                    && item.start + item.size
                        <= f32::from(if vertical {
                            bounds.size.height
                        } else {
                            bounds.size.width
                        }) + 0.5
            })
        {
            visible = layout.visible(focused as f32, self.direction);
        }
        let (start, end) = layout.clip_range(self.direction);
        *self.layout.borrow_mut() = Some(layout);
        let mask = if vertical {
            Bounds::new(
                bounds.origin + point(inset, px(start)),
                size(cross, px(end - start)),
            )
        } else {
            Bounds::new(
                bounds.origin + point(px(start), inset),
                size(px(end - start), cross),
            )
        };
        let mut children = Vec::with_capacity(visible.len());
        let count = self.items.len();
        window.with_content_mask(Some(ContentMask { bounds: mask }), |window| {
            for (index, geometry) in visible {
                let Some(item) = self.items[index].take() else {
                    continue;
                };
                let (size, offset) = if vertical {
                    (
                        size(cross, px(geometry.size)),
                        point(inset, px(geometry.start)),
                    )
                } else {
                    (
                        size(px(geometry.size), cross),
                        point(px(geometry.start), inset),
                    )
                };
                let radius = if vertical {
                    0.
                } else {
                    CornerRole::ExtraLarge.shape().radii(
                        f32::from(size.width),
                        f32::from(size.height),
                        self.direction,
                    )[0]
                };
                let id = item.id;
                let activate_id = id.clone();
                let accessible_id = id.clone();
                let activate = self.activate.clone();
                let activate_accessible = activate.clone();
                let state = self.state.clone();
                let accessible_state = state.clone();
                let accessible_focus = self.focus[index].focus.clone();
                let user_scroll_enabled = self.user_scroll_enabled;
                let content = match item.content {
                    Content::Element(content) => content,
                    Content::Image(source) => img(source)
                        .size_full()
                        .object_fit(ObjectFit::Cover)
                        .rounded(px(radius))
                        .into_any_element(),
                };
                let focused = self.focused == Some(index) && window.last_input_was_keyboard();
                let mut child = Button::new((self.id.clone(), id))
                    .accessibility_label(item.label)
                    .aria_position_in_set(index + 1)
                    .aria_size_of_set(count)
                    .track_focus(&self.focus[index].focus)
                    .tab_stop(self.tab_stop == Some(index))
                    .disabled(item.disabled)
                    .w(size.width)
                    .h(size.height)
                    .bg(self.surface)
                    .rounded(px(radius))
                    .overflow_hidden()
                    .relative()
                    .group("carousel-item")
                    .when(!item.disabled, |button| {
                        button
                            .on_click(move |_, window, cx| {
                                if user_scroll_enabled {
                                    state.update(cx, |state, cx| {
                                        state.animate_scroll_to_item(index, cx)
                                    });
                                }
                                window.refresh();
                                if let Some(activate) = &activate {
                                    activate(&activate_id, window, cx);
                                }
                            })
                            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                if user_scroll_enabled {
                                    accessible_state.update(cx, |state, cx| {
                                        state.animate_scroll_to_item(index, cx)
                                    });
                                }
                                accessible_focus.focus(window, cx);
                                window.refresh();
                                if let Some(activate) = &activate_accessible {
                                    activate(&accessible_id, window, cx);
                                }
                            })
                    })
                    .child(content)
                    .child(
                        div()
                            .id("state-layer")
                            .absolute()
                            .inset_0()
                            .rounded(px(radius))
                            .when(!item.disabled, |state| {
                                state
                                    .group_hover("carousel-item", |style| {
                                        style.bg(self.hover.opacity(0.08))
                                    })
                                    .group_active("carousel-item", |style| {
                                        style.bg(self.hover.opacity(0.1))
                                    })
                                    .when(focused, |state| state.border_2().border_color(self.ring))
                            }),
                    )
                    .into_any_element();
                child.prepaint_as_root(bounds.origin + offset, size.map(Into::into), window, cx);
                children.push(child);
            }
        });
        Prepaint { children, mask }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        state: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_content_mask(Some(ContentMask { bounds: state.mask }), |window| {
            for child in &mut state.children {
                child.paint(window, cx);
            }
        });
    }
}

fn layout<'a>(
    variant: CarouselVariant,
    width: Pixels,
    height: Pixels,
    preferred_width: Pixels,
    options: LayoutOptions,
    items: impl IntoIterator<Item = &'a CarouselItem>,
) -> Layout {
    let (main, cross) = if variant.is_vertical() {
        (height, width)
    } else {
        (width, height)
    };
    Layout::new(
        variant,
        main.into(),
        cross.into(),
        preferred_width.into(),
        options,
        items.into_iter().map(|item| item.aspect),
    )
}

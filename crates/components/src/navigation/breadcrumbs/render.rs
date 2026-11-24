use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, AvailableSpace, Bounds, ContentMask, Element, ElementId,
    FocusHandle, GlobalElementId, InspectorElementId, InteractiveElement, IntoElement, LayoutId,
    ParentElement, Pixels, Point, RenderOnce, Role, SharedString, Size, StatefulInteractiveElement,
    Style, Styled, Window, div, point, prelude::FluentBuilder, px, relative, size,
};
use gpui_base::{Button as BaseButton, TestSupportExt};
use gpui_material_foundation::{ThemeScope, current_material_theme, text_style};
use gpui_material_theme::{Direction, TypeRole, TypographyEmphasis};

use crate::{Icon, IconName, Menu, MenuItem, navigation::overflow_strip::OverflowStripHandle};

use super::{
    BreadcrumbItem, Breadcrumbs,
    layout::{CollapsePlan, compute_collapse_plan},
    types::{IconFactory, ResolvedColors},
};

const ROW_MIN_HEIGHT: Pixels = px(32.);
const ITEM_GAP: Pixels = px(4.);
const OVERFLOW_TRIGGER_WIDTH: Pixels = px(32.);
const OVERFLOW_TRIGGER_HEIGHT: Pixels = px(32.);

impl RenderOnce for Breadcrumbs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let label = self.accessible_label.clone();
        div()
            .id(id)
            .test_support()
            .role(Role::Navigation)
            .aria_label(label)
            .w_full()
            .overflow_hidden()
            .child(BreadcrumbsLayout::new(self, window, cx))
    }
}

struct PreparedItem {
    id: SharedString,
    label: SharedString,
    icon: Option<IconFactory>,
    disabled: bool,
    is_current: bool,
    focus: Option<FocusHandle>,
    inline: AnyElement,
    preferred: Size<Pixels>,
}

struct BreadcrumbsLayout {
    breadcrumbs: Breadcrumbs,
    direction: Direction,
    colors: ResolvedColors,
    overflow: OverflowStripHandle<SharedString>,
}

impl BreadcrumbsLayout {
    fn new(breadcrumbs: Breadcrumbs, window: &mut Window, cx: &mut App) -> Self {
        let overflow =
            OverflowStripHandle::use_state(&breadcrumbs.id, "breadcrumbs-overflow", window, cx);
        Self {
            direction: current_material_theme(window, cx).layout.direction,
            colors: breadcrumbs.colors.resolve(window, cx),
            overflow,
            breadcrumbs,
        }
    }

    fn make_separator(&self) -> AnyElement {
        let glyph = match &self.breadcrumbs.separator {
            Some(factory) => factory(),
            None => Icon::new(IconName::ChevronRight)
                .size(px(16.))
                .mirror_in_rtl(true)
                .into_any_element(),
        };
        div()
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .px(ITEM_GAP)
            .text_color(self.colors.separator)
            .child(glyph)
            .into_any_element()
    }

    fn prepare_item(
        &self,
        item: BreadcrumbItem,
        is_current: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> PreparedItem {
        let rtl = self.direction == Direction::Rtl;
        let elem_id = ElementId::from((self.breadcrumbs.id.clone(), item.id.clone()));

        if is_current {
            let fg = if item.disabled {
                self.colors.disabled
            } else {
                self.colors.current
            };
            let current_aria: SharedString = format!("{}, current page", item.label).into();
            let mut inline = div()
                .id(elem_id)
                .test_support()
                .role(Role::Label)
                .aria_label(current_aria)
                .h(ROW_MIN_HEIGHT)
                .px(px(6.))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.))
                .when(rtl, Styled::flex_row_reverse)
                .text_color(fg)
                .children(
                    item.icon
                        .as_ref()
                        .map(|icon| div().flex_none().text_size(px(16.)).child(icon())),
                )
                .child(
                    text_style(
                        div().whitespace_nowrap(),
                        TypeRole::TitleSmall.style(TypographyEmphasis::Baseline),
                        1.,
                        window,
                        cx,
                    )
                    .child(item.label.clone()),
                )
                .into_any_element();
            let preferred = inline.layout_as_root(
                size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
                window,
                cx,
            );
            return PreparedItem {
                id: item.id,
                label: item.label,
                icon: item.icon,
                disabled: item.disabled,
                is_current: true,
                focus: None,
                inline,
                preferred,
            };
        }

        let focus = window.with_id(
            ElementId::from((self.breadcrumbs.id.clone(), "breadcrumb-focus")),
            |window| {
                window
                    .use_keyed_state(item.id.clone(), cx, |_, cx| cx.focus_handle())
                    .read(cx)
                    .clone()
            },
        );

        let fg = if item.disabled {
            self.colors.disabled
        } else {
            self.colors.ancestor
        };
        let hover_bg = self.colors.hover;
        let focus_ring = self.colors.focus_ring;

        let activate = (!item.disabled).then(|| {
            self.breadcrumbs.on_select.as_ref().map(|on_select| {
                let on_select = on_select.clone();
                let item_id = item.id.clone();
                Rc::new(ThemeScope::bind_action(
                    move |window, cx| on_select(item_id.clone(), window, cx),
                    window,
                    cx,
                ))
            })
        });
        let activate = activate.flatten();

        let click_handler = activate.clone();
        let a11y_handler = activate;

        let mut inline = BaseButton::new(elem_id)
            .track_focus(&focus)
            .disabled(item.disabled)
            .accessibility_label(item.label.clone())
            .h(ROW_MIN_HEIGHT)
            .px(px(6.))
            .rounded(px(8.))
            .border_2()
            .border_color(gpui::transparent_black())
            .flex()
            .flex_row()
            .items_center()
            .gap(px(4.))
            .when(rtl, Styled::flex_row_reverse)
            .text_color(fg)
            .when(!item.disabled, move |button| {
                button
                    .hover(move |b| b.bg(hover_bg))
                    .focus_visible(move |b| b.border_color(focus_ring))
            })
            .when_some(click_handler, |button, handler| {
                button.on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    handler(window, cx);
                })
            })
            .when_some(a11y_handler, |button, handler| {
                button.on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                    handler(window, cx);
                })
            })
            .children(
                item.icon
                    .as_ref()
                    .map(|icon| div().flex_none().text_size(px(16.)).child(icon())),
            )
            .child(
                text_style(
                    div().whitespace_nowrap(),
                    TypeRole::LabelLarge.style(TypographyEmphasis::Baseline),
                    1.,
                    window,
                    cx,
                )
                .child(item.label.clone()),
            )
            .into_any_element();

        let preferred = inline.layout_as_root(
            size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
            window,
            cx,
        );

        PreparedItem {
            id: item.id,
            label: item.label,
            icon: item.icon,
            disabled: item.disabled,
            is_current: false,
            focus: Some(focus),
            inline,
            preferred,
        }
    }

    fn transfer_focus(
        &self,
        plan: &CollapsePlan,
        items: &[PreparedItem],
        window: &mut Window,
        cx: &mut App,
    ) {
        let has_overflow = !plan.collapsed.is_empty();
        let hidden_focused = has_overflow.then(|| {
            items[plan.collapsed.clone()]
                .iter()
                .find(|item| {
                    item.focus
                        .as_ref()
                        .is_some_and(|focus| focus.contains_focused(window, cx))
                })
                .map(|item| {
                    let fallback = item
                        .focus
                        .clone()
                        .unwrap_or_else(|| self.overflow.focus.clone());
                    (item.id.clone(), fallback)
                })
        });
        self.overflow.transfer_focus(
            has_overflow,
            hidden_focused.flatten(),
            |id| {
                plan.leading
                    .clone()
                    .chain(plan.trailing.clone())
                    .any(|idx| {
                        !items[idx].disabled && !items[idx].is_current && items[idx].id == *id
                    })
            },
            || {
                items
                    .iter()
                    .find(|item| !item.disabled && !item.is_current)
                    .and_then(|item| item.focus.clone())
            },
            window,
            cx,
        );
    }

    fn build_overflow_menu(
        &self,
        collapsed: &[PreparedItem],
        trigger_size: Size<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let entries: Vec<MenuItem> = collapsed
            .iter()
            .map(|item| {
                let mut entry = MenuItem::new(
                    ElementId::from((self.breadcrumbs.id.clone(), item.id.clone())),
                    item.label.clone(),
                )
                .disabled(item.disabled);
                if let Some(icon) = item.icon.clone() {
                    entry = entry.leading(move || icon());
                }
                if !item.disabled
                    && let Some(on_select) = self.breadcrumbs.on_select.clone()
                {
                    let item_id = item.id.clone();
                    let bound = ThemeScope::bind_action(
                        move |window, cx| on_select(item_id.clone(), window, cx),
                        window,
                        cx,
                    );
                    entry = entry.on_activate(move |window, cx| bound(window, cx));
                }
                entry
            })
            .collect();

        let controller = self.overflow.controller.clone();
        let observed = controller.clone();
        let trigger_focus = self.overflow.focus.clone();
        let trigger_id = ElementId::from((self.breadcrumbs.id.clone(), "overflow"));
        let label = self.breadcrumbs.overflow_label.clone();
        let ink = self.colors.ancestor;
        let hover_bg = self.colors.hover;
        let focus_ring = self.colors.focus_ring;

        Menu::composed(
            (self.breadcrumbs.id.clone(), "overflow-menu"),
            entries,
            move |_, toggle| {
                let return_focus = trigger_focus.clone();
                let dots = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(2.5))
                    .children((0..3).map(|_| div().size(px(3.5)).rounded_full().bg(ink)));
                BaseButton::new(trigger_id)
                    .track_focus(&trigger_focus)
                    .accessibility_label(label)
                    .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                        return_focus.focus(window, cx);
                        toggle(window, cx);
                    })
                    .w(trigger_size.width)
                    .h(trigger_size.height)
                    .p_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    .rounded(px(8.))
                    .border_2()
                    .border_color(gpui::transparent_black())
                    .text_color(ink)
                    .hover(move |button| button.bg(hover_bg))
                    .focus_visible(move |button| button.border_color(focus_ring))
                    .child(dots)
                    .into_any_element()
            },
        )
        .open(controller.is_showing())
        .on_open_change(move |open, window, cx| observed.set(open, window, cx))
        .into_any_element()
    }

    fn origin(&self, bounds: Bounds<Pixels>, child: Size<Pixels>, offset: Pixels) -> Point<Pixels> {
        let dy = (bounds.size.height - child.height).max(px(0.)) / 2.;
        match self.direction {
            Direction::Ltr => point(bounds.left() + offset, bounds.top() + dy),
            Direction::Rtl => point(bounds.right() - offset - child.width, bounds.top() + dy),
        }
    }
}

struct BreadcrumbsLayoutState {
    items: Vec<PreparedItem>,
    sep_width: Pixels,
}

struct BreadcrumbsPrepaintState {
    plan: CollapsePlan,
    separators: Vec<AnyElement>,
    menu: Option<AnyElement>,
}

impl Element for BreadcrumbsLayout {
    type RequestLayoutState = BreadcrumbsLayoutState;
    type PrepaintState = BreadcrumbsPrepaintState;

    fn id(&self) -> Option<ElementId> {
        Some(ElementId::from((self.breadcrumbs.id.clone(), "layout")))
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn a11y_role(&self) -> Option<Role> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let raw_items = std::mem::take(&mut self.breadcrumbs.items);
        let len = raw_items.len();
        let items: Vec<PreparedItem> = raw_items
            .into_iter()
            .enumerate()
            .map(|(idx, item)| self.prepare_item(item, idx + 1 == len, window, cx))
            .collect();

        let mut probe_sep = self.make_separator();
        let sep_size = probe_sep.layout_as_root(
            size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
            window,
            cx,
        );
        let sep_width = sep_size.width;

        let widths: Vec<Pixels> = items.iter().map(|i| i.preferred.width).collect();
        let preferred_plan = compute_collapse_plan(
            &widths,
            sep_width,
            OVERFLOW_TRIGGER_WIDTH,
            self.breadcrumbs.max_items,
            px(f32::INFINITY),
        );

        let visible_segments = preferred_plan.leading.len()
            + usize::from(!preferred_plan.collapsed.is_empty())
            + preferred_plan.trailing.len();
        let mut preferred_width = preferred_plan
            .leading
            .clone()
            .chain(preferred_plan.trailing.clone())
            .map(|i| items[i].preferred.width)
            .sum::<Pixels>();
        if !preferred_plan.collapsed.is_empty() {
            preferred_width += OVERFLOW_TRIGGER_WIDTH;
        }
        if visible_segments > 1 {
            preferred_width += sep_width * (visible_segments - 1) as f32;
        }

        let cross = items.iter().map(|item| item.preferred.height).fold(
            if items.is_empty() {
                px(0.)
            } else {
                ROW_MIN_HEIGHT.max(sep_size.height)
            },
            Pixels::max,
        );

        let style = Style {
            size: size(preferred_width.into(), cross.into()),
            min_size: size(px(0.).into(), px(0.).into()),
            max_size: size(relative(1.).into(), relative(1.).into()),
            ..Default::default()
        };

        (
            window.request_layout(style, None, cx),
            BreadcrumbsLayoutState { items, sep_width },
        )
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let widths: Vec<Pixels> = state.items.iter().map(|i| i.preferred.width).collect();
        let plan = compute_collapse_plan(
            &widths,
            state.sep_width,
            OVERFLOW_TRIGGER_WIDTH,
            self.breadcrumbs.max_items,
            bounds.size.width,
        );
        self.transfer_focus(&plan, &state.items, window, cx);

        let mut offset = px(0.);
        let mut separators = Vec::new();
        let mut placed_any = false;
        let mask = Some(ContentMask { bounds });

        let mut place_separator = |offset: &mut Pixels, window: &mut Window, cx: &mut App| {
            let mut sep = self.make_separator();
            let sep_size = sep.layout_as_root(
                size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
                window,
                cx,
            );
            let origin = self.origin(bounds, sep_size, *offset);
            window.with_content_mask(mask, |window| {
                sep.prepaint_at(origin, window, cx);
            });
            *offset += sep_size.width;
            separators.push(sep);
        };

        for idx in plan.leading.clone() {
            if placed_any {
                place_separator(&mut offset, window, cx);
            }
            let item = &mut state.items[idx];
            let origin = self.origin(bounds, item.preferred, offset);
            window.with_content_mask(mask, |window| {
                item.inline.prepaint_at(origin, window, cx);
            });
            offset += item.preferred.width;
            placed_any = true;
        }

        let menu = (!plan.collapsed.is_empty()).then(|| {
            if placed_any {
                place_separator(&mut offset, window, cx);
            }
            let trigger_size = size(
                OVERFLOW_TRIGGER_WIDTH.min((bounds.size.width - offset).max(px(0.))),
                OVERFLOW_TRIGGER_HEIGHT.min(bounds.size.height.max(ROW_MIN_HEIGHT)),
            );
            let mut menu = self.build_overflow_menu(
                &state.items[plan.collapsed.clone()],
                trigger_size,
                window,
                cx,
            );
            let measured =
                menu.layout_as_root(trigger_size.map(AvailableSpace::Definite), window, cx);
            let origin = self.origin(bounds, measured, offset);
            window.with_content_mask(mask, |window| {
                menu.prepaint_at(origin, window, cx);
            });
            offset += measured.width;
            placed_any = true;
            menu
        });

        for idx in plan.trailing.clone() {
            if placed_any {
                place_separator(&mut offset, window, cx);
            }
            let item = &mut state.items[idx];
            let origin = self.origin(bounds, item.preferred, offset);
            window.with_content_mask(mask, |window| {
                item.inline.prepaint_at(origin, window, cx);
            });
            offset += item.preferred.width;
            placed_any = true;
        }

        BreadcrumbsPrepaintState {
            plan,
            separators,
            menu,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let mask = Some(ContentMask { bounds });
        window.with_content_mask(mask, |window| {
            for idx in prepaint.plan.leading.clone() {
                state.items[idx].inline.paint(window, cx);
            }
            for sep in &mut prepaint.separators {
                sep.paint(window, cx);
            }
            for idx in prepaint.plan.trailing.clone() {
                state.items[idx].inline.paint(window, cx);
            }
            if let Some(menu) = &mut prepaint.menu {
                menu.paint(window, cx);
            }
        });
    }
}

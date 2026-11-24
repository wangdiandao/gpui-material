use super::*;

/// Logical position within a connected horizontal group.
#[derive(Clone, Copy)]
pub enum Segment {
    Only,
    Leading,
    Middle,
    Trailing,
    SplitLeading,
    SplitTrailing,
}

/// A Material action with a composed shape and a width assigned by its group.
#[derive(IntoElement)]
pub struct ButtonPart {
    common: Common,
    variant: ButtonVariant,
    selected: Option<bool>,
    presentation_selected: Option<bool>,
    on_activate: Option<InvokeAction>,
    on_change: Option<Change>,
    segment: Option<Segment>,
    icon: Option<AnyElement>,
    slots: Slots,
    width: Option<Pixels>,
    tab_stop: bool,
    icon_width: Option<IconButtonWidth>,
}

impl ButtonPart {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, width: Pixels) -> Self {
        Self {
            common: Common::new(id, label),
            variant: ButtonVariant::default(),
            selected: None,
            presentation_selected: None,
            on_activate: None,
            on_change: None,
            segment: None,
            icon: None,
            slots: Slots::default(),
            width: Some(width),
            tab_stop: true,
            icon_width: None,
        }
    }

    pub fn intrinsic(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        let mut part = Self::new(id, label, px(0.));
        part.width = None;
        part
    }

    common_builders!();
    appearance_builders!();
    slot_builders!();

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn segment(mut self, segment: Option<Segment>) -> Self {
        self.segment = segment;
        self
    }

    pub fn selected(mut self, selected: Option<bool>) -> Self {
        self.selected = selected;
        self
    }

    pub fn presentation_selected(mut self, selected: bool) -> Self {
        self.presentation_selected = Some(selected);
        self
    }

    pub fn on_change(mut self, callback: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(callback));
        self
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    pub fn icon_width(mut self, width: IconButtonWidth) -> Self {
        self.icon_width = Some(width);
        self
    }

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    pub fn on_activate(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(InvokeAction::Full(Rc::new(callback)));
        self
    }

    pub fn on_accessible_activate(
        mut self,
        callback: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(InvokeAction::AccessibleOnly(Rc::new(callback)));
        self
    }
}

impl RenderOnce for ButtonPart {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut metrics = self.icon_width.map_or_else(
            || self.common.size.metrics(),
            |width| self.common.size.icon_metrics(width).0,
        );
        let segment = self.segment.map(|segment| {
            use CornerRole::*;
            let (inner, pressed) = match segment {
                Segment::SplitLeading | Segment::SplitTrailing => match self.common.size {
                    ControlSize::ExtraSmall => (ExtraSmall, Small),
                    ControlSize::Small | ControlSize::Medium => (ExtraSmall, Medium),
                    ControlSize::Large => (Small, LargeIncreased),
                    ControlSize::ExtraLarge => (Medium, LargeIncreased),
                },
                _ => (Small, ExtraSmall),
            };
            SegmentShape {
                leading: matches!(
                    segment,
                    Segment::Only | Segment::Leading | Segment::SplitLeading
                ),
                trailing: matches!(
                    segment,
                    Segment::Only | Segment::Trailing | Segment::SplitTrailing
                ),
                outer: self.common.shape.corner(metrics.square, false),
                inner,
                pressed,
            }
        });
        let split = matches!(
            self.segment,
            Some(Segment::SplitLeading | Segment::SplitTrailing)
        );
        let palette_selection = if split { None } else { self.selected };
        let mut style = button_appearance(self.variant, palette_selection, self.common.tone).style(
            metrics,
            self.common.shape.corner(metrics.square, false),
            0.,
        );
        if self.selected.is_some() || self.presentation_selected.is_some() {
            style.selected_corner = Some(self.common.shape.corner(metrics.square, true));
        }
        if matches!(self.segment, Some(Segment::SplitTrailing)) {
            metrics.icon = split_trailing_size(self.common.size).0;
        }
        if split {
            let colors = &mut self.common.appearance.colors;
            colors.checked_content = colors.checked_content.or(colors.content);
            colors.checked_container = colors.checked_container.or(colors.container);
        }
        let checked = self
            .presentation_selected
            .or(self.selected)
            .unwrap_or(false);
        let tint = self
            .common
            .appearance
            .colors
            .content(self.common.disabled, checked)
            .unwrap_or_else(|| style.foreground(self.common.disabled, window, cx));
        let has_slots = self.slots.content.is_some()
            || self.slots.leading.is_some()
            || self.slots.trailing.is_some()
            || self.slots.padding.is_some();
        let content = if let Some(width) = self.width.filter(|_| !has_slots) {
            if let Some(icon) = self.icon {
                icon_slot(icon, metrics.icon).into_any_element()
            } else {
                let typography = metrics.label.style(TypographyEmphasis::Baseline);
                let padding = metrics.padding.min((f32::from(width) - 16.).max(0.) / 2.);
                let label_width = (width - px(2. * padding)).max(px(0.));
                div()
                    .flex()
                    .min_w_0()
                    .max_w_full()
                    .w(width)
                    .justify_center()
                    .px(px(padding))
                    .child(crate::text::single_line(
                        self.common.label.clone(),
                        typography,
                        1.,
                        tint,
                        (Some(label_width), 1., 1.),
                        window,
                        cx,
                    ))
                    .into_any_element()
            }
        } else {
            self.slots.width = self.width;
            if let Some(icon) = self.icon {
                self.slots.content = Some(icon_slot(icon, metrics.icon).into_any_element());
            }
            self.slots.padding = self
                .slots
                .padding
                .or_else(|| split_padding(self.common.size, self.segment));
            self.slots
                .render(self.common.label.clone(), metrics, 1., window, tint, cx)
        };
        let action = if let Some(selected) = self.selected {
            Action::Toggle {
                selected,
                on_change: self.on_change.or_else(|| {
                    self.on_activate.map(|action| {
                        let callback = action.into_callback();
                        Rc::new(move |_, window: &mut Window, cx: &mut App| callback(window, cx))
                            as Change
                    })
                }),
            }
        } else {
            Action::Invoke(self.on_activate)
        };
        ActionControl {
            id: self.common.id,
            label: self.common.label,
            focus: self.common.focus,
            disabled: self.common.disabled,
            action,
            style,
            content,
        }
        .render_layout(
            1.,
            ActionLayout {
                segment,
                width: self.width,
                tab_stop: self.tab_stop,
                appearance: self.common.appearance,
                presentation_selected: self.presentation_selected,
                ..Default::default()
            },
            self.common.interaction_source.as_ref(),
            window,
            cx,
        )
    }
}

fn split_padding(size: ControlSize, segment: Option<Segment>) -> Option<ButtonPadding> {
    let (start, end) = match segment {
        Some(Segment::SplitLeading) => match size {
            ControlSize::ExtraSmall => (12., 10.),
            ControlSize::Small => (16., 12.),
            ControlSize::Medium => (24., 24.),
            ControlSize::Large => (48., 48.),
            ControlSize::ExtraLarge => (64., 64.),
        },
        Some(Segment::SplitTrailing) => {
            let (icon, footprint) = split_trailing_size(size);
            let edge = (footprint - icon) / 2.;
            (edge, edge)
        }
        _ => return None,
    };
    Some(ButtonPadding {
        start: px(start),
        end: px(end),
        ..Default::default()
    })
}

pub fn label_width(size: ControlSize, label: SharedString, window: &Window, cx: &App) -> Pixels {
    group_label_width(size, label, None, false, window, cx)
}

pub fn group_label_width(
    size: ControlSize,
    label: SharedString,
    padding: Option<ButtonPadding>,
    icon: bool,
    window: &Window,
    cx: &App,
) -> Pixels {
    let metrics = size.metrics();
    let theme = current_material_theme(window, cx);
    let style = metrics.label.style(TypographyEmphasis::Baseline);
    let width = gpui_material_foundation::shape_text(
        if label.contains(['\r', '\n']) {
            label.replace(['\r', '\n'], " ").into()
        } else {
            label
        },
        style,
        &theme.typography,
        1.,
        gpui::transparent_black(),
        window,
        cx,
    )
    .map_or(px(0.), |text| text.line().width.ceil());
    let padding = padding.map_or(px(metrics.padding * 2.), |padding| {
        finite_length(padding.start) + finite_length(padding.end)
    });
    (width
        + padding
        + if icon {
            px(metrics.icon + metrics.gap)
        } else {
            px(0.)
        })
    .max(px(theme.layout.density.interactive_size(1.)))
}

pub fn group_icon_width(
    size: ControlSize,
    width: IconButtonWidth,
    padding: Option<ButtonPadding>,
) -> Pixels {
    let (metrics, width) = size.icon_metrics(width);
    padding.map_or(px(width), |padding| {
        px(metrics.icon) + finite_length(padding.start) + finite_length(padding.end)
    })
}

pub fn split_trailing_size(size: ControlSize) -> (f32, f32) {
    // Trailing icon and sum of the two horizontal spaces from SplitButton*Tokens.
    match size {
        ControlSize::ExtraSmall | ControlSize::Small => (22., 48.),
        ControlSize::Medium => (26., 56.),
        ControlSize::Large => (38., 96.),
        ControlSize::ExtraLarge => (50., 136.),
    }
}

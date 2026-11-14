use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, FocusHandle, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window, div, px,
};
use gpui_material_foundation::window_material_theme;
use gpui_material_theme::{
    ColorRole, CornerRole, Direction, Elevation, MotionKind, MotionSpeed, TypeRole,
    TypographyEmphasis,
};

use crate::action::{
    Action, ActionControl, ActionLayout, ActionStyle, Activate, Change, DisabledStyle,
    SegmentShape, icon_slot,
};

/// Material Expressive sizes, from extra small to extra large.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ControlSize {
    ExtraSmall,
    #[default]
    Small,
    Medium,
    Large,
    ExtraLarge,
}

#[derive(Clone, Copy)]
struct Metrics {
    height: f32,
    icon: f32,
    gap: f32,
    padding: f32,
    outline: f32,
    square: CornerRole,
    pressed: CornerRole,
    label: TypeRole,
}

impl ControlSize {
    // Button{XSmall,Small,Medium,Large,XLarge}Tokens in Compose Material 3 1.5.0-rc01.
    fn metrics(self) -> Metrics {
        use CornerRole::*;
        use TypeRole::*;
        let (height, icon, gap, padding, outline, square, pressed, label) = match self {
            Self::ExtraSmall => (32., 20., 8., 16., 1., Medium, Small, LabelLarge),
            Self::Small => (40., 20., 8., 16., 1., Medium, Small, LabelLarge),
            Self::Medium => (56., 24., 8., 24., 1., Large, Medium, TitleMedium),
            Self::Large => (96., 32., 12., 48., 2., ExtraLarge, Large, HeadlineSmall),
            Self::ExtraLarge => (136., 40., 16., 64., 3., ExtraLarge, Large, HeadlineLarge),
        };
        Metrics {
            height,
            icon,
            gap,
            padding,
            outline,
            square,
            pressed,
            label,
        }
    }

    fn icon_metrics(self, width: IconButtonWidth) -> (Metrics, f32) {
        // Expressive icon buttons have their own icon and horizontal space.
        let mut metrics = self.metrics();
        let (icon, narrow, uniform, wide) = match self {
            Self::ExtraSmall => (20., 4., 6., 10.),
            Self::Small => (24., 4., 8., 14.),
            Self::Medium => (24., 12., 16., 24.),
            Self::Large => (32., 16., 32., 48.),
            Self::ExtraLarge => (40., 32., 48., 72.),
        };
        metrics.icon = icon;
        let space = match width {
            IconButtonWidth::Narrow => narrow,
            IconButtonWidth::Uniform => uniform,
            IconButtonWidth::Wide => wide,
        };
        (metrics, icon + 2. * space)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonShape {
    #[default]
    Round,
    Square,
}

impl ButtonShape {
    fn corner(self, square: CornerRole, invert: bool) -> CornerRole {
        if (self == Self::Round) != invert {
            CornerRole::Full
        } else {
            square
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonVariant {
    Elevated,
    #[default]
    Filled,
    FilledTonal,
    Outlined,
    Text,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToggleButtonVariant {
    Elevated,
    #[default]
    Filled,
    FilledTonal,
    Outlined,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum IconButtonVariant {
    #[default]
    Standard,
    Filled,
    FilledTonal,
    Outlined,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum IconButtonWidth {
    Narrow,
    #[default]
    Uniform,
    Wide,
}

struct Common {
    id: ElementId,
    label: SharedString,
    disabled: bool,
    focus: Option<FocusHandle>,
    size: ControlSize,
    shape: ButtonShape,
}

impl Common {
    fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            focus: None,
            size: ControlSize::default(),
            shape: ButtonShape::default(),
        }
    }

    fn render(
        self,
        content: AnyElement,
        style: ActionStyle,
        action: Action,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.render_scaled(content, style, action, 1., window, cx)
    }

    fn render_scaled(
        self,
        content: AnyElement,
        style: ActionStyle,
        action: Action,
        scale: f32,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        ActionControl {
            id: self.id,
            label: self.label,
            focus: self.focus,
            disabled: self.disabled,
            action,
            style,
            content,
        }
        .render_scaled(scale, window, cx)
    }
}

// All four buttons expose the same shape, size, and focus contract.
macro_rules! common_builders {
    () => {
        /// Sets the Material size tier. Small is the default.
        pub fn size(mut self, size: ControlSize) -> Self {
            self.common.size = size;
            self
        }

        /// Sets the resting shape. Round is the default.
        pub fn shape(mut self, shape: ButtonShape) -> Self {
            self.common.shape = shape;
            self
        }

        /// Controls availability and keyboard traversal.
        pub fn disabled(mut self, disabled: bool) -> Self {
            self.common.disabled = disabled;
            self
        }

        /// Uses the caller's stable focus handle.
        pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
            self.common.focus = Some(focus.clone());
            self
        }
    };
}

#[derive(Default)]
struct Slots {
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
}

impl Slots {
    fn render(
        self,
        label: SharedString,
        metrics: Metrics,
        scale: f32,
        window: &Window,
        tint: gpui::Hsla,
        cx: &App,
    ) -> AnyElement {
        let direction = window_material_theme(window, cx).layout.direction;
        let typography = metrics.label.style(TypographyEmphasis::Baseline);
        let mut row = div()
            .flex()
            .items_center()
            .justify_center()
            .min_w_0()
            .max_w_full()
            .gap(px(metrics.gap * scale))
            .px(px(metrics
                .padding
                .min(f32::from(window.viewport_size().width) * 0.125)
                * scale))
            .py(px(4. * scale))
            .text_size(px(typography.size * scale))
            .line_height(px(typography.line_height * scale));
        if direction == Direction::Rtl {
            row = row.flex_row_reverse();
        }
        row.children(
            self.leading
                .map(|icon| icon_slot(icon, metrics.icon * scale)),
        )
        .child(crate::text::single_line(
            label,
            typography,
            scale,
            tint,
            (None, 1., 1.),
            window,
            cx,
        ))
        .children(
            self.trailing
                .map(|icon| icon_slot(icon, metrics.icon * scale)),
        )
        .into_any_element()
    }
}

macro_rules! slot_builders {
    () => {
        /// Supplies a decorative leading icon in the Material icon slot.
        pub fn leading_icon(mut self, icon: impl IntoElement) -> Self {
            self.slots.leading = Some(icon.into_any_element());
            self
        }

        /// Supplies a decorative trailing icon in the Material icon slot.
        pub fn trailing_icon(mut self, icon: impl IntoElement) -> Self {
            self.slots.trailing = Some(icon.into_any_element());
            self
        }
    };
}

/// A labeled Material action. Activation uses Base's pointer, keyboard and
/// accessibility behavior and invokes the application callback once.
#[derive(IntoElement)]
pub struct Button {
    common: Common,
    variant: ButtonVariant,
    slots: Slots,
    on_activate: Option<Activate>,
    scale: f32,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            common: Common::new(id, label),
            variant: ButtonVariant::default(),
            slots: Slots::default(),
            on_activate: None,
            scale: 1.,
        }
    }

    common_builders!();
    slot_builders!();

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub(crate) fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let metrics = self.common.size.metrics();
        let appearance = button_appearance(self.variant, None);
        let style = appearance.style(
            metrics,
            self.common.shape.corner(metrics.square, false),
            58.,
        );
        let content = self.slots.render(
            self.common.label.clone(),
            metrics,
            self.scale,
            window,
            style.foreground(self.common.disabled, window, cx),
            cx,
        );
        self.common.render_scaled(
            content,
            style,
            Action::Invoke(self.on_activate),
            self.scale,
            window,
            cx,
        )
    }
}

/// A controlled labeled toggle. The caller re-renders with the selected value
/// after accepting a requested change.
#[derive(IntoElement)]
pub struct ToggleButton {
    common: Common,
    variant: ToggleButtonVariant,
    selected: bool,
    slots: Slots,
    on_change: Option<Change>,
}

impl ToggleButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, selected: bool) -> Self {
        Self {
            common: Common::new(id, label),
            variant: ToggleButtonVariant::default(),
            selected,
            slots: Slots::default(),
            on_change: None,
        }
    }

    common_builders!();
    slot_builders!();

    pub fn variant(mut self, variant: ToggleButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ToggleButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let metrics = self.common.size.metrics();
        let variant = match self.variant {
            ToggleButtonVariant::Elevated => ButtonVariant::Elevated,
            ToggleButtonVariant::Filled => ButtonVariant::Filled,
            ToggleButtonVariant::FilledTonal => ButtonVariant::FilledTonal,
            ToggleButtonVariant::Outlined => ButtonVariant::Outlined,
        };
        let mut style = button_appearance(variant, Some(self.selected)).style(
            metrics,
            self.common.shape.corner(metrics.square, false),
            0.,
        );
        style.selected_corner = Some(self.common.shape.corner(metrics.square, true));
        let content = self.slots.render(
            self.common.label.clone(),
            metrics,
            1.,
            window,
            style.foreground(self.common.disabled, window, cx),
            cx,
        );
        self.common.render(
            content,
            style,
            Action::Toggle {
                selected: self.selected,
                on_change: self.on_change,
            },
            window,
            cx,
        )
    }
}

/// An icon-only action with a required accessibility name.
#[derive(IntoElement)]
pub struct IconButton {
    common: Common,
    icon: AnyElement,
    variant: IconButtonVariant,
    width: IconButtonWidth,
    on_activate: Option<Activate>,
}

impl IconButton {
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self {
            common: Common::new(id, name),
            icon: icon.into_any_element(),
            variant: IconButtonVariant::default(),
            width: IconButtonWidth::default(),
            on_activate: None,
        }
    }

    common_builders!();

    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn width(mut self, width: IconButtonWidth) -> Self {
        self.width = width;
        self
    }

    pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (metrics, width) = self.common.size.icon_metrics(self.width);
        let mut style = icon_appearance(self.variant, None).style(
            metrics,
            self.common.shape.corner(metrics.square, false),
            width,
        );
        style.corner_motion = (MotionKind::Effects, MotionSpeed::Default);
        self.common.render(
            icon_slot(self.icon, metrics.icon).into_any_element(),
            style,
            Action::Invoke(self.on_activate),
            window,
            cx,
        )
    }
}

/// An icon-only controlled toggle. The selected icon and native toggle state
/// both reflect the caller's value.
#[derive(IntoElement)]
pub struct IconToggleButton {
    common: Common,
    icon: AnyElement,
    selected_icon: Option<AnyElement>,
    selected: bool,
    variant: IconButtonVariant,
    width: IconButtonWidth,
    on_change: Option<Change>,
}

impl IconToggleButton {
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
        selected: bool,
    ) -> Self {
        Self {
            common: Common::new(id, name),
            icon: icon.into_any_element(),
            selected_icon: None,
            selected,
            variant: IconButtonVariant::default(),
            width: IconButtonWidth::default(),
            on_change: None,
        }
    }

    common_builders!();

    pub fn selected_icon(mut self, icon: impl IntoElement) -> Self {
        self.selected_icon = Some(icon.into_any_element());
        self
    }

    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn width(mut self, width: IconButtonWidth) -> Self {
        self.width = width;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for IconToggleButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (metrics, width) = self.common.size.icon_metrics(self.width);
        let mut style = icon_appearance(self.variant, Some(self.selected)).style(
            metrics,
            self.common.shape.corner(metrics.square, false),
            width,
        );
        style.selected_corner = Some(self.common.shape.corner(metrics.square, true));
        style.corner_motion = (MotionKind::Effects, MotionSpeed::Default);
        let icon = if self.selected {
            self.selected_icon.unwrap_or(self.icon)
        } else {
            self.icon
        };
        self.common.render(
            icon_slot(icon, metrics.icon).into_any_element(),
            style,
            Action::Toggle {
                selected: self.selected,
                on_change: self.on_change,
            },
            window,
            cx,
        )
    }
}

/// Internal composition hooks shared with the optional Material extras crate.
#[doc(hidden)]
pub mod composition {
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
        on_activate: Option<Activate>,
        segment: Option<Segment>,
        icon: Option<AnyElement>,
        width: Pixels,
        tab_stop: bool,
        semantic_only: bool,
    }

    impl ButtonPart {
        pub fn new(
            id: impl Into<ElementId>,
            label: impl Into<SharedString>,
            width: Pixels,
        ) -> Self {
            Self {
                common: Common::new(id, label),
                variant: ButtonVariant::default(),
                selected: None,
                on_activate: None,
                segment: None,
                icon: None,
                width,
                tab_stop: true,
                semantic_only: false,
            }
        }

        common_builders!();

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

        pub fn icon(mut self, icon: impl IntoElement) -> Self {
            self.icon = Some(icon.into_any_element());
            self
        }

        pub fn tab_stop(mut self, tab_stop: bool) -> Self {
            self.tab_stop = tab_stop;
            self
        }

        pub fn on_activate(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
            self.on_activate = Some(Rc::new(callback));
            self
        }

        pub fn on_accessible_activate(
            mut self,
            callback: impl Fn(&mut Window, &mut App) + 'static,
        ) -> Self {
            self.on_activate = Some(Rc::new(callback));
            self.semantic_only = true;
            self
        }
    }

    impl RenderOnce for ButtonPart {
        fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
            let mut metrics = self.common.size.metrics();
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
                    inner,
                    pressed,
                }
            });
            let mut style =
                button_appearance(self.variant, self.selected).style(metrics, CornerRole::Full, 0.);
            if self.selected.is_some() {
                style.selected_corner = Some(metrics.square);
            }
            if matches!(self.segment, Some(Segment::SplitTrailing)) {
                metrics.icon = split_trailing_size(self.common.size).0;
            }
            let content = if let Some(icon) = self.icon {
                icon_slot(icon, metrics.icon).into_any_element()
            } else {
                let tint = style.foreground(self.common.disabled, window, cx);
                let typography = metrics.label.style(TypographyEmphasis::Baseline);
                let padding = metrics
                    .padding
                    .min((f32::from(self.width) - 16.).max(0.) / 2.);
                let label_width = (self.width - px(2. * padding)).max(px(0.));
                div()
                    .flex()
                    .min_w_0()
                    .max_w_full()
                    .w(self.width)
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
            };
            let action = if let Some(selected) = self.selected {
                Action::Toggle {
                    selected,
                    on_change: self.on_activate.map(|callback| {
                        Rc::new(move |_, window: &mut Window, cx: &mut App| callback(window, cx))
                            as Change
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
                    width: Some(self.width),
                    tab_stop: self.tab_stop,
                    semantic_only: self.semantic_only,
                },
                window,
                cx,
            )
        }
    }

    pub fn label_width(
        size: ControlSize,
        label: SharedString,
        window: &Window,
        cx: &App,
    ) -> Pixels {
        let metrics = size.metrics();
        let theme = window_material_theme(window, cx);
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
        (width + px(metrics.padding * 2.)).max(px(theme.layout.density.interactive_size(1.)))
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
}

struct Appearance {
    background: Option<ColorRole>,
    foreground: ColorRole,
    outlined: bool,
    disabled: DisabledStyle,
    elevation: Elevation,
    hover_elevation: Elevation,
}

impl Appearance {
    fn style(self, metrics: Metrics, corner: CornerRole, min_width: f32) -> ActionStyle {
        ActionStyle {
            background: self.background,
            background_color: None,
            foreground: self.foreground,
            disabled: self.disabled,
            height: metrics.height,
            min_width,
            corner,
            pressed_corner: metrics.pressed,
            selected_corner: None,
            corner_motion: (MotionKind::Spatial, MotionSpeed::Fast),
            outline: if self.outlined { metrics.outline } else { 0. },
            elevation: self.elevation,
            hover_elevation: self.hover_elevation,
        }
    }
}

fn button_appearance(variant: ButtonVariant, selected: Option<bool>) -> Appearance {
    use ColorRole::*;
    use Elevation::*;
    let (background, foreground, outlined, elevation, hover_elevation) = match (variant, selected) {
        (ButtonVariant::Elevated, Some(true)) => (Some(Primary), OnPrimary, false, Level1, Level2),
        (ButtonVariant::Elevated, _) => (Some(SurfaceContainerLow), Primary, false, Level1, Level2),
        (ButtonVariant::Filled, Some(false)) => (
            Some(SurfaceContainer),
            OnSurfaceVariant,
            false,
            Level0,
            Level1,
        ),
        (ButtonVariant::Filled, _) => (Some(Primary), OnPrimary, false, Level0, Level1),
        (ButtonVariant::FilledTonal, Some(true)) => {
            (Some(Secondary), OnSecondary, false, Level0, Level1)
        }
        (ButtonVariant::FilledTonal, _) => (
            Some(SecondaryContainer),
            OnSecondaryContainer,
            false,
            Level0,
            Level1,
        ),
        (ButtonVariant::Outlined, Some(true)) => (
            Some(InverseSurface),
            InverseOnSurface,
            false,
            Level0,
            Level0,
        ),
        (ButtonVariant::Outlined, _) => (None, OnSurfaceVariant, true, Level0, Level0),
        // Compose overrides the outdated TextButtonTokens.LabelColor to Primary.
        (ButtonVariant::Text, _) => (None, Primary, false, Level0, Level0),
    };
    Appearance {
        background,
        foreground,
        outlined,
        disabled: DisabledStyle {
            // FilledTonalButton retains the baseline tokens; the expressive
            // toggle uses TonalButtonTokens instead.
            foreground: if variant == ButtonVariant::FilledTonal && selected.is_none() {
                OnSurface
            } else {
                OnSurfaceVariant
            },
            background: match (variant, selected) {
                (ButtonVariant::Outlined, Some(_)) => Some((OutlineVariant, 0.10)),
                (ButtonVariant::Outlined | ButtonVariant::Text, _) => None,
                (ButtonVariant::FilledTonal, None) => Some((OnSurface, 0.12)),
                _ => Some((OnSurface, 0.10)),
            },
            outline: (OutlineVariant, 0.10),
        },
        elevation,
        hover_elevation,
    }
}

fn icon_appearance(variant: IconButtonVariant, selected: Option<bool>) -> Appearance {
    let mut appearance = match variant {
        IconButtonVariant::Standard => button_appearance(ButtonVariant::Text, selected),
        IconButtonVariant::Filled => button_appearance(ButtonVariant::Filled, selected),
        IconButtonVariant::FilledTonal => button_appearance(ButtonVariant::FilledTonal, selected),
        IconButtonVariant::Outlined => button_appearance(ButtonVariant::Outlined, selected),
    };
    if variant == IconButtonVariant::Standard {
        appearance.foreground = if selected == Some(true) {
            ColorRole::Primary
        } else {
            ColorRole::OnSurfaceVariant
        };
    }
    appearance.disabled = DisabledStyle {
        foreground: ColorRole::OnSurface,
        // Compose icon toggle disabled colors do not depend on selection.
        background: match variant {
            IconButtonVariant::Filled | IconButtonVariant::FilledTonal => {
                Some((ColorRole::OnSurface, 0.10))
            }
            IconButtonVariant::Standard | IconButtonVariant::Outlined => None,
        },
        outline: (ColorRole::OutlineVariant, 0.38),
    };
    appearance.hover_elevation = Elevation::Level0;
    appearance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_action_and_standard_icon_keep_distinct_compose_colors() {
        // Button.kt corrects TextButtonTokens.LabelColor, whereas a standard
        // icon remains neutral until selected (IconButtonDefaults.kt).
        assert_eq!(
            button_appearance(ButtonVariant::Text, None).foreground,
            ColorRole::Primary
        );
        for selected in [None, Some(false)] {
            assert_eq!(
                icon_appearance(IconButtonVariant::Standard, selected).foreground,
                ColorRole::OnSurfaceVariant
            );
        }
        assert_eq!(
            icon_appearance(IconButtonVariant::Standard, Some(true)).foreground,
            ColorRole::Primary
        );
    }

    #[test]
    fn disabled_tokens_are_not_inferred_from_enabled_selection_colors() {
        use ColorRole::*;
        for selected in [Some(false), Some(true)] {
            let outlined = button_appearance(ButtonVariant::Outlined, selected);
            assert_eq!(outlined.disabled.background, Some((OutlineVariant, 0.10)));
            assert_eq!(outlined.disabled.foreground, OnSurfaceVariant);
            assert_eq!(outlined.disabled.outline, (OutlineVariant, 0.10));
            let icon = icon_appearance(IconButtonVariant::Outlined, selected);
            assert_eq!(icon.disabled.background, None);
            assert_eq!(icon.disabled.foreground, OnSurface);
            assert_eq!(icon.disabled.outline, (OutlineVariant, 0.38));
        }
        assert_eq!(
            button_appearance(ButtonVariant::Outlined, None)
                .disabled
                .background,
            None
        );
        assert_eq!(
            button_appearance(ButtonVariant::Filled, None)
                .disabled
                .background,
            Some((OnSurface, 0.10))
        );
        // FilledTonalButton and its expressive toggle reference different token
        // generations in the pinned Compose release.
        assert_eq!(
            button_appearance(ButtonVariant::FilledTonal, None)
                .disabled
                .background,
            Some((OnSurface, 0.12))
        );
        assert_eq!(
            button_appearance(ButtonVariant::FilledTonal, Some(false))
                .disabled
                .background,
            Some((OnSurface, 0.10))
        );
        assert_eq!(
            icon_appearance(IconButtonVariant::FilledTonal, None)
                .disabled
                .background,
            Some((OnSurface, 0.10))
        );
    }
}

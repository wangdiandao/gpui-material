mod appearance;
mod spinner;

use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, SharedString, Styled, Window, div, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{InteractionSource, current_material_theme};
use gpui_material_theme::{
    CornerRole, Direction, MotionKind, MotionSpeed, TypeRole, TypographyEmphasis,
};

use crate::action::{
    Action, ActionControl, ActionLayout, ActionStyle, Activate, Change, SegmentShape, icon_slot,
};
use crate::button_style::{
    ButtonBorder, ButtonColors, ButtonElevations, ButtonPadding, ButtonShapes, ButtonStyle,
    ControlSize, finite_length,
};

use appearance::{button_appearance, icon_appearance};
use spinner::render_loading_spinner;

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

/// Semantic palette shortcut for Material buttons and icon buttons.
/// Backwards-compatible alias for [`crate::SemanticTone`].
/// Per-instance [`ButtonColors`] and [`ButtonBorder`] overrides still take precedence.
pub type ButtonTone = crate::SemanticTone;

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
    loading: bool,
    focus: Option<FocusHandle>,
    interaction_source: Option<Entity<InteractionSource>>,
    appearance: ButtonStyle,
    size: ControlSize,
    shape: ButtonShape,
    tone: ButtonTone,
}

impl Common {
    fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            loading: false,
            focus: None,
            interaction_source: None,
            appearance: ButtonStyle::default(),
            size: ControlSize::default(),
            shape: ButtonShape::default(),
            tone: ButtonTone::default(),
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
        .render_layout(
            scale,
            ActionLayout {
                tab_stop: true,
                loading: self.loading,
                appearance: self.appearance,
                ..Default::default()
            },
            self.interaction_source.as_ref(),
            window,
            cx,
        )
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

        /// Applies a Material 3 semantic tone shortcut. Explicit `.colors` and `.border`
        /// overrides still take precedence.
        pub fn tone(mut self, tone: ButtonTone) -> Self {
            self.common.tone = tone;
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
    content: Option<AnyElement>,
    padding: Option<ButtonPadding>,
    width: Option<Pixels>,
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
        let direction = current_material_theme(window, cx).layout.direction;
        let typography = metrics.label.style(TypographyEmphasis::Baseline);
        let max_padding = self.width.map_or_else(
            || f32::from(window.viewport_size().width) * 0.125,
            |width| (f32::from(width) - 16.).max(0.) / 2.,
        );
        let horizontal = px(metrics.padding.min(max_padding));
        let ButtonPadding {
            top,
            end,
            bottom,
            start,
        } = self.padding.unwrap_or(ButtonPadding {
            top: px(4.),
            end: horizontal,
            bottom: px(4.),
            start: horizontal,
        });
        let (left, right) = if direction == Direction::Rtl {
            (end, start)
        } else {
            (start, end)
        };
        let icons = usize::from(self.leading.is_some()) + usize::from(self.trailing.is_some());
        let label_width = self.width.map(|width| {
            (width
                - finite_length(left)
                - finite_length(right)
                - px((metrics.icon + metrics.gap) * icons as f32))
            .max(px(0.))
        });
        let row_typography = current_material_theme(window, cx)
            .typography
            .scale_style(typography);
        let mut row = div()
            .flex()
            .items_center()
            .justify_center()
            .min_w_0()
            .max_w_full()
            .gap(px(metrics.gap * scale))
            .pl(finite_length(left) * scale)
            .pr(finite_length(right) * scale)
            .pt(finite_length(top) * scale)
            .pb(finite_length(bottom) * scale)
            .text_size(px(row_typography.size * scale))
            .line_height(px(row_typography.line_height * scale));
        if direction == Direction::Rtl {
            row = row.flex_row_reverse();
        }
        if let Some(width) = self.width {
            row = row.w(width).overflow_hidden();
        }
        row.children(
            self.leading
                .map(|icon| icon_slot(icon, metrics.icon * scale)),
        )
        .child(self.content.unwrap_or_else(|| {
            crate::text::single_line(
                label,
                typography,
                scale,
                tint,
                (label_width, 1., 1.),
                window,
                cx,
            )
            .into_any_element()
        }))
        .children(
            self.trailing
                .map(|icon| icon_slot(icon, metrics.icon * scale)),
        )
        .into_any_element()
    }
}

macro_rules! appearance_builders {
    () => {
        /// Observes this button's interaction presentation through a caller-owned source.
        /// The source does not handle input or change selection.
        pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
            self.common.interaction_source = Some(source.clone());
            self
        }

        /// Overrides state-specific corners. Unspecified states use the size and variant defaults.
        pub fn shapes(mut self, shapes: ButtonShapes) -> Self {
            self.common.appearance.shapes = shapes;
            self
        }

        /// Overrides container and content colors for individual states.
        pub fn colors(mut self, colors: ButtonColors) -> Self {
            self.common.appearance.colors = colors;
            self
        }

        /// Supplies a border in place of the variant's outline.
        pub fn border(mut self, border: ButtonBorder) -> Self {
            self.common.appearance.border = Some(border);
            self
        }
    };
}

macro_rules! slot_builders {
    () => {
        /// Replaces the visible label with an element. The constructor's label
        /// remains the accessible name. Keep interactive children outside the button.
        pub fn content(mut self, content: impl IntoElement) -> Self {
            self.slots.content = Some(content.into_any_element());
            self
        }

        /// Sets logical insets around the label, content and icon slots.
        pub fn content_padding(mut self, padding: ButtonPadding) -> Self {
            self.slots.padding = Some(padding);
            self
        }

        /// Overrides elevation levels for individual interaction states.
        pub fn elevations(mut self, elevations: ButtonElevations) -> Self {
            self.common.appearance.elevations = elevations;
            self
        }

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

/// Internal composition hooks shared with the optional Material extras crate.
#[doc(hidden)]
pub mod composition;

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

    /// Creates a Filled button.
    pub fn filled(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).variant(ButtonVariant::Filled)
    }

    /// Creates a Filled Tonal button.
    pub fn tonal(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).variant(ButtonVariant::FilledTonal)
    }

    /// Creates an Elevated button.
    pub fn elevated(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).variant(ButtonVariant::Elevated)
    }

    /// Creates an Outlined button.
    pub fn outlined(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).variant(ButtonVariant::Outlined)
    }

    /// Creates a Text button.
    pub fn text(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label).variant(ButtonVariant::Text)
    }

    common_builders!();
    appearance_builders!();
    slot_builders!();

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shows a centered inline circular spinner while suppressing activation and preserving the
    /// button's resting width. Keeps the active palette unless `.disabled(true)` is also set.
    pub fn loading(mut self, loading: bool) -> Self {
        self.common.loading = loading;
        self
    }

    #[cfg(feature = "feedback")]
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
        let appearance = button_appearance(self.variant, None, self.common.tone);
        let style = appearance.style(
            metrics,
            self.common.shape.corner(metrics.square, false),
            58.,
        );
        let tint = self
            .common
            .appearance
            .colors
            .content(self.common.disabled, false)
            .unwrap_or_else(|| style.foreground(self.common.disabled, window, cx));
        let resting_content = self.slots.render(
            self.common.label.clone(),
            metrics,
            self.scale,
            window,
            tint,
            cx,
        );
        let content = if self.common.loading {
            div()
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .child(div().opacity(0.).child(resting_content))
                .child(
                    div()
                        .id((self.common.id.clone(), "loading-spinner"))
                        .test_support()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(render_loading_spinner(
                            &self.common.id,
                            metrics.icon * self.scale,
                            tint,
                            window,
                            cx,
                        )),
                )
                .into_any_element()
        } else {
            resting_content
        };
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
    appearance_builders!();
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
        let mut style = button_appearance(variant, Some(self.selected), self.common.tone).style(
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
            self.common
                .appearance
                .colors
                .content(self.common.disabled, self.selected)
                .unwrap_or_else(|| style.foreground(self.common.disabled, window, cx)),
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

    /// Creates a Standard icon button.
    pub fn standard(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self::new(id, name, icon).variant(IconButtonVariant::Standard)
    }

    /// Creates a Filled icon button.
    pub fn filled(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self::new(id, name, icon).variant(IconButtonVariant::Filled)
    }

    /// Creates a Filled Tonal icon button.
    pub fn tonal(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self::new(id, name, icon).variant(IconButtonVariant::FilledTonal)
    }

    /// Creates an Outlined icon button.
    pub fn outlined(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self::new(id, name, icon).variant(IconButtonVariant::Outlined)
    }

    common_builders!();
    appearance_builders!();

    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn width(mut self, width: IconButtonWidth) -> Self {
        self.width = width;
        self
    }

    /// Shows an inline circular spinner in place of the icon while suppressing activation.
    /// Keeps the active palette unless `.disabled(true)` is also set.
    pub fn loading(mut self, loading: bool) -> Self {
        self.common.loading = loading;
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
        let mut style = icon_appearance(self.variant, None, self.common.tone).style(
            metrics,
            self.common.shape.corner(metrics.square, false),
            width,
        );
        style.corner_motion = (MotionKind::Effects, MotionSpeed::Default);
        let content = if self.common.loading {
            let tint = self
                .common
                .appearance
                .colors
                .content(self.common.disabled, false)
                .unwrap_or_else(|| style.foreground(self.common.disabled, window, cx));
            div()
                .id((self.common.id.clone(), "loading-spinner"))
                .test_support()
                .flex()
                .items_center()
                .justify_center()
                .child(render_loading_spinner(
                    &self.common.id,
                    metrics.icon,
                    tint,
                    window,
                    cx,
                ))
                .into_any_element()
        } else {
            icon_slot(self.icon, metrics.icon).into_any_element()
        };
        self.common
            .render(content, style, Action::Invoke(self.on_activate), window, cx)
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
    appearance_builders!();

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
        let mut style = icon_appearance(self.variant, Some(self.selected), self.common.tone).style(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_and_icon_button_variant_shorthands_set_expected_variants() {
        assert_eq!(
            Button::filled("b1", "Filled").variant,
            ButtonVariant::Filled
        );
        assert_eq!(
            Button::tonal("b2", "Tonal").variant,
            ButtonVariant::FilledTonal
        );
        assert_eq!(
            Button::elevated("b3", "Elevated").variant,
            ButtonVariant::Elevated
        );
        assert_eq!(
            Button::outlined("b4", "Outlined").variant,
            ButtonVariant::Outlined
        );
        assert_eq!(Button::text("b5", "Text").variant, ButtonVariant::Text);

        assert_eq!(
            IconButton::standard("i1", "Std", div()).variant,
            IconButtonVariant::Standard
        );
        assert_eq!(
            IconButton::filled("i2", "Fill", div()).variant,
            IconButtonVariant::Filled
        );
        assert_eq!(
            IconButton::tonal("i3", "Tonal", div()).variant,
            IconButtonVariant::FilledTonal
        );
        assert_eq!(
            IconButton::outlined("i4", "Out", div()).variant,
            IconButtonVariant::Outlined
        );

        let tone: ButtonTone = crate::SemanticTone::Danger;
        assert_eq!(tone, ButtonTone::Danger);
    }
}

use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, FocusHandle, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, px,
};
use gpui_material_foundation::{animate, material_theme};
use gpui_material_theme::{
    ColorRole, CornerRole, Direction, Elevation, MotionKind, MotionSpeed, TypeRole,
    TypographyEmphasis,
};

use crate::action::{Action, ActionControl, ActionStyle, Activate, icon_slot};

/// Expressive FAB sizes. Small is the 56 dp FAB and Small Extended FAB.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FabSize {
    #[default]
    Small,
    Medium,
    Large,
}

impl FabSize {
    fn metrics(self) -> (f32, f32, CornerRole, f32, f32, TypeRole) {
        // FabBaseline, FabMedium/Large, and ExtendedFabSmall/Medium/Large.
        match self {
            Self::Small => (56., 24., CornerRole::Large, 16., 8., TypeRole::TitleMedium),
            Self::Medium => (
                80.,
                28.,
                CornerRole::LargeIncreased,
                26.,
                16.,
                TypeRole::TitleLarge,
            ),
            Self::Large => (
                96.,
                32.,
                CornerRole::ExtraLarge,
                28.,
                20.,
                TypeRole::HeadlineSmall,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FabColor {
    #[default]
    Primary,
    Secondary,
    Tertiary,
}

struct FabContent {
    id: ElementId,
    name: SharedString,
    icon: AnyElement,
    size: FabSize,
    color: FabColor,
    disabled: bool,
    focus: Option<FocusHandle>,
    on_activate: Option<Activate>,
}

impl FabContent {
    fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: icon.into_any_element(),
            size: FabSize::default(),
            color: FabColor::default(),
            disabled: false,
            focus: None,
            on_activate: None,
        }
    }

    fn style(&self) -> ActionStyle {
        let (height, _, corner, _, _, _) = self.size.metrics();
        let (background, foreground) = match self.color {
            FabColor::Primary => (ColorRole::PrimaryContainer, ColorRole::OnPrimaryContainer),
            FabColor::Secondary => (
                ColorRole::SecondaryContainer,
                ColorRole::OnSecondaryContainer,
            ),
            FabColor::Tertiary => (ColorRole::TertiaryContainer, ColorRole::OnTertiaryContainer),
        };
        ActionStyle {
            background: Some(background),
            foreground,
            height,
            min_width: height,
            corner,
            pressed_corner: corner,
            outline: 0.,
            elevation: Elevation::Level3,
            hover_elevation: Elevation::Level4,
        }
    }
}

macro_rules! fab_builders {
    () => {
        pub fn size(mut self, size: FabSize) -> Self {
            self.content.size = size;
            self
        }

        pub fn color(mut self, color: FabColor) -> Self {
            self.content.color = color;
            self
        }

        pub fn disabled(mut self, disabled: bool) -> Self {
            self.content.disabled = disabled;
            self
        }

        pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
            self.content.focus = Some(focus.clone());
            self
        }

        pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
            self.content.on_activate = Some(Rc::new(handler));
            self
        }
    };
}

/// An elevated icon action with a required accessibility name. Mount it beside
/// the scroll viewport to keep its position fixed while content scrolls.
#[derive(IntoElement)]
pub struct Fab {
    content: FabContent,
}

impl Fab {
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self {
            content: FabContent::new(id, name, icon),
        }
    }

    fab_builders!();
}

impl RenderOnce for Fab {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = self.content.style();
        let (_, icon_size, _, _, _, _) = self.content.size.metrics();
        ActionControl {
            id: self.content.id,
            label: self.content.name,
            focus: self.content.focus,
            disabled: self.content.disabled,
            action: Action::Invoke(self.content.on_activate),
            style,
            content: icon_slot(self.content.icon, icon_size).into_any_element(),
        }
        .render(window, cx)
    }
}

/// An elevated icon and label action. Expansion is controlled by the caller.
/// Both presentations share the same focus handle, action and accessible name.
#[derive(IntoElement)]
pub struct ExtendedFab {
    content: FabContent,
    expanded: bool,
}

impl ExtendedFab {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self {
            content: FabContent::new(id, label, icon),
            expanded: true,
        }
    }

    fab_builders!();

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

impl RenderOnce for ExtendedFab {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = self.content.style();
        let (_, icon_size, _, padding, gap, role) = self.content.size.metrics();
        let theme = material_theme(cx);
        let direction = theme.layout.direction;
        let motion = theme.motion.spec(
            MotionKind::Spatial,
            MotionSpeed::Default,
            cx.reduce_motion(),
        );
        let progress = animate(
            ElementId::from((self.content.id.clone(), "expansion")),
            f32::from(self.expanded),
            motion,
            window,
            cx,
        )
        .clamp(0., 1.);
        let typography = role.style(TypographyEmphasis::Baseline);
        let shaped = gpui_material_foundation::shape_text(
            self.content.name.clone(),
            typography,
            &material_theme(cx).typography,
            1.,
            gpui::black(),
            window,
            cx,
        );
        let max_label_width = (window.viewport_size().width - px(style.height + 32.)).max(px(0.));
        let label_width = shaped
            .map_or(px(0.), |shaped| shaped.line().width)
            .min(max_label_width);
        let mut row = div()
            .flex()
            .items_center()
            .min_w_0()
            .px(px(padding * progress))
            .gap(px(gap * progress))
            .text_size(px(typography.size))
            .line_height(px(typography.line_height));
        if direction == Direction::Rtl {
            row = row.flex_row_reverse();
        }
        row = row.child(icon_slot(self.content.icon, icon_size));
        if progress > 0. {
            row = row.child(
                div()
                    .min_w_0()
                    .w(label_width * progress)
                    .overflow_hidden()
                    .opacity(progress)
                    .child(
                        div()
                            .w(label_width)
                            .truncate()
                            .child(self.content.name.clone()),
                    ),
            );
        }
        ActionControl {
            id: self.content.id,
            label: self.content.name,
            focus: self.content.focus,
            disabled: self.content.disabled,
            action: Action::Invoke(self.content.on_activate),
            style,
            content: row.into_any_element(),
        }
        .render(window, cx)
    }
}

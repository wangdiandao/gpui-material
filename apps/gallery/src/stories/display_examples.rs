use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, ScrollHandle, SharedString,
    Window,
};
use gpui_material::{CheckboxState, LoadingIndicatorShapes, SliderState};

use crate::story::{Story, section, stack};

#[cfg(all(test, feature = "test-support"))]
mod avatar_tests;
#[cfg(all(test, feature = "test-support"))]
mod badge_tests;
#[cfg(all(test, feature = "test-support"))]
mod banner_tests;
#[cfg(all(test, feature = "test-support"))]
mod card_tests;
mod content;
#[cfg(all(test, feature = "test-support"))]
mod display_scrollbar_tests;
#[cfg(all(test, feature = "test-support"))]
mod divider_tests;
#[cfg(all(test, feature = "test-support"))]
mod drop_target_tests;
#[cfg(all(test, feature = "test-support"))]
mod empty_state_tests;
mod feedback;
#[cfg(all(test, feature = "test-support"))]
mod icon_tests;
mod list_examples;
mod lists_and_tiles;
#[cfg(all(test, feature = "test-support"))]
mod loading_tests;
#[cfg(all(test, feature = "test-support"))]
mod progress_tests;
mod pull_to_refresh_demo;
#[cfg(all(test, feature = "test-support"))]
mod pull_to_refresh_tests;
#[cfg(all(test, feature = "test-support"))]
mod settings_tiles_tests;
#[cfg(all(test, feature = "test-support"))]
mod skeleton_tests;
mod text_examples;

pub(crate) struct DisplayExamples {
    story: Option<Story>,
    progress: f32,
    loading: bool,
    loading_progress: f32,
    loading_shapes: Option<LoadingIndicatorShapes>,
    pull_to_refresh: Option<Entity<pull_to_refresh_demo::RefreshExample>>,
    selected: bool,
    opened: bool,
    child_count: usize,
    indeterminate: bool,
    selected_file: Option<usize>,
    pinned_file: Option<usize>,
    files_scroll: ScrollHandle,
    list_examples: Option<Entity<list_examples::ListExamples>>,
    wide_scroll: ScrollHandle,
    drop_status: SharedString,
    empty_state_status: SharedString,
    banner_visible: bool,
    banner_status: SharedString,
    skeleton_animated: bool,
    auto_sync: bool,
    telemetry_opt_in: CheckboxState,
    release_channel: SharedString,
    notification_volume: Option<Entity<SliderState>>,
    choice_density: SharedString,
}

impl Default for DisplayExamples {
    fn default() -> Self {
        Self {
            story: None,
            progress: 0.35,
            loading: false,
            loading_progress: 0.,
            loading_shapes: None,
            pull_to_refresh: None,
            selected: false,
            opened: false,
            child_count: 0,
            indeterminate: false,
            selected_file: None,
            pinned_file: None,
            files_scroll: ScrollHandle::new(),
            list_examples: None,
            wide_scroll: ScrollHandle::new(),
            drop_status: "Drop files or click the drop zone".into(),
            empty_state_status: "No project action triggered yet".into(),
            banner_visible: true,
            banner_status: "Ready".into(),
            skeleton_animated: true,
            auto_sync: true,
            telemetry_opt_in: CheckboxState::Indeterminate,
            release_channel: "stable".into(),
            notification_volume: None,
            choice_density: "comfortable".into(),
        }
    }
}

impl DisplayExamples {
    pub(crate) fn for_story(story: Story) -> Self {
        Self {
            story: Some(story),
            loading: story == Story::LoadingIndicator,
            ..Self::default()
        }
    }

    fn shows(&self, story: Story) -> bool {
        self.story.is_none_or(|active| active == story)
    }
}

impl Render for DisplayExamples {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        stack()
            .children(self.shows(Story::Text).then(text_examples::examples))
            .children(self.shows(Story::Icon).then(|| self.icon_symbols_section()))
            .children(
                self.shows(Story::Icon)
                    .then(|| self.icon_inherited_section()),
            )
            .children(
                self.shows(Story::Icon)
                    .then(|| self.icon_directional_section()),
            )
            .children(
                self.shows(Story::Card)
                    .then(|| self.card_section(window, cx)),
            )
            .children(self.shows(Story::Avatar).then(|| self.avatar_section()))
            .children(self.shows(Story::Divider).then(|| self.divider_section()))
            .children(
                self.shows(Story::DropTarget)
                    .then(|| self.drop_target_section(cx)),
            )
            .children(
                self.shows(Story::EmptyState)
                    .then(|| self.empty_state_section(cx)),
            )
            .children(self.shows(Story::Banner).then(|| self.banner_section(cx)))
            .children(
                self.shows(Story::Skeleton)
                    .then(|| self.skeleton_section(cx)),
            )
            .children(self.shows(Story::Badge).then(|| self.badge_section()))
            .children(
                self.shows(Story::List)
                    .then(|| self.settings_tiles_section(cx)),
            )
            .children((self.story == Some(Story::List)).then(|| {
                self.list_examples
                    .get_or_insert_with(|| cx.new(list_examples::ListExamples::new))
                    .clone()
            }))
            .children(
                self.shows(Story::List)
                    .then(|| self.segmented_list_section(cx)),
            )
            .children(
                self.shows(Story::Progress)
                    .then(|| self.progress_section(cx)),
            )
            .children(self.story.eq(&Some(Story::LoadingIndicator)).then(|| {
                let refresh = self
                    .pull_to_refresh
                    .get_or_insert_with(|| cx.new(pull_to_refresh_demo::RefreshExample::new));
                section("Pull to refresh")
                    .description("Pull down at the top of the list or use Refresh files.")
                    .child(refresh.clone())
            }))
            .children(
                self.shows(Story::LoadingIndicator)
                    .then(|| self.indeterminate_loading_section(cx)),
            )
            .children(
                self.shows(Story::LoadingIndicator)
                    .then(|| self.determinate_loading_section(cx)),
            )
            .children(
                self.shows(Story::List)
                    .then(|| self.standard_list_section(cx)),
            )
            .children(
                (self.story == Some(Story::List))
                    .then(|| self.horizontal_scroll_section(window, cx)),
            )
    }
}

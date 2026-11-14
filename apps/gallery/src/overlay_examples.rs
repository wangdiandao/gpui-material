use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use gpui_base::input::{Input, InputState};
use gpui_material::{Button, Dialog, DialogState, DialogVariant, Sheet, SheetEdge, SheetMode};

use crate::story::{Story, stack};

pub(crate) struct ModalExamples {
    story: Story,
    primary: Entity<DialogState>,
    secondary: Entity<DialogState>,
    standard: Entity<DialogState>,
    name: Entity<InputState>,
    expanded: bool,
}

impl ModalExamples {
    pub(crate) fn new(story: Story, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let primary = cx.new(DialogState::new);
        let secondary = cx.new(DialogState::new);
        let standard = cx.new(DialogState::new);
        cx.observe(&primary, |_, _, cx| cx.notify()).detach();
        cx.observe(&secondary, |_, _, cx| cx.notify()).detach();
        Self {
            story,
            primary,
            secondary,
            standard,
            name: cx.new(|cx| InputState::new(window, cx).placeholder("名称")),
            expanded: false,
        }
    }

    fn trigger(id: &'static str, label: &'static str, state: &Entity<DialogState>) -> Button {
        let state = state.clone();
        Button::new(id, label).on_activate(move |window, cx| {
            state.update(cx, |state, cx| state.open(window, cx));
        })
    }

    fn dialogs(&self) -> impl IntoElement {
        let editor = self.name.clone();
        stack()
            .items_start()
            .child(Self::trigger("rename", "Rename project…", &self.primary))
            .child(
                Dialog::new("project-dialog", &self.primary, "Rename project")
                    .description("Use a name people can recognize in the project list.")
                    .body(
                        stack()
                            .child(
                                div()
                                    .rounded_sm()
                                    .border_1()
                                    .p_2()
                                    .child(Input::new(&editor)),
                            )
                            .child(Self::trigger(
                                "guidelines",
                                "Naming guidelines…",
                                &self.secondary,
                            )),
                    )
                    .confirm("Rename", move |_, cx| {
                        !editor.read(cx).value().trim().is_empty()
                    }),
            )
            .child(
                Dialog::new("guidelines-dialog", &self.secondary, "Naming guidelines")
                    .layer(1)
                    .variant(DialogVariant::FullScreen)
                    .description("Use the project name as it appears in your team’s roadmap.")
                    .cancel_label("Close"),
            )
    }

    fn sheets(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity().downgrade();
        stack()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(Self::trigger(
                        "activity",
                        "Project activity…",
                        &self.primary,
                    ))
                    .child(Self::trigger(
                        "details",
                        "Project details…",
                        &self.secondary,
                    )),
            )
            .child(
                Sheet::new(
                    "standard",
                    &self.standard,
                    "Project notes",
                    "The project name is used in the activity log.",
                )
                .extent(px(160.))
                .expandable(self.expanded, move |expanded, _, cx| {
                    let _ = owner.update(cx, |this, cx| {
                        this.expanded = expanded;
                        cx.notify();
                    });
                }),
            )
            .child(
                Sheet::new(
                    "activity-sheet",
                    &self.primary,
                    "Project activity",
                    stack()
                        .children((1..=20).map(|entry| div().child(format!("Revision {entry}")))),
                )
                .mode(SheetMode::Modal)
                .extent(px(300.)),
            )
            .child(
                Sheet::new(
                    "details-sheet",
                    &self.secondary,
                    "Project details",
                    "Visible to project members",
                )
                .edge(SheetEdge::End)
                .mode(SheetMode::Modal)
                .extent(px(360.)),
            )
    }
}

impl Render for ModalExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.story == Story::Dialog {
            self.dialogs().into_any_element()
        } else {
            self.sheets(cx).into_any_element()
        }
    }
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn escape_closes_nested_dialogs_and_restores_each_trigger(cx: &mut gpui::TestAppContext) {
    use gpui::WindowOptions;
    use gpui_material::MaterialRoot;

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| ModalExamples::new(Story::Dialog, window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let (first, second) = cx.update(|cx| {
        let view = view.read(cx);
        (view.primary.clone(), view.secondary.clone())
    });
    let trigger = cx
        .update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            window.focus_next(cx);
            let trigger = window.focused(cx).unwrap();
            first.update(cx, |state, cx| state.open(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
            second.update(cx, |state, cx| state.open(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
            trigger
        })
        .unwrap();
    cx.simulate_keystrokes(window.into(), "escape");
    cx.update(|cx| {
        assert!(first.read(cx).is_open());
        assert!(!second.read(cx).is_open());
    });
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "escape");
    cx.update_window(window.into(), |_, window, cx| {
        assert!(!first.read(cx).is_open());
        assert!(trigger.is_focused(window));
    })
    .unwrap();
}

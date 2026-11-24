use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use gpui_material::{
    BottomSheetState, BottomSheetValue, Button, LogicalEdges, Scaffold, Sheet, SheetMode, Text,
};

use crate::story::{section, stack};

pub(super) struct BottomSheetExamples {
    standard: Entity<BottomSheetState>,
    modal: Entity<BottomSheetState>,
}

impl BottomSheetExamples {
    pub(super) fn new(cx: &mut Context<Self>) -> Self {
        let standard = cx.new(|cx| {
            BottomSheetState::new(BottomSheetValue::PartiallyExpanded, cx).enabled_values([
                BottomSheetValue::PartiallyExpanded,
                BottomSheetValue::Expanded,
            ])
        });
        let modal = cx.new(|cx| BottomSheetState::new(BottomSheetValue::Hidden, cx));
        cx.observe(&standard, |_, _, cx| cx.notify()).detach();
        cx.observe(&modal, |_, _, cx| cx.notify()).detach();
        Self { standard, modal }
    }
}

impl Render for BottomSheetExamples {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let modal = self.modal.clone();
        stack()
            .child(
                section("Bottom sheet stops")
                    .child(Button::new("show-retained-bottom-sheet", "Show modal bottom sheet")
                        .on_activate(move |window, cx| {
                            modal.update(cx, |state, cx| state.show(window, cx));
                        }))
                    .child(div().h(px(330.)).child(
                        Scaffold::new("bottom-sheet-scaffold", Text::new("Project workbench").id("workbench-name"))
                            .bottom_sheet(
                                Sheet::bottom(
                                    "package-bottom-sheet", &self.standard, "Build packages",
                                    "Build output stays available while the workbench remains interactive.",
                                )
                                .extent(px(230.))
                                .peek_height(px(64.))
                                .handle_label("Resize package sheet")
                                .handle_content(Text::new("Build packages").id("packages-grip")),
                            )
                            .content_insets(LogicalEdges { bottom: 72., ..LogicalEdges::default() }),
                    )),
            )
            .child(
                Sheet::bottom(
                    "retained-modal-sheet", &self.modal, "Build history",
                    stack().children((1..=12).map(|build| div().child(format!("Build {build}")))),
                )
                .mode(SheetMode::Modal)
                .extent(px(380.))
                .peek_height(px(180.))
                .max_width(px(520.))
                .close_label("Close build history")
                .handle_label("Resize build history"),
            )
    }
}

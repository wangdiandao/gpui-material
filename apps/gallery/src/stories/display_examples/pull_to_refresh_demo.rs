use std::time::Duration;

use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use gpui_material::{Divider, PullToRefresh, PullToRefreshState, Text, theme::TypeRole};

pub(super) struct RefreshExample {
    state: Entity<PullToRefreshState>,
    refreshing: bool,
    revision: usize,
}

impl RefreshExample {
    pub(super) fn new(cx: &mut Context<Self>) -> Self {
        let state = cx.new(|_| PullToRefreshState::new());
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        Self {
            state,
            refreshing: false,
            revision: 0,
        }
    }
}

impl Render for RefreshExample {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity().downgrade();
        PullToRefresh::new(
            "files-refresh",
            &self.state,
            self.refreshing,
            move |window, cx| {
                let _ = owner.update(cx, |demo, cx| {
                    demo.refreshing = true;
                    cx.notify();
                    let timer = cx.background_executor().timer(Duration::from_millis(800));
                    cx.spawn_in(window, async move |demo, cx| {
                        timer.await;
                        let _ = demo.update_in(cx, |demo, _, cx| {
                            demo.revision += 1;
                            demo.refreshing = false;
                            cx.notify();
                        });
                    })
                    .detach();
                });
            },
        )
        .refresh_label("Refresh files")
        .h(px(218.))
        .children((0_usize..8).map(|index| {
            div()
                .flex_none()
                .px(px(12.))
                .py(px(8.))
                .child(
                    Text::new(format!(
                        "Project file {} · revision {}",
                        index + 1,
                        self.revision
                    ))
                    .id(("refresh-file", index))
                    .role(TypeRole::BodyMedium),
                )
                .child(Divider::horizontal())
        }))
    }
}

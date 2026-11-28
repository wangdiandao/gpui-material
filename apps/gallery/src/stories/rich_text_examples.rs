use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled, Window,
    div, prelude::FluentBuilder, px,
};
use gpui_material::{
    Button, ButtonVariant, MaterialTheme, RichText, RichTextSelectionFormat, RichTextState,
    SelectableText, SelectableTextGroup, Surface, Text, ThemeScope,
    theme::{Appearance, ColorRole, ColorScheme, CornerRole, TypeRole},
};

use crate::story::{Story, section, stack};

#[cfg(all(test, feature = "test-support"))]
mod tests;

const SAMPLE_MARKDOWN: &str = r#"# Material 3 Expressive

Rich text projects the active **MaterialTheme** onto headings, body copy, [links](https://m3.material.io), and code blocks.

## Checklist

- [x] Typography scale (`HeadlineLarge` – `TitleSmall`, `BodyLarge`)
- [ ] Click a task checkbox to toggle source state

> Blockquotes use `OutlineVariant` borders and `OnSurfaceVariant` copy for clean visual hierarchy.

```rust
let theme = MaterialTheme::from_seed(seed, Appearance::Dark, 0.0);
```

| Token | Role |
| --- | --- |
| Code surface | `SurfaceContainerHighest` |
| Link accent | `Primary` |
| Selection | `PrimaryContainer` |
"#;

pub(crate) struct RichTextExamples {
    story: Story,
    selection_group: SelectableTextGroup,
    markdown_source: String,
    rich_state: Entity<RichTextState>,
    last_clicked_link: SharedString,
    stream_step: usize,
}

impl RichTextExamples {
    pub(crate) fn new(story: Story, cx: &mut Context<Self>) -> Self {
        let selection_group = SelectableTextGroup::new("Selectable document", cx);
        let markdown_source = SAMPLE_MARKDOWN.to_owned();
        let rich_state = cx.new(|cx| RichTextState::markdown(&markdown_source, cx));
        cx.observe(&rich_state, |_, _, cx| cx.notify()).detach();
        Self {
            story,
            selection_group,
            markdown_source,
            rich_state,
            last_clicked_link: SharedString::default(),
            stream_step: 0,
        }
    }

    fn render_selectable_text(&self) -> impl IntoElement {
        stack()
            .max_w(px(560.))
            .child(
                section("Typography roles & selection")
                    .description(
                        "Drag across text to highlight with PrimaryContainer and press Ctrl/Cmd+C to copy.",
                    )
                    .child(
                        SelectableText::headline_small("sel-heading", "Selectable Material headline")
                            .emphasized(true),
                    )
                    .child(SelectableText::body_large(
                        "sel-body",
                        "SelectableText honors Material 3 type roles, color roles, wrapping, and ThemeScope overrides while integrating with MaterialRoot's window selection layer.",
                    ))
                    .child(
                        SelectableText::body_medium(
                            "sel-muted",
                            "Muted supporting copy uses OnSurfaceVariant.",
                        )
                        .muted(),
                    ),
            )
            .child(
                section("Multi-paragraph shared selection group")
                    .description("Drag from the first paragraph into the second to select across blocks.")
                    .child(
                        SelectableText::body_large(
                            "sel-group-1",
                            "First paragraph in a shared SelectableTextGroup.",
                        )
                        .group(&self.selection_group)
                        .document_order(0),
                    )
                    .child(
                        SelectableText::body_large(
                            "sel-group-2",
                            "Second paragraph continues the same selection order cleanly.",
                        )
                        .group(&self.selection_group)
                        .document_order(1),
                    ),
            )
    }

    fn render_rich_text(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let owner_link = cx.entity().downgrade();
        let owner_task = cx.entity().downgrade();
        let owner_stream = cx.entity().downgrade();
        let dark_scope = MaterialTheme::default().colors(ColorScheme::baseline(Appearance::Dark));

        stack()
            .max_w(px(640.))
            .child(
                section("Interactive Markdown")
                    .description(
                        "Supports headings, links, task toggles, code surfaces, blockquotes, tables, and streaming updates.",
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("rich-append", "Append note")
                                    .variant(ButtonVariant::Outlined)
                                    .on_activate(move |_, cx| {
                                        let _ = owner_stream.update(cx, |view, cx| {
                                            view.stream_step += 1;
                                            let note = format!(
                                                "\n\n*Streamed update #{}: incremental Markdown append.*",
                                                view.stream_step
                                            );
                                            view.markdown_source.push_str(&note);
                                            view.rich_state.update(cx, |state, cx| {
                                                state.push_str(&note, cx);
                                            });
                                            cx.notify();
                                        });
                                    }),
                            )
                            .when(!self.last_clicked_link.is_empty(), |this| {
                                this.child(
                                    Text::body_small(format!(
                                        "Last link clicked: {}",
                                        self.last_clicked_link
                                    ))
                                    .color(ColorRole::Primary),
                                )
                            }),
                    )
                    .child(
                        RichText::from_state("rich-markdown", &self.rich_state)
                            .selection_format(RichTextSelectionFormat::Plain)
                            .on_link_click(move |url, _, _, cx| {
                                let url = url.clone();
                                let _ = owner_link.update(cx, |view, cx| {
                                    view.last_clicked_link = url;
                                    cx.notify();
                                });
                            })
                            .on_task_toggle(move |toggle, _, cx| {
                                let _ = owner_task.update(cx, |view, cx| {
                                    let range = toggle.marker_range();
                                    if view.markdown_source.get(range.clone()).is_some() {
                                        let replacement = if toggle.checked() { "[x]" } else { "[ ]" };
                                        view.markdown_source.replace_range(range, replacement);
                                        let updated = view.markdown_source.clone();
                                        view.rich_state.update(cx, |state, cx| {
                                            state.set_text(&updated, cx);
                                        });
                                    }
                                    cx.notify();
                                });
                            }),
                    ),
            )
            .child(
                section("ThemeScope dark preview")
                    .description("RichText projects its style directly from the enclosing ThemeScope.")
                    .child(
                        ThemeScope::new(
                            Surface::new(ColorRole::SurfaceContainer)
                                .shape(CornerRole::Large.shape())
                                .p_4()
                                .child(
                                    RichText::markdown(
                                        "rich-dark-scope",
                                        "### Scoped dark surface\n\nCode spans like `SurfaceContainerHighest` and [Primary links](https://m3.material.io) adapt automatically inside `ThemeScope`.",
                                    )
                                    .body_role(TypeRole::BodyMedium),
                                ),
                        )
                        .theme(dark_scope),
                    ),
            )
    }
}

impl Render for RichTextExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.story {
            Story::SelectableText => self.render_selectable_text().into_any_element(),
            Story::RichText => self.render_rich_text(cx).into_any_element(),
            _ => div().into_any_element(),
        }
    }
}

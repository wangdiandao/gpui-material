use gpui::{
    AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, Styled, Subscription, Window, div,
};
use gpui_material::{
    Button, ButtonVariant, CommandGroup, CommandItem, CommandPalette, CommandPaletteEvent,
    CommandPaletteState, Icon, IconName, Text,
    theme::{ColorRole, TypeRole},
};

use crate::story::{section, stack};

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub(crate) struct CommandPaletteExamples {
    palette: Entity<CommandPaletteState>,
    last_command: SharedString,
    _subscription: Subscription,
}

impl CommandPaletteExamples {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let palette = cx.new(|cx| CommandPaletteState::new(window, cx));
        let subscription = cx.subscribe(&palette, |this, _, event, cx| {
            if let CommandPaletteEvent::Execute(item) = event {
                this.last_command = format!("Executed: {} ({})", item.label(), item.id()).into();
                cx.notify();
            }
        });
        Self {
            palette,
            last_command: "Press Ctrl+K / Cmd+K or click the button to open the command palette."
                .into(),
            _subscription: subscription,
        }
    }

    fn command_groups() -> Vec<CommandGroup> {
        vec![
            CommandGroup::new(
                "Navigation",
                [
                    CommandItem::new("go-inbox", "Go to Inbox")
                        .supporting("Jump to unread messages")
                        .icon(IconName::Inbox)
                        .shortcut("G I")
                        .keywords(["mail", "messages"]),
                    CommandItem::new("go-docs", "Open Documentation")
                        .supporting("Browse component specs and examples")
                        .icon(IconName::Description)
                        .shortcut("Ctrl+Shift+D")
                        .keywords(["guide", "help", "reference"]),
                ],
            ),
            CommandGroup::new(
                "Appearance",
                [
                    CommandItem::new("theme-dark", "Switch to Dark Theme")
                        .supporting("Use Material 3 dark surface tokens")
                        .icon(IconName::DarkMode)
                        .shortcut("Ctrl+Shift+L")
                        .keywords(["night", "appearance", "color"]),
                    CommandItem::new("theme-light", "Switch to Light Theme")
                        .supporting("Use Material 3 light surface tokens")
                        .icon(IconName::LightMode)
                        .keywords(["day", "bright", "color"]),
                ],
            ),
            CommandGroup::new(
                "Workspace",
                [
                    CommandItem::new("new-note", "Create Release Note")
                        .supporting("Draft a new changelog entry")
                        .icon(IconName::Add)
                        .shortcut("Ctrl+N"),
                    CommandItem::new("archive-all", "Archive Workspace")
                        .supporting("Requires owner permissions")
                        .icon(IconName::Archive)
                        .disabled(true),
                ],
            ),
        ]
    }
}

impl Render for CommandPaletteExamples {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let open_palette = self.palette.clone();
        let shortcut_palette = self.palette.clone();

        div()
            .w_full()
            .on_key_down(move |event, window, cx| {
                if event.keystroke.key == "k"
                    && (event.keystroke.modifiers.control || event.keystroke.modifiers.platform)
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.shift
                {
                    shortcut_palette.update(cx, |state, cx| state.toggle(window, cx));
                    cx.stop_propagation();
                }
            })
            .child(
                stack()
                    .child(
                        section("Modal command palette")
                            .description(
                                "Filter grouped commands by label, description, or keywords; navigate with Up/Down/Home/End and execute with Enter.",
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        Button::new("story-open-palette", "Open command palette")
                                            .variant(ButtonVariant::FilledTonal)
                                            .leading_icon(Icon::new(IconName::Search))
                                            .on_activate(move |window, cx| {
                                                open_palette
                                                    .update(cx, |state, cx| state.open(window, cx));
                                            }),
                                    )
                                    .child(
                                        Text::new(self.last_command.clone())
                                            .id("story-palette-status")
                                            .role(TypeRole::BodyMedium)
                                            .color(ColorRole::OnSurfaceVariant)
                                            .wrap(),
                                    ),
                            ),
                    )
                    .child(
                        CommandPalette::grouped(
                            "story-command-palette",
                            &self.palette,
                            Self::command_groups(),
                        )
                        .placeholder("Type a command or search...")
                        .footer(
                            Text::new("↑↓ Navigate · Home/End Jump · Enter Execute · Esc Close")
                                .role(TypeRole::LabelSmall)
                                .color(ColorRole::OnSurfaceVariant),
                        ),
                    ),
            )
    }
}

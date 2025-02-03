use std::{
    cell::Cell,
    fmt::{Display, Formatter},
};

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::WidgetRef};

use crate::{
    actions::{Action, Actions, FileBrowserAction, KeyBinding},
    components::file_browser::AddMode,
    config::Theme,
};

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Action::FileBrowser(action) = self else {
            return Ok(());
        };
        f.write_str(match action {
            FileBrowserAction::AddToQueue => "add to Queue",
            FileBrowserAction::AddToLibrary => "add to Library",
            FileBrowserAction::AddToPlaylist => "add to Playlist",
            FileBrowserAction::ToggleMode => "toggle add mode",
            FileBrowserAction::OpenTerminal => "open terminal",
            FileBrowserAction::NavigateUp => "navigate up",
        })
    }
}

impl Display for KeyBinding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if !self.modifiers().is_empty() {
            write!(f, "{}+", self.modifiers())?;
        }
        let code = format!("{}", self.code()).replace(" ", "");
        write!(f, "{code}")
    }
}

pub struct FileBrowserHelp<'a> {
    theme: Theme,
    pills: Vec<(KeyBinding, Cell<Action>)>,
    actions: &'a Actions,
}

impl<'a> FileBrowserHelp<'a> {
    pub fn new(actions: &'a Actions, theme: Theme) -> Self {
        let pills = vec![
            (
                actions.list_primary(),
                Cell::new(Action::FileBrowser(FileBrowserAction::AddToQueue)),
            ),
            (
                actions.list_secondary(),
                Cell::new(Action::FileBrowser(FileBrowserAction::AddToLibrary)),
            ),
            (
                actions
                    .key_by_action(Action::FileBrowser(FileBrowserAction::ToggleMode))
                    .unwrap(),
                Cell::new(Action::FileBrowser(FileBrowserAction::ToggleMode)),
            ),
            (
                actions
                    .key_by_action(Action::FileBrowser(FileBrowserAction::OpenTerminal))
                    .unwrap(),
                Cell::new(Action::FileBrowser(FileBrowserAction::OpenTerminal)),
            ),
        ];

        Self { theme, pills, actions }
    }

    pub fn set_add_mode(&self, add_mode: AddMode) {
        let kb_secondary = self.actions.list_secondary();

        let key_binding = self.pills.iter().find(|(k, _)| *k == kb_secondary);

        let action = if add_mode == AddMode::AddToLibrary {
            Action::FileBrowser(FileBrowserAction::AddToLibrary)
        } else {
            Action::FileBrowser(FileBrowserAction::AddToPlaylist)
        };

        if let Some(key_binding) = key_binding {
            key_binding.1.set(action);
        }
    }
}

impl WidgetRef for FileBrowserHelp<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let help_pill_style = Style::new().fg(self.theme.foreground).bg(self.theme.top_bar_background);
        let help_pill_border_style = Style::new().fg(self.theme.top_bar_background);

        let mut pills = vec![];

        for (key, action) in self.pills.iter() {
            pills.push(ratatui::text::Span::raw("▐").style(help_pill_border_style));
            pills.push(ratatui::text::Span::raw(format!("{key}: {}", action.get())).style(help_pill_style));
            pills.push(ratatui::text::Span::raw("▌ ").style(help_pill_border_style));
        }

        let line = ratatui::text::Line::from(pills);
        line.render_ref(area, buf);
    }
}

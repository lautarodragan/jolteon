use std::fmt::{Display, Formatter};

use ratatui::{buffer::Buffer, layout::Rect, prelude::Widget, style::Style, text::Span, widgets::WidgetRef};
use ratatui::style::Color;
use crate::{
    actions::{Action, Actions, FileBrowserAction, KeyBinding},
    components::file_browser::AddMode,
    theme::Theme,
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
            FileBrowserAction::ToggleShowHidden => "toggle show hidden files",
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
    pills: Vec<KeyBindingPill<'a>>,
    actions: &'a Actions,
}

impl<'a> FileBrowserHelp<'a> {
    pub fn new(actions: &'a Actions, theme: Theme) -> Self {
        let pills = vec![
            KeyBindingPill::new(
                theme,
                actions.list_primary(),
                Action::FileBrowser(FileBrowserAction::AddToQueue),
            ),
            KeyBindingPill::new(
                theme,
                actions.list_secondary(),
                Action::FileBrowser(FileBrowserAction::AddToLibrary),
            ),
            KeyBindingPill::new(
                theme,
                actions
                    .key_by_action(Action::FileBrowser(FileBrowserAction::ToggleMode))
                    .unwrap(),
                Action::FileBrowser(FileBrowserAction::ToggleMode),
            ),
            KeyBindingPill::new(
                theme,
                actions
                    .key_by_action(Action::FileBrowser(FileBrowserAction::OpenTerminal))
                    .unwrap(),
                Action::FileBrowser(FileBrowserAction::OpenTerminal),
            ),
            KeyBindingPill::new(
                theme,
                actions
                    .key_by_action(Action::FileBrowser(FileBrowserAction::ToggleShowHidden))
                    .unwrap(),
                Action::FileBrowser(FileBrowserAction::ToggleShowHidden),
            ),
        ];

        Self { pills, actions }
    }

    pub fn set_add_mode(&mut self, add_mode: AddMode) {
        let kb_secondary = self.actions.list_secondary();
        let Some(pill) = self.pills.iter_mut().find(|p| p.key_binding == kb_secondary) else {
            log::error!("Missing pill for {kb_secondary:?}?");
            return;
        };

        let action = if add_mode == AddMode::AddToLibrary {
            Action::FileBrowser(FileBrowserAction::AddToLibrary)
        } else {
            Action::FileBrowser(FileBrowserAction::AddToPlaylist)
        };

        pill.set_action(action);
    }
}

impl WidgetRef for FileBrowserHelp<'_> {
    fn render_ref(&self, mut area: Rect, buf: &mut Buffer) {
        for kbp in &self.pills {
            kbp.render_ref(area, buf);
            area.x += kbp.width() + 1;
        }
    }
}

struct KeyBindingPill<'a> {
    key_binding: KeyBinding,
    action: Action,
    theme: Theme,
    span_key_binding: Span<'a>,
    span_action: Span<'a>,
    span_key_binding_width: u16,
    span_action_width: u16,
}

impl KeyBindingPill<'_> {
    pub fn new(theme: Theme, key_binding: KeyBinding, action: Action) -> Self {
        let help_pill_style = Style::new().fg(theme.foreground).bg(theme.top_bar_background);

        let span_key_binding = Span::raw(key_binding.to_string()).style(help_pill_style);
        let span_key_binding_width = u16::try_from(span_key_binding.width()).unwrap_or(u16::MAX);

        let span_action = Span::raw(action.to_string()).style(help_pill_style);
        let span_action_width = u16::try_from(span_action.width()).unwrap_or(u16::MAX);

        Self {
            key_binding,
            action,
            theme,
            span_key_binding,
            span_action,
            span_key_binding_width,
            span_action_width,
        }
    }

    pub fn width(&self) -> u16 {
        self.span_key_binding_width + self.span_action_width + 4
    }

    // pub fn set_key_binding(&mut self, key_binding: KeyBinding) {
    //     self.key_binding = key_binding;
    // }

    pub fn set_action(&mut self, action: Action) {
        self.action = action;
        let help_pill_style = Style::new().fg(self.theme.foreground).bg(self.theme.top_bar_background);
        self.span_action = Span::raw(action.to_string()).style(help_pill_style);
        self.span_action_width = u16::try_from(self.span_action.width()).unwrap_or(u16::MAX);
    }
}

impl WidgetRef for KeyBindingPill<'_> {
    fn render_ref(&self, mut area: Rect, buf: &mut Buffer) {
        if area.x + self.width() > buf.area().right() {
            return;
        }

        if self.theme.top_bar_background != Color::Reset {
            buf[area].set_symbol("▐").set_fg(self.theme.top_bar_background);
        }

        area.x += 1;

        (&self.span_key_binding).render(area, buf);
        area.x += self.span_key_binding_width;

        buf[area]
            .set_symbol(":")
            .set_fg(self.theme.foreground)
            .set_bg(self.theme.top_bar_background);
        area.x += 1;

        buf[area]
            .set_symbol(" ")
            .set_fg(self.theme.foreground)
            .set_bg(self.theme.top_bar_background);
        area.x += 1;

        (&self.span_action).render(area, buf);
        area.x += self.span_action_width;

        if self.theme.top_bar_background != Color::Reset {
            buf[area].set_symbol("▌").set_fg(self.theme.top_bar_background);
        }
    }
}

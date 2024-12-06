use std::fmt::{Display, Formatter};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Text,
    widgets::WidgetRef,
};

use super::{AddMode, FileBrowser, FileBrowserSelection};
use crate::components::file_browser::help::FileBrowserHelp;

impl Display for FileBrowserSelection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let path = self.to_path();
        let file_name = path.file_name().map(|p| p.to_string_lossy());
        f.write_str(file_name.unwrap_or(path.to_string_lossy()).as_ref())?;
        Ok(())
    }
}

impl From<&FileBrowserSelection> for Text<'_> {
    fn from(value: &FileBrowserSelection) -> Self {
        Text::raw(value.to_string())
    }
}

impl Display for AddMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AddMode::AddToLibrary => "add to Library",
            AddMode::AddToPlaylist => "add to Playlist",
        })
    }
}

impl<'a> WidgetRef for FileBrowser<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [area_top, area_main, _, area_help] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(10),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .horizontal_margin(2)
        .areas(area);

        let [area_main_left, _, area_main_right] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Length(5),
            Constraint::Percentage(50),
        ])
        .areas(area_main);

        let [area_right_top, _, area_right_bottom] = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Length(1),
            Constraint::Percentage(50),
        ])
        .areas(area_main_right);

        self.current_directory.render_ref(area_top, buf);
        self.parents_list.render_ref(area_main_left, buf);
        self.children_list.render_ref(area_right_top, buf);
        self.file_meta.render_ref(area_right_bottom, buf);
        self.help.render_ref(area_help, buf);
    }
}

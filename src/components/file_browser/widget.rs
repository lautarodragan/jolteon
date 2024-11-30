use std::fmt::{Display, Formatter};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Text,
    widgets::WidgetRef,
};

use super::{FileBrowser, FileBrowserSelection};

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

impl<'a> WidgetRef for FileBrowser<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let [area_top, area_main] = Layout::vertical([Constraint::Length(2), Constraint::Min(1)])
            .horizontal_margin(2)
            .areas(area);

        let [area_main_left, area_main_separator, _area_main_right] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Length(5),
            Constraint::Percentage(50),
        ])
        .areas(area_main);

        self.current_directory.render_ref(area_top, buf);
        self.parents_list.render_ref(area_main_left, buf);
        self.children_list.render_ref(_area_main_right, buf);

        let [_separator_left, _, _separator_right] =
            Layout::horizontal([Constraint::Min(1), Constraint::Length(1), Constraint::Min(1)])
                .areas(area_main_separator);
    }
}

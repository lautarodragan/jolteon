use std::fmt::{Display, Formatter};
use std::path::Path;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    prelude::{Line, Modifier, Span, Style},
    text::Text,
    widgets::{block::Position, Block, Borders, List, ListItem, ListState, StatefulWidget, WidgetRef},
};

use crate::config::Theme;

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
        let (area_top, area_main_left, area_main_separator, _) = create_areas(area);

        *self.height.lock().unwrap() = area_main_left.height as usize;

        let tb = top_bar(&self.theme, self.current_directory(), &self.filter);
        tb.render_ref(area_top, buf);

        let fl = file_list(&self.theme, &self.items, self.filter());
        StatefulWidget::render(
            fl,
            area_main_left,
            buf,
            &mut ListState::default()
                .with_offset(self.offset)
                .with_selected(Some(self.selected_index)),
        );

        let [_separator_left, _, _separator_right] =
            Layout::horizontal([Constraint::Min(1), Constraint::Length(1), Constraint::Min(1)])
                .areas(area_main_separator);
    }
}

fn create_areas(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let [area_top, area_main] = Layout::vertical([Constraint::Length(2), Constraint::Min(1)])
        .horizontal_margin(2)
        .areas(area);

    let [area_main_left, area_main_separator, area_main_right] = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Length(5),
        Constraint::Percentage(50),
    ])
    .areas(area_main);

    (area_top, area_main_left, area_main_separator, area_main_right)
}

fn top_bar(theme: &Theme, current_directory: &Path, filter: &Option<String>) -> Block<'static> {
    let folder_name = current_directory
        .file_name()
        .and_then(|s| s.to_str())
        .map(String::from)
        .unwrap_or("".to_string());

    let browser_title = match filter {
        Some(filter) => Line::from(vec![
            Span::styled("Search: ", Style::default()),
            Span::styled(filter.clone(), Style::default().fg(theme.search)),
        ]),
        _ => Line::from(folder_name),
    };

    let top_bar = Block::default()
        .borders(Borders::NONE)
        .title(browser_title)
        .title_alignment(Alignment::Left)
        .title_position(Position::Top)
        .title_style(Style::new().bg(theme.background).fg(theme.foreground));

    top_bar
}

fn file_list(theme: &Theme, items: &[FileBrowserSelection], filter: &Option<String>) -> List<'static> {
    let browser_items: Vec<ListItem> = items
        .iter()
        // .map(|i| i.to_path().to_string_lossy().to_string())
        .map(|i| {
            let fg = match filter.as_ref() {
                Some(s) if i.to_path().to_string_lossy().to_lowercase().contains(&s.to_lowercase()) => theme.search,
                _ => theme.foreground_secondary,
            };
            ListItem::new(Text::from(i)).style(Style::default().fg(fg))
        })
        .collect();

    let browser_list = List::new(browser_items)
        .style(Style::default().fg(theme.foreground))
        .highlight_style(
            Style::default()
                .bg(theme.background_selected)
                .fg(theme.foreground_selected)
                .add_modifier(Modifier::BOLD),
        )
        .scroll_padding(0)
        .highlight_symbol("");

    browser_list
}

use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::{Line, Modifier, Span, Style, Widget},
    widgets::WidgetRef,
};

use crate::theme::Theme;

fn split_path(path: &Path) -> (String, String) {
    let folder_name = path
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| path.to_string_lossy())
        .to_string();
    let parent_path = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .map(|mut p| {
            p.push(std::path::MAIN_SEPARATOR);
            p
        })
        .unwrap_or_default();

    (parent_path, folder_name)
}

pub struct CurrentDirectory {
    theme: Theme,
    path: RefCell<PathBuf>,
    path_split: RefCell<(String, String)>,
}

impl CurrentDirectory {
    pub fn new(theme: Theme, path: PathBuf) -> Self {
        Self {
            theme,
            path_split: RefCell::new(split_path(path.as_path())),
            path: RefCell::new(path),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.path.borrow().clone()
    }

    pub fn set_path(&self, path: PathBuf) {
        let mut path_split = self.path_split.borrow_mut();
        *path_split = split_path(path.as_path());
        *self.path.borrow_mut() = path;
    }
}

impl WidgetRef for CurrentDirectory {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let path_split = self.path_split.borrow();

        let (parent_path, folder_name) = &*path_split;

        let parent_path = Span::raw(parent_path).style(
            Style::new()
                .bg(self.theme.background)
                .fg(self.theme.foreground)
                .add_modifier(Modifier::DIM),
        );

        let folder_name =
            Span::raw(folder_name).style(Style::new().bg(self.theme.background).fg(self.theme.foreground));

        let browser_title = Line::from(vec![parent_path, folder_name]);

        browser_title.render(area, buf);
    }
}

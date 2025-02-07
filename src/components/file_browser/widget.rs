use std::fmt::{Display, Formatter};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Text,
    widgets::WidgetRef,
};

use super::{AddMode, FileBrowser, FileBrowserSelection};

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

impl WidgetRef for FileBrowser<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if let Ok(mut files_from_io_thread) = self.files_from_io_thread.try_lock() {
            // If the IO thread has something for us, and the lock is free, let's grab that stuff.
            //
            // We use `try_lock` instead of `lock` to avoid blocking the UI while the other thread holds the lock.
            // A responsive UI with stale data is better UX than a choppy UI.
            //
            // Conceptually, this code doesn't belong in a "render" method.
            // We should probably have a "component.update()" method or something.
            // In practice, it'd be the same — we'll do this in the same thread that
            // processes input and does the rendering, between those two.
            if !files_from_io_thread.is_empty() {
                let files = std::mem::take(&mut *files_from_io_thread);
                if let Some(f) = files.first() {
                    self.file_meta.set_file(f.clone());
                } else {
                    self.file_meta.clear();
                }

                self.children_list.set_items(files);
            }
        };

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

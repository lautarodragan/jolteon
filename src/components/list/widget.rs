use std::{
    borrow::Cow,
    sync::atomic::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Widget, WidgetRef},
};

use super::component::List;

pub struct ListLine<'a> {
    theme: &'a crate::config::Theme,
    text: Cow<'a, str>,
    list_has_focus: bool,
    is_selected: bool,
    is_match: bool,
    is_renaming: bool,
    overrides: Option<Style>,
}

impl<'a> Widget for ListLine<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut style = if self.is_renaming {
            Style::default().fg(self.theme.background).bg(self.theme.search)
        } else if self.is_selected {
            if self.list_has_focus {
                Style::default()
                    .fg(self.theme.foreground_selected)
                    .bg(self.theme.background_selected)
            } else {
                Style::default()
                    .fg(self.theme.foreground_selected)
                    .bg(self.theme.background_selected_blur)
            }
        } else {
            let fg = if self.is_match {
                self.theme.search
            } else {
                self.theme.foreground_secondary
            };
            Style::default().fg(fg).bg(self.theme.background)
        };

        if let Some(overrides) = self.overrides {
            style = style.patch(overrides);
        }

        let line: Cow<'a, str> = if self.is_renaming {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            let caret = if now % 500 < 250 { '⎸' } else { ' ' };
            format!("{}{}", self.text, caret).into()
        } else {
            self.text
        };

        let line = ratatui::text::Line::from(line).style(style);
        line.render_ref(area, buf);
    }
}

impl<T> WidgetRef for List<'_, T>
where
    T: std::fmt::Display,
{
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.height.store(area.height as usize, Ordering::Relaxed);

        let items = &self.items.lock().unwrap();

        if items.len() < 1 {
            return;
        }

        let selected_item_index = self.selected_item_index.load(Ordering::Relaxed);
        let offset = self.offset.load(Ordering::Relaxed);

        let rename = self.rename.lock().unwrap();

        for i in 0..items.len().min(area.height as usize) {
            let item_index = i + offset;

            if item_index >= items.len() {
                log::error!("item index {item_index} > items.len() {} offset={offset}", items.len());
                break;
            }

            let item = &items[item_index];
            let area = Rect {
                y: area.y + i as u16,
                height: 1,
                ..area
            };

            let is_selected = item_index == selected_item_index;
            let is_renaming = is_selected && rename.is_some();

            let text = match *rename {
                Some(ref rename) if is_selected => rename.as_str().into(),
                _ => item.inner.to_string().into(),
            };

            let style_overrides = {
                let line_style = self.line_style.lock().unwrap();
                line_style.as_ref().and_then(|ls| ls(&item.inner))
            };

            let line = ListLine {
                theme: &self.theme,
                text,
                list_has_focus: self.is_focused.load(Ordering::Acquire),
                is_selected,
                is_match: item.is_match,
                is_renaming,
                overrides: style_overrides,
            };

            line.render(area, buf);
        }
    }
}

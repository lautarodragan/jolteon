use std::{
    borrow::Cow,
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
    T: std::fmt::Display + Clone,
{
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.height.set(area.height as usize);

        let items = self.items.borrow();
        let visible_items = self.visible_items.borrow();

        if visible_items.is_empty() {
            return;
        }

        let selected_item_index = self.selected_item_index.get();
        let offset = self.offset.get();

        let rename = self.rename.borrow();

        for i in 0..visible_items.len().min(area.height as usize) {
            let item_index = i + offset;

            if item_index >= visible_items.len() {
                log::error!(
                    "item index {item_index} > items.len() {} offset={offset}",
                    visible_items.len()
                );
                break;
            }

            let true_index = visible_items[item_index];
            let item = &items[true_index];
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

            let style_overrides = self.line_style.as_ref().and_then(|ls| ls(&item.inner));

            let line = ListLine {
                theme: &self.theme,
                text,
                list_has_focus: self.is_focused.get(),
                is_selected,
                is_match: item.is_match,
                is_renaming,
                overrides: style_overrides,
            };

            line.render(area, buf);
        }
    }
}

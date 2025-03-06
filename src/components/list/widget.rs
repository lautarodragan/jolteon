use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
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
    renaming_caret_position: usize,
    overrides: Option<Style>,
}

impl Widget for ListLine<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut style = if self.is_renaming {
            Style::default()
                .fg(self.theme.background)
                .bg(self.theme.search)
                .add_modifier(Modifier::SLOW_BLINK)
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

        let line = if self.is_renaming {
            let spans = if self.renaming_caret_position >= self.text.len() {
                let l = Span::from(self.text);
                let r = Span::from(" ").style(style.bg(self.theme.foreground_selected).fg(self.theme.search));
                vec![l, r]
            } else {
                let (l, r) = self.text.split_at(self.renaming_caret_position);
                let (c, r) = r.split_at(1);

                let l = Span::from(l);
                let c = Span::from(c).style(style.bg(self.theme.foreground_selected).fg(self.theme.search));
                let r = Span::from(r);
                vec![l, c, r]
            };
            Line::from(spans)
        } else {
            Line::from(self.text)
        };
        line.style(style).render_ref(area, buf);
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
                renaming_caret_position: *self.renaming_caret_position.borrow(),
                overrides: style_overrides,
            };

            line.render(area, buf);
        }
    }
}

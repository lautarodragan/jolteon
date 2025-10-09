use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::{Line, Span},
    style::{Modifier, Style},
    widgets::Widget,
};

use crate::theme::Theme;

pub struct ListLine<'a> {
    pub theme: &'a Theme,
    pub text: Cow<'a, str>,
    pub list_has_focus: bool,
    pub is_selected: bool,
    pub is_match: bool,
    pub is_renaming: bool,
    pub renaming_caret_position: usize,
    pub overrides: Option<Style>,
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
        line.style(style).render(area, buf);
    }
}

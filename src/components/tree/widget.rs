use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style, Styled},
    widgets::{Widget, WidgetRef},
};

use super::{Tree, TreeNode, TreeNodePath};
use crate::{config::Theme, ui::Focusable};

pub struct ListLine<'a> {
    theme: &'a Theme,
    text: String,
    list_has_focus: bool,
    is_selected: bool,
    is_match: bool,
    is_renaming: bool,
    overrides: Option<Style>,
}

impl Widget for ListLine<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut style = if self.is_renaming {
            Style::default().fg(self.theme.background).bg(self.theme.search)
        } else if self.is_selected {
            if self.list_has_focus {
                if self.is_match {
                    Style::default()
                        .fg(self.theme.search_selected)
                        .bg(self.theme.background_selected)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(self.theme.foreground_selected)
                        .bg(self.theme.background_selected)
                }
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
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            let caret = if now % 500 < 250 { '⎸' } else { ' ' };
            format!("{}{}", self.text, caret)
        } else {
            self.text
        };

        let line = ratatui::text::Line::from(line).style(style);
        line.render_ref(area, buf);
    }
}

impl<T> WidgetRef for Tree<'_, T>
where
    T: std::fmt::Display + Clone,
{
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.height.set(area.height as usize);

        let items = self.items.borrow();
        let selected_item_path = self.selected_item_path.borrow();
        let offset = self.offset.get();
        let line_style = &self.line_style;

        let rename = self.rename.borrow();

        let mut y = 0;
        let mut skip = offset as u16;

        for (i, node) in items.iter().enumerate() {
            render_node(
                area,
                buf,
                &self.theme,
                &mut y,
                &mut skip,
                self.is_focused(),
                node,
                &rename,
                line_style,
                &selected_item_path,
                TreeNodePath::from_vec(vec![i]),
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_node<'a, T>(
    area: Rect,
    buf: &mut Buffer,
    theme: &Theme,
    y: &mut u16,
    skip: &mut u16,
    is_focused: bool,
    node: &TreeNode<T>,
    rename: &Option<String>,
    line_style: &Option<Box<dyn Fn(&T) -> Option<Style> + 'a>>,
    selected_item_path: &TreeNodePath,
    path: TreeNodePath,
) where
    T: std::fmt::Display + Clone,
{
    if *y >= area.height {
        return;
    }

    if *skip > 0 {
        *skip -= 1;
    } else {
        let parent_area = Rect {
            y: area.y + *y,
            height: 1,
            ..area
        };

        // log::debug!("rendering {path} selected i {selected_item_path}");
        let is_selected = selected_item_path == &path;
        let is_renaming = is_selected && rename.is_some();

        let indent = if !path.is_empty() {
            let i = (path.len() - 1) * 2;
            " ".repeat(i)
        } else {
            "".to_string()
        };

        let text = match *rename {
            Some(ref rename) if is_selected => rename.clone(),
            _ => node.inner.to_string(),
        };

        let text = indent + text.as_str();

        let style_overrides = line_style.as_ref().and_then(|ls| ls(&node.inner));

        let line = ListLine {
            theme,
            text,
            list_has_focus: is_focused,
            is_selected,
            is_match: node.is_match,
            is_renaming,
            overrides: style_overrides,
        };

        line.render(parent_area, buf);
        *y += 1;
    }

    if !node.is_open {
        return;
    }

    for (i, node) in node.children.iter().enumerate() {
        render_node(
            area,
            buf,
            theme,
            y,
            skip,
            is_focused,
            node,
            rename,
            line_style,
            selected_item_path,
            path.with_child(i),
        );
    }
}

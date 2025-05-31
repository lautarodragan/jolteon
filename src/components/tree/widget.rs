use ratatui::{
    buffer::Buffer,
    layout::{Offset, Rect},
    style::Style,
    widgets::{Widget, WidgetRef},
};

use super::{Tree, TreeNode, TreeNodePath};
use crate::{components::ListLine, config::Theme, ui::Focusable};

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

        let indent = if !path.is_empty() { (path.len() - 1) * 2 } else { 0 };

        let text = match *rename {
            Some(ref rename) if is_selected => rename.into(),
            _ => node.inner.to_string().into(), // TODO: cache this
        };

        let style_overrides = line_style.as_ref().and_then(|ls| ls(&node.inner));

        let line = ListLine {
            theme,
            text,
            list_has_focus: is_focused,
            is_selected,
            is_match: node.is_match,
            is_renaming,
            overrides: style_overrides,
            renaming_caret_position: 0,
        };

        line.render(parent_area.offset(Offset { x: indent as i32, y: 0 }), buf);
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

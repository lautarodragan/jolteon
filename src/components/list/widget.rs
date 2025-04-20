use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Widget, WidgetRef},
};

use super::component::List;
use crate::components::ListLine;

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
        let render_fn = self.render_fn.borrow();

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
                _ => {
                    if let Some(render_fn) = &*render_fn {
                        render_fn(&item.inner).into()
                    } else {
                        item.inner.to_string().into()
                    }
                }
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

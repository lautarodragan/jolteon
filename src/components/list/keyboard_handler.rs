use std::sync::atomic::Ordering;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{ui::KeyboardHandlerRef};

use super::component::List;

impl<'a, T: 'a + Clone> KeyboardHandlerRef<'a> for List<'a, T> {

    fn on_key(&self, key: KeyEvent) {
        let target = "::List.on_key";
        log::trace!(target: target, "{:?}", key);

        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Home | KeyCode::End => {
                self.on_directional_key(key);
            },
            KeyCode::Enter | KeyCode::Char(_) => {
                let items = self.items.lock().unwrap();

                let i = self.selected_item_index.load(Ordering::SeqCst);
                if i >= items.len() {
                    log::error!(target: target, "selected_item_index > items.len");
                    return;
                }
                let item = items[i].clone();
                drop(items);
                self.on_select_fn.lock().unwrap()(item, key);
            },
            KeyCode::Delete => {
                let i = self.selected_item_index.load(Ordering::Acquire);
                let mut items = self.items.lock().unwrap();
                let removed_item = items.remove(i);

                if i >= items.len() {
                    self.selected_item_index.store(items.len().saturating_sub(1), Ordering::Release);
                }

                drop(items);

                self.on_delete_fn.lock().unwrap()(removed_item, i);
            },
            _ => {},
        }
    }
}


impl<'a, T> List<'a, T> {

    fn on_directional_key(&self, key: KeyEvent) {
        let mut items = self.items.lock().unwrap();
        let length = items.len() as i32;

        let height = self.height.load(Ordering::Relaxed) as i32;
        let padding = 5;

        let mut offset = self.offset.load(Ordering::SeqCst) as i32;
        let mut i = self.selected_item_index.load(Ordering::SeqCst) as i32;

        let mut swapped: Option<(usize, usize)> = None;

        match key.code {
            KeyCode::Up | KeyCode::Down => {
                if key.modifiers == KeyModifiers::NONE {
                    if key.code == KeyCode::Up {
                        i -= 1;
                    } else {
                        i += 1;
                    }
                // } else if key.modifiers == KeyModifiers::ALT {
                //     if let Some(next) = next_index_by_album(&*items, i, key.code) {
                //         i = next as i32;
                //     }
                } else if key.modifiers == KeyModifiers::CONTROL && items.len() > 1 {
                    // swap
                    let nexti = if key.code == KeyCode::Up && i > 0 {
                        i - 1
                    } else if key.code == KeyCode::Down && (i as usize) < items.len().saturating_sub(1) {
                        i + 1
                    } else {
                        i
                    };

                    if nexti != i {
                        items.swap(i as usize, nexti as usize);
                        swapped = Some((i as usize, nexti as usize));
                        i = nexti;
                    }

                } else {
                    return;
                }

                let padding = if key.code == KeyCode::Up {
                    padding
                } else {
                    height.saturating_sub(padding).saturating_sub(1)
                };

                if (key.code == KeyCode::Up && i < offset + padding) || (key.code == KeyCode::Down && i > offset + padding) {
                    offset = if i > padding {
                        i - padding
                    } else {
                        0
                    };
                }

            },

            KeyCode::Home => {
                i = 0;
                offset = 0;
            },
            KeyCode::End => {
                i = length - 1;
                offset = i - height + padding;
            },
            _ => {},
        }

        offset = offset.min(length - height).max(0);
        i = i.min(length - 1).max(0);

        self.offset.store(offset as usize, Ordering::SeqCst);
        self.selected_item_index.store(i as usize, Ordering::SeqCst);

        drop(items);

        if let Some(swapped) = swapped {
            self.on_reorder_fn.lock().unwrap()(swapped.0, swapped.1);
        }
    }

}

use std::sync::atomic::Ordering;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{ui::KeyboardHandlerRef};

use super::component::List;

impl<'a, T: 'a + Clone> KeyboardHandlerRef<'a> for List<'a, T>
where T: std::fmt::Display
{

    fn on_key(&self, key: KeyEvent) {
        let target = "::List.on_key";
        log::trace!(target: target, "{:?}", key);

        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Home | KeyCode::End => {
                self.on_directional_key(key);
            },
            KeyCode::Enter => {
                self.filter_mut(|filter| {
                    filter.clear();
                });

                let items = self.items.lock().unwrap();

                let i = self.selected_item_index.load(Ordering::Acquire);
                if i >= items.len() {
                    log::error!(target: target, "selected_item_index > items.len");
                    return;
                }
                let item = items[i].inner.clone();
                drop(items);

                if key.modifiers == KeyModifiers::ALT {
                    self.on_enter_fn.lock().unwrap()(item);
                } else {
                    self.on_select_fn.lock().unwrap()(item, key);
                }

            },
            KeyCode::Delete => {
                let i = self.selected_item_index.load(Ordering::Acquire);
                let mut items = self.items.lock().unwrap();
                let removed_item = items.remove(i);

                if i >= items.len() {
                    self.selected_item_index.store(items.len().saturating_sub(1), Ordering::Release);
                }

                drop(items);

                self.on_delete_fn.lock().unwrap()(removed_item.inner, i);
            },
            KeyCode::Char(char) => {
                self.filter_mut(|filter| {
                    filter.push(char);
                });
            },
            KeyCode::Esc => {
                self.filter_mut(|filter| {
                    filter.clear();
                });
            },
            _ => {},
        }
    }
}


impl<'a, T> List<'a, T>
where T: std::fmt::Display + Clone
{
    fn on_directional_key(&self, key: KeyEvent) {
        let is_filtering = !self.filter.lock().unwrap().is_empty();
        let mut items = self.items.lock().unwrap();
        let length = items.len() as i32;

        let height = self.height.load(Ordering::Relaxed) as i32;
        let padding = 5;

        let padding = if key.code == KeyCode::Down || key.code == KeyCode::End {
            height.saturating_sub(padding).saturating_sub(1)
        } else {
            padding
        };

        let mut i = self.selected_item_index.load(Ordering::SeqCst) as i32;

        let mut swapped: Option<(usize, usize)> = None;

        match key.code {
            KeyCode::Up | KeyCode::Down => {
                if key.modifiers == KeyModifiers::NONE {
                    if key.code == KeyCode::Up {
                        if is_filtering {
                            if let Some(n) = items.iter().take(i as usize).rposition(|item| item.is_match) {
                                i = n as i32;
                            }
                        } else {
                            i -= 1;
                        }
                    } else {
                        if is_filtering {
                            if let Some(n) = items.iter().skip(i as usize + 1).position(|item| item.is_match) {
                                i += n as i32 + 1;
                            }
                        } else {
                            i += 1;
                        }
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
            },
            KeyCode::Home => {
                if is_filtering {
                    if let Some(n) = items.iter().position(|item| item.is_match) {
                        i = n as i32;
                    } else {
                        i = 0;
                    }
                } else {
                    i = 0;
                }
            },
            KeyCode::End => {
                if is_filtering {
                    if let Some(n) = items.iter().rposition(|item| item.is_match) {
                        i = n as i32;
                    } else {
                        i = length - 1;
                    }
                } else {
                    i = length - 1;
                }
            },
            _ => {},
        }

        let offset = self.offset.load(Ordering::Acquire) as i32;
        if (key.code == KeyCode::Up && i < offset + padding) || (key.code == KeyCode::Down && i > offset + padding) || key.code == KeyCode::Home || key.code == KeyCode::End {
            let offset = if i > padding {
                (i - padding).min(length - height).max(0)
            } else {
                0
            };
            self.offset.store(offset as usize, Ordering::Release);
        }

        i = i.min(length - 1).max(0);
        self.selected_item_index.store(i as usize, Ordering::SeqCst);

        let newly_selected_item = items[i as usize].inner.clone();

        drop(items);

        if let Some(swapped) = swapped {
            self.on_reorder_fn.lock().unwrap()(swapped.0, swapped.1);
        } else { // TODO: if i != previous_i
            self.on_select_fn.lock().unwrap()(newly_selected_item, key);
        }
    }

}

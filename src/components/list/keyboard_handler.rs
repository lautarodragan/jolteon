use std::sync::{atomic::Ordering, MutexGuard};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::ui::KeyboardHandlerRef;

use super::component::{Direction, List};

impl<'a, T> KeyboardHandlerRef<'a> for List<'a, T>
where
    T: 'a + Clone + std::fmt::Display,
{
    fn on_key(&self, key: KeyEvent) {
        let target = "::List.on_key";
        log::trace!(target: target, "{:?}", key);

        let mut rename = self.rename.lock().unwrap();

        if rename.is_some() {
            self.on_rename_key(key, rename);
            return;
        }

        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Home | KeyCode::End | KeyCode::PageUp | KeyCode::PageDown => {
                self.on_directional_key(key);
            }
            KeyCode::Enter => {
                self.filter_mut(|filter| {
                    filter.clear();
                });

                let items = self.items.lock().unwrap();

                let i = self.selected_item_index.get();
                if i >= items.len() {
                    log::error!(target: target, "selected_item_index > items.len");
                    return;
                }
                let item = items[i].inner.clone();
                drop(items);

                if key.modifiers == KeyModifiers::NONE {
                    self.on_enter_fn.lock().unwrap()(item);
                    if self.auto_select_next.load(Ordering::Acquire) {
                        self.on_directional_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
                    }
                } else if key.modifiers == KeyModifiers::ALT {
                    if let Some(on_enter_alt_fn) = &*self.on_enter_alt_fn.lock().unwrap() {
                        on_enter_alt_fn(item);
                        self.on_directional_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
                    }
                }
            }
            KeyCode::Insert => {
                let f = self.on_insert_fn.lock().unwrap();
                let Some(f) = &*f else {
                    return;
                };
                f();
            }
            KeyCode::Delete => {
                let Some(on_delete) = &*self.on_delete_fn.lock().unwrap() else {
                    return;
                };

                let mut items = self.items.lock().unwrap();

                if items.is_empty() {
                    return;
                }

                let i = self.selected_item_index.get();
                let removed_item = items.remove(i);

                if i >= items.len() {
                    self.selected_item_index.set(items.len().saturating_sub(1));
                }

                drop(items);

                on_delete(removed_item.inner, i);
            }
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                if self.on_rename_fn.lock().unwrap().is_none() {
                    return;
                }
                *rename = self.with_selected_item(|item| Some(item.to_string()));
                drop(rename);
                self.on_request_focus_trap_fn.lock().unwrap()(true);
            }
            KeyCode::Char(char) => {
                self.filter_mut(|filter| {
                    filter.push(char);
                });
            }
            KeyCode::Esc => {
                self.filter_mut(|filter| {
                    filter.clear();
                });
            }
            _ => {}
        }
    }
}

fn is_key_dir_upwards(key_code: KeyCode) -> bool {
    key_code == KeyCode::Up || key_code == KeyCode::Home || key_code == KeyCode::PageUp
}

fn is_key_dir_downwards(key_code: KeyCode) -> bool {
    key_code == KeyCode::Down || key_code == KeyCode::End || key_code == KeyCode::PageDown
}

impl<T> List<'_, T>
where
    T: std::fmt::Display + Clone,
{
    fn on_directional_key(&self, key: KeyEvent) {
        let is_filtering = !self.filter.lock().unwrap().is_empty();
        let mut items = self.items.lock().unwrap();
        let length = items.len() as i32;

        if length < 2 {
            return;
        }

        let height = self.height.get() as i32;
        let padding = self.padding.get() as i32;
        let page_size = self.page_size.get() as i32;

        let padding = if is_key_dir_downwards(key.code) {
            height.saturating_sub(padding).saturating_sub(1)
        } else {
            padding
        };

        let mut i = self.selected_item_index.get() as i32;
        let initial_i = i;

        let on_reorder = self.on_reorder_fn.lock().unwrap();
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
                    } else if is_filtering {
                        if let Some(n) = items.iter().skip(i as usize + 1).position(|item| item.is_match) {
                            i += n as i32 + 1;
                        }
                    } else {
                        i += 1;
                    }
                } else if key.modifiers == KeyModifiers::ALT {
                    if let Some(next_item_special) = &*self.find_next_item_by_fn.lock().unwrap() {
                        let inners: Vec<&T> = items.iter().map(|i| &i.inner).collect();

                        if let Some(ii) = next_item_special(&inners, i as usize, Direction::from(key.code)) {
                            i = ii as i32;
                        }
                    }
                } else if on_reorder.is_some() && key.modifiers == KeyModifiers::CONTROL {
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
            }
            KeyCode::PageUp if !is_filtering => {
                i -= page_size;
            }
            KeyCode::PageDown if !is_filtering => {
                i += page_size;
            }
            KeyCode::Home => {
                if is_filtering {
                    if let Some(n) = items.iter().position(|item| item.is_match) {
                        i = n as i32;
                    };
                } else {
                    i = 0;
                }
            }
            KeyCode::End => {
                if is_filtering {
                    if let Some(n) = items.iter().rposition(|item| item.is_match) {
                        i = n as i32;
                    };
                } else {
                    i = length - 1;
                }
            }
            _ => {}
        }

        if i == initial_i {
            return;
        }

        i = i.min(length - 1).max(0);
        self.selected_item_index.set(i as usize);

        let offset = self.offset.get() as i32;
        if (is_key_dir_upwards(key.code) && i < offset + padding)
            || (is_key_dir_downwards(key.code) && i > offset + padding)
        {
            let offset = if i > padding {
                (i - padding).min(length - height).max(0)
            } else {
                0
            };
            self.offset.set(offset as usize);
        }

        let newly_selected_item = items[i as usize].inner.clone();

        drop(items);

        if let Some(swapped) = swapped {
            if let Some(f) = &*on_reorder {
                f(swapped.0, swapped.1);
            }
        } else {
            drop(on_reorder);
            self.on_select_fn.lock().unwrap()(newly_selected_item);
        }
    }

    fn on_rename_key(&self, key: KeyEvent, mut rename_opt: MutexGuard<Option<String>>) {
        let Some(ref mut rename) = *rename_opt else {
            return;
        };

        match key.code {
            KeyCode::Char(char) => {
                rename.push(char);
            }
            KeyCode::Backspace => {
                if key.modifiers == KeyModifiers::ALT {
                    rename.clear();
                } else if !rename.is_empty() {
                    rename.remove(rename.len().saturating_sub(1));
                }
            }
            KeyCode::Esc => {
                *rename_opt = None;
                self.on_request_focus_trap_fn.lock().unwrap()(false);
            }
            KeyCode::Enter => {
                if rename.is_empty() {
                    return;
                }

                let on_rename_fn = self.on_rename_fn.lock().unwrap();

                let Some(ref on_rename_fn) = *on_rename_fn else {
                    return;
                };

                on_rename_fn((*rename).to_string());

                *rename_opt = None;
                self.on_request_focus_trap_fn.lock().unwrap()(false);
            }
            _ => {}
        }
    }
}

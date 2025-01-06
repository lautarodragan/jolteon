use std::cell::RefMut;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::structs::{Action, ListAction, NavigationAction, OnAction};

use super::component::{Direction, List};

// TODO: OnAction
// impl<'a, T> KeyboardHandlerRef<'a> for List<'a, T>
// where
//     T: 'a + Clone + std::fmt::Display,
// {
//     fn on_key(&self, key: KeyEvent) {
//         match key.code {
//             KeyCode::Char(char) => {
//                 self.filter_mut(|filter| {
//                     filter.push(char);
//                 });
//             }
//         }
//     }
// }

impl<T> List<'_, T>
where
    T: std::fmt::Display + Clone,
{
    fn on_navigation_action(&self, action: NavigationAction) {
        let is_filtering = !self.filter.borrow_mut().is_empty();
        let length = self.items.borrow().len();

        if length < 2 {
            return;
        }

        let initial_i = self.selected_item_index.get();

        let i = match action {
            NavigationAction::NextSpecial | NavigationAction::PreviousSpecial => {
                let Some(next_item_special_fn) = &*self.find_next_item_by_fn.borrow_mut() else {
                    return;
                };
                let items = self.items.borrow();
                let inners: Vec<&T> = items.iter().map(|i| &i.inner).collect();

                let Some(ii) = next_item_special_fn(&inners, initial_i, Direction::from(action)) else {
                    return;
                };

                ii
            }
            NavigationAction::Up if !is_filtering && initial_i > 1 => initial_i - 1,
            NavigationAction::Down if !is_filtering => initial_i + 1,
            NavigationAction::Up if is_filtering => {
                let items = self.items.borrow();
                let Some(n) = items.iter().take(initial_i).rposition(|item| item.is_match) else {
                    return;
                };
                n
            }
            NavigationAction::Down if is_filtering => {
                let items = self.items.borrow();
                let Some(n) = items.iter().skip(initial_i + 1).position(|item| item.is_match) else {
                    return;
                };
                initial_i + n + 1
            }
            NavigationAction::PageUp if !is_filtering => initial_i.saturating_sub(self.page_size.get() as usize),
            NavigationAction::PageDown if !is_filtering => initial_i + self.page_size.get() as usize,
            NavigationAction::Home if !is_filtering => 0,
            NavigationAction::End if !is_filtering => usize::MAX,
            NavigationAction::Home if is_filtering => {
                let items = self.items.borrow();
                let Some(n) = items.iter().position(|item| item.is_match) else {
                    return;
                };
                n
            }
            NavigationAction::End if is_filtering => {
                let items = self.items.borrow();
                let Some(n) = items.iter().rposition(|item| item.is_match) else {
                    return;
                };
                n
            }
            _ => {
                return;
            }
        };

        let i = i.min(length - 1); // SAFETY: if length < 2, function exits early

        if i == initial_i {
            return;
        }

        self.set_selected_index(i);

        let newly_selected_item = self.items.borrow()[i].inner.clone();

        self.on_select_fn.borrow_mut()(newly_selected_item);
    }

}

impl<'a, T> OnAction for List<'a, T>
where
    T: 'a + Clone + std::fmt::Display,
{
    fn on_action(&self, action: Action) {
        let target = "::List.on_action";

        let mut rename_option = self.rename.borrow_mut();

        match action {
            Action::Navigation(action) if rename_option.is_none() => {
                self.on_navigation_action(action);
            }
            Action::ListAction(action) if rename_option.is_none() => match action {
                ListAction::Primary | ListAction::Secondary => {
                    self.filter_mut(|filter| {
                        filter.clear();
                    });

                    let items = self.items.borrow();

                    let i = self.selected_item_index.get();
                    if i >= items.len() {
                        log::error!(target: target, "selected_item_index > items.len");
                        return;
                    }
                    let item = items[i].inner.clone();
                    drop(items);

                    if action == ListAction::Primary {
                        self.on_enter_fn.borrow_mut()(item);
                        if self.auto_select_next.get() {
                            self.on_navigation_action(NavigationAction::Down);
                        }
                    } else if action == ListAction::Secondary {
                        if let Some(on_enter_alt_fn) = &*self.on_enter_alt_fn.borrow_mut() {
                            on_enter_alt_fn(item);
                            if self.auto_select_next.get() {
                                self.on_navigation_action(NavigationAction::Down);
                            }
                        }
                    }
                }
                ListAction::Insert => {
                    let f = self.on_insert_fn.borrow_mut();
                    let Some(f) = &*f else {
                        return;
                    };
                    f();
                }
                ListAction::Delete => {
                    let Some(on_delete) = &*self.on_delete_fn.borrow_mut() else {
                        return;
                    };

                    let mut items = self.items.borrow_mut();

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
                ListAction::SwapUp | ListAction::SwapDown => {
                    let on_reorder = self.on_reorder_fn.borrow_mut();

                    let Some(on_reorder) = &*on_reorder else {
                        return;
                    };

                    let i = self.selected_item_index.get();
                    let mut items = self.items.borrow_mut();

                    let next_i;
                    if action == ListAction::SwapUp && i > 0 {
                        next_i = i - 1;
                    } else if action == ListAction::SwapDown && i < items.len().saturating_sub(1) {
                        next_i = i + 1;
                    } else {
                        return;
                    };

                    items.swap(i, next_i);
                    drop(items);
                    self.set_selected_index(next_i);
                    on_reorder(i, next_i);
                }
                ListAction::RenameStart if self.on_rename_fn.borrow().is_some() => {
                    *rename_option = self.with_selected_item(|item| Some(item.to_string()));
                    self.on_request_focus_trap_fn.borrow_mut()(true);
                }
                ListAction::Cancel => {
                    self.filter_mut(|filter| {
                        filter.clear();
                    });
                }
                _ => {}
            },
            Action::ListAction(action) if rename_option.is_some() => match action {
                ListAction::Primary => {
                    self.on_request_focus_trap_fn.borrow_mut()(false);

                    if rename_option.as_ref().is_none_or(|r| r.is_empty()) {
                        return;
                    }

                    let on_rename_fn = self.on_rename_fn.borrow_mut();

                    let Some(ref on_rename_fn) = *on_rename_fn else {
                        return;
                    };

                    on_rename_fn(rename_option.take().unwrap());
                }
                ListAction::Cancel => {
                    *rename_option = None;
                    self.on_request_focus_trap_fn.borrow_mut()(false);
                }
                ListAction::RenameChar(c) => {
                    let Some(ref mut rename) = *rename_option else {
                        return;
                    };
                    rename.push(c);
                }
                ListAction::RenameDeleteCharBack => {
                    let Some(ref mut rename) = *rename_option else {
                        return;
                    };
                    rename.remove(rename.len().saturating_sub(1));
                }
                ListAction::RenameClear => {
                    let Some(ref mut rename) = *rename_option else {
                        return;
                    };
                    rename.clear();
                }
                _ => {}
            },
            _ => {}
        }
    }
}

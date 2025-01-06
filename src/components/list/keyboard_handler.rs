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
//         let target = "::List.on_key";
//         log::trace!(target: target, "{:?}", key);
//
//         let mut rename = self.rename.borrow_mut();
//
//         if rename.is_some() {
//             self.on_rename_key(key, rename);
//             return;
//         }
//
//         match key.code {
//             KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
//                 if self.on_rename_fn.borrow_mut().is_none() {
//                     return;
//                 }
//                 *rename = self.with_selected_item(|item| Some(item.to_string()));
//                 drop(rename);
//                 self.on_request_focus_trap_fn.borrow_mut()(true);
//             }
//             KeyCode::Char(char) => {
//                 self.filter_mut(|filter| {
//                     filter.push(char);
//                 });
//             }
//             KeyCode::Esc => {
//                 self.filter_mut(|filter| {
//                     filter.clear();
//                 });
//             }
//             _ => {}
//         }
//     }
// }

fn is_navigation_action_upwards(action: NavigationAction) -> bool {
    action == NavigationAction::Up
        || action == NavigationAction::Home
        || action == NavigationAction::PageUp
        || action == NavigationAction::PreviousSpecial
}

fn is_navigation_action_downwards(action: NavigationAction) -> bool {
    action == NavigationAction::Down
        || action == NavigationAction::End
        || action == NavigationAction::PageDown
        || action == NavigationAction::NextSpecial
}

impl<T> List<'_, T>
where
    T: std::fmt::Display + Clone,
{
    fn on_directional_action(&self, action: NavigationAction) {
        let is_filtering = !self.filter.borrow_mut().is_empty();
        let items = self.items.borrow_mut();
        let length = items.len() as i32;

        if length < 2 {
            return;
        }

        let mut i = self.selected_item_index.get() as i32;
        let initial_i = i;

        match action {
            NavigationAction::NextSpecial | NavigationAction::PreviousSpecial => {
                if let Some(next_item_special) = &*self.find_next_item_by_fn.borrow_mut() {
                    let inners: Vec<&T> = items.iter().map(|i| &i.inner).collect();

                    if let Some(ii) = next_item_special(&inners, i as usize, Direction::from(action)) {
                        i = ii as i32;
                    }
                }
            }
            NavigationAction::Up | NavigationAction::Down => {
                if action == NavigationAction::Up {
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
            }
            NavigationAction::PageUp | NavigationAction::PageDown if !is_filtering => {
                let page_size = self.page_size.get() as i32;

                if action == NavigationAction::PageUp {
                    i -= page_size;
                } else {
                    i += page_size;
                }
            }
            NavigationAction::Home => {
                if is_filtering {
                    if let Some(n) = items.iter().position(|item| item.is_match) {
                        i = n as i32;
                    };
                } else {
                    i = 0;
                }
            }
            NavigationAction::End => {
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
        let i = i as usize;

        drop(items);
        self.set_selected_index(i);

        let newly_selected_item = {
            let items = self.items.borrow();
            items[i].inner.clone()
        };

        self.on_select_fn.borrow_mut()(newly_selected_item);
    }

    fn on_rename_key(&self, key: KeyEvent, mut rename_opt: RefMut<Option<String>>) {
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
                self.on_request_focus_trap_fn.borrow_mut()(false);
            }
            KeyCode::Enter => {
                if rename.is_empty() {
                    return;
                }

                let on_rename_fn = self.on_rename_fn.borrow_mut();

                let Some(ref on_rename_fn) = *on_rename_fn else {
                    return;
                };

                on_rename_fn((*rename).to_string());

                *rename_opt = None;
                self.on_request_focus_trap_fn.borrow_mut()(false);
            }
            _ => {}
        }
    }
}

impl<'a, T> OnAction for List<'a, T>
where
    T: 'a + Clone + std::fmt::Display,
{
    fn on_action(&self, action: Action) {
        let target = "::List.on_action";

        match action {
            Action::Navigation(action) => {
                self.on_directional_action(action);
            }
            Action::ListAction(action) => match action {
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
                            self.on_directional_action(NavigationAction::Down);
                        }
                    } else if action == ListAction::Secondary {
                        if let Some(on_enter_alt_fn) = &*self.on_enter_alt_fn.borrow_mut() {
                            on_enter_alt_fn(item);
                            self.on_directional_action(NavigationAction::Down);
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

                    let nexti;
                    if action == ListAction::SwapUp && i > 0 {
                        nexti = i - 1;
                    } else if action == ListAction::SwapDown && i < items.len().saturating_sub(1) {
                        nexti = i + 1;
                    } else {
                        return;
                    };

                    items.swap(i, nexti);
                    drop(items);
                    self.set_selected_index(nexti);
                    on_reorder(i, nexti);
                }
            },
            _ => {}
        }
    }
}

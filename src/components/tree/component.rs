use std::{
    cell::{Cell, RefCell},
    cmp::Ordering,
    fmt::{Debug, Display},
};

use crate::{
    actions::{Action, ListAction, NavigationAction, TextAction},
    config::Theme,
    structs::Direction,
    ui::Focusable,
};

use super::{TreeNode, TreeNodePath};

pub struct Tree<'a, T: 'a> {
    pub(super) theme: Theme,

    pub(super) items: RefCell<Vec<TreeNode<T>>>,
    pub(super) selected_item_path: RefCell<TreeNodePath>,

    pub(super) on_select_fn: Option<Box<dyn Fn(&TreeNode<T>) + 'a>>,
    pub(super) on_enter_fn: Option<Box<dyn Fn(&T) + 'a>>,
    pub(super) on_enter_alt_fn: Option<Box<dyn Fn(&T) + 'a>>,
    pub(super) on_reorder_fn: Option<Box<dyn Fn(TreeNodePath, usize, usize) + 'a>>,
    pub(super) on_insert_fn: Option<Box<dyn Fn() + 'a>>,
    pub(super) on_delete_fn: Option<Box<dyn Fn(T, Vec<usize>) + 'a>>,
    pub(super) on_rename_fn: Option<Box<dyn Fn(String) + 'a>>,
    pub(super) on_request_focus_trap: Option<Box<dyn Fn(bool) + 'a>>,
    pub(super) find_next_item_by_fn: Option<Box<dyn Fn(&[&T], usize, Direction) -> Option<usize> + 'a>>,

    pub(super) auto_select_next: Cell<bool>,

    pub(super) offset: Cell<usize>,
    pub(super) height: Cell<usize>,
    pub(super) line_style: Option<Box<dyn Fn(&T) -> Option<ratatui::style::Style> + 'a>>,
    pub(super) is_focused: Cell<bool>,

    pub(super) filter: RefCell<String>,
    pub(super) rename: RefCell<Option<String>>,

    pub(super) padding: u8,
    pub(super) page_size: u8,
}

impl<'a, T> Tree<'a, T>
where
    T: 'a + Display + Debug,
{
    pub fn new(theme: Theme, items: Vec<TreeNode<T>>) -> Self {
        Self {
            theme,

            on_select_fn: None,
            on_enter_fn: None,
            on_enter_alt_fn: None,
            on_reorder_fn: None,
            on_insert_fn: None,
            on_delete_fn: None,
            on_rename_fn: None,
            on_request_focus_trap: None,
            find_next_item_by_fn: None,

            items: RefCell::new(items),
            selected_item_path: RefCell::new(TreeNodePath::zero()),

            auto_select_next: Cell::new(true),

            offset: Cell::new(0),
            height: Cell::new(0),
            line_style: None,
            is_focused: Cell::default(),

            filter: RefCell::new("".to_string()),
            rename: RefCell::new(None),

            padding: 5,
            page_size: 5,
        }
    }

    pub fn set_auto_select_next(&self, v: bool) {
        self.auto_select_next.set(v)
    }

    pub fn line_style(&mut self, cb: impl Fn(&T) -> Option<ratatui::style::Style> + 'a) {
        self.line_style = Some(Box::new(cb));
    }

    pub fn with_node_at_path<R>(&self, path: &TreeNodePath, cb: impl FnOnce(&TreeNode<T>) -> R) -> R {
        let items = self.items.borrow();
        let node = TreeNode::get_node_at_path(path, &items).unwrap();
        cb(node)
    }

    pub fn with_node_at_path_mut<R>(&self, path: TreeNodePath, cb: impl FnOnce(&mut TreeNode<T>) -> R) -> R {
        let mut items = self.items.borrow_mut();
        let node = &mut TreeNode::get_node_at_path_mut(path.clone(), &mut items);
        cb(node)
    }

    pub fn with_selected_node<R>(&self, cb: impl FnOnce(&TreeNode<T>) -> R) -> R {
        let selected_item_path = self.selected_item_path.borrow();
        self.with_node_at_path(&selected_item_path, cb)
    }

    pub fn with_selected_item_mut(&self, cb: impl FnOnce(&mut TreeNode<T>)) {
        self.with_node_at_path_mut((*self.selected_item_path.borrow()).clone(), cb)
    }

    /// Triggered by moving the selection around, with the Up and Down arrow keys by default.
    pub fn on_select(&mut self, cb: impl Fn(&TreeNode<T>) + 'a) {
        self.on_select_fn = Some(Box::new(cb));
    }

    /// Triggered, by default, with Enter.
    /// Not the most intuitive name, but it is what it is.
    pub fn on_enter(&mut self, cb: impl Fn(&T) + 'a) {
        self.on_enter_fn = Some(Box::new(cb));
    }

    /// An alternative "on_enter", triggered, by default, with Alt+Enter.
    /// This is somewhat tightly coupled to functionality required by consumers of this List component.
    pub fn on_enter_alt(&mut self, cb: impl Fn(&T) + 'a) {
        self.on_enter_alt_fn = Some(Box::new(cb));
    }

    /// Callback will be called with (parent's path, old index, new index).
    pub fn on_reorder(&mut self, cb: impl Fn(TreeNodePath, usize, usize) + 'a) {
        self.on_reorder_fn = Some(Box::new(cb));
    }

    pub fn on_insert(&mut self, cb: impl Fn() + 'a) {
        self.on_insert_fn = Some(Box::new(cb));
    }

    pub fn on_delete(&mut self, cb: impl Fn(T, Vec<usize>) + 'a) {
        self.on_delete_fn = Some(Box::new(cb));
    }

    pub fn on_rename(&mut self, cb: impl Fn(String) + 'a) {
        self.on_rename_fn = Some(Box::new(cb));
    }

    pub fn on_request_focus_trap_fn(&mut self, cb: impl Fn(bool) + 'a) {
        self.on_request_focus_trap = Some(Box::new(cb));
    }

    /// Function used to select next/previous item by some custom logic.
    /// Triggered by Alt+Up/Down by default.
    /// Currently used only to jump to the first song of the next/previous album
    /// in the Library's song list (right panel).
    ///
    /// If the List component was a Tree component, we wouldn't need this "special" behavior.
    /// We'd be jumping to the first child of the next/previous parent.
    ///
    /// If `tree.selected_path` was `Vec<usize>`, we'd do something like the following:
    ///
    /// ```
    /// tree.selected_path[tree.selected_path.len() - 2] += 1;
    /// tree.selected_path[tree.selected_path.len() - 1] = 0;
    /// ```
    pub fn find_next_item_by_fn(&mut self, cb: impl Fn(&[&T], usize, Direction) -> Option<usize> + 'a) {
        self.find_next_item_by_fn = Some(Box::new(cb));
    }

    /// Sets the list of items and resets selection and scroll
    pub fn set_items(&self, items: Vec<TreeNode<T>>) {
        self.set_items_s(items, TreeNodePath::zero(), 0);
    }

    // /// Sets the list of items but tries to conserve selection and scroll
    // pub fn set_items_k(&self, new_items: Vec<TreeNode<T>>) {
    //     let mut items = self.items.borrow_mut();
    //
    //     if new_items.len() < items.len() {
    //         let difference = items.len().saturating_sub(new_items.len());
    //         // let selected_item_index = self.selected_item_index.get();
    //         // let new_selected_item_index = selected_item_index.saturating_sub(difference).min(new_items.len());
    //         // self.selected_item_index.set(new_selected_item_index);
    //
    //         let current_offset = self.offset.get();
    //         if current_offset > new_items.len().saturating_sub(self.height.get()) {
    //             self.offset.set(current_offset.saturating_sub(difference));
    //         }
    //     }
    //
    //     *items = new_items;
    // }

    /// Sets the list of items, selection and scroll
    fn set_items_s(&self, new_items: Vec<TreeNode<T>>, i: TreeNodePath, o: usize) {
        *self.selected_item_path.borrow_mut() = i;
        self.offset.set(o);
        *self.items.borrow_mut() = new_items;
    }

    pub fn set_is_open_all(&self, v: bool) {
        let mut items = self.items.borrow_mut();

        for item in &mut *items {
            item.is_open = v;
        }
    }

    pub fn filter(&self) -> String {
        self.filter.borrow().clone()
    }

    pub fn filter_mut(&self, cb: impl FnOnce(&mut String)) {
        // let mut items = self.items.borrow_mut();
        //
        // if items.len() < 2 {
        //     return;
        // }
        //
        // let mut filter = self.filter.borrow_mut();
        //
        // cb(&mut filter);
        //
        // for item in items.iter_mut() {
        //     if filter.is_empty() {
        //         item.is_match = false;
        //     } else {
        //         item.is_match = item
        //             .inner
        //             .to_string()
        //             .to_lowercase()
        //             .contains(filter.to_lowercase().as_str());
        //     }
        // }

        // let selected_item_index = self.selected_item_index.get();
        // if !items[selected_item_index].is_match {
        //     if let Some(i) = items.iter().skip(selected_item_index).position(|item| item.is_match) {
        //         let i = i + selected_item_index;
        //         self.selected_item_index.set(i);
        //     } else if let Some(i) = items.iter().position(|item| item.is_match) {
        //         self.selected_item_index.set(i);
        //     }
        // }
    }

    pub fn scroll_position(&self) -> usize {
        self.offset.get()
    }

    // pub fn selected_index(&self) -> Vec<usize> {
    //     self.selected_item_index.borrow()
    // }

    pub fn set_selected_index(&self, new_i: TreeNodePath) {
        log::debug!("set_selected_index {new_i:?}");
        *self.selected_item_path.borrow_mut() = new_i;
    }

    pub fn exec_action(&mut self, action: Action) {
        let target = "::List.on_action";

        if self.rename.borrow().is_some() {
            self.exec_rename_action(action);
        } else {
            match action {
                Action::Navigation(action) => self.exec_navigation_action(action),
                Action::Confirm | Action::ConfirmAlt => {
                    self.filter_mut(|filter| {
                        filter.clear();
                    });

                    let items = self.items.borrow();
                    let node = TreeNode::get_node_at_path(&self.selected_item_path.borrow(), &items).unwrap();

                    if action == Action::Confirm {
                        if let Some(on_enter_fn) = &self.on_enter_fn {
                            on_enter_fn(&node.inner);
                        }
                        if self.auto_select_next.get() {
                            self.exec_navigation_action(NavigationAction::Down);
                        }
                    } else if action == Action::ConfirmAlt {
                        if let Some(on_enter_alt_fn) = &self.on_enter_alt_fn {
                            on_enter_alt_fn(&node.inner);
                            if self.auto_select_next.get() {
                                self.exec_navigation_action(NavigationAction::Down);
                            }
                        }
                    }
                }
                Action::Cancel => {
                    self.filter_mut(|filter| {
                        filter.clear();
                    });
                }
                Action::ListAction(action) => self.exec_list_action(action),
                Action::Text(action) => self.exec_text_action(action),
                _ => {}
            }
        };
    }

    fn exec_navigation_action(&self, action: NavigationAction) {
        let is_filtering = !self.filter.borrow_mut().is_empty();
        let nodes = self.items.borrow();

        if nodes.is_empty() {
            return;
        }

        let mut initial_i = self.selected_item_path.borrow_mut();

        if initial_i.is_empty() {
            log::error!("exec_navigation_action: self.selected_item_path was empty.");
            *initial_i = TreeNodePath::zero();
        }

        let i = match action {
            NavigationAction::PreviousSpecial => {
                if initial_i.len() > 1 {
                    initial_i.parent()
                } else if initial_i.first() > 0 {
                    initial_i.with_value(0, initial_i.first() - 1)
                } else {
                    initial_i.clone()
                }
            }
            NavigationAction::NextSpecial => {
                // TODO: maybe define these as "next / previous node with children"? (and implement them so)
                if initial_i.len() > 1 {
                    let new_i = initial_i.parent();

                    let sibling_count = {
                        if new_i.len() == 1 {
                            nodes.len()
                        } else {
                            let parent_path = new_i.parent();
                            let node = TreeNode::get_node_at_path(&parent_path, &nodes).unwrap();
                            node.children.len()
                        }
                    };

                    if new_i.last() + 1 < sibling_count {
                        new_i.with_value(new_i.len() - 1, new_i.last() + 1)
                    } else {
                        new_i
                    }
                } else if initial_i.first() + 1 < nodes.len() {
                    initial_i.with_value(0, initial_i.first() + 1)
                } else {
                    initial_i.clone()
                }
            }
            NavigationAction::Up if !is_filtering => {
                if initial_i.last() > 0 {
                    let new_path = initial_i.with_value(initial_i.len() - 1, initial_i.last() - 1);
                    let node = TreeNode::get_node_at_path(&new_path, &nodes).unwrap();

                    if node.is_open && !node.children.is_empty() {
                        new_path.with_child(node.children.len() - 1)
                    } else {
                        new_path
                    }
                } else if initial_i.len() > 1 {
                    initial_i.parent()
                } else {
                    initial_i.clone()
                }
            }
            NavigationAction::Down if !is_filtering => {
                let node = TreeNode::get_node_at_path(&initial_i, &nodes).unwrap();

                if node.is_open && !node.children.is_empty() {
                    // Walk down / into
                    initial_i.with_child(0)
                } else {
                    // Walk next/up/next
                    let mut dynamic_path = initial_i.clone();

                    loop {
                        if dynamic_path.is_empty() {
                            log::error!("NavigationAction::Down: dynamic_path is empty already! {initial_i:?}");
                            break initial_i.clone();
                        };

                        let last = dynamic_path.last();
                        dynamic_path = dynamic_path.parent();

                        if dynamic_path.is_empty() {
                            break TreeNodePath::from_vec(vec![(last + 1).min(nodes.len().saturating_sub(1))]);
                        }

                        let parent_node = TreeNode::get_node_at_path(&dynamic_path, &nodes).unwrap();

                        if parent_node.children.len() > last + 1 {
                            break dynamic_path.with_child(last + 1);
                        }
                    }
                }
            }
            // NavigationAction::Up if is_filtering => {
            //     let items = self.items.borrow();
            //     let Some(n) = items.iter().take(initial_i).rposition(|item| item.is_match) else {
            //         return;
            //     };
            //     n
            // }
            // NavigationAction::Down if is_filtering => {
            //     let items = self.items.borrow();
            //     let Some(n) = items.iter().skip(initial_i + 1).position(|item| item.is_match) else {
            //         return;
            //     };
            //     initial_i + n + 1
            // }
            NavigationAction::PageUp if !is_filtering => {
                // NOTE: PageUp MUST result in the same as pressing Up `page_size` times.
                // It'll make more sense to implement the logic here, and have the "normal" Up/Down
                // call this code with a page_size of 1.
                initial_i.clone()
            }
            NavigationAction::PageDown if !is_filtering => initial_i.clone(),
            NavigationAction::Home if !is_filtering => TreeNodePath::zero(),
            NavigationAction::End if !is_filtering => {
                let mut new_i = vec![nodes.len() - 1];
                let mut node = &nodes[nodes.len() - 1];

                loop {
                    if !node.is_open || node.children.is_empty() {
                        break TreeNodePath::from_vec(new_i);
                    }

                    let i = node.children.len() - 1;
                    new_i.push(i);
                    node = &node.children[i];
                }
            }
            // NavigationAction::Home if is_filtering => {
            //     let v_items = self.visible_items.borrow();
            //     let items = self.items.borrow();
            //     let Some(n) = v_items.iter().position(|item| items[*item].is_match) else {
            //         return;
            //     };
            //     n
            // }
            // NavigationAction::End if is_filtering => {
            //     let v_items = self.visible_items.borrow();
            //     let items = self.items.borrow();
            //     let Some(n) = v_items.iter().rposition(|item| items[*item].is_match) else {
            //         return;
            //     };
            //     n
            // }
            _ => {
                return;
            }
        };

        let dir = i.cmp(&initial_i);

        if dir == Ordering::Equal {
            return;
        }

        let total_visible_node_count = TreeNode::total_open_count(&nodes) as isize;
        let visible_node_count_until_selection = TreeNode::open_count(&nodes, &i) as isize;
        let height = self.height.get() as isize;
        let offset = self.offset.get() as isize;
        let padding = self.padding as isize;
        let padding = if dir == Ordering::Greater {
            height - padding - 1
        } else {
            padding
        };

        if (dir == Ordering::Less && visible_node_count_until_selection < offset + padding)
            || (dir == Ordering::Greater && visible_node_count_until_selection > offset + padding)
        {
            let offset = if visible_node_count_until_selection > padding {
                (visible_node_count_until_selection - padding)
                    .min(total_visible_node_count - height)
                    .max(0)
            } else {
                0
            };
            self.offset.set(offset as usize);
        }

        *initial_i = i;

        let newly_selected_item = TreeNode::get_node_at_path(&initial_i, &nodes).unwrap();

        if let Some(on_select_fn) = &self.on_select_fn {
            on_select_fn(newly_selected_item);
        }
    }

    fn exec_rename_action(&mut self, action: Action) {
        let mut rename_option = self.rename.borrow_mut();
        let Some(ref mut rename) = *rename_option else {
            return;
        };
        match action {
            Action::Confirm => {
                if let Some(on_request_focus_trap) = &self.on_request_focus_trap {
                    on_request_focus_trap(false);
                }

                if rename.is_empty() {
                    return;
                }

                let Some(ref on_rename_fn) = &self.on_rename_fn else {
                    return;
                };

                on_rename_fn(rename_option.take().unwrap());
            }
            Action::Cancel => {
                *rename_option = None;
                if let Some(on_request_focus_trap) = &self.on_request_focus_trap {
                    on_request_focus_trap(false);
                }
            }
            Action::Text(TextAction::Char(char)) => {
                rename.push(char);
            }
            Action::Text(TextAction::DeleteBack) => {
                rename.remove(rename.len().saturating_sub(1));
            }
            Action::Text(TextAction::Delete) => {
                rename.remove(rename.len().saturating_sub(1));
            }
            Action::ListAction(ListAction::RenameClear) => {
                rename.clear();
            }
            _ => {}
        }
    }

    fn exec_list_action(&self, action: ListAction) {
        match action {
            ListAction::Insert => {
                let Some(f) = &self.on_insert_fn else {
                    return;
                };
                f();
            }
            ListAction::Delete => {
                let Some(on_delete) = &self.on_delete_fn else {
                    return;
                };

                let items = self.items.borrow_mut();

                if items.is_empty() {
                }

                // let i = self.selected_item_index.borrow();
                // let removed_item = items.remove(i);
                //
                // if i >= items.len() {
                //     // self.selected_item_index.set(items.len().saturating_sub(1));
                // }
                //
                // drop(items);
                //
                // on_delete(removed_item.inner, i.clone());
            }
            ListAction::SwapUp | ListAction::SwapDown => {
                let Some(on_reorder) = &self.on_reorder_fn else {
                    return;
                };

                let mut nodes = self.items.borrow_mut();
                let mut selected_item_path = self.selected_item_path.borrow_mut();
                let path_parent = selected_item_path.parent();
                let selected_node_index = selected_item_path.last();

                let siblings = if path_parent.is_empty() {
                    &mut *nodes
                } else {
                    &mut TreeNode::get_node_at_path_mut(path_parent.clone(), &mut nodes).children
                };

                if siblings.len() < 2 {
                    return;
                }

                let next_i;
                if action == ListAction::SwapUp && selected_node_index > 0 {
                    next_i = selected_node_index - 1;
                } else if action == ListAction::SwapDown && selected_node_index < siblings.len().saturating_sub(1) {
                    next_i = selected_node_index + 1;
                } else {
                    return;
                };

                siblings.swap(selected_node_index, next_i);
                *selected_item_path = path_parent.with_child(next_i);

                drop(nodes);
                drop(selected_item_path);

                on_reorder(path_parent, selected_node_index, next_i);
            }
            ListAction::RenameStart if self.on_rename_fn.is_some() => {
                *self.rename.borrow_mut() = self.with_selected_node(|item| Some(item.inner.to_string()));
                if let Some(on_request_focus_trap) = &self.on_request_focus_trap {
                    on_request_focus_trap(false);
                }
            }
            ListAction::OpenClose => {
                let mut path = self.selected_item_path.borrow_mut();
                let mut items = self.items.borrow_mut();
                let selected_node = TreeNode::get_node_at_path_mut(path.clone(), &mut items);

                log::debug!("ListAction::OpenClose {path} {}", selected_node.inner);

                if !selected_node.children.is_empty() {
                    selected_node.is_open = !selected_node.is_open;
                } else if path.len() > 1 {
                    let parent_path = path.parent();
                    log::debug!("ListAction::OpenClose {parent_path} (parent of {path})");
                    let node = TreeNode::get_node_at_path_mut(parent_path.clone(), &mut items);
                    node.is_open = false;
                    *path = parent_path;

                    // TODO: collapsing the selected node may require lowering the offset by up to node.children.len()
                }
            }
            ListAction::ExpandAll => {
                let mut nodes = self.items.borrow_mut();

                for node in &mut *nodes {
                    node.is_open = true;
                }
            }
            ListAction::CollapseAll => {
                let mut nodes = self.items.borrow_mut();

                for node in &mut *nodes {
                    node.is_open = false;
                }
            }
            _ => {}
        }
    }

    fn exec_text_action(&self, action: TextAction) {
        match action {
            TextAction::Char(char) => {
                self.filter_mut(|filter| {
                    filter.push(char);
                });
            }
            TextAction::DeleteBack => {
                self.filter_mut(|filter| {
                    if !filter.is_empty() {
                        filter.remove(filter.len().saturating_sub(1));
                    }
                });
            }
            _ => {}
        }
    }
}

impl<T> Drop for Tree<'_, T> {
    fn drop(&mut self) {
        log::trace!("Tree.drop()");
    }
}

impl<T> Focusable for Tree<'_, T> {
    fn set_is_focused(&self, v: bool) {
        self.is_focused.set(v);
    }

    fn is_focused(&self) -> bool {
        self.is_focused.get()
    }
}

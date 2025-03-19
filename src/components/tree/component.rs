use std::{
    cell::{Cell, RefCell},
    cmp::Ordering,
    fmt::{Debug, Display},
};

use super::{TreeNode, TreeNodePath};
use crate::{
    actions::{Action, ListAction, NavigationAction, TextAction},
    config::Theme,
    ui::Focusable,
};

pub struct Tree<'a, T: 'a> {
    pub(super) theme: Theme,

    pub(super) items: RefCell<Vec<TreeNode<T>>>,
    pub(super) selected_item_path: RefCell<TreeNodePath>,

    pub(super) on_select_fn: Option<Box<dyn Fn(&TreeNode<T>) + 'a>>,
    pub(super) on_enter_fn: Option<Box<dyn Fn(&T) + 'a>>,
    pub(super) on_enter_alt_fn: Option<Box<dyn Fn(&T) + 'a>>,
    pub(super) on_reorder_fn: Option<Box<dyn Fn(TreeNodePath, usize, usize) + 'a>>,
    pub(super) on_insert_fn: Option<Box<dyn Fn() + 'a>>,
    pub(super) on_delete_fn: Option<Box<dyn Fn(TreeNode<T>, TreeNodePath) + 'a>>,
    pub(super) on_rename_fn: Option<Box<dyn Fn(String) + 'a>>,
    pub(super) on_request_focus_trap: Option<Box<dyn Fn(bool) + 'a>>,

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

    pub fn with_nodes_mut(&mut self, cb: impl FnOnce(&mut Vec<TreeNode<T>>)) {
        let mut items = self.items.borrow_mut();
        cb(&mut *items)
    }

    #[allow(unused)]
    pub fn set_auto_select_next(&self, v: bool) {
        self.auto_select_next.set(v)
    }

    #[allow(unused)]
    pub fn line_style(&mut self, cb: impl Fn(&T) -> Option<ratatui::style::Style> + 'a) {
        self.line_style = Some(Box::new(cb));
    }

    #[allow(unused)]
    pub fn with_node_at_path<R>(&self, path: &TreeNodePath, cb: impl FnOnce(&TreeNode<T>) -> R) -> R {
        let items = self.items.borrow();
        let node = TreeNode::get_node_at_path(path, &items).unwrap();
        cb(node)
    }

    #[allow(unused)]
    pub fn with_node_at_path_mut<R>(&self, path: TreeNodePath, cb: impl FnOnce(&mut TreeNode<T>) -> R) -> R {
        let mut items = self.items.borrow_mut();
        let node = &mut TreeNode::get_node_at_path_mut(path.clone(), &mut items);
        cb(node)
    }

    pub fn with_selected_node<R>(&self, cb: impl FnOnce(&TreeNode<T>) -> R) -> R {
        let selected_item_path = self.selected_item_path.borrow();
        self.with_node_at_path(&selected_item_path, cb)
    }

    #[allow(unused)]
    pub fn with_selected_node_mut(&self, cb: impl FnOnce(&mut TreeNode<T>)) {
        self.with_node_at_path_mut((*self.selected_item_path.borrow()).clone(), cb)
    }

    /// Triggered by moving the selection around, with the Up and Down arrow keys by default.
    pub fn on_select(&mut self, cb: impl Fn(&TreeNode<T>) + 'a) {
        self.on_select_fn = Some(Box::new(cb));
    }

    /// Triggered, by default, with Enter.
    /// Not the most intuitive name, but it is what it is.
    pub fn on_confirm(&mut self, cb: impl Fn(&T) + 'a) {
        self.on_enter_fn = Some(Box::new(cb));
    }

    /// An alternative "on_enter", triggered, by default, with Alt+Enter.
    /// This is somewhat tightly coupled to functionality required by consumers of this List component.
    #[allow(unused)]
    pub fn on_confirm_alt(&mut self, cb: impl Fn(&T) + 'a) {
        self.on_enter_alt_fn = Some(Box::new(cb));
    }

    /// Callback will be called with (parent's path, old index, new index).
    pub fn on_reorder(&mut self, cb: impl Fn(TreeNodePath, usize, usize) + 'a) {
        self.on_reorder_fn = Some(Box::new(cb));
    }

    #[allow(unused)]
    pub fn on_insert(&mut self, cb: impl Fn() + 'a) {
        self.on_insert_fn = Some(Box::new(cb));
    }

    pub fn on_delete(&mut self, cb: impl Fn(TreeNode<T>, TreeNodePath) + 'a) {
        self.on_delete_fn = Some(Box::new(cb));
    }

    #[allow(unused)]
    pub fn on_rename(&mut self, cb: impl Fn(String) + 'a) {
        self.on_rename_fn = Some(Box::new(cb));
    }

    #[allow(unused)]
    pub fn on_request_focus_trap_fn(&mut self, cb: impl Fn(bool) + 'a) {
        self.on_request_focus_trap = Some(Box::new(cb));
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

    pub fn filter_mut(&self, cb: impl FnOnce(&mut String)) {
        if self.items.borrow().len() < 2 {
            return;
        }

        let mut filter = self.filter.borrow_mut();

        cb(&mut filter);

        let mut nodes = self.items.borrow_mut();
        let selected_item_path = self.selected_item_path.borrow_mut();
        let mut selected_node_match_status_changed = false;

        // TODO: recursive iter_mut. this is hard-coded to go down 1 level deep.
        for (index, node) in nodes.iter_mut().enumerate() {
            node.is_match = !filter.is_empty()
                && node
                    .inner
                    .to_string()
                    .to_lowercase()
                    .contains(filter.to_lowercase().as_str());

            if selected_item_path.len() == 1 && selected_item_path[0] == index && !node.is_match {
                selected_node_match_status_changed = true;
            }

            if node.is_open {
                for (child_index, node) in node.children.iter_mut().enumerate() {
                    node.is_match = !filter.is_empty()
                        && node
                            .inner
                            .to_string()
                            .to_lowercase()
                            .contains(filter.to_lowercase().as_str());

                    if selected_item_path.len() == 2
                        && selected_item_path[0] == index
                        && selected_item_path[1] == child_index
                        && !node.is_match
                    {
                        selected_node_match_status_changed = true;
                    }
                }
            }
        }

        if selected_node_match_status_changed {
            log::debug!("selected_node_match_status_changed {filter} {selected_item_path:?}");

            let mut new_path = selected_item_path.clone();

            'outer: for (root_node_index, root_node) in nodes.iter().enumerate() {
                let path = TreeNodePath::from_vec(vec![root_node_index]);

                if root_node.is_match && path > *selected_item_path {
                    new_path = path;
                    break 'outer;
                } else if root_node.is_open {
                    for (path, node) in root_node.iter() {
                        let path = path.with_parent(root_node_index);
                        if path <= *selected_item_path {
                            continue;
                        } else if node.is_match {
                            new_path = path;
                            break 'outer;
                        }
                    }
                }
            }

            drop(nodes);
            drop(selected_item_path);
            drop(filter);

            self.set_selected_path(new_path);
        }
    }

    pub fn exec_action(&mut self, action: Action) {
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
        if let Some(new_path) = self.navigation_action_to_new_path(action) {
            self.set_selected_path(new_path);
        }
    }

    fn navigation_action_to_new_path(&self, action: NavigationAction) -> Option<TreeNodePath> {
        let is_filtering = !self.filter.borrow().is_empty();
        let current_path = {
            let current_path = self.selected_item_path.borrow().clone();
            if current_path.is_empty() {
                log::error!("exec_navigation_action: self.selected_item_path was empty.");
                TreeNodePath::zero()
            } else {
                current_path
            }
        };
        let root_nodes = self.items.borrow();

        if root_nodes.is_empty() {
            return None;
        }

        match action {
            NavigationAction::PreviousSpecial if !is_filtering => {
                if current_path.len() > 1 {
                    Some(current_path.parent())
                } else {
                    current_path.prev_sibling()
                }
            }
            NavigationAction::NextSpecial if !is_filtering => {
                // TODO: maybe define these as "next / previous node with children"? (and implement them so)
                if current_path.len() > 1 {
                    let parent_path = current_path.parent();

                    let sibling_count = {
                        if parent_path.len() == 1 {
                            root_nodes.len()
                        } else {
                            let grand_parent_path = parent_path.parent();
                            let node = TreeNode::get_node_at_path(&grand_parent_path, &root_nodes).unwrap();
                            node.children.len()
                        }
                    };

                    if parent_path.last() + 1 < sibling_count {
                        Some(parent_path.next_sibling())
                    } else {
                        Some(parent_path)
                    }
                } else if current_path.first() + 1 < root_nodes.len() {
                    Some(current_path.next_sibling())
                } else {
                    None
                }
            }
            NavigationAction::Up if !is_filtering => {
                if let Some(new_path) = current_path.prev_sibling() {
                    let node = TreeNode::get_node_at_path(&new_path, &root_nodes).unwrap(); // safety: we can unwrap because there will always be a previous node!

                    Some(if node.is_open && !node.children.is_empty() {
                        new_path.with_child(node.children.len() - 1)
                    } else {
                        new_path
                    })
                } else if current_path.len() > 1 {
                    Some(current_path.parent())
                } else {
                    None
                }
            }
            NavigationAction::Down if !is_filtering => {
                let Some(node) = TreeNode::get_node_at_path(&current_path, &root_nodes) else {
                    panic!("No node at path {current_path:?}. Either TreeNode::get_node_at_path is broken, or something is incorrectly setting current_path.");
                };

                if node.is_open && !node.children.is_empty() {
                    // Walk down / into
                    Some(current_path.with_child(0))
                } else {
                    // Walk next/up/next
                    let mut path = current_path.clone();

                    loop {
                        let last = path.last();
                        path = path.parent();

                        if path.is_empty() {
                            return if last == root_nodes.len() - 1 {
                                None
                            } else {
                                Some(TreeNodePath::from_vec(vec![
                                    (last + 1).min(root_nodes.len().saturating_sub(1))
                                ]))
                            };
                        }

                        let parent_node = TreeNode::get_node_at_path(&path, &root_nodes).unwrap();

                        if parent_node.children.len() > last + 1 {
                            return Some(path.with_child(last + 1));
                        }
                    }
                }
            }
            NavigationAction::Up if is_filtering => {
                for (root_node_index, root_node) in root_nodes.iter().enumerate().rev() {
                    if let Some(path) = root_node
                        .iter()
                        .rev()
                        .filter(|(_, node)| node.is_match)
                        .map(|(path, _)| path)
                        .map(|path| path.with_parent(root_node_index))
                        .find(|path| *path < current_path)
                    {
                        return Some(path);
                    }

                    if root_node.is_match {
                        let path = TreeNodePath::from_vec(vec![root_node_index]);
                        if path < current_path {
                            return Some(path);
                        }
                    }
                }

                None
            }
            NavigationAction::Down if is_filtering => {
                for (root_node_index, root_node) in root_nodes.iter().enumerate() {
                    let path = TreeNodePath::from_vec(vec![root_node_index]);

                    if root_node.is_match && path > current_path {
                        return Some(path);
                    } else if root_node.is_open {
                        if let Some(path) = root_node
                            .iter()
                            .filter(|(path, node)| node.is_match)
                            .map(|(path, node)| path)
                            .map(|path| path.with_parent(root_node_index))
                            .find(|path| *path > current_path)
                        {
                            return Some(path);
                        }
                    }
                }

                None
            }
            NavigationAction::PageUp if !is_filtering => {
                // NOTE: PageUp MUST result in the same as pressing Up `page_size` times.
                // It'll make more sense to implement the logic here, and have the "normal" Up/Down
                // call this code with a page_size of 1.
                None
            }
            NavigationAction::PageDown if !is_filtering => None,
            NavigationAction::Home if !is_filtering => Some(TreeNodePath::zero()),
            NavigationAction::End if !is_filtering => {
                let mut new_path = vec![root_nodes.len() - 1];
                let mut node = &root_nodes[root_nodes.len() - 1];

                loop {
                    if !node.is_open || node.children.is_empty() {
                        return Some(TreeNodePath::from_vec(new_path));
                    }

                    let i = node.children.len() - 1;
                    new_path.push(i);
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
            _ => None,
        }
    }

    fn set_selected_path(&self, new_path: TreeNodePath) {
        log::debug!("set_selected_path {new_path:?}");

        let mut current_path = self.selected_item_path.borrow_mut();

        let dir = new_path.cmp(&current_path);

        if dir == Ordering::Equal {
            return;
        }

        let nodes = self.items.borrow();

        let Some(node_at_new_path) = TreeNode::get_node_at_path(&new_path, &nodes) else {
            panic!("No node at path {new_path:?}");
        };

        let total_visible_node_count = TreeNode::total_open_count(&nodes) as isize;
        let visible_node_count_until_selection = TreeNode::open_count(&nodes, &new_path) as isize - 1;
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

        *current_path = new_path;
        drop(current_path);

        if let Some(on_select_fn) = &self.on_select_fn {
            on_select_fn(node_at_new_path);
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

                let mut items = self.items.borrow_mut();

                if items.is_empty() {
                    return;
                }

                let selected_item_path = self.selected_item_path.borrow_mut();
                if selected_item_path.is_empty() {
                    log::warn!("selected_item_path.is_empty()");
                    return;
                }

                let removed_item = if selected_item_path.len() == 1 {
                    items.remove(selected_item_path.first())
                } else {
                    let parent_path = selected_item_path.parent();
                    let parent = TreeNode::get_node_at_path_mut(parent_path.clone(), &mut items);
                    parent.children.remove(selected_item_path.last())
                };

                drop(items);

                on_delete(removed_item, selected_item_path.clone());
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

                // TODO: if the selected node was closed, `selected_path = selected_path.parent()`
                // and update the scroll position if necessary
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

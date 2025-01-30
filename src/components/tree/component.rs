use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter, Write};
use std::ops::{Deref, DerefMut};

use crossterm::event::KeyCode;

use crate::{
    actions::{Action, ListAction, NavigationAction, TextAction},
    config::Theme,
    ui::Focusable,
    structs::Direction,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct TreeNodePath(Vec<usize>);

impl TreeNodePath {
    fn new() -> Self {
        Self(vec![])
    }

    fn parent(&self) -> Self {
        let mut parent = self.clone();
        let new_len = parent.len().saturating_sub(1);
        parent.truncate(new_len);
        parent
    }

    fn deepest(&self) -> usize {
        self[self.len().saturating_sub(1)]
    }

    fn with_child(&self, i: usize) -> Self {
        let mut path = self.clone();
        path.push(i);
        path
    }
}

impl Deref for TreeNodePath {
    type Target = Vec<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TreeNodePath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Ord for TreeNodePath {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut j = 0;
        loop {
            if j >= self.0.len().min(other.0.len()) {
                break self.0.len().cmp(&other.0.len());
            }

            let ord = self.0[j].cmp(&other.0[j]);

            if ord != Ordering::Equal {
                break ord;
            }

            j += 1;
        }
    }
}

impl Display for TreeNodePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in &self.0 {
            write!(f, "/{i}")?;
        }
        Ok(())
    }
}

fn cmp_vec(vec_a: &[usize], vec_b: &[usize]) -> Ordering {
    let vec_a = TreeNodePath(vec_a.to_vec());
    let vec_b = TreeNodePath(vec_b.to_vec());
    vec_a.cmp(&vec_b)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeNode<T> {
    pub inner: T,
    pub is_visible: bool,
    pub is_match: bool,
    pub is_open: bool,
    pub children: Vec<Self>,
}

impl<T> TreeNode<T> {
    pub fn new(t: T) -> Self {
        Self {
            inner: t,
            is_visible: true,
            is_match: false,
            is_open: true,
            children: vec![],
        }
    }

    fn total_open_children_count(&self) -> usize {
        fn recursive_total_open_count<T>(nodes: &[TreeNode<T>]) -> usize {
            let mut count = 0;
            for i in 0..nodes.len() {
                count += 1;

                if !nodes[i].is_open || nodes[i].children.is_empty() {
                    continue
                }

                count += recursive_total_open_count(&nodes[i].children);
            }
            count
        }
        if !self.is_open || self.children.is_empty() {
            0
        } else {
            recursive_total_open_count(&self.children)
        }
    }

    fn total_open_count(nodes: &[TreeNode<T>]) -> usize {
        nodes.len() + nodes.iter().map(|node| node.total_open_children_count()).sum::<usize>()
    }

    fn open_count(nodes: &[TreeNode<T>], until_path: &TreeNodePath) -> usize {
        fn recursive_open_count<T>(nodes: &[TreeNode<T>], path: TreeNodePath, until_path: &TreeNodePath) -> usize {
            let mut count = 0;
            for i in 0..nodes.len() {
                let mut new_path = path.clone();
                new_path.push(i);

                if new_path >= *until_path {
                    break;
                }

                count += 1;

                if !nodes[i].is_open || nodes[i].children.is_empty() {
                    continue
                }

                count += recursive_open_count(&nodes[i].children, new_path, &until_path);
            }
            count
        }
        recursive_open_count(&*nodes, TreeNodePath::new(), until_path)
    }
}

fn get_node_at_path<T>(mut path: VecDeque<usize>, nodes: &[TreeNode<T>]) -> &TreeNode<T> {
    let Some(next_level) = path.pop_front() else {
        panic!("get_node_at_path panic");
    };

    if path.is_empty() {
        &nodes[next_level]
    } else {
        get_node_at_path(path, &nodes[next_level].children)
    }
}


fn get_node_at_path_mut<T>(mut path: VecDeque<usize>, nodes: &mut[TreeNode<T>]) -> &mut TreeNode<T> {
    let Some(next_level) = path.pop_front() else {
        panic!("get_node_at_path_mut panic");
    };

    if path.is_empty() {
        &mut nodes[next_level]
    } else {
        get_node_at_path_mut(path, &mut nodes[next_level].children)
    }
}

pub struct Tree<'a, T: 'a> {
    pub(super) theme: Theme,

    pub(super) items: RefCell<Vec<TreeNode<T>>>,
    pub(super) selected_item_path: RefCell<Vec<usize>>,

    pub(super) on_select_fn: Box<dyn Fn(TreeNode<T>) + 'a>,
    pub(super) on_enter_fn: RefCell<Box<dyn Fn(T) + 'a>>,
    pub(super) on_enter_alt_fn: RefCell<Option<Box<dyn Fn(T) + 'a>>>,
    pub(super) on_reorder_fn: RefCell<Option<Box<dyn Fn(TreeNodePath, usize, usize) + 'a>>>,
    pub(super) on_insert_fn: RefCell<Option<Box<dyn Fn() + 'a>>>,
    pub(super) on_delete_fn: RefCell<Option<Box<dyn Fn(T, Vec<usize>) + 'a>>>,
    pub(super) on_rename_fn: RefCell<Option<Box<dyn Fn(String) + 'a>>>,
    pub(super) on_request_focus_trap_fn: RefCell<Box<dyn Fn(bool) + 'a>>,
    pub(super) find_next_item_by_fn: RefCell<Option<Box<dyn Fn(&[&T], usize, Direction) -> Option<usize> + 'a>>>,

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
    T: 'a + Clone + Display + Debug,
{
    pub fn new(theme: Theme, items: Vec<TreeNode<T>>) -> Self {
        Self {
            theme,

            on_select_fn: Box::new(|_| {}) as _,
            on_enter_fn: RefCell::new(Box::new(|_| {}) as _),
            on_enter_alt_fn: RefCell::new(None),
            on_reorder_fn: RefCell::new(None),
            on_insert_fn: RefCell::new(None),
            on_delete_fn: RefCell::new(None),
            on_rename_fn: RefCell::new(None),
            on_request_focus_trap_fn: RefCell::new(Box::new(|_| {}) as _),
            find_next_item_by_fn: RefCell::new(None),

            items: RefCell::new(items),
            selected_item_path: RefCell::new(vec![0]),

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

    pub fn with_node_at_path<R>(&self, path: VecDeque<usize>, cb: impl FnOnce(&TreeNode<T>) -> R) -> R {
        let items = self.items.borrow();
        let node = get_node_at_path(path, &*items);
        cb(node)
    }

    pub fn with_node_at_path_mut<R>(&self, path: VecDeque<usize>, cb: impl FnOnce(&mut TreeNode<T>) -> R) -> R {
        let mut items = self.items.borrow_mut();
        let node = &mut get_node_at_path_mut(path, &mut *items);
        cb(node)
    }

    pub fn with_selected_node<R>(&self, cb: impl FnOnce(&TreeNode<T>) -> R) -> R {
        self.with_node_at_path((*self.selected_item_path.borrow()).clone().into(), cb)
    }

    pub fn with_selected_item_mut(&self, cb: impl FnOnce(&mut TreeNode<T>)) {
        self.with_node_at_path_mut((*self.selected_item_path.borrow()).clone().into(), cb)
    }

    /// Triggered by moving the selection around, with the Up and Down arrow keys by default.
    pub fn on_select(&mut self, cb: impl Fn(TreeNode<T>) + 'a) {
        self.on_select_fn = Box::new(cb);
    }

    /// Triggered, by default, with Enter.
    /// Not the most intuitive name, but it is what it is.
    pub fn on_enter(&self, cb: impl Fn(T) + 'a) {
        *self.on_enter_fn.borrow_mut() = Box::new(cb);
    }

    /// An alternative "on_enter", triggered, by default, with Alt+Enter.
    /// This is somewhat tightly coupled to functionality required by consumers of this List component.
    pub fn on_enter_alt(&self, cb: impl Fn(T) + 'a) {
        *self.on_enter_alt_fn.borrow_mut() = Some(Box::new(cb));
    }

    /// Callback will be called with (parent's path, old index, new index).
    pub fn on_reorder(&self, cb: impl Fn(TreeNodePath, usize, usize) + 'a) {
        *self.on_reorder_fn.borrow_mut() = Some(Box::new(cb));
    }

    pub fn on_insert(&self, cb: impl Fn() + 'a) {
        *self.on_insert_fn.borrow_mut() = Some(Box::new(cb));
    }

    pub fn on_delete(&self, cb: impl Fn(T, Vec<usize>) + 'a) {
        *self.on_delete_fn.borrow_mut() = Some(Box::new(cb));
    }

    pub fn on_rename(&self, cb: impl Fn(String) + 'a) {
        *self.on_rename_fn.borrow_mut() = Some(Box::new(cb));
    }

    pub fn on_request_focus_trap_fn(&self, cb: impl Fn(bool) + 'a) {
        *self.on_request_focus_trap_fn.borrow_mut() = Box::new(cb);
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
    pub fn find_next_item_by_fn(&self, cb: impl Fn(&[&T], usize, Direction) -> Option<usize> + 'a) {
        *self.find_next_item_by_fn.borrow_mut() = Some(Box::new(cb));
    }

    /// Sets the list of items and resets selection and scroll
    pub fn set_items(&self, items: Vec<TreeNode<T>>) {
        self.set_items_s(items, vec![0], 0);
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
    fn set_items_s(&self, new_items: Vec<TreeNode<T>>, i: Vec<usize>, o: usize) {
        *self.selected_item_path.borrow_mut() = i;
        self.offset.set(o);
        *self.items.borrow_mut() = new_items;
    }

    fn is_open(&self, i: usize) -> bool {
        self.items.borrow()[i].is_open
    }

    fn set_is_open(&self, i: usize, v: bool) {
        let mut items = self.items.borrow_mut();
        items[i].is_open = v;
    }

    pub fn toggle_is_open(&self, i: usize) -> bool {
        let is_open = !self.is_open(i);
        self.set_is_open(i, is_open);
        is_open
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

    pub fn set_selected_index(&self, new_i: Vec<usize>) {
        log::debug!("set_selected_index {new_i:?}");
        *self.selected_item_path.borrow_mut() = new_i;
    }

    pub fn exec_action(&self, action: Action) {
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
                    let i = self.selected_item_path.borrow().clone();
                    let node = get_node_at_path(i.into(), &*items);
                    let item = node.inner.clone();
                    drop(items);

                    if action == Action::Confirm {
                        self.on_enter_fn.borrow_mut()(item);
                        if self.auto_select_next.get() {
                            self.exec_navigation_action(NavigationAction::Down);
                        }
                    } else if action == Action::ConfirmAlt {
                        if let Some(on_enter_alt_fn) = &*self.on_enter_alt_fn.borrow_mut() {
                            on_enter_alt_fn(item);
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
            *initial_i = vec![0];
        }

        let i = match action {
            NavigationAction::PreviousSpecial => {
                let mut new_i = initial_i.clone();
                if initial_i.len() > 1 {
                    new_i.truncate(initial_i.len() - 1);
                } else if new_i[0] > 0{
                    new_i[0] -= 1;
                }
                new_i
            }
            NavigationAction::NextSpecial => {
                // TODO: maybe define these as "next / previous node with children"? (and implement them so)
                let mut new_i = initial_i.clone();
                if initial_i.len() > 1 {
                    let new_len = initial_i.len() - 1;
                    new_i.truncate(new_len);

                    let sibling_count = {
                        if new_i.len() == 1 {
                            nodes.len()
                        } else {
                            let parent_path = new_i[..new_i.len() - 1].to_vec();
                            let node = get_node_at_path(parent_path.into(), &*nodes);
                            node.children.len()
                        }
                    };

                    if new_i[new_len - 1] + 1 < sibling_count {
                        new_i[new_len - 1] += 1;
                    }
                } else if new_i[0] + 1 < nodes.len() {
                    new_i[0] += 1;
                }
                new_i
            }
            NavigationAction::Up if !is_filtering => {
                if initial_i[initial_i.len() - 1] > 0 {
                    let mut new_i = initial_i.clone();
                    new_i[initial_i.len() - 1] -= 1;

                    let node = get_node_at_path(new_i.clone().into(), &*nodes);

                    if node.is_open && !node.children.is_empty() {
                        new_i.push(node.children.len() - 1)
                    }

                    new_i
                } else if initial_i.len() > 1 {
                    let mut new_i = initial_i.clone();
                    new_i.truncate(initial_i.len() - 1);
                    new_i
                } else {
                    initial_i.clone()
                }
            },
            NavigationAction::Down if !is_filtering => {
                let node = get_node_at_path(initial_i.clone().into(), &*nodes);

                if node.is_open && !node.children.is_empty() {
                    // Walk down / into
                    let mut new_path = initial_i.clone();
                    new_path.push(0);
                    new_path
                } else {
                    // Walk next/up/next
                    let mut dynamic_path: VecDeque<usize> = initial_i.clone().into();

                    loop {
                        let Some(discarded) = dynamic_path.pop_back() else {
                            log::error!("NavigationAction::Down: dynamic_path is empty already! {initial_i:?}");
                            break initial_i.clone();
                        };

                        if dynamic_path.is_empty() {
                            break vec![(discarded + 1).min(nodes.len().saturating_sub(1))];
                        }

                        let parent_node = get_node_at_path(dynamic_path.clone(), &*nodes); // TODO: get_node_at_path -> Option<...>

                        if parent_node.children.len() > discarded + 1 {
                            dynamic_path.push_back(discarded + 1);
                            break dynamic_path.into();
                        }
                    }
                }
            },
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
            },
            NavigationAction::PageDown if !is_filtering => {
                initial_i.clone()
            },
            NavigationAction::Home if !is_filtering => vec![0],
            NavigationAction::End if !is_filtering => {
                let mut new_i = vec![nodes.len() - 1];
                let mut node = &nodes[nodes.len() - 1];

                loop {
                    if !node.is_open || node.children.is_empty() {
                        break new_i;
                    }

                    let i = node.children.len() - 1;
                    new_i.push(i);
                    node = &node.children[i];
                }
            },
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

        let dir = cmp_vec(&i, &initial_i);

        if dir == Ordering::Equal {
            return;
        }

        let total_visible_node_count = TreeNode::total_open_count(&*nodes) as isize;
        let visible_node_count_until_selection = TreeNode::open_count(&*nodes, &TreeNodePath(i.clone())) as isize;
        let height = self.height.get() as isize;
        let offset = self.offset.get() as isize;
        let padding = self.padding as isize;
        let padding = if dir == Ordering::Greater { height - padding - 1 } else { padding };

        if (dir == Ordering::Less && visible_node_count_until_selection < offset + padding) || (dir == Ordering::Greater && visible_node_count_until_selection > offset + padding) {
            let offset = if visible_node_count_until_selection > padding {
                (visible_node_count_until_selection - padding).min(total_visible_node_count - height).max(0)
            } else {
                0
            };
            self.offset.set(offset as usize);
        }

        *initial_i = i;

        let newly_selected_item = get_node_at_path(initial_i.clone().into(), &*nodes);
        let inner_clone = newly_selected_item.clone();
        drop(nodes);
        (self.on_select_fn)(inner_clone);
    }

    fn exec_rename_action(&self, action: Action) {
        let mut rename_option = self.rename.borrow_mut();
        let Some(ref mut rename) = *rename_option else {
            return;
        };
        match action {
            Action::Confirm => {
                self.on_request_focus_trap_fn.borrow_mut()(false);

                if rename.is_empty() {
                    return;
                }

                let on_rename_fn = self.on_rename_fn.borrow_mut();

                let Some(ref on_rename_fn) = *on_rename_fn else {
                    return;
                };

                on_rename_fn(rename_option.take().unwrap());
            }
            Action::Cancel => {
                *rename_option = None;
                self.on_request_focus_trap_fn.borrow_mut()(false);
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
                let on_reorder = self.on_reorder_fn.borrow_mut();

                let Some(on_reorder) = &*on_reorder else {
                    return;
                };

                let mut nodes = self.items.borrow_mut();
                let mut selected_item_path = self.selected_item_path.borrow_mut();
                let path_p = TreeNodePath(selected_item_path.clone());
                let path_parent = path_p.parent();
                let selected_node_index = path_p.deepest();

                let mut siblings = if path_parent.is_empty() {
                    &mut *nodes
                } else {
                    &mut get_node_at_path_mut((&*path_parent).clone().into(), &mut *nodes).children
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
                *selected_item_path = path_parent.with_child(next_i).0;

                drop(nodes);
                drop(selected_item_path);

                on_reorder(path_parent, selected_node_index, next_i);
            }
            ListAction::RenameStart if self.on_rename_fn.borrow().is_some() => {
                *self.rename.borrow_mut() = self.with_selected_node(|item| Some(item.inner.to_string()));
                self.on_request_focus_trap_fn.borrow_mut()(true);
            }
            ListAction::OpenClose => {
                let mut path = self.selected_item_path.borrow_mut();
                let mut items = self.items.borrow_mut();
                let node = get_node_at_path_mut(path.clone().into(), &mut *items);
                log::debug!("ListAction::OpenClose {path:?} {}", node.inner);

                if !node.children.is_empty() {
                    node.is_open = !node.is_open;
                } else if path.len() > 1 {
                    let parent_path: Vec<usize> = path[..path.len() - 1].into();
                    log::debug!("ListAction::OpenClose {parent_path:?} (parent of {path:?})");
                    let node = get_node_at_path_mut(parent_path.into(), &mut *items);
                    node.is_open = false;
                    let new_len = path.len().saturating_sub(1);
                    path.truncate(new_len);

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

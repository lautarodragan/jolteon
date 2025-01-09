use std::cell::{Cell, RefCell};

use crossterm::event::KeyCode;

use crate::{
    structs::{Action, ListAction, TextAction},
    {config::Theme, structs::NavigationAction},
};

#[derive(Eq, PartialEq)]
pub enum Direction {
    Backwards,
    Forwards,
}

impl From<KeyCode> for Direction {
    fn from(key_code: KeyCode) -> Self {
        if key_code == KeyCode::Up || key_code == KeyCode::Home || key_code == KeyCode::PageUp {
            Self::Backwards
        } else {
            Self::Forwards
        }
    }
}

impl From<NavigationAction> for Direction {
    fn from(action: NavigationAction) -> Self {
        if action == NavigationAction::Up
            || action == NavigationAction::Home
            || action == NavigationAction::PageUp
            || action == NavigationAction::PreviousSpecial
        {
            Self::Backwards
        } else {
            Self::Forwards
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ListItem<T> {
    pub inner: T,
    pub is_visible: bool,
    pub is_match: bool,
    pub is_open: bool,
}

impl<T> ListItem<T> {
    pub fn new(t: T) -> Self {
        Self {
            inner: t,
            is_visible: true,
            is_match: false,
            is_open: true,
        }
    }
}

pub struct List<'a, T: 'a>
where
    T: std::fmt::Display,
{
    pub(super) theme: Theme,

    pub(super) items: RefCell<Vec<ListItem<T>>>,
    pub(super) visible_items: RefCell<Vec<usize>>,
    pub(super) selected_item_index: Cell<usize>,

    pub(super) on_select_fn: RefCell<Box<dyn Fn(T) + 'a>>,
    pub(super) on_enter_fn: RefCell<Box<dyn Fn(T) + 'a>>,
    pub(super) on_enter_alt_fn: RefCell<Option<Box<dyn Fn(T) + 'a>>>,
    pub(super) on_reorder_fn: RefCell<Option<Box<dyn Fn(usize, usize) + 'a>>>,
    pub(super) on_insert_fn: RefCell<Option<Box<dyn Fn() + 'a>>>,
    pub(super) on_delete_fn: RefCell<Option<Box<dyn Fn(T, usize) + 'a>>>,
    pub(super) on_rename_fn: RefCell<Option<Box<dyn Fn(String) + 'a>>>,
    pub(super) on_request_focus_trap_fn: RefCell<Box<dyn Fn(bool) + 'a>>,
    pub(super) find_next_item_by_fn: RefCell<Option<Box<dyn Fn(&[&T], usize, Direction) -> Option<usize> + 'a>>>,

    pub(super) auto_select_next: Cell<bool>,

    pub(super) offset: Cell<usize>,
    pub(super) height: Cell<usize>,
    pub(super) line_style: RefCell<Option<Box<dyn Fn(&T) -> Option<ratatui::style::Style> + 'a>>>,
    pub(super) is_focused: Cell<bool>,

    pub(super) filter: RefCell<String>,
    pub(super) rename: RefCell<Option<String>>,

    pub(super) padding: Cell<u8>,
    pub(super) page_size: Cell<u8>,
}

impl<'a, T> List<'a, T>
where
    T: 'a + Clone + std::fmt::Display,
{
    pub fn new(theme: Theme, items: Vec<T>) -> Self {
        let items: Vec<ListItem<T>> = items.into_iter().map(ListItem::new).collect();

        let s = Self {
            theme,

            on_select_fn: RefCell::new(Box::new(|_| {}) as _),
            on_enter_fn: RefCell::new(Box::new(|_| {}) as _),
            on_enter_alt_fn: RefCell::new(None),
            on_reorder_fn: RefCell::new(None),
            on_insert_fn: RefCell::new(None),
            on_delete_fn: RefCell::new(None),
            on_rename_fn: RefCell::new(None),
            on_request_focus_trap_fn: RefCell::new(Box::new(|_| {}) as _),
            find_next_item_by_fn: RefCell::new(None),

            items: RefCell::new(items),
            visible_items: RefCell::default(),
            selected_item_index: Cell::new(0),

            auto_select_next: Cell::new(true),

            offset: Cell::new(0),
            height: Cell::new(0),
            line_style: RefCell::new(None),
            is_focused: Cell::default(),

            filter: RefCell::new("".to_string()),
            rename: RefCell::new(None),

            padding: Cell::new(5),
            page_size: Cell::new(5),
        };

        s.refresh_visible_items();
        s
    }

    pub fn focus(&self) {
        self.is_focused.set(true);
    }

    pub fn blur(&self) {
        self.is_focused.set(false);
    }

    pub fn set_auto_select_next(&self, v: bool) {
        self.auto_select_next.set(v);
    }

    pub fn line_style(&self, cb: impl Fn(&T) -> Option<ratatui::style::Style> + 'a) {
        let mut line_style = self.line_style.borrow_mut();
        *line_style = Some(Box::new(cb));
    }

    pub fn with_items<R>(&self, cb: impl FnOnce(Vec<&T>) -> R) -> R {
        let items = self.items.borrow();
        let items_inner = (*items).iter().map(|a| &a.inner).collect();
        cb(items_inner)
    }

    pub fn with_selected_item<R>(&self, cb: impl FnOnce(&T) -> R) -> R {
        let items = self.items.borrow();
        let i = self.selected_item_index.get();
        cb(&items[i].inner)
    }

    pub fn with_selected_item_mut(&self, cb: impl FnOnce(&mut T)) {
        let mut items = self.items.borrow_mut();
        let i = self.selected_item_index.get();
        cb(&mut items[i].inner);
    }

    /// Triggered by moving the selection around, with the Up and Down arrow keys by default.
    pub fn on_select(&self, cb: impl Fn(T) + 'a) {
        *self.on_select_fn.borrow_mut() = Box::new(cb);
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

    pub fn on_reorder(&self, cb: impl Fn(usize, usize) + 'a) {
        *self.on_reorder_fn.borrow_mut() = Some(Box::new(cb));
    }

    pub fn on_insert(&self, cb: impl Fn() + 'a) {
        *self.on_insert_fn.borrow_mut() = Some(Box::new(cb));
    }

    pub fn on_delete(&self, cb: impl Fn(T, usize) + 'a) {
        *self.on_delete_fn.borrow_mut() = Some(Box::new(cb));
    }

    pub fn on_rename(&self, cb: impl Fn(String) + 'a) {
        *self.on_rename_fn.borrow_mut() = Some(Box::new(cb));
    }

    pub fn on_request_focus_trap_fn(&self, cb: impl Fn(bool) + 'a) {
        *self.on_request_focus_trap_fn.borrow_mut() = Box::new(cb);
    }

    pub fn find_next_item_by_fn(&self, cb: impl Fn(&[&T], usize, Direction) -> Option<usize> + 'a) {
        *self.find_next_item_by_fn.borrow_mut() = Some(Box::new(cb));
    }

    /// Sets the list of items and resets selection and scroll
    pub fn set_items(&self, items: Vec<T>) {
        self.set_items_s(items, 0, 0);
    }

    /// Sets the list of items but tries to conserve selection and scroll
    pub fn set_items_k(&self, new_items: Vec<T>) {
        let mut items = self.items.borrow_mut();

        if new_items.len() < items.len() {
            let difference = items.len().saturating_sub(new_items.len());
            let selected_item_index = self.selected_item_index.get();
            let new_selected_item_index = selected_item_index.saturating_sub(difference).min(new_items.len());
            self.selected_item_index.set(new_selected_item_index);

            let current_offset = self.offset.get();
            if current_offset > new_items.len().saturating_sub(self.height.get()) {
                self.offset.set(current_offset.saturating_sub(difference));
            }
        }

        *items = new_items.into_iter().map(ListItem::new).collect();

        let mut visible_items = self.visible_items.borrow_mut();
        visible_items.resize(items.len(), 0);
        for i in 0..visible_items.len() {
            visible_items[i] = i;
        }
    }

    /// Sets the list of items, selection and scroll
    pub fn set_items_s(&self, new_items: Vec<T>, i: usize, o: usize) {
        self.selected_item_index.set(i);
        self.offset.set(o);
        *self.items.borrow_mut() = new_items.into_iter().map(ListItem::new).collect();
        self.refresh_visible_items();
    }

    fn refresh_visible_items(&self) {
        let items = self.items.borrow_mut();
        let mut visible_items = self.visible_items.borrow_mut();
        visible_items.clear();
        for i in 0..items.len() {
            if items[i].is_visible {
                visible_items.push(i)
            }
        }
    }

    pub fn set_is_visible(&self, i: usize, v: bool) {
        let mut items = self.items.borrow_mut();
        items[i].is_visible = v;
        drop(items);
        self.refresh_visible_items();
    }

    #[allow(unused)]
    pub fn is_visible(&self, i: usize) -> bool {
        self.items.borrow()[i].is_visible
    }

    pub fn set_is_open(&self, i: usize, v: bool) {
        let mut items = self.items.borrow_mut();
        items[i].is_open = v;
    }

    pub fn is_open(&self, i: usize) -> bool {
        self.items.borrow()[i].is_open
    }

    pub fn push_item(&self, item: T) {
        let mut items = self.items.borrow_mut();
        items.push(ListItem::new(item));
    }

    pub fn append_items(&self, items_to_append: impl IntoIterator<Item = T>) {
        let mut items = self.items.borrow_mut();
        let mut items_to_append: Vec<ListItem<T>> = items_to_append.into_iter().map(ListItem::new).collect();

        items.append(&mut items_to_append);
    }

    pub fn filter_mut(&self, cb: impl FnOnce(&mut String)) {
        let mut filter = self.filter.borrow_mut();

        cb(&mut filter);

        let mut items = self.items.borrow_mut();

        if items.len() < 2 {
            return;
        }

        for item in items.iter_mut() {
            if filter.is_empty() {
                item.is_match = false;
            } else {
                item.is_match = item
                    .inner
                    .to_string()
                    .to_lowercase()
                    .contains(filter.to_lowercase().as_str());
            }
        }

        let selected_item_index = self.selected_item_index.get();
        if !items[selected_item_index].is_match {
            if let Some(i) = items.iter().skip(selected_item_index).position(|item| item.is_match) {
                let i = i + selected_item_index;
                self.selected_item_index.set(i);
            } else if let Some(i) = items.iter().position(|item| item.is_match) {
                self.selected_item_index.set(i);
            }
        }
    }

    pub fn scroll_position(&self) -> usize {
        self.offset.get()
    }

    pub fn selected_index(&self) -> usize {
        self.selected_item_index.get()
    }

    pub fn selected_index_true(&self) -> usize {
        let i = self.selected_item_index.get();
        self.visible_items.borrow()[i]
    }

    pub fn set_selected_index(&self, new_i: usize) {
        let current_i = self.selected_item_index.get();

        if new_i == current_i {
            return;
        }

        let visible_items = self.visible_items.borrow();

        assert!(new_i < visible_items.len());

        self.selected_item_index.set(new_i);

        let is_down = new_i > current_i;
        let is_up = new_i < current_i;

        let new_i = new_i as isize;
        let height = self.height.get() as isize;
        let offset = self.offset.get() as isize;
        let padding = self.padding.get() as isize;
        let padding = if is_down { height - padding - 1 } else { padding };

        if (is_up && new_i < offset + padding) || (is_down && new_i > offset + padding) {
            let offset = if new_i > padding {
                (new_i - padding).min(visible_items.len() as isize - height).max(0)
            } else {
                0
            };
            self.offset.set(offset as usize);
        }
    }

    pub fn set_selected_index_true(&self, new_i: usize) {
        let i = {
            let visible_items = self.visible_items.borrow();
            visible_items.iter().position(|i| *i == new_i).unwrap()
        };
        log::debug!("set_selected_index_true {new_i} -> {i}");
        self.set_selected_index(i);
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

                    let i = self.selected_item_index.get();
                    if i >= items.len() {
                        log::error!(target: target, "selected_item_index > items.len");
                        return;
                    }
                    let item = items[i].inner.clone();
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

    #[allow(unused)]
    pub(super) fn next_visible_index(&self, start: usize) -> usize {
        let items = self.items.borrow();
        let mut next = start + 1;
        loop {
            if let Some(item) = items.get(next)
                && item.is_visible
            {
                return next;
            } else if next >= items.len() - 1 {
                return start;
            } else {
                next += 1;
            }
        }
    }

    #[allow(unused)]
    pub(super) fn previous_visible_index(&self, start: usize) -> usize {
        if start == 0 {
            return start;
        }
        let items = self.items.borrow();
        let mut next = start - 1;
        loop {
            if let Some(item) = items.get(next)
                && item.is_visible
            {
                return next;
            } else if next == 0 {
                return start;
            } else {
                next -= 1;
            }
        }
    }

    fn exec_navigation_action(&self, action: NavigationAction) {
        let is_filtering = !self.filter.borrow_mut().is_empty();
        let length = self.visible_items.borrow().len();

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
            NavigationAction::Up if !is_filtering && initial_i > 0 => initial_i - 1,
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
                let v_items = self.visible_items.borrow();
                let items = self.items.borrow();
                let Some(n) = v_items.iter().position(|item| items[*item].is_match) else {
                    return;
                };
                n
            }
            NavigationAction::End if is_filtering => {
                let v_items = self.visible_items.borrow();
                let items = self.items.borrow();
                let Some(n) = v_items.iter().rposition(|item| items[*item].is_match) else {
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

        let item_index = self.visible_items.borrow()[i];
        let newly_selected_item = self.items.borrow()[item_index].inner.clone();

        self.on_select_fn.borrow_mut()(newly_selected_item);
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

                let i = self.selected_index_true();
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
                *self.rename.borrow_mut() = self.with_selected_item(|item| Some(item.to_string()));
                self.on_request_focus_trap_fn.borrow_mut()(true);
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
                    filter.remove(filter.len().saturating_sub(1));
                });
            }
            _ => {}
        }
    }
}

impl<T: std::fmt::Display> Drop for List<'_, T> {
    fn drop(&mut self) {
        log::trace!("List.drop()");
    }
}

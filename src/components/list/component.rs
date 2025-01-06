use std::cell::{Cell, RefCell};

use crossterm::event::KeyCode;

use crate::{config::Theme, structs::NavigationAction};

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
pub struct ListItem<T> {
    pub inner: T,
    // pub is_visible: bool,
    pub is_match: bool,
}

impl<T> ListItem<T> {
    pub fn new(t: T) -> Self {
        Self {
            inner: t,
            is_match: false,
        }
    }
}

pub struct List<'a, T: 'a>
where
    T: std::fmt::Display,
{
    pub(super) theme: Theme,

    pub(super) items: RefCell<Vec<ListItem<T>>>,
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
    T: std::fmt::Display,
{
    pub fn new(theme: Theme, items: Vec<T>) -> Self {
        let items = items
            .into_iter()
            .map(|item| ListItem {
                inner: item,
                is_match: false,
            })
            .collect();

        Self {
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
        }
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
    }

    /// Sets the list of items, selection and scroll
    pub fn set_items_s(&self, items: Vec<T>, i: usize, o: usize) {
        self.selected_item_index.set(i);
        self.offset.set(o);

        *self.items.borrow_mut() = items.into_iter().map(ListItem::new).collect();
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

    pub fn selected_index(&self) -> usize {
        self.selected_item_index.get()
    }

    pub fn scroll_position(&self) -> usize {
        self.offset.get()
    }

    pub fn set_selected_index(&self, new_i: usize) {
        let length = self.items.borrow().len() as isize;
        let current_i = self.selected_item_index.get() as isize;
        let new_i = new_i as isize;
        let new_i = new_i.min(length - 1).max(0);

        if new_i == current_i {
            return;
        }

        self.selected_item_index.set(new_i as usize);

        let is_down = new_i > current_i;
        let is_up = new_i < current_i;

        let height = self.height.get() as isize;
        let offset = self.offset.get() as isize;
        let padding = self.padding.get() as isize;
        let padding = if is_down { height - padding - 1 } else { padding };

        if (is_up && new_i < offset + padding) || (is_down && new_i > offset + padding) {
            let offset = if new_i > padding {
                (new_i - padding).min(length - height).max(0)
            } else {
                0
            };
            self.offset.set(offset as usize);
        }
    }
}

impl<T: std::fmt::Display> Drop for List<'_, T> {
    fn drop(&mut self) {
        log::trace!("List.drop()");
    }
}

use std::{
    cell::Cell,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};

use crossterm::event::KeyCode;

use crate::config::Theme;

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

    pub(super) items: Mutex<Vec<ListItem<T>>>,
    pub(super) selected_item_index: Cell<usize>,

    pub(super) on_select_fn: Mutex<Box<dyn Fn(T) + 'a>>,
    pub(super) on_enter_fn: Mutex<Box<dyn Fn(T) + 'a>>,
    pub(super) on_enter_alt_fn: Mutex<Option<Box<dyn Fn(T) + 'a>>>,
    pub(super) on_reorder_fn: Mutex<Option<Box<dyn Fn(usize, usize) + 'a>>>,
    pub(super) on_insert_fn: Mutex<Option<Box<dyn Fn() + 'a>>>,
    pub(super) on_delete_fn: Mutex<Option<Box<dyn Fn(T, usize) + 'a>>>,
    pub(super) on_rename_fn: Mutex<Option<Box<dyn Fn(String) + 'a>>>,
    pub(super) on_request_focus_trap_fn: Mutex<Box<dyn Fn(bool) + 'a>>,
    pub(super) find_next_item_by_fn: Mutex<Option<Box<dyn Fn(&[&T], usize, Direction) -> Option<usize> + 'a>>>,

    pub(super) auto_select_next: AtomicBool,

    pub(super) offset: Cell<usize>,
    pub(super) height: Cell<usize>,
    pub(super) line_style: Mutex<Option<Box<dyn Fn(&T) -> Option<ratatui::style::Style> + 'a>>>,
    pub(super) is_focused: AtomicBool,

    pub(super) filter: Mutex<String>,
    pub(super) rename: Mutex<Option<String>>,

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

            on_select_fn: Mutex::new(Box::new(|_| {}) as _),
            on_enter_fn: Mutex::new(Box::new(|_| {}) as _),
            on_enter_alt_fn: Mutex::new(None),
            on_reorder_fn: Mutex::new(None),
            on_insert_fn: Mutex::new(None),
            on_delete_fn: Mutex::new(None),
            on_rename_fn: Mutex::new(None),
            on_request_focus_trap_fn: Mutex::new(Box::new(|_| {}) as _),
            find_next_item_by_fn: Mutex::new(None),

            items: Mutex::new(items),
            selected_item_index: Cell::new(0),

            auto_select_next: AtomicBool::new(true),

            offset: Cell::new(0),
            height: Cell::new(0),
            line_style: Mutex::new(None),
            is_focused: AtomicBool::default(),

            filter: Mutex::new("".to_string()),
            rename: Mutex::new(None),

            padding: Cell::new(5),
            page_size: Cell::new(5),
        }
    }

    pub fn focus(&self) {
        self.is_focused.store(true, Ordering::Release);
    }

    pub fn blur(&self) {
        self.is_focused.store(false, Ordering::Release);
    }

    pub fn set_auto_select_next(&self, v: bool) {
        self.auto_select_next.store(v, Ordering::Release);
    }

    pub fn line_style(&self, cb: impl Fn(&T) -> Option<ratatui::style::Style> + 'a) {
        let mut line_style = self.line_style.lock().unwrap();
        *line_style = Some(Box::new(cb));
    }

    pub fn with_items<R>(&self, cb: impl FnOnce(Vec<&T>) -> R) -> R {
        let items = self.items.try_lock().unwrap();
        let items_inner = (*items).iter().map(|a| &a.inner).collect();
        cb(items_inner)
    }

    pub fn with_selected_item<R>(&self, cb: impl FnOnce(&T) -> R) -> R {
        let items = self.items.lock().unwrap();
        let i = self.selected_item_index.get();
        cb(&items[i].inner)
    }

    pub fn with_selected_item_mut(&self, cb: impl FnOnce(&mut T)) {
        let mut items = self.items.lock().unwrap();
        let i = self.selected_item_index.get();
        cb(&mut items[i].inner);
    }

    /// Triggered by moving the selection around, with the Up and Down arrow keys by default.
    pub fn on_select(&self, cb: impl Fn(T) + 'a) {
        *self.on_select_fn.lock().unwrap() = Box::new(cb);
    }

    /// Triggered, by default, with Enter.
    /// Not the most intuitive name, but it is what it is.
    pub fn on_enter(&self, cb: impl Fn(T) + 'a) {
        *self.on_enter_fn.lock().unwrap() = Box::new(cb);
    }
    /// An alternative "on_enter", triggered, by default, with Alt+Enter.
    /// This is somewhat tightly coupled to functionality required by consumers of this List component.
    pub fn on_enter_alt(&self, cb: impl Fn(T) + 'a) {
        *self.on_enter_alt_fn.lock().unwrap() = Some(Box::new(cb));
    }

    pub fn on_reorder(&self, cb: impl Fn(usize, usize) + 'a) {
        *self.on_reorder_fn.lock().unwrap() = Some(Box::new(cb));
    }

    pub fn on_insert(&self, cb: impl Fn() + 'a) {
        *self.on_insert_fn.lock().unwrap() = Some(Box::new(cb));
    }

    pub fn on_delete(&self, cb: impl Fn(T, usize) + 'a) {
        *self.on_delete_fn.lock().unwrap() = Some(Box::new(cb));
    }

    pub fn on_rename(&self, cb: impl Fn(String) + 'a) {
        *self.on_rename_fn.lock().unwrap() = Some(Box::new(cb));
    }

    pub fn on_request_focus_trap_fn(&self, cb: impl Fn(bool) + 'a) {
        *self.on_request_focus_trap_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn find_next_item_by_fn(&self, cb: impl Fn(&[&T], usize, Direction) -> Option<usize> + 'a) {
        *self.find_next_item_by_fn.lock().unwrap() = Some(Box::new(cb));
    }

    pub fn set_items(&self, items: Vec<T>) {
        self.set_items_s(items, 0, 0);
    }

    pub fn set_items_k(&self, new_items: Vec<T>) {
        let mut items = self.items.lock().unwrap();

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

    pub fn set_items_s(&self, items: Vec<T>, i: usize, o: usize) {
        self.selected_item_index.set(i);
        self.offset.set(o);

        *self.items.lock().unwrap() = items.into_iter().map(ListItem::new).collect();
    }

    #[allow(dead_code)]
    pub fn push_item(&self, item: T) {
        let mut items = self.items.lock().unwrap();
        items.push(ListItem::new(item));
    }

    #[allow(dead_code)]
    pub fn append_items(&self, items_to_append: impl IntoIterator<Item = T>) {
        let mut items = self.items.lock().unwrap();
        let mut items_to_append: Vec<ListItem<T>> = items_to_append.into_iter().map(ListItem::new).collect();

        items.append(&mut items_to_append);
    }

    #[allow(dead_code)]
    pub fn pop_item(&self) -> Option<T> {
        let mut items = self.items.lock().unwrap();

        if items.is_empty() {
            None
        } else {
            Some(items.remove(0).inner)
        }
    }

    pub fn filter_mut(&self, cb: impl FnOnce(&mut String)) {
        let mut filter = self.filter.lock().unwrap();

        cb(&mut filter);

        let mut items = self.items.lock().unwrap();

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
}

impl<T: std::fmt::Display> Drop for List<'_, T> {
    fn drop(&mut self) {
        log::trace!("List.drop()");
    }
}

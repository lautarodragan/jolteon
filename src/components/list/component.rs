use std::{
    sync::{
        atomic::{AtomicUsize, AtomicU8, Ordering},
        Mutex,
    },
};

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    config::Theme,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListItem<T> {
    pub inner: T,
    // pub is_visible: bool,
    pub is_match: bool,
}

pub struct List<'a, T: 'a>
where T: std::fmt::Display
{
    pub(super) theme: Theme,

    pub(super) items: Mutex<Vec<ListItem<T>>>,
    pub(super) selected_item_index: AtomicUsize,

    pub(super) on_select_fn: Mutex<Box<dyn Fn(T, KeyEvent) + 'a>>,
    pub(super) on_enter_fn: Mutex<Box<dyn Fn(T) + 'a>>,
    pub(super) on_reorder_fn: Mutex<Option<Box<dyn Fn(usize, usize) + 'a>>>,
    pub(super) on_insert_fn: Mutex<Option<Box<dyn Fn() + 'a>>>,
    pub(super) on_delete_fn: Mutex<Option<Box<dyn Fn(T, usize) + 'a>>>,
    pub(super) on_rename_fn: Mutex<Option<Box<dyn Fn(String) + 'a>>>,
    pub(super) on_request_focus_trap_fn: Mutex<Box<dyn Fn(bool) + 'a>>,
    pub(super) find_next_item_by_fn: Mutex<Option<Box<dyn Fn(&[&T], usize, Direction) -> Option<usize> + 'a>>>,

    pub(super) offset: AtomicUsize,
    pub(super) height: AtomicUsize,

    pub(super) filter: Mutex<String>,
    pub(super) rename: Mutex<Option<String>>,

    pub(super) padding: AtomicU8,
    pub(super) page_size: AtomicU8,
}

impl<'a, T> List<'a, T>
where T: std::fmt::Display
{
    pub fn new(theme: Theme, items: Vec<T>) -> Self {
        let items = items.into_iter().map(|item| ListItem {
            inner: item,
            is_match: false,
        }).collect();

        Self {
            theme,

            on_select_fn: Mutex::new(Box::new(|_, _| {}) as _),
            on_enter_fn: Mutex::new(Box::new(|_| {}) as _),
            on_reorder_fn: Mutex::new(None),
            on_insert_fn: Mutex::new(None),
            on_delete_fn: Mutex::new(None),
            on_rename_fn: Mutex::new(None),
            on_request_focus_trap_fn: Mutex::new(Box::new(|_| {}) as _),
            find_next_item_by_fn: Mutex::new(None),

            items: Mutex::new(items),
            selected_item_index: AtomicUsize::new(0),

            offset: AtomicUsize::new(0),
            height: AtomicUsize::new(0),

            filter: Mutex::new("".to_string()),
            rename: Mutex::new(None),

            padding: AtomicU8::new(5),
            page_size: AtomicU8::new(5),
        }
    }

    pub fn with_items<R>(&self, cb: impl FnOnce(Vec<&T>) -> R) -> R {
        let items = self.items.try_lock().unwrap();
        let items_inner = (*items).iter().map(|a| &a.inner).collect();
        cb(items_inner)
    }

    pub fn with_selected_item<R>(&self, cb: impl FnOnce(&T) -> R) -> R {
        let items = self.items.lock().unwrap();
        let i = self.selected_item_index.load(Ordering::Acquire);
        cb(&items[i].inner)
    }

    pub fn with_selected_item_mut(&self, cb: impl FnOnce(&mut T)) {
        let mut items = self.items.lock().unwrap();
        let i = self.selected_item_index.load(Ordering::Acquire);
        cb(&mut items[i].inner);
    }

    pub fn on_select(&self, cb: impl Fn(T, KeyEvent) + 'a) {
        *self.on_select_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_enter(&self, cb: impl Fn(T) + 'a) {
        *self.on_enter_fn.lock().unwrap() = Box::new(cb);
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
        self.selected_item_index.store(0, Ordering::SeqCst);
        self.offset.store(0, Ordering::SeqCst);
        *self.items.lock().unwrap() = items.into_iter().map(|item| ListItem {
            inner: item,
            is_match: false,
        }).collect();
    }

    pub fn push_item(&self, item: T) {
        let mut items = self.items.lock().unwrap();
        items.push(ListItem {
            inner: item,
            is_match: false,
        });
    }

    pub fn filter_mut(&self, cb: impl FnOnce(&mut String)) {
        let mut filter = self.filter.lock().unwrap();

        cb(&mut *filter);

        let mut items = self.items.lock().unwrap();

        if items.len() < 2 {
            return;
        }

        for item in items.iter_mut() {
            if filter.is_empty() {
                item.is_match = false;
            } else {
                item.is_match = item.inner.to_string().to_lowercase().contains(filter.to_lowercase().as_str());
            }
        }

        let selected_item_index = self.selected_item_index.load(Ordering::Acquire);
        if !items[selected_item_index].is_match {
            if let Some(i) = items.iter().skip(selected_item_index).position(|item| item.is_match) {
                let i = i + selected_item_index;
                self.selected_item_index.store(i, Ordering::Release);
            } else if let Some(i) = items.iter().position(|item| item.is_match) {
                self.selected_item_index.store(i, Ordering::Release);
            }
        }
    }

}

impl<T: std::fmt::Display> Drop for List<'_, T> {
    fn drop(&mut self) {
        log::trace!("List.drop()");
    }
}

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};

use crossterm::event::KeyEvent;

use crate::{
    config::Theme,
};

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

    pub(super) on_select_fn: Mutex<Box<dyn FnMut(T, KeyEvent) + 'a>>,
    pub(super) on_enter_fn: Mutex<Box<dyn FnMut(T) + 'a>>,
    pub(super) on_reorder_fn: Mutex<Box<dyn FnMut(usize, usize) + 'a>>,
    pub(super) on_delete_fn: Mutex<Box<dyn FnMut(T, usize) + 'a>>,
    pub(super) on_request_focus_trap_fn: Mutex<Box<dyn FnMut() + 'a>>,

    pub(super) offset: AtomicUsize,
    pub(super) height: AtomicUsize,

    pub(super) filter: Mutex<String>,
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
            on_reorder_fn: Mutex::new(Box::new(|_, _| {}) as _),
            on_delete_fn: Mutex::new(Box::new(|_, _| {}) as _),
            on_request_focus_trap_fn: Mutex::new(Box::new(|| {}) as _),

            items: Mutex::new(items),
            selected_item_index: AtomicUsize::new(0),

            offset: AtomicUsize::new(0),
            height: AtomicUsize::new(0),

            filter: Mutex::new("".to_string()),
        }
    }

    pub fn with_items<R>(&self, cb: impl FnOnce(Vec<&T>) -> R) -> R {
        let items = self.items.lock().unwrap();
        let items_inner = (*items).iter().map(|a| &a.inner).collect();
        cb(items_inner)
    }

    pub fn with_selected_item_mut(&self, cb: impl FnOnce(&mut T)) {
        let mut items = self.items.lock().unwrap();
        let i = self.selected_item_index.load(Ordering::Acquire);
        cb(&mut items[i].inner);
    }

    pub fn on_select(&self, cb: impl FnMut(T, KeyEvent) + 'a) {
        *self.on_select_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_enter(&self, cb: impl FnMut(T) + 'a) {
        *self.on_enter_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_reorder(&self, cb: impl FnMut(usize, usize) + 'a) {
        *self.on_reorder_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_delete(&self, cb: impl FnMut(T, usize) + 'a) {
        *self.on_delete_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_request_focus_trap_fn(&self, cb: impl FnMut() + 'a) {
        *self.on_request_focus_trap_fn.lock().unwrap() = Box::new(cb);
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

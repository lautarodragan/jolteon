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

pub struct List<'a, T: 'a> {
    pub(super) theme: Theme,

    pub(super) items: Mutex<Vec<T>>,
    pub(super) selected_item_index: AtomicUsize,

    pub(super) on_select_fn: Mutex<Box<dyn FnMut(T, KeyEvent) + 'a>>,
    pub(super) on_reorder_fn: Mutex<Box<dyn FnMut(usize, usize) + 'a>>,
    pub(super) on_delete_fn: Mutex<Box<dyn FnMut(T, usize) + 'a>>,

    pub(super) offset: AtomicUsize,
    pub(super) height: AtomicUsize,
}

impl<'a, T> List<'a, T> {
    pub fn new(theme: Theme, items: Vec<T>) -> Self {
        Self {
            theme,

            on_select_fn: Mutex::new(Box::new(|_, _| {}) as _),
            on_reorder_fn: Mutex::new(Box::new(|_, _| {}) as _),
            on_delete_fn: Mutex::new(Box::new(|_, _| {}) as _),

            items: Mutex::new(items),
            selected_item_index: AtomicUsize::new(0),

            offset: AtomicUsize::new(0),
            height: AtomicUsize::new(0),
        }
    }

    pub fn on_select(&self, cb: impl FnMut(T, KeyEvent) + 'a) {
        *self.on_select_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_reorder(&self, cb: impl FnMut(usize, usize) + 'a) {
        *self.on_reorder_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn on_delete(&self, cb: impl FnMut(T, usize) + 'a) {
        *self.on_delete_fn.lock().unwrap() = Box::new(cb);
    }

    pub fn set_items(&self, items: Vec<T>) {
        self.selected_item_index.store(0, Ordering::SeqCst);
        self.offset.store(0, Ordering::SeqCst);
        *self.items.lock().unwrap() = items;
    }
}

impl<T> Drop for List<'_, T> {
    fn drop(&mut self) {
        log::trace!("List.drop()");
    }
}
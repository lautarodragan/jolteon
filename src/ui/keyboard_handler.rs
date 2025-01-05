use std::{cell::RefCell, rc::Rc};

use crossterm::event::KeyEvent;
use ratatui::widgets::WidgetRef;

use crate::structs::{Action, OnAction, OnActionMut};

pub trait KeyboardHandlerRef<'a>: 'a {
    fn on_key(&self, key: KeyEvent) {

    }
}
pub trait KeyboardHandlerMut<'a>: 'a {
    fn on_key(&mut self, key: KeyEvent) {

    }
}

// impl<T> KeyboardHandlerRef<'_> for T {
//
// }

pub trait ComponentRef<'a>: KeyboardHandlerRef<'a> + WidgetRef + OnAction {}
pub trait ComponentMut<'a>: KeyboardHandlerMut<'a> + WidgetRef + OnActionMut {}

impl<'a, T: KeyboardHandlerRef<'a> + OnAction + WidgetRef> ComponentRef<'a> for T {}
impl<'a, T: KeyboardHandlerMut<'a> + OnActionMut + WidgetRef> ComponentMut<'a> for T {}

pub enum Component<'a> {
    Ref(Rc<dyn 'a + ComponentRef<'a>>),
    Mut(Rc<RefCell<dyn 'a + ComponentMut<'a>>>),
}

impl<'a> KeyboardHandlerRef<'a> for Component<'a> {
    fn on_key(&self, key: KeyEvent) {
        match self {
            Component::Ref(ref target) => {
                target.on_key(key);
            }
            Component::Mut(ref target) => {
                let mut target = target.borrow_mut();
                target.on_key(key);
            }
        }
    }
}

impl<'a> OnAction for Component<'a> {
    fn on_action(&self, action: Action) {
        match self {
            Component::Ref(ref target) => {
                target.on_action(action);
            }
            Component::Mut(ref target) => {
                let mut target = target.borrow_mut();
                target.on_action(action);
            }
        }
    }
}

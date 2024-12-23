use std::{cell::RefCell, rc::Rc};

use crossterm::event::KeyEvent;
use ratatui::widgets::WidgetRef;

pub trait KeyboardHandlerRef<'a>: 'a {
    fn on_key(&self, key: KeyEvent);
}
pub trait KeyboardHandlerMut<'a>: 'a {
    fn on_key(&mut self, key: KeyEvent);
}

pub trait ComponentRef<'a>: KeyboardHandlerRef<'a> + WidgetRef {}
pub trait ComponentMut<'a>: KeyboardHandlerMut<'a> + WidgetRef {}

impl<'a, T: KeyboardHandlerRef<'a> + WidgetRef> ComponentRef<'a> for T {}
impl<'a, T: KeyboardHandlerMut<'a> + WidgetRef> ComponentMut<'a> for T {}

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

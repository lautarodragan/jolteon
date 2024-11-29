use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

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
    RefRc(Rc<dyn 'a + ComponentRef<'a>>),
    RefArc(Arc<dyn 'a + ComponentRef<'a>>),
    Mut(Arc<Mutex<dyn 'a + ComponentMut<'a>>>),
}

impl<'a> KeyboardHandlerRef<'a> for Component<'a> {
    fn on_key(&self, key: KeyEvent) {
        match self {
            Component::RefRc(ref target) => {
                target.on_key(key);
            }
            Component::RefArc(ref target) => {
                target.on_key(key);
            }
            Component::Mut(ref target) => {
                target.lock().unwrap().on_key(key);
            }
        }
    }
}

use std::sync::{Arc, Mutex};
use crossterm::event::KeyEvent;

pub trait KeyboardHandlerRef<'a, R = ()>: 'a {
    fn on_key(&self, key: KeyEvent) -> R;
}
pub trait KeyboardHandlerMut<'a>: 'a {
    fn on_key(&mut self, key: KeyEvent);
}

#[derive(Clone)]
pub enum KeyboardHandler<'a> {
    Ref(Arc<dyn 'a + KeyboardHandlerRef<'a>>),
    Mut(Arc<Mutex<dyn 'a + KeyboardHandlerMut<'a>>>),
}

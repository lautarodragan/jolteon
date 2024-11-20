use std::sync::{Arc, Mutex};

use crossterm::event::KeyEvent;
use ratatui::widgets::WidgetRef;

pub trait KeyboardHandlerRef<'a, R = ()>: 'a {
    fn on_key(&self, key: KeyEvent) -> R;
}
pub trait KeyboardHandlerMut<'a>: 'a {
    fn on_key(&mut self, key: KeyEvent);
}

pub trait ComponentRef<'a>: KeyboardHandlerRef<'a> + WidgetRef {}
pub trait ComponentMut<'a>: KeyboardHandlerMut<'a> + WidgetRef {}

impl<'a, T: KeyboardHandlerRef<'a> + WidgetRef> ComponentRef<'a> for T {}
impl<'a, T: KeyboardHandlerMut<'a> + WidgetRef> ComponentMut<'a> for T {}

pub enum Component<'a> {
    Ref(Arc<dyn 'a + ComponentRef<'a>>),
    Mut(Arc<Mutex<dyn 'a + ComponentMut<'a>>>),
}

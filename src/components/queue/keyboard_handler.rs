use crossterm::event::KeyEvent;
use crate::structs::{Action, OnAction};
use crate::ui::KeyboardHandlerRef;

use super::Queue;

impl<'a> KeyboardHandlerRef<'a> for Queue<'a> {
    fn on_key(&self, key: KeyEvent) {
        self.song_list.on_key(key);
    }
}

impl OnAction for Queue<'_> {
    fn on_action(&self, action: Action) {
        todo!()
    }
}

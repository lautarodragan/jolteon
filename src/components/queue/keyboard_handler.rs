use crate::structs::{Action, OnAction, OnActionMut};

use super::Queue;

impl OnActionMut for Queue<'_> {
    fn on_action(&mut self, action: Action) {
        self.song_list.on_action(action);
    }
}

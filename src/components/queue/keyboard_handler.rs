use crate::structs::{Action, OnAction};

use super::Queue;

impl OnAction for Queue<'_> {
    fn on_action(&self, action: Action) {
        self.song_list.on_action(action);
    }
}

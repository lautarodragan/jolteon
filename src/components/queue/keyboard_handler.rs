use super::Queue;
use crate::actions::{Action, OnAction, OnActionMut};

impl OnActionMut for Queue<'_> {
    fn on_action(&mut self, actions: Vec<Action>) {
        self.song_list.on_action(actions);
    }
}

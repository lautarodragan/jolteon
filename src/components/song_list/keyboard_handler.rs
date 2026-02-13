use super::component::SongList;
use crate::actions::{Action, OnAction};

impl<'a> OnAction for SongList<'a> {
    fn on_action(&self, actions: Vec<Action>) {
        self.list.exec_action(actions);
    }
}

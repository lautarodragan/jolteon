use super::component::Soundtracks;
use crate::actions::{Action, OnAction, OnActionMut};

impl OnActionMut for Soundtracks<'_> {
    fn on_action(&mut self, actions: Vec<Action>) {
        self.focus_group.on_action(actions);
    }
}

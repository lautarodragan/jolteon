use crate::actions::{Action, OnAction, OnActionMut};

use super::library::Library;

impl OnActionMut for Library<'_> {
    fn on_action(&mut self, actions: Vec<Action>) {
        // log::trace!(target: "::library.on_action", "{action:?}");
        self.focus_group.on_action(actions);
    }
}

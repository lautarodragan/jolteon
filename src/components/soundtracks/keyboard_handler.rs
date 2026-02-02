use crate::actions::{Action, OnAction, OnActionMut};
use crate::components::soundtracks::Soundtracks;

impl OnActionMut for Soundtracks<'_> {
    fn on_action(&mut self, actions: Vec<Action>) {
        self.focus_group.on_action(actions);
    }
}

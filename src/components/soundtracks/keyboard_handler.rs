use super::component::Soundtracks;
use crate::actions::{Action, NavigationAction, OnAction, OnActionMut};

impl OnActionMut for Soundtracks<'_> {
    fn on_action(&mut self, actions: Vec<Action>) {
        match actions[0] {
            Action::Navigation(NavigationAction::Right) => {
                // TODO: either prioritize self.focus_group.on_action(actions) or respect focus_stolen
                self.focus_group.focus_nth(1);
            }
            Action::Navigation(NavigationAction::Left) => {
                // TODO: either prioritize self.focus_group.on_action(actions) or respect focus_stolen,
                self.focus_group.focus_nth(0);
            }
            _ => {
                self.focus_group.on_action(actions);
            }
        }
    }
}

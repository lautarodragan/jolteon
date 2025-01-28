use std::fmt::Debug;

use crate::actions::{Action, OnAction};

use super::component::Tree;

impl<'a, T> OnAction for Tree<'a, T>
where
    T: 'a + Clone + std::fmt::Display + Debug,
{
    fn on_action(&self, action: Vec<Action>) {
        self.exec_action(action[0]);
    }
}

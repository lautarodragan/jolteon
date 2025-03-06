use std::fmt::Debug;

use super::component::List;
use crate::actions::{Action, OnAction};

impl<'a, T> OnAction for List<'a, T>
where
    T: 'a + Clone + std::fmt::Display + Debug,
{
    fn on_action(&self, actions: Vec<Action>) {
        self.exec_action(actions);
    }
}

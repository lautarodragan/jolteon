use std::fmt::Debug;

use crate::actions::{Action, OnActionMut};

use super::component::Tree;

impl<'a, T> OnActionMut for Tree<'a, T>
where
    T: 'a + Clone + std::fmt::Display + Debug,
{
    fn on_action(&mut self, action: Vec<Action>) {
        self.exec_action(action[0]);
    }
}

use std::fmt::Debug;

use super::component::Tree;
use crate::actions::{Action, OnActionMut};

impl<'a, T> OnActionMut for Tree<'a, T>
where
    T: 'a + Clone + std::fmt::Display + Debug,
{
    fn on_action(&mut self, action: Vec<Action>) {
        self.exec_action(action[0]);
    }
}

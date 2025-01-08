use crate::structs::{Action, OnAction};

use super::component::List;

impl<'a, T> OnAction for List<'a, T>
where
    T: 'a + Clone + std::fmt::Display,
{
    fn on_action(&self, action: Action) {
        self.exec_action(action);
    }
}

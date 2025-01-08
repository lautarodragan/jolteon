use crate::structs::{Action, OnAction};

use super::component::List;

// TODO: OnAction
// impl<'a, T> KeyboardHandlerRef<'a> for List<'a, T>
// where
//     T: 'a + Clone + std::fmt::Display,
// {
//     fn on_key(&self, key: KeyEvent) {
//         match key.code {
//             KeyCode::Char(char) => {
//                 self.filter_mut(|filter| {
//                     filter.push(char);
//                 });
//             }
//         }
//     }
// }

impl<'a, T> OnAction for List<'a, T>
where
    T: 'a + Clone + std::fmt::Display,
{
    fn on_action(&self, action: Action) {
        self.exec_action(action);
    }
}

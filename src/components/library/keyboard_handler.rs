
use crate::{
    structs::{Action, OnAction},
    ui::*,
};
use crate::structs::NavigationAction;
use super::library::Library;

impl<'a> KeyboardHandlerRef<'a> for Library<'a> {

}

impl OnAction for Library<'_> {
    fn on_action(&self, action: Action) {
        log::trace!(target: "::library.on_action", "{action:?}");

        let i = self.focused_component.get();

        match action {
            Action::Navigation(NavigationAction::FocusNext) => {
                self.focused_component.set({
                    if i < self.components.len().saturating_sub(1) {
                        i + 1
                    } else {
                        0
                    }
                });
            }
            _ => {
                if let Some(a) = self.components.get(i) {
                    a.on_action(action);
                }
            }
        }
    }
}

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    ui::{KeyboardHandlerRef},
};
use super::library::{Library};

impl<'a> KeyboardHandlerRef<'a> for Library<'a> {

    fn on_key(&self, key: KeyEvent) {
        log::trace!(target: "::library.on_key", "{:?}", key);

        let i = self.focused_component.get();

        match key.code {
            KeyCode::Tab => {
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
                    a.on_key(key);
                }
            }
        }
    }
}

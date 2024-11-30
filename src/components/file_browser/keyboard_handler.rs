use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::KeyboardHandlerRef;

use super::FileBrowser;

impl<'a> KeyboardHandlerRef<'a> for FileBrowser<'a> {
    fn on_key(&self, key: KeyEvent) {
        match key.code {
            KeyCode::Backspace => {
                self.navigate_up();
            }
            _ => self.parents_list.on_key(key),
        }
    }
}

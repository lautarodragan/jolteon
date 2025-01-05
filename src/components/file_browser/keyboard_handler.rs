use std::sync::atomic::Ordering;

use crossterm::event::KeyEvent;

use crate::ui::KeyboardHandlerRef;

use super::FileBrowser;

impl<'a> KeyboardHandlerRef<'a> for FileBrowser<'a> {
    fn on_key(&self, key: KeyEvent) {
        // log::debug!("keyboard handler for file browser: {:?}", key);
        let focus = self.focus.load(Ordering::Acquire);
        if focus == 0 {
            self.parents_list.on_key(key)
        } else if focus == 1 {
            self.children_list.on_key(key);
        } else if focus == 2 {
            self.file_meta.on_key(key);
        }
    }
}

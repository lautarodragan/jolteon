use std::sync::atomic::Ordering;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::ui::KeyboardHandlerRef;

use super::FileBrowser;

impl<'a> KeyboardHandlerRef<'a> for FileBrowser<'a> {
    fn on_key(&self, key: KeyEvent) {
        match key.code {
            KeyCode::Backspace => {
                self.navigate_up();
            }
            KeyCode::Tab | KeyCode::BackTab => {
                let mut focus = self.focus.load(Ordering::Acquire);

                if key.modifiers == KeyModifiers::NONE || key.code == KeyCode::Tab {
                    if focus > 1 {
                        focus = 0
                    } else {
                        focus += 1;
                    }
                } else if key.modifiers == KeyModifiers::SHIFT || key.code == KeyCode::BackTab {
                    if focus == 0 {
                        focus = 2
                    } else {
                        focus -= 1;
                    }
                }

                self.focus.store(focus, Ordering::Release);

                if focus == 0 {
                    self.parents_list.focus();
                    self.children_list.blur();
                    self.file_meta.blur();
                } else if focus == 1 {
                    self.parents_list.blur();
                    self.children_list.focus();
                    self.file_meta.blur();
                } else if focus == 2 {
                    self.parents_list.blur();
                    self.children_list.blur();
                    self.file_meta.focus();
                }
            }
            _ => {
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
    }
}

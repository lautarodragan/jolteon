use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::KeyboardHandlerRef;

use super::queue::Queue;

impl KeyboardHandlerRef<'_> for Queue {
    fn on_key(&self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                if let Some(song) = self.selected_song() {
                    // self.play_song(song);
                    log::error!(" queue play song {:?}", song);
                };
            }
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
            KeyCode::Delete => self.remove_selected(),
            _ => {}
        };
    }
}

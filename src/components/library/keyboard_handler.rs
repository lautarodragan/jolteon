use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    ui::KeyboardHandlerRef,
};

use super::library::{Library, LibraryScreenElement};

impl<'a> KeyboardHandlerRef<'a> for Library<'a> {

    fn on_key(&self, key: KeyEvent) {
        log::trace!(target: "::library.on_key", "start {:?}", key);

        let focused_element = self.focused_element();

        match key.code {
            KeyCode::Tab => {
                self.set_focused_element(match focused_element {
                    LibraryScreenElement::AlbumTree => LibraryScreenElement::SongList,
                    LibraryScreenElement::SongList => LibraryScreenElement::AlbumTree,
                });
            }
            _ if focused_element == LibraryScreenElement::AlbumTree => {
                self.album_tree.on_key(key)
            },
            _ if focused_element == LibraryScreenElement::SongList  => {
                self.song_list.on_key(key)
            },
            _ => (),
        }
    }
}

use crate::structs::{Action, ListAction, NavigationAction, OnAction};

use super::library::Library;

impl OnAction for Library<'_> {
    fn on_action(&self, action: Action) {
        // log::trace!(target: "::library.on_action", "{action:?}");

        match action {
            Action::Navigation(NavigationAction::FocusNext) => {
                let focused_component = self.focused_component.get();
                self.focused_component.set({
                    if focused_component < self.components.len().saturating_sub(1) {
                        focused_component + 1
                    } else {
                        0
                    }
                });
            }
            Action::ListAction(ListAction::OpenClose) => {
                // TODO: implement a concept of "children" directly into the List component
                let (artist_index, artist_album_count) = self.album_tree.with_items(|items| {
                    let artist_index = {
                        let mut artist_index = self.album_tree.selected_index_true();

                        loop {
                            if items[artist_index].is_artist() {
                                break artist_index;
                            } else {
                                artist_index -= 1;
                            }
                        }
                    };

                    let artist_album_count = {
                        let mut artist_album_count = 1;
                        loop {
                            let item_index = artist_index + artist_album_count;

                            if item_index >= items.len() - 1 {
                                break artist_album_count;
                            }

                            if items[item_index].is_album() {
                                artist_album_count += 1;
                            } else {
                                break artist_album_count - 1;
                            }
                        }
                    };

                    (artist_index, artist_album_count)
                });

                let is_open = !self.album_tree.is_open(artist_index);
                self.album_tree.set_is_open(artist_index, is_open);

                for i in artist_index + 1..artist_index + 1 + artist_album_count {
                    self.album_tree.set_is_visible(i, is_open);
                }

                if !is_open {
                    self.album_tree.set_selected_index_true(artist_index);
                }
            }
            _ => {
                if let Some(a) = self.components.get(self.focused_component.get()) {
                    a.on_action(action);
                }
            }
        }
    }
}

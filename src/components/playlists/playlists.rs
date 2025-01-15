use std::rc::Rc;

use chrono::Local;

use crate::{
    components::{FocusGroup, List},
    config::Theme,
    structs::{Playlist, Song},
};

pub struct Playlists<'a> {
    pub(super) theme: Theme,
    pub(super) playlist_list: Rc<List<'a, Playlist>>,
    pub(super) deleted_playlist_list: Rc<List<'a, Playlist>>,
    pub(super) song_list: Rc<List<'a, Song>>,
    pub(super) focus_group: FocusGroup<'a>,
    pub(super) show_deleted_playlists: bool,
}

impl<'a> Playlists<'a> {
    pub fn new(theme: Theme) -> Self {
        let playlists_file = crate::files::Playlists::from_file();

        let song_list = Rc::new(List::new(
            theme,
            playlists_file
                .playlists
                .first()
                .map(|pl| pl.songs.clone())
                .unwrap_or_default(),
        ));
        let mut playlist_list = List::new(theme, playlists_file.playlists);
        let deleted_playlist_list = Rc::new(List::new(theme, playlists_file.deleted));

        playlist_list.on_select({
            let song_list = song_list.clone();
            move |pl| {
                song_list.set_items(pl.songs.clone());
            }
        });

        let playlist_list = Rc::new(playlist_list);
        playlist_list.on_rename({
            let playlist_list = playlist_list.clone();
            let deleted_playlist_list = deleted_playlist_list.clone();

            move |v| {
                playlist_list.with_selected_item_mut(|i| {
                    i.name = v;
                });
                save(&playlist_list, &deleted_playlist_list);
            }
        });

        playlist_list.on_insert({
            let playlist_list = playlist_list.clone();
            let deleted_playlist_list = deleted_playlist_list.clone();
            move || {
                let playlist = Playlist::new(format!(
                    "New playlist created at {}",
                    Local::now().format("%A %-l:%M:%S%P")
                ));
                playlist_list.push_item(playlist);
                save(&playlist_list, &deleted_playlist_list);
            }
        });

        playlist_list.on_delete({
            let playlist_list = playlist_list.clone();
            let deleted_playlist_list = deleted_playlist_list.clone();
            move |pl, _| {
                deleted_playlist_list.push_item(pl);
                save(&playlist_list, &deleted_playlist_list);
            }
        });

        song_list.on_reorder({
            let playlist_list = playlist_list.clone();
            let deleted_playlist_list = deleted_playlist_list.clone();

            move |a, b| {
                log::debug!(target: "::playlists", "on_reorder {a} {b}");
                playlist_list.with_selected_item_mut(move |pl| {
                    pl.songs.swap(a, b);
                });
                save(&playlist_list, &deleted_playlist_list);
            }
        });
        song_list.on_delete({
            let playlist_list = playlist_list.clone();
            let deleted_playlist_list = deleted_playlist_list.clone();

            move |song, index| {
                log::trace!(target: "::playlists", "on_delete {index} {}", song.title);
                playlist_list.with_selected_item_mut(move |pl| {
                    pl.songs.remove(index);
                });
                save(&playlist_list, &deleted_playlist_list);
            }
        });

        let focus_group = FocusGroup::new(vec![playlist_list.clone(), song_list.clone()]);

        Self {
            // playlists: Mutex::new(vec![
            //     Playlist::new("My first Jolteon playlist".to_string()),
            //     Playlist::new("Ctrl+N to create new ones".to_string()),
            //     Playlist::new("Alt+N to rename".to_string()),
            // ]),
            theme,
            playlist_list,
            deleted_playlist_list,
            song_list,
            focus_group,
            show_deleted_playlists: false,
        }
    }

    pub fn on_enter_song(&self, cb: impl Fn(Song) + 'a) {
        self.song_list.on_enter(cb);
    }

    pub fn on_enter_song_alt(&self, cb: impl Fn(Song) + 'a) {
        self.song_list.on_enter_alt(cb);
    }

    pub fn on_enter_playlist(&self, cb: impl Fn(Playlist) + 'a) {
        self.playlist_list.on_enter(cb);
    }

    pub fn on_request_focus_trap_fn(&self, cb: impl Fn(bool) + 'a) {
        self.playlist_list.on_request_focus_trap_fn(cb);
    }

    pub fn selected_playlist_mut(&self, f: impl FnOnce(&mut Playlist)) {
        self.playlist_list.with_selected_item_mut(f);
        save(&self.playlist_list, &self.deleted_playlist_list);
    }

    pub fn add_songs(&self, songs: &mut Vec<Song>) {
        self.selected_playlist_mut(move |pl| {
            pl.songs.append(songs);
        });
    }
}

impl Drop for Playlists<'_> {
    fn drop(&mut self) {
        log::trace!("Playlists.drop()");
    }
}

fn clone_vec(v: Vec<&Playlist>) -> Vec<Playlist> {
    v.into_iter().cloned().collect()
}

fn save(playlist_list: &List<Playlist>, deleted_playlist_list: &List<Playlist>) {
    let f = crate::files::Playlists {
        playlists: playlist_list.with_items(clone_vec),
        deleted: deleted_playlist_list.with_items(clone_vec),
    };
    f.save();
}

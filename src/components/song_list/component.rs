use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;

use crate::{
    components::List,
    structs::{Direction, Song},
    theme::Theme,
    ui::Focusable,
};

pub struct SongList<'a> {
    pub(super) list: List<'a, Song>,
}

#[serde_inline_default::serde_inline_default]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, DefaultFromSerde)]
pub struct SongListViewOptions {
    pub artist: bool,
    pub album: bool,
    pub year: bool,
    #[serde_inline_default(true)]
    pub track_number: bool,
    #[serde_inline_default(true)]
    pub title: bool,
    pub show_missing_album: bool,
    pub show_missing_year: bool,
}

impl SongListViewOptions {
    pub fn short() -> Self {
        Self {
            artist: false,
            album: false,
            year: false,
            track_number: true,
            title: true,
            show_missing_album: false,
            show_missing_year: false,
        }
    }

    pub fn long() -> Self {
        Self {
            artist: false,
            album: true,
            year: true,
            track_number: true,
            title: true,
            show_missing_album: true,
            show_missing_year: true,
        }
    }
}

impl<'a> SongList<'a> {
    pub fn new(theme: Theme, songs: Vec<Song>) -> Self {
        let list = List::new(theme, songs);
        let mut song_list = Self { list };

        song_list.configure();
        song_list
    }

    fn configure(&mut self) {
        self.set_view_options_long();

        self.list.find_next_item_by_fn({
            |songs, i, direction| {
                let Some(ref selected_album) = songs[i].album else {
                    log::warn!("no selected song album");
                    return None;
                };

                if direction == Direction::Forwards {
                    songs
                        .iter()
                        .skip(i)
                        .position(|s| s.album.as_ref().is_some_and(|a| a != selected_album))
                        .map(|ns| ns.saturating_add(i))
                } else {
                    songs
                        .iter()
                        .take(i)
                        .rposition(|s| s.album.as_ref().is_some_and(|a| a != selected_album))
                        .and_then(|ns| songs.get(ns))
                        .and_then(|s| s.album.as_ref())
                        .and_then(|next_song_album| {
                            songs
                                .iter()
                                .position(|song| song.album.as_ref().is_some_and(|a| a.as_str() == next_song_album))
                        })
                }
            }
        });

        self.list
            .on_select(move |song| log::debug!("SongList: selected song {song:#?}"));
    }

    pub fn set_view_options_short(&self) {
        self.set_view_options(SongListViewOptions::short());
    }

    pub fn set_view_options_long(&self) {
        self.set_view_options(SongListViewOptions::long());
    }

    pub fn set_view_options(&self, parts: SongListViewOptions) {
        self.list.render_fn(move |song| render_song_with_parts(song, parts));
    }

    pub fn set_items(&self, songs: Vec<Song>) {
        self.list.set_items(songs);
    }

    pub fn on_confirm(&self, cb: impl Fn(Song) + 'a) {
        self.list.on_confirm(cb);
    }

    pub fn on_confirm_alt(&self, cb: impl Fn(Song) + 'a) {
        self.list.on_confirm_alt(cb);
    }

    pub fn on_delete(&self, cb: impl Fn(Song, usize) + 'a) {
        self.list.on_delete(cb);
    }

    pub fn on_reorder(&self, cb: impl Fn(usize, usize) + 'a) {
        self.list.on_reorder(cb);
    }
}

fn render_song_with_parts(song: &Song, parts: SongListViewOptions) -> String {
    let mut pieces = Vec::new();

    if parts.artist
        && let Some(ref artist) = song.artist
    {
        pieces.push(artist.clone());
    }

    if parts.year {
        if let Some(year) = song.year {
            pieces.push(year.to_string());
        } else if parts.show_missing_year {
            pieces.push("(no year)".to_string());
        }
    }

    if parts.album {
        if let Some(ref album) = song.album {
            pieces.push(album.clone());
        } else if parts.show_missing_album {
            pieces.push("(no_album)".to_string());
        }
    }

    if parts.track_number {
        pieces.push(song.track.unwrap_or(0).to_string());
    }

    if parts.title {
        pieces.push(song.title.clone());
    }

    pieces.join(" - ")
}

impl Focusable for SongList<'_> {
    fn set_is_focused(&self, v: bool) {
        self.list.set_is_focused(v);
    }

    fn is_focused(&self) -> bool {
        self.list.is_focused()
    }
}

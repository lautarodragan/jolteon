use serde::{Deserialize, Serialize};

use crate::structs::Song;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlaylistView {
    pub artist: bool,
    pub album: bool,
    pub year: bool,
    pub track_number: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>,
    pub is_deleted: bool,
    #[serde(default)]
    pub playlist_view: PlaylistView,
}

impl Playlist {
    pub fn new(name: String) -> Self {
        Self {
            name,
            songs: vec![],
            is_deleted: false,
            playlist_view: PlaylistView::default(),
        }
    }
}

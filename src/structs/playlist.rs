use serde::{Deserialize, Serialize};

use crate::{components::SongListViewOptions, structs::Song};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>,
    pub is_deleted: bool,
    #[serde(default)]
    pub view_options: SongListViewOptions,
}

impl Playlist {
    pub fn new(name: String) -> Self {
        Self {
            name,
            songs: vec![],
            is_deleted: false,
            view_options: SongListViewOptions::default(),
        }
    }
}

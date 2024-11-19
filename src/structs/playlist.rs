use serde::{Deserialize, Serialize};

use crate::structs::Song;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub songs: Vec<Song>,
    pub is_deleted: bool,
}

impl Playlist {
    pub fn new(name: String) -> Self {
        Self {
            name,
            songs: vec![],
            is_deleted: false,
        }
    }
}

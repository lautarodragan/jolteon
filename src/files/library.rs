use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    structs::Song,
    toml::{read_toml_file_or_default, write_toml_file, TomlFileError},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Library {
    pub songs: Vec<Song>,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            songs: vec![],
        }
    }
}

impl Library {
    pub fn from_file() -> Self {
        read_toml_file_or_default("library")
    }

    pub fn from_hash_map(songs_by_artist: &HashMap<String, Vec<Song>>) -> Self {
        let mut songs = vec![];

        for (_artist, artist_songs) in songs_by_artist {
            for song in artist_songs {
                songs.push(song.clone());
            }
        }

        Self {
            songs,
        }
    }

    pub fn to_file(&self) -> Result<(), TomlFileError> {
        write_toml_file("library", self)
    }

    pub fn save(&self) {
        if let Err(err) = self.to_file() {
            log::error!("Could not save library! {:#?}", err);
        }
    }

    pub fn save_hash_map(songs_by_artist: &HashMap<String, Vec<Song>>) {
        let library = Self::from_hash_map(&*songs_by_artist);
        library.save();
    }
}

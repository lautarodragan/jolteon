use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{structs::Song, toml::read_toml_file_or_default};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Library {
    pub songs_by_artist: HashMap<String, Vec<Song>>,
}

impl Library {
    pub fn from_file() -> Self {
        read_toml_file_or_default("library")
    }

    // pub fn to_file(&self) -> Result<(), TomlFileError> {
    //     write_toml_file("library", self)
    // }
    //
    // pub fn save(&self) {
    //     if let Err(err) = self.to_file() {
    //         log::error!("Could not save library! {:#?}", err);
    //     }
    // }
    //
    // pub fn add_songs(&mut self, songs_to_add: Vec<Song>) {
    //     for song in songs_to_add {
    //         let Some(ref artist) = song.artist else {
    //             log::error!("Library.add_song() -> no artist! {:?}", song);
    //             continue;
    //         };
    //
    //         let artist_songs = self.songs_by_artist.entry(artist.clone()).or_default();
    //         if let Err(i) = artist_songs.binary_search(&song) {
    //             artist_songs.insert(i, song);
    //         }
    //     }
    //
    //     self.save();
    // }
    //
    // pub fn remove_artist(&mut self, artist: &str) {
    //     self.songs_by_artist.remove(artist);
    //     self.save();
    // }
    //
    // pub fn remove_album(&mut self, artist: &str, album: &str) {
    //     let Some(artist_songs) = self.songs_by_artist.get_mut(artist) else {
    //         log::error!(target: "::library.album_tree.on_delete", "Tried to delete artist's songs, but the artist has no songs.");
    //         return;
    //     };
    //     artist_songs.retain(|s| s.album.as_ref().is_some_and(|a| *a != album));
    //     self.save();
    // }
}

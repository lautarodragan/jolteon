use serde::{Deserialize, Serialize};

use crate::{
    structs::Playlist,
    toml::{read_toml_file_or_default, write_toml_file, TomlFileError},
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Playlists {
    pub playlists: Vec<Playlist>,
    pub deleted: Vec<Playlist>,
}

impl Playlists {
    pub fn from_file() -> Self {
        read_toml_file_or_default("playlists")
    }

    pub fn to_file(&self) -> Result<(), TomlFileError> {
        write_toml_file("playlists", self)
    }

    pub fn save(&self) {
        if let Err(err) = self.to_file() {
            log::error!("Could not save playlists! {:#?}", err);
        }
    }
}

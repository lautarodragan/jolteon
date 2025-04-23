use serde::{Deserialize, Serialize};

use crate::{
    structs::Song,
    toml::{TomlFileError, read_toml_file_or_default, write_toml_file},
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct State {
    pub last_visited_path: Option<String>,
    #[serde(default)]
    pub queue_items: Vec<Song>,
}

impl State {
    pub fn from_file() -> Self {
        read_toml_file_or_default("state")
    }

    pub fn to_file(&self) -> Result<(), TomlFileError> {
        write_toml_file("state", self)
    }
}

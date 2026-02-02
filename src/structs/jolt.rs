use std::{fs::read_to_string, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub enum WorkType {
    #[default]
    Album,
    Soundtrack,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Jolt {
    #[serde(skip)]
    pub path: PathBuf,
    pub work_type: WorkType,
    pub artist: Option<String>,
    /// If this entity is a Soundtrack, soundtrack_subject is the name of the related work of art.
    /// For example, the album `Back To The Future (Music From The Motion Picture Soundtrack)`
    /// should have this set to `Back To The Future`.
    pub soundtrack_subject: Option<String>,
    pub album: Option<String>,
    pub disc_number: Option<u32>,
    pub year: Option<u32>,
}

impl Jolt {
    pub fn from_path(path: PathBuf) -> Result<Self, JoltCreationError> {
        let jolt = read_to_string(path.as_path())?;
        let jolt: Jolt = toml::from_str(&jolt)?;

        Ok(Jolt { path, ..jolt })
    }
}

#[derive(Debug)]
pub enum JoltCreationError {
    Toml(toml::de::Error),
    Fs(std::io::Error),
}

impl From<toml::de::Error> for JoltCreationError {
    fn from(value: toml::de::Error) -> Self {
        JoltCreationError::Toml(value)
    }
}

impl From<std::io::Error> for JoltCreationError {
    fn from(value: std::io::Error) -> Self {
        JoltCreationError::Fs(value)
    }
}

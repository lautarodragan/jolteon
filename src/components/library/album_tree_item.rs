use std::{
    fmt::{Display, Formatter},
};

use crate::structs::Song;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Album {
    pub artist: String,
    pub name: String,
    pub year: u32,
    pub songs: Vec<Song>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlbumTreeItem {
    Artist(String),
    Album(Album),
}

impl AlbumTreeItem {
    pub fn is_artist(&self) -> bool {
        matches!(self, Self::Artist(_))
    }

    pub fn is_album(&self) -> bool {
        matches!(self, Self::Album(_))
    }
}

impl Display for AlbumTreeItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlbumTreeItem::Artist(s) => write!(f, "{s}"),
            AlbumTreeItem::Album(album) => write!(f, "  {} - {}", album.year, album.name),
        }
    }
}

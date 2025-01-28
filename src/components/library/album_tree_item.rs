use std::{
    fmt::{Display, Formatter},
};
use std::cmp::Ordering;
use crate::structs::Song;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Artist {
    pub name: String,
    pub albums: Vec<Album>,
}

// impl Ord for Artist {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.name.cmp(&other.name)
//     }
// }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Album {
    pub artist: String,
    pub name: String,
    pub year: u32,
    pub songs: Vec<Song>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlbumTreeItem {
    Artist(Artist),
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
            AlbumTreeItem::Artist(s) => write!(f, "{}", s.name),
            AlbumTreeItem::Album(album) => write!(f, "  {} - {}", album.year, album.name),
        }
    }
}

use std::fmt::{Display, Formatter};

use crate::structs::Song;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Artist {
    pub name: String,
    pub albums: Vec<Album>,
}

impl Artist {
    pub fn songs(&self) -> Vec<Song> {
        self.albums.iter().flat_map(|album| album.songs.clone()).collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Album {
    pub artist: String,
    pub name: String,
    pub year: Option<u32>,
    pub songs: Vec<Song>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlbumTreeItem {
    Artist(Artist),
    Album(Album),
}

impl AlbumTreeItem {
    pub fn songs(&self) -> Vec<Song> {
        match self {
            AlbumTreeItem::Artist(_) => vec![],
            AlbumTreeItem::Album(a) => a.songs.clone(),
        }
    }
}

impl Display for AlbumTreeItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlbumTreeItem::Artist(s) => write!(f, "{}", s.name),
            AlbumTreeItem::Album(album) => write!(f, "  {} - {}", album.year.unwrap_or_default(), album.name),
        }
    }
}

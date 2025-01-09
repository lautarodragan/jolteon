use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlbumTreeItem {
    Artist(String),
    Album(String, String),
}

impl AlbumTreeItem {
    pub fn is_artist(&self) -> bool {
        matches!(self, Self::Artist(_))
    }

    pub fn is_album(&self) -> bool {
        matches!(self, Self::Album(_, _))
    }
}

impl Ord for AlbumTreeItem {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (AlbumTreeItem::Artist(a), AlbumTreeItem::Artist(b)) => a.cmp(b),
            (AlbumTreeItem::Artist(_), AlbumTreeItem::Album(_, _)) => Ordering::Greater,
            (AlbumTreeItem::Album(_, _), AlbumTreeItem::Artist(_)) => Ordering::Less,
            (AlbumTreeItem::Album(_, ref a), AlbumTreeItem::Album(_, ref b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for AlbumTreeItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for AlbumTreeItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlbumTreeItem::Artist(s) => write!(f, "{s}"),
            AlbumTreeItem::Album(_, s) => write!(f, "  {s}"),
        }
    }
}

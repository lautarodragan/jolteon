use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlbumTreeItem {
    Artist(String),
    Album(String, String),
}

impl Ord for AlbumTreeItem {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (AlbumTreeItem::Artist(a), AlbumTreeItem::Artist(b)) => {
                a.cmp(b)
            }
            (AlbumTreeItem::Artist(_), AlbumTreeItem::Album(_, _)) => {
                Ordering::Greater
            }
            (AlbumTreeItem::Album(_, _), AlbumTreeItem::Artist(_)) => {
                Ordering::Less
            }
            (AlbumTreeItem::Album(_, ref a), AlbumTreeItem::Album(_, ref b)) => {
                a.cmp(b)
            }
        }
    }
}

impl PartialOrd for AlbumTreeItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl AlbumTreeItem {
    pub fn contains(&self, needle: &str) -> bool {
        let haystack = self.to_string();
        haystack.contains(needle) || haystack.to_lowercase().contains(needle.to_lowercase().as_str())
    }
}

impl Display for AlbumTreeItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            AlbumTreeItem::Artist(s) => s,
            AlbumTreeItem::Album(_, s) => s,
        };
        write!(f, "{}", x)
    }
}

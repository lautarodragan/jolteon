use strum::Display;

use crate::{components::Callback, structs::Song, theme::Theme};

pub struct CommandLine<'a> {
    pub(super) theme: Theme,
    pub(super) query: Option<Query>,
    pub(super) query_error: Option<String>,
    pub(super) on_confirm_fn: Callback<'a, Query>,
}

impl<'a> CommandLine<'a> {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            query: None,
            query_error: None,
            on_confirm_fn: Callback::default(),
        }
    }

    pub fn query(&self) -> Option<&Query> {
        self.query.as_ref()
    }

    pub fn set_query(&mut self, query: Option<Query>) {
        self.query = query
    }

    pub fn on_confirm(&self, f: impl Fn(Query) + 'a) {
        self.on_confirm_fn.set(f);
    }
}

#[derive(Debug, Display)]
pub enum Query {
    AddSongs {
        songs: Vec<Song>,
        step: usize,
        target: QueryAddSongsTarget,
        target_name: Option<String>,
        playlists: Vec<String>,
    },
}
#[derive(Debug, Display, Clone, PartialEq, Eq)]
pub enum QueryAddSongsTarget {
    Library,
    Soundtracks,
    Playlist,
}

impl QueryAddSongsTarget {
    pub fn next(&self) -> Self {
        match self {
            Self::Library => Self::Soundtracks,
            Self::Soundtracks => Self::Playlist,
            Self::Playlist => Self::Playlist,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Library => Self::Library,
            Self::Soundtracks => Self::Library,
            Self::Playlist => Self::Soundtracks,
        }
    }
}

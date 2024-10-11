use std::{
    path::PathBuf,
    time::Duration,
};

use lofty::{Accessor, AudioFile, LoftyError, Probe, TaggedFileExt};
use serde::{Deserialize, Serialize};

use crate::{
    cue::CueSheet,
    components::{FileBrowserSelection, directory_to_songs_and_folders},
};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Song {
    pub path: PathBuf,
    pub start_time: Duration,
    pub length: Duration,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub track: Option<u32>,
    pub year: Option<u32>,
}

impl Song {
    pub fn from_file(path: &PathBuf) -> Result<Self, LoftyError> {
        let tagged_file = Probe::open(path)?.read()?;

        // for tag in tagged_file.tags() {
        //     for tag_item in tag.items() {
        //         log::debug!("tag_item {:?}", tag_item);
        //     }
        // }

        let (artist, album, title, track, year) = match tagged_file.primary_tag() {
            Some(primary_tag) => (
                primary_tag.artist().map(String::from),
                primary_tag.album().map(String::from),
                primary_tag.title().map(String::from),
                primary_tag.track(),
                primary_tag.year(),
            ),
            _ => (None, None, None, None, None),
        };

        Ok(Song {
            path: PathBuf::from(path),
            start_time: Duration::ZERO,
            length: tagged_file.properties().duration(),
            title: title.unwrap_or(path.file_name().unwrap().to_str().unwrap().to_string()),
            artist,
            album,
            track,
            year,
        })
    }

    pub fn from_dir(path: &PathBuf) -> Vec<Self> {
        let songs = directory_to_songs_and_folders(path);
        songs.iter().filter_map(|s| {
            if let FileBrowserSelection::Song(song) = s {
                Some(song.clone())
            } else {
                None
            }
        }).collect()
    }

    pub fn from_cue_sheet(cue_sheet: CueSheet) -> Vec<Self> {
        let cue_file = cue_sheet.file().unwrap();
        let performer = cue_sheet.performer();
        let file_name = cue_file.name();
        let tracks = cue_file.tracks();

        let cue_path = cue_sheet.cue_sheet_file_path();
        let song_path = cue_path.parent().unwrap().join(file_name);

        let song = match Song::from_file(&song_path) {
            Ok(s) => s,
            Err(err) => {
                log::warn!(target: "::song.from_cue_sheet", "Could not load songs from cue sheet.");
                log::warn!(target: "::song.from_cue_sheet", "Cue sheet path: {:?}", cue_path);
                log::warn!(target: "::song.from_cue_sheet", "Error: {:#?}", err);
                log::warn!(target: "::song.from_cue_sheet", "Full cue sheet: {:#?}", cue_sheet);
                return Vec::new();
            }
        };

        // log::debug!(target: "::Song.from_cue_sheet()", "{:#?}", tracks);

        let mut songs: Vec<Song> = tracks
            .iter()
            .map(|t| Song {
                path: song_path.clone(),
                length: Duration::ZERO,
                artist: performer.clone().or(t.performer()),
                title: t.title(),
                start_time: t.start_time(),
                album: cue_sheet.title(),
                track: t.index().split_whitespace().nth(0).map(|i| i.parse().ok()).flatten(),
                year: song.year, // TODO: cue sheet year as a fallback? (it's usually stored as a comment in it...)
            })
            .collect();

        for i in 0..songs.len() {
            let next_start = if i < songs.len() - 1 {
                songs[i + 1].start_time
            } else {
                song.length
            };
            let this_start = songs[i].start_time;
            songs[i].length = next_start.saturating_sub(this_start);
        }

        songs
    }
}

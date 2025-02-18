use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    time::Duration,
};

use lofty::{
    error::LoftyError,
    file::{AudioFile, TaggedFileExt},
    probe::Probe,
    tag::Accessor,
};
use serde::{Deserialize, Serialize};

use crate::{
    components::{directory_to_songs_and_folders, FileBrowserSelection},
    cue::CueSheet,
    structs::Jolt,
};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Song {
    pub path: PathBuf,
    pub start_time: Duration,
    pub length: Duration,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub disc_number: Option<u32>,
    pub track: Option<u32>,
    pub year: Option<u32>,
}

impl Song {
    pub fn from_file(path: &PathBuf) -> Result<Self, LoftyError> {
        let tagged_file = Probe::open(path)?.read()?;
        let jolt = Jolt::from_path(path.parent().unwrap().join(".jolt")).ok();

        let (artist, album, title, track, year, disc_number) = match tagged_file.primary_tag() {
            Some(primary_tag) => (
                primary_tag.artist().map(String::from),
                primary_tag.album().map(String::from),
                primary_tag.title().map(String::from),
                primary_tag.track(),
                primary_tag.year(),
                primary_tag.disk(),
            ),
            _ => (None, None, None, None, None, None),
        };

        Ok(Song {
            path: PathBuf::from(path),
            start_time: Duration::ZERO,
            length: tagged_file.properties().duration(),
            title: title.unwrap_or(path.file_name().unwrap().to_str().unwrap().to_string()),
            artist: jolt.as_ref().and_then(|j| j.artist.clone()).or(artist),
            album: jolt.as_ref().and_then(|j| j.album.clone()).or(album),
            disc_number,
            track,
            year,
        })
    }

    pub fn from_dir(path: &Path) -> Vec<Self> {
        // TODO: improve this. stop using the FileBrowser stuff.
        //   check for songs, cue
        let entries = directory_to_songs_and_folders(path);

        let jolt = entries.iter().find_map(|e| match e {
            FileBrowserSelection::Jolt(j) => Some(j),
            _ => None,
        });

        log::trace!(target: "::Song::from_dir", "{:#?}", jolt);

        entries
            .iter()
            .filter_map(|s| {
                if let FileBrowserSelection::Song(song) = s {
                    let mut song = song.clone();

                    if let Some(jolt) = jolt {
                        if jolt.album.is_some() {
                            song.album.clone_from(&jolt.album);
                        }
                        if jolt.artist.is_some() {
                            song.artist.clone_from(&jolt.artist);
                        }
                    }

                    Some(song)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn from_cue_sheet(cue_sheet: CueSheet) -> Vec<Self> {
        cue_sheet.files().into_iter().flat_map(|cue_file| {
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

            let jolt = Jolt::from_path(song_path.parent().unwrap().join(".jolt")).ok();

            // TODO: attempt to read date from REM DATE comment
            let cue_date = cue_sheet
                .comments()
                .into_iter()
                .find(|comment| comment.starts_with("DATE "));

            log::debug!("DATE from cue sheet: {cue_date:?}");

            let cue_year: Option<u32> = cue_date.unwrap()[5..].parse().ok(); // TODO: 100% not safe lol

            log::debug!("DATE from cue sheet: {cue_year:?}");

            let mut songs: Vec<Song> = tracks
                .iter()
                .map(|t| Song {
                    path: song_path.clone(),
                    length: Duration::ZERO,
                    artist: jolt
                        .as_ref()
                        .and_then(|j| j.artist.clone())
                        .or(performer.clone())
                        .or(t.performer()),
                    title: t.title(),
                    start_time: t.start_time(),
                    album: jolt.as_ref().and_then(|j| j.album.clone()).or(cue_sheet.title()),
                    track: t.index().split_whitespace().nth(0).and_then(|i| i.parse().ok()),
                    year: song.year.or(cue_year),
                    disc_number: jolt.as_ref().and_then(|j| j.disc_number), // There seems to be no standard disc number field for Cue Sheets...
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
        }).collect()
    }

    // pub fn debug_tags(&self) {
    //     let tagged_file = Probe::open(&self.path).unwrap().read().unwrap();
    //
    //     // log::debug!("properties {:?}", tagged_file.properties());
    //
    //     let tags = tagged_file.tags();
    //
    //     for tag in tags {
    //         let items: Vec<_> = tag.items().map(|i| (i.key(), i.value())).collect();
    //         log::debug!("tag {:?} {:#?}", tag.tag_type(), items);
    //
    //         for item in tag.items() {
    //             log::debug!("tag item {:?}", item);
    //         }
    //     }
    // }

    pub fn get_tags(&self) -> Vec<lofty::tag::Tag> {
        let tagged_file = Probe::open(&self.path).unwrap().read().unwrap();
        tagged_file.tags().to_vec()
    }
}

impl Ord for Song {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.album, &other.album) {
            (Some(album_a), Some(album_b)) if album_a == album_b => match self.disc_number.cmp(&other.disc_number) {
                Ordering::Equal => match (&self.track, &other.track) {
                    (Some(a), Some(b)) => a.cmp(b),
                    (Some(_), None) => Ordering::Greater,
                    (None, Some(_)) => Ordering::Less,
                    _ => self.title.cmp(&other.title),
                },
                o => o,
            },
            (Some(album_a), Some(album_b)) if album_a != album_b => match (self.year, other.year) {
                (Some(ref year_a), Some(ref year_b)) => {
                    if year_a != year_b {
                        year_a.cmp(year_b)
                    } else {
                        album_a.cmp(album_b)
                    }
                }
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                _ => album_a.cmp(album_b),
            },
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            _ => self.title.cmp(&other.title),
        }
    }
}

impl PartialOrd for Song {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

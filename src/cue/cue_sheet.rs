use std::{
    io::{self},
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    components::dir_entry_is_song,
    cue::{cue_line::CueLine, cue_line_node::CueLineNode, cue_sheet_item::CueSheetItem},
};

#[allow(dead_code)]
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct CueSheet {
    cue_sheet_file_path: PathBuf,
    unknown: Vec<String>,
    comments: Vec<String>,
    performer: Option<String>,
    title: Option<String>,
    files: Vec<CueFile>,
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct CueFile {
    name: String,
    tracks: Vec<Track>,
}

impl CueFile {
    fn new(name: String, mut c: Vec<CueSheetItem>) -> Self {
        let mut tracks = Vec::new();

        while let Some(t) = c.pop() {
            if let CueSheetItem::Track(track_index, track_properties) = t {
                tracks.push(Track::new(track_index, track_properties));
            }
        }

        tracks.sort_by(|a, b| a.index.partial_cmp(&b.index).unwrap());

        Self { name, tracks }
    }

    pub fn name(&self) -> String {
        let name = self.name.clone();
        let name_parts: Vec<&str> = name.split('"').filter(|s| !s.is_empty()).collect();
        name_parts[0].to_string()
    }

    pub fn tracks(&self) -> Vec<Track> {
        self.tracks.clone()
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Track {
    index: String,
    // type: String (could be enum. always "audio" for now)
    title: String,
    start_time: String,
    performer: Option<String>,
}

impl Track {
    fn new(track_index: String, mut track_properties: Vec<CueSheetItem>) -> Self {
        let mut track = Self::default();
        track.index = track_index;

        while let Some(t) = track_properties.pop() {
            match t {
                CueSheetItem::Title(s) => track.title = s,
                CueSheetItem::Performer(s) => track.performer = Some(s),
                CueSheetItem::Index(s) => track.start_time = s,
                _ => {}
            }
        }

        track
    }

    pub fn index(&self) -> String {
        self.index.clone()
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn performer(&self) -> Option<String> {
        self.performer.clone()
    }

    pub fn start_time(&self) -> Duration {
        let start_time = self.start_time.clone();
        let start_time_parts: Vec<&str> = start_time.split_whitespace().filter(|s| !s.is_empty()).collect();
        let start_time_parts = start_time_parts[1].to_string();
        let mut time_parts: Vec<&str> = start_time_parts.split(':').collect();
        // MINUTES:SECONDS:FRAMES

        let _frames = match time_parts.pop() {
            Some(f) => str::parse(f).unwrap(),
            _ => 0,
        };

        let mut multiplier = 1u64;
        let mut seconds = 0u64;
        while let Some(t) = time_parts.pop() {
            let n: u64 = str::parse(t).unwrap();
            seconds += n * multiplier;
            multiplier *= 60;
        }

        Duration::from_secs(seconds)
    }
}

impl CueSheet {
    pub fn from_file(path: &Path) -> io::Result<CueSheet> {
        let cue_lines = CueLine::from_file(path)?;
        let cue_nodes = CueLineNode::from_lines(cue_lines);
        let mut top_cue_items: Vec<CueSheetItem> = cue_nodes.iter().map(CueSheetItem::from_cue_line_node).collect();

        let mut sheet = CueSheet::default();
        sheet.cue_sheet_file_path = path.to_path_buf();

        for e in top_cue_items {
            match e {
                CueSheetItem::Comment(s) => sheet.comments.push(s),
                CueSheetItem::Title(s) => sheet.title = Some(s),
                CueSheetItem::Performer(s) => sheet.performer = Some(s),
                CueSheetItem::File(s, c) => {
                    if s.contains(char::REPLACEMENT_CHARACTER) {
                        // Super primitive way to support non-utf encodings.
                        log::warn!("File name has invalid UTF8! {s}");

                        if !path.join(Path::new(&s)).exists() {
                            log::warn!(
                                "The file with invalid UTF8 does not exist. Will try to guess which file it may be."
                            );
                        }

                        let Ok(entries) = path.parent().unwrap().read_dir() else {
                            log::warn!("No entries at path {path:?}.");
                            continue;
                        };

                        for entry in entries.filter_map(|e| e.ok()).filter(dir_entry_is_song) {
                            let entry_path = entry.path();
                            let entry_path = entry_path.to_string_lossy();

                            log::trace!("attempting entry {entry_path}");

                            if !PathBuf::from(entry_path.to_string()).exists() {
                                log::warn!("No dice. Will have to ignore {entry_path}");
                                continue;
                            }

                            log::warn!("This one seems to work: {entry_path}");

                            sheet.files.push(CueFile::new(entry_path.to_string(), c));
                            break;
                        }
                    } else {
                        sheet.files.push(CueFile::new(s, c));
                    }
                }
                _ => {}
            }
        }

        sheet.comments.sort();

        Ok(sheet)
    }

    pub fn cue_sheet_file_path(&self) -> PathBuf {
        self.cue_sheet_file_path.clone()
    }

    pub fn files(&self) -> Vec<CueFile> {
        self.files.clone()
    }

    pub fn title(&self) -> Option<String> {
        self.title.clone()
    }

    pub fn performer(&self) -> Option<String> {
        self.performer.clone()
    }

    pub fn comments(&self) -> Vec<String> {
        self.comments.clone()
    }

    pub fn unknowns(&self) -> Vec<String> {
        self.unknown.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cue_sheet_from_file() {
        let path = Path::new("./src/cue/Tim Buckley - Happy Sad.cue");
        let cue = CueSheet::from_file(path).unwrap();

        assert_eq!(cue.unknown.len(), 0);
        assert_eq!(cue.comments.len(), 4);

        assert_eq!(
            cue.comments,
            vec![
                "COMMENT \"Nice album\"",
                "DATE 1969",
                "DISCID 5B0A7D06",
                "GENRE Folk/Blues",
            ]
        );

        assert_eq!(cue.performer, Some("Tim Buckley".to_string()));

        assert_eq!(cue.files.len(), 1);

        assert_eq!(cue.files[0].tracks.len(), 6, "{:#?}", cue.files[0]);

        assert_eq!(
            cue.files[0].tracks[0],
            Track {
                index: "01 AUDIO".to_string(),
                title: "Strange Feelin'".to_string(),
                start_time: "01 00:00:00".to_string(),
                performer: Some("Tim Buckley".to_string())
            }
        );

        assert_eq!(
            cue.files[0].tracks[1],
            Track {
                index: "02 AUDIO".to_string(),
                title: "Buzzin' Fly".to_string(),
                start_time: "01 07:41:25".to_string(),
                performer: Some("Tim Buckley".to_string())
            }
        );

        assert_eq!(
            cue.files[0].tracks[5],
            Track {
                index: "06 AUDIO".to_string(),
                title: "Sing A Song For You".to_string(),
                performer: Some("Tim Buckley".to_string()),
                start_time: "01 42:06:30".to_string(),
            }
        );
    }

    #[test]
    fn cue_sheet_from_file_2() {
        let path = Path::new("./src/cue/Moroccan Roll.cue");
        let cue = CueSheet::from_file(path).unwrap();

        assert_eq!(cue.unknown.len(), 0);
        assert_eq!(cue.comments.len(), 5);

        assert_eq!(
            cue.comments,
            vec![
                "DATE \"1977\"",
                "DISCID 5B171C07",
                "DISCNUMBER 1",
                "GENRE \"Jazz-Rock\"",
                "TOTALDISCS 1",
            ]
        );

        assert_eq!(cue.performer, Some("Brand X".to_string()));

        assert_eq!(cue.files.len(), 9);

        assert_eq!(cue.files[0].tracks.len(), 1, "{:#?}", cue.files[0]);

        assert_eq!(
            cue.files[0].tracks[0],
            Track {
                index: "01 AUDIO".to_string(),
                title: "Sun In The Night".to_string(),
                start_time: "01 00:00:00".to_string(),
                performer: None,
            }
        );
    }
}

use super::cue_line_node::CueLineNode;

#[derive(PartialEq, Eq, Debug)]
pub enum CueSheetItem {
    Comment(String),
    Title(String),
    Performer(String),
    #[allow(dead_code)]
    Album(Option<String>),
    File(String, Vec<CueSheetItem>),
    Track(String, Vec<CueSheetItem>),
    Index(String),
    Unknown,
}

impl CueSheetItem {
    pub fn from_cue_line_node(cue_line_node: &CueLineNode) -> Self {
        match cue_line_node.line.key.as_str() {
            "REM" => Self::Comment(cue_line_node.line.value.clone()),
            "PERFORMER" => Self::Performer(cue_line_node.line.value.trim_matches('"').to_string()),
            "TITLE" => Self::Title(cue_line_node.line.value.trim_matches('"').to_string()),
            "INDEX" => Self::Index(cue_line_node.line.value.clone()),
            "FILE" => {
                let children = cue_line_node.children.iter().map(Self::from_cue_line_node).collect();
                Self::File(cue_line_node.line.value.clone(), children)
            }
            "TRACK" => {
                let children = cue_line_node.children.iter().map(Self::from_cue_line_node).collect();
                Self::Track(cue_line_node.line.value.clone(), children)
            }
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::cue::{cue_line::CueLine, cue_line_node::CueLineNode};

    #[test]
    fn cue_sheet_item_from_cue_line_node() {
        let cue_line_node = CueLineNode {
            line: CueLine {
                indentation: 4,
                key: "REM".to_string(),
                value: "GENRE Folk/Blues".to_string(),
            },
            children: Vec::new(),
        };

        let cue_sheet_item = CueSheetItem::from_cue_line_node(&cue_line_node);

        assert_eq!(cue_sheet_item, CueSheetItem::Comment("GENRE Folk/Blues".to_string()));

        let cue_line_node = CueLineNode {
            line: CueLine {
                indentation: 4,
                key: "TITLE".to_string(),
                value: "Happy Sad".to_string(),
            },
            children: Vec::new(),
        };

        let cue_sheet_item = CueSheetItem::from_cue_line_node(&cue_line_node);

        assert_eq!(cue_sheet_item, CueSheetItem::Title("Happy Sad".to_string()));
    }

    #[test]
    fn cue_sheet_items_from_file() {
        let path = Path::new("src/cue/Tim Buckley - Happy Sad.cue");
        let cue_lines = CueLine::from_file(path).unwrap();
        let cue_nodes = CueLineNode::from_lines(cue_lines);
        let top_cue_items: Vec<CueSheetItem> = cue_nodes.iter().map(CueSheetItem::from_cue_line_node).collect();

        assert_eq!(top_cue_items.len(), 7);

        assert_eq!(top_cue_items[..4], vec![
            CueSheetItem::Comment("GENRE Folk/Blues".to_string()),
            CueSheetItem::Comment("DATE 1969".to_string()),
            CueSheetItem::Comment("DISCID 5B0A7D06".to_string()),
            CueSheetItem::Comment("COMMENT \"Nice album\"".to_string()), // TODO: bug! need to be smarter with the stripping of double-quotes.
        ]);

        assert_eq!(top_cue_items[4..6], vec![
            CueSheetItem::Performer("Tim Buckley".to_string()),
            CueSheetItem::Title("Happy Sad".to_string()),
        ]);

        let CueSheetItem::File(file_name, children) = &top_cue_items[6] else {
            panic!("")
        };

        assert_eq!(
            file_name.to_owned(),
            "\"Tim Buckley - Happy Sad.flac\" WAVE".to_string()
        );
        assert_eq!(children.len(), 6);

        assert_eq!(
            children[0],
            CueSheetItem::Track("01 AUDIO".to_string(), vec![
                CueSheetItem::Title("Strange Feelin'".to_string()),
                CueSheetItem::Performer("Tim Buckley".to_string()),
                CueSheetItem::Index("01 00:00:00".to_string()),
            ])
        );

        assert_eq!(
            children[1],
            CueSheetItem::Track("02 AUDIO".to_string(), vec![
                CueSheetItem::Title("Buzzin' Fly".to_string()),
                CueSheetItem::Performer("Tim Buckley".to_string()),
                CueSheetItem::Index("01 07:41:25".to_string()),
            ])
        );

        assert_eq!(
            children[5],
            CueSheetItem::Track("06 AUDIO".to_string(), vec![
                CueSheetItem::Title("Sing A Song For You".to_string()),
                CueSheetItem::Performer("Tim Buckley".to_string()),
                CueSheetItem::Index("01 42:06:30".to_string()),
            ])
        );
    }
}

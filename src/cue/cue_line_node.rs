use std::cmp::Ordering;

use crate::cue::cue_line::CueLine;

#[derive(Debug, Eq, PartialEq)]
pub struct CueLineNode {
    pub line: CueLine,
    pub children: Vec<CueLineNode>,
}

impl CueLineNode {
    fn from_line(line: CueLine) -> Self {
        Self {
            line,
            children: Vec::new(),
        }
    }

    pub fn from_lines(cue_lines: Vec<CueLine>) -> Vec<CueLineNode> {
        fn node_by_depth(nodes: &mut [CueLineNode], depth: usize) -> &mut CueLineNode {
            assert!(!nodes.is_empty());

            let node = &mut nodes[nodes.len() - 1];

            if depth == 0 {
                node
            } else {
                node_by_depth(&mut node.children, depth - 1)
            }
        }

        let mut top_nodes = vec![];
        let mut depth = 0;

        for node in cue_lines.into_iter().map(Self::from_line) {
            if node.line.indentation == 0 {
                depth = 0;
                top_nodes.push(node);
                continue;
            }

            if top_nodes.is_empty() {
                // this would mean the first node has non-zero indentation!
                // TODO: we could be more permissive and apply some heuristics here
                //   (allow any indentation for the first cue_line,
                //   check whether it makes sense for a node of type X to be a child of
                //   a node of type Y and guess the correct indentation otherwise, etc)
                log::warn!("Ignoring Cue Sheet invalid data: {node:?}");
                continue;
            }

            match node.line.indentation.cmp(&depth) {
                Ordering::Less => {
                    // current `node` is a sibling of _some_ ancestor of previous`node`
                    depth = node.line.indentation;

                    if depth == 0 {
                        top_nodes.push(node);
                    } else {
                        let parent = node_by_depth(&mut top_nodes, depth - 1);
                        parent.children.push(node);
                    }
                }
                Ordering::Equal => {
                    // current `node` is a sibling of previous `node`
                    let parent = node_by_depth(&mut top_nodes, depth - 1);
                    parent.children.push(node);
                }
                Ordering::Greater => {
                    // current `node` is a child of previous `node`
                    let parent = node_by_depth(&mut top_nodes, depth);
                    parent.children.push(node);
                    depth += 1;
                }
            }
        }

        top_nodes
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn cue_line_nodes_from_lines_single_file() {
        let path = Path::new("src/cue/Tim Buckley - Happy Sad.cue");
        let cue_lines = CueLine::from_file(path).unwrap();

        let cue_nodes = CueLineNode::from_lines(cue_lines);

        assert_eq!(cue_nodes.len(), 7);

        let keys: Vec<String> = cue_nodes.iter().map(|n| n.line.key.clone()).collect();
        assert_eq!(keys, vec!["REM", "REM", "REM", "REM", "PERFORMER", "TITLE", "FILE"]);

        let file = &cue_nodes[cue_nodes.len() - 1];

        assert_eq!(file.line, CueLine {
            indentation: 0,
            key: "FILE".to_string(),
            value: "\"Tim Buckley - Happy Sad.flac\" WAVE".to_string(),
        });

        assert_eq!(file.children[0], CueLineNode {
            line: CueLine {
                indentation: 1,
                key: "TRACK".to_string(),
                value: "01 AUDIO".to_string(),
            },
            children: vec![
                CueLineNode {
                    line: CueLine {
                        indentation: 2,
                        key: "TITLE".to_string(),
                        value: "\"Strange Feelin'\"".to_string(),
                    },
                    children: vec![],
                },
                CueLineNode {
                    line: CueLine {
                        indentation: 2,
                        key: "PERFORMER".to_string(),
                        value: "\"Tim Buckley\"".to_string(),
                    },
                    children: vec![],
                },
                CueLineNode {
                    line: CueLine {
                        indentation: 2,
                        key: "INDEX".to_string(),
                        value: "01 00:00:00".to_string(),
                    },
                    children: vec![],
                },
            ]
        });

        assert_eq!(file.children[1], CueLineNode {
            line: CueLine {
                indentation: 1,
                key: "TRACK".to_string(),
                value: "02 AUDIO".to_string(),
            },
            children: vec![
                CueLineNode {
                    line: CueLine {
                        indentation: 2,
                        key: "TITLE".to_string(),
                        value: "\"Buzzin' Fly\"".to_string(),
                    },
                    children: vec![],
                },
                CueLineNode {
                    line: CueLine {
                        indentation: 2,
                        key: "PERFORMER".to_string(),
                        value: "\"Tim Buckley\"".to_string(),
                    },
                    children: vec![],
                },
                CueLineNode {
                    line: CueLine {
                        indentation: 2,
                        key: "INDEX".to_string(),
                        value: "01 07:41:25".to_string(),
                    },
                    children: vec![],
                },
            ]
        });

        assert_eq!(file.children[5], CueLineNode {
            line: CueLine {
                indentation: 1,
                key: "TRACK".to_string(),
                value: "06 AUDIO".to_string(),
            },
            children: vec![
                CueLineNode {
                    line: CueLine {
                        indentation: 2,
                        key: "TITLE".to_string(),
                        value: "\"Sing A Song For You\"".to_string(),
                    },
                    children: vec![],
                },
                CueLineNode {
                    line: CueLine {
                        indentation: 2,
                        key: "PERFORMER".to_string(),
                        value: "\"Tim Buckley\"".to_string(),
                    },
                    children: vec![],
                },
                CueLineNode {
                    line: CueLine {
                        indentation: 2,
                        key: "INDEX".to_string(),
                        value: "01 42:06:30".to_string(),
                    },
                    children: vec![],
                },
            ]
        });
    }

    #[test]
    fn cue_line_nodes_from_lines_multi_file() {
        let path = Path::new("src/cue/Moroccan Roll.cue");
        let cue_lines = CueLine::from_file(path).unwrap();

        let cue_top_nodes = CueLineNode::from_lines(cue_lines);

        assert_eq!(cue_top_nodes.len(), 16);

        let keys: Vec<String> = cue_top_nodes.iter().map(|n| n.line.key.clone()).collect();
        assert_eq!(keys, vec![
            "TITLE",
            "PERFORMER",
            "REM",
            "REM",
            "REM",
            "REM",
            "REM",
            "FILE",
            "FILE",
            "FILE",
            "FILE",
            "FILE",
            "FILE",
            "FILE",
            "FILE",
            "FILE"
        ]);

        assert_eq!(cue_top_nodes[0], CueLineNode {
            line: CueLine {
                indentation: 0,
                key: "TITLE".to_string(),
                value: "\"Moroccan Roll (LP)\"".to_string(),
            },
            children: vec![],
        });

        assert_eq!(cue_top_nodes[7], CueLineNode {
            line: CueLine {
                indentation: 0,
                key: "FILE".to_string(),
                value: "\"01 Sun In The Night.flac\" WAVE".to_string(),
            },
            children: vec![CueLineNode {
                line: CueLine {
                    indentation: 1,
                    key: "TRACK".to_string(),
                    value: "01 AUDIO".to_string(),
                },
                children: vec![
                    CueLineNode {
                        line: CueLine {
                            indentation: 2,
                            key: "TITLE".to_string(),
                            value: "\"Sun In The Night\"".to_string(),
                        },
                        children: vec![],
                    },
                    CueLineNode {
                        line: CueLine {
                            indentation: 2,
                            key: "INDEX".to_string(),
                            value: "01 00:00:00".to_string(),
                        },
                        children: vec![],
                    },
                ],
            }]
        });
    }
}

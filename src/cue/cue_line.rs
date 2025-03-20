use std::{
    fmt::{Debug, Display, Formatter},
    fs::File,
    io::{self, Read},
    path::Path,
};

#[derive(Eq, PartialEq, Debug)]
pub struct CueLine {
    pub indentation: usize,
    pub key: String,
    pub value: String,
}

impl Display for CueLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.key, self.value)
    }
}

impl CueLine {
    pub fn from_file(path: &Path) -> io::Result<Vec<CueLine>> {
        let mut file = File::open(path)?;

        let mut buf = vec![];
        file.read_to_end(&mut buf)?;

        let contents = String::from_utf8_lossy(&buf);

        let mut cue_lines = Vec::new();
        for line in contents.split(&['\n', '\r']).filter(|l| !l.trim().is_empty()) {
            if line.contains(char::REPLACEMENT_CHARACTER) {
                log::warn!("this line has invalid UTF8 {line}");
            }

            let key_value = line.trim_start();
            let indentation = line.len() - key_value.len();

            let Some((key, value)) = key_value.split_once(char::is_whitespace) else {
                log::warn!("lines should be key value {:?}", line);
                continue;
            };

            cue_lines.push(Self {
                indentation: indentation / 2,
                key: key.to_string(),
                value: value.to_string(),
            });
        }

        Ok(cue_lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cue_lines_from_file() {
        let path = Path::new("./src/cue/Tim Buckley - Happy Sad.cue");
        let cue_lines = CueLine::from_file(path).unwrap();

        assert_eq!(cue_lines.len(), 31, "{cue_lines:#?}");

        assert_eq!(cue_lines[0], CueLine {
            indentation: 0,
            key: "REM".to_string(),
            value: "GENRE Folk/Blues".to_string(),
        });

        assert_eq!(cue_lines[4], CueLine {
            indentation: 0,
            key: "PERFORMER".to_string(),
            value: "\"Tim Buckley\"".to_string(),
        });

        assert_eq!(cue_lines[5], CueLine {
            indentation: 0,
            key: "TITLE".to_string(),
            value: "\"Happy Sad\"".to_string(),
        });
    }
}

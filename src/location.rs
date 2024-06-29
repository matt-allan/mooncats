//!Locations within source code files as line and character offsets.
use std::{hash::Hash, path::PathBuf};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::errors::{self, *};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileUri(Url);

impl FileUri {
    pub fn parse(input: &str) -> Result<Self> {
        let uri = Url::parse(input)?;

        Self::try_from(uri)
    }

    pub fn starts_with_path(&self, base: &FileUri) -> bool {
        self.0.path().starts_with(base.0.path())
    }

    pub fn strip_path_prefix(&self, base: &FileUri) -> Result<Self> {
        if ! self.0.path().starts_with(base.0.path()) {
            bail!("Not a prefix")
        }

        let mut stripped = self.clone();
        stripped.0.set_path(&self.0.path().strip_prefix(base.0.path()).unwrap().to_string());

        Ok(stripped)
    }

    pub fn file_name(&self) -> String {
        self.0.path_segments().unwrap().next_back().unwrap().to_string()
    }

    pub fn split_file_at_dot(&self) -> (String, String) {
        let name = self.file_name();

        if let Some((stem, ext)) = name.rsplitn(2, ".").collect_tuple() {
            return (stem.to_string(), ext.to_string())
        }

        return (name.clone(), String::new())
    }

    pub fn file_stem(&self) -> String {
        let (stem, _) = self.split_file_at_dot();

        stem
    }

    pub fn extension(&self) -> String {
        let (_, ext) = self.split_file_at_dot();

        ext
    }

    pub fn to_file_path(&self) -> Result<PathBuf> {
        self.0.to_file_path().map_err(|_| anyhow!("File URI is not a valid path"))
    }

    pub fn depth(&self) -> usize {
        let segments = self.0.path_segments().unwrap();

        let mut n = 0;
        for _ in segments {
            n += 1
        }
        n = (n-1).max(0); // don't count the last segment, which is the filename

        n
    }
}

impl TryFrom<PathBuf> for FileUri {
    type Error = errors::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let uri = Url::from_file_path(value).map_err(|_| anyhow!("invalid file URI"))?;

        Ok(Self(uri))
    }
}

impl TryFrom<Url> for FileUri {
    type Error = errors::Error;

    fn try_from(uri: Url) -> Result<Self, Self::Error> {
        if uri.scheme() != "file" {
            bail!("Missing file scheme")
        }

        let path = uri.path();

        if path.len() == 0 {
            bail!("Missing file path")
        }

        if path.len() == 1 || path.ends_with("..") {
            bail!("Missing file name")
        }

        Ok(Self(uri))
    }
}

/// A position within a document. 
/// The position is packed into as a single u64 when encoding to JSON.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(from = "u64")]
pub struct Position {
    /// Line position in a document (zero-based).
    pub line: u64,
    /// Character offset on a line in a document (zero-based).
    /// The offset counts UTF-16 code units.
    pub character: u64,
}

impl Position {
    /// Unpack a single integer into a position using the LuaLS encoding.
    pub fn unpack(position: u64) -> Self {
        Self {
            line: position / 10_000,
            character: position % 10_000,
        }
    }

    /// Pack a a position into a single integer using the LuaLS encoding.
    #[allow(dead_code)]
    pub fn pack(&self) -> u64 {
        self.line * 10_000 + self.character.min(10_000 - 1)
    }
}

impl From<u64> for Position {
    fn from(value: u64) -> Self {
        Self::unpack(value.into())
    }
}

/// A range of characters between two positions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Range {
    start: Position,
    #[serde(alias = "finish")]
    end: Position,
}

impl From<(Position, Position)> for Range {
    fn from(value: (Position, Position)) -> Self {
        let (start, end) = value;

        assert!(start <= end);

        Self {
            start,
            end,
        }
    }
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        assert!(start <= end);

        Self {
            start,
            end,
        }
    }

    pub fn join(&self, other: &Range) -> Range {
        if other.start < self.end {
            return self.clone()
        }

        return Range {
            start: self.start,
            end: other.end,
        }
    }

    pub fn start(&self) -> Position {
        self.start
    }

    pub fn end(&self) -> Position {
        self.end
    }

    pub fn bounds(&self) -> (Position, Position) {
        (self.start, self.end)
    }
}

/// A location specifies a source file and a range of characters.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Location {
    #[serde(alias = "file")]
	pub file: FileUri,
    #[serde(flatten)]
	pub range: Range,
}

/// A span of text from within a file.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Span {
    pub text: String,
	pub location: Location,
}

impl Span {
    pub fn empty(location: Location) -> Self {
        Self {
            text: String::new(),
            location,
        }
    }

    pub fn is_empty(&self) -> bool {
        return self.text.len() == 0
    }
}

/// Read a range from the given text.
pub fn read_range<'a>(text: &'a str, range: &Range) -> String {
    let (start, end) = range.bounds();

    text
        .lines()
        .enumerate()
        .map(|(i, l)| (u64::try_from(i).expect("overflow"), l))
        .filter(|(i, _)| {
            *i >= start.line && *i <= end.line
        })
        .map(|(i, l)| {
            let codepoints: Vec<u16> = l.encode_utf16()
                .into_iter()
                .collect();

            if i == start.line {
                let start = usize::try_from(start.character).expect("oveflow");
                String::from_utf16_lossy(&codepoints[start..])
            } else if i == end.line {
                let end = usize::try_from(end.character).expect("oveflow");
                String::from_utf16_lossy(&codepoints[0..end])
            } else {
                l.to_string()
            }
        })
        .join("")
}
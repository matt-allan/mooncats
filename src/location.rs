//!Locations within source code files as line and character offsets.
use std::hash::Hash;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use url::Url;

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
	pub file: Url,
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
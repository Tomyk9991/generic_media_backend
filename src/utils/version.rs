use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::num::ParseFloatError;
use std::str::FromStr;

#[derive(Debug)]
pub enum Error {
    UndefinedSequence(char),
    F32Parsing(ParseFloatError)
}
#[derive(Debug, Clone)]
pub struct Version {
    pub version: f32
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Error::F32Parsing(value)
    }
}

impl Eq for Version {}

impl PartialEq<Self> for Version {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
    }
}

impl PartialOrd<Self> for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.version.partial_cmp(&other.version)
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version.total_cmp(&other.version)
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Error::UndefinedSequence(c) => format!("Cannot parse character: {}", c),
            Error::F32Parsing(f) => f.to_string()
        })
    }
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();

        if let ["version", version] = &s.split(' ').collect::<Vec<&str>>()[..] {
            let mut iter = version.split('.');
            let first_part = iter.next().unwrap_or("0");
            let rest_parts: String = iter.collect();

            let version_str = format!("{}.{}", first_part, rest_parts);
            let s = version_str.parse()?;

            return Ok(Self {
                version: s
            })
        }

        for char in s.chars() {
            if char.is_ascii_digit() || char == '.' {
                continue;
            } else {
                return Err(Error::UndefinedSequence(char))
            }
        }

        let mut iter = s.split('.');
        let first_part = iter.next().unwrap_or("0");
        let rest_parts: String = iter.collect();

        let version_str = format!("{}.{}", first_part, rest_parts);
        let s = version_str.parse()?;

        Ok(Self {
            version: s
        })
    }
}

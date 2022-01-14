use colored::Colorize;
use derive_more::Display;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

static VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)
                ^ # start
                ([1-9]+\d*|0) # major
                (?:
                    \.
                    ([1-9]+\d*|0) # minor
                    (?:
                        \.
                        ([1-9]+\d*|0) # patch
                        (?:
                            (alpha|beta|RC) # pre_type
                            ([1-9]+) # pre_version
                        )?
                    )?
                )?
                $ # end
            ",
    )
    .unwrap()
});

pub type Version = Major;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Major {
    version: usize,
    minor: Option<Minor>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
struct Minor {
    version: usize,
    patch: Option<Patch>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
struct Patch {
    version: usize,
    pre: Option<Pre>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
struct Pre {
    version: usize,
    pre_type: PreType,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display, Serialize)]
pub enum PreType {
    #[display(fmt = "alpha")]
    Alpha,
    #[display(fmt = "beta")]
    Beta,
    #[display(fmt = "RC")]
    Rc,
}

impl Version {
    pub fn from_numbers(major: usize, minor: Option<usize>, patch: Option<usize>) -> Self {
        Self {
            version: major,
            minor: minor.map(|version| Minor {
                version,
                patch: patch.map(|version| Patch { version, pre: None }),
            }),
        }
    }
    pub fn from_major(major: usize) -> Self {
        Self {
            version: major,
            minor: None,
        }
    }
    pub fn includes(&self, other: &Self) -> bool {
        self.major_version() == other.major_version()
            && self.minor.map_or(true, |Minor { version, patch }| {
                Some(version) == other.minor_version()
                    && (patch.map_or(true, |Patch { version, pre }| {
                        Some(version) == other.patch_version()
                            && pre.map_or(true, |pre| Some(pre) == other.pre())
                    }))
            })
    }
    fn minor(self) -> Option<Minor> {
        self.minor
    }
    fn patch(self) -> Option<Patch> {
        self.minor().and_then(|minor| minor.patch)
    }
    fn pre(self) -> Option<Pre> {
        self.patch().and_then(|patch| patch.pre)
    }
    pub fn major_version(self) -> usize {
        self.version
    }
    pub fn minor_version(self) -> Option<usize> {
        self.minor().map(|minor| minor.version)
    }
    pub fn patch_version(self) -> Option<usize> {
        self.patch().map(|patch| patch.version)
    }
    pub fn pre_type(self) -> Option<PreType> {
        self.pre().map(|pre| pre.pre_type)
    }
    pub fn pre_version(self) -> Option<usize> {
        self.pre().map(|pre| pre.version)
    }

    pub fn is_same_major(self, other: Self) -> bool {
        self.major_version() == other.major_version()
    }
    pub fn is_same_minor(self, other: Self) -> bool {
        self.minor_version().is_some()
            && other.minor_version().is_some()
            && self.is_same_major(other)
            && self.minor_version() == other.minor_version()
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid version format: \"{0}\"")]
    InvalidVersionFormat(String),
}

impl FromStr for Version {
    type Err = Error;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        if s == "3.0.x (latest)" {
            return Ok(Version::from_numbers(
                3,
                Some(9),
                Some(if cfg!(target_os = "windows") { 17 } else { 18 }),
            ));
        }
        let cap = VERSION_REGEX
            .captures(s)
            .ok_or(Error::InvalidVersionFormat(s.to_owned()))?;
        let to_num = |m: regex::Match| m.as_str().parse().unwrap();
        let major = Major {
            version: to_num(cap.get(1).unwrap()),
            minor: cap.get(2).map(to_num).map(|version| Minor {
                version,
                patch: cap.get(3).map(to_num).map(|version| Patch {
                    version,
                    pre: cap.get(5).map(to_num).map(|version| Pre {
                        version,
                        pre_type: PreType::from_str(&cap[4]).unwrap(),
                    }),
                }),
            }),
        };
        Ok(major)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format!(
            "{}{}{}{}",
            self.major_version(),
            self.minor_version()
                .map_or("".to_owned(), |v| format!(".{}", v)),
            self.patch_version()
                .map_or("".to_owned(), |v| format!(".{}", v)),
            self.pre_type().map_or("".to_owned(), |t| format!(
                "{}{}",
                t,
                self.pre_version().unwrap()
            )),
        )
        .fmt(f)
    }
}

struct VersionVisitor;
impl<'de> de::Visitor<'de> for VersionVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        formatter.write_str("struct Version")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse().map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(VersionVisitor)
    }
}

impl FromStr for PreType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alpha" => Ok(PreType::Alpha),
            "beta" => Ok(PreType::Beta),
            "RC" => Ok(PreType::Rc),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn parsed_from_str() {
        let version3_1_4: Result<Version, _> = "3.1.4".parse();
        assert!(matches!(version3_1_4, Ok(_)));
        assert_eq!(
            version3_1_4.unwrap(),
            Version::from_numbers(3, Some(1), Some(4))
        );
    }

    #[test]
    fn deserialize_from_json() {
        let json = r#"
            { "3.1.4": ["abc", "cdf"] }
        "#;
        let parsed: Result<HashMap<Version, Vec<&str>>, _> = serde_json::from_str(json);
        println!("{:?}", parsed);
        assert!(parsed.is_ok());

        let version3_1_4 = Version::from_numbers(3, Some(1), Some(4));
        assert_eq!(
            parsed.unwrap().get(&version3_1_4),
            Some(&vec!["abc", "cdf"])
        );
    }

    #[test]
    fn includes_test() {
        let version3_1_4 = Version::from_numbers(3, Some(1), Some(4));
        let version3_1 = Version::from_numbers(3, Some(1), None);
        let version3 = Version::from_numbers(3, None, None);

        assert_eq!(version3.includes(&version3), true);
        assert_eq!(version3.includes(&version3_1), true);
        assert_eq!(version3.includes(&version3_1_4), true);
        assert_eq!(version3_1.includes(&version3), false);
        assert_eq!(version3_1.includes(&version3_1), true);
        assert_eq!(version3_1.includes(&version3_1_4), true);
        assert_eq!(version3_1_4.includes(&version3), false);
        assert_eq!(version3_1_4.includes(&version3_1), false);
        assert_eq!(version3_1_4.includes(&version3_1_4), true);
    }
}

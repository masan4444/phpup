use crate::curl;
use crate::version::Version;
use chrono::{Datelike, NaiveDate, Utc};
use derive_more::Display;
use serde::{de, Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("Can't find releases that matches {0}")]
    NotFoundRelease(Version),

    #[error(transparent)]
    CurlError(#[from] curl::Error),

    #[error("Receive error message from release site: {0}")]
    Other(String),
}

fn fetch_and_parse(
    version: Option<Version>,
    max: Option<usize>,
) -> Result<BTreeMap<Version, Release>, FetchError> {
    let base_url = "https://www.php.net/releases/index.php";
    let query = format!(
        "?json=1{}{}",
        version
            .map(|version| format!("&version={}", version))
            .unwrap_or_default(),
        max.map(|max| format!("&max={}", max)).unwrap_or_default(),
    );
    let url = &format!("{}{}", base_url, query);
    let json = curl::get_as_slice(url)?;

    let resp: Response =
        serde_json::from_slice(&json).unwrap_or_else(|_| panic!("Can't parse json from {}", url));
    match resp {
        Response::Map(releases) => Ok(releases),
        Response::One(release) => Ok([(release.version.unwrap(), release)].into_iter().collect()),
        Response::Error { msg } => {
            if msg.starts_with("Unknown version") {
                Err(FetchError::NotFoundRelease(version.unwrap()))
            } else {
                Err(FetchError::Other(msg.to_owned()))
            }
        }
    }
}

pub fn fetch_all(version: Version) -> Result<BTreeMap<Version, Release>, FetchError> {
    fetch_and_parse(Some(version), Some(1000))
}

pub fn fetch_latest(version: Version) -> Result<Release, FetchError> {
    let mut latest = fetch_and_parse(Some(version), None)?;
    let version = *latest.keys().next().unwrap();
    Ok(latest.remove(&version).unwrap())
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
#[serde_as]
enum Response<'a> {
    Map(#[serde_as(as = "BTreeMap<_, _>")] BTreeMap<Version, Release>),
    One(Release),
    Error {
        #[serde(rename = "error")]
        msg: &'a str,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Release {
    announcement: Option<Announcement>,
    tags: Option<Vec<Tag>>,
    source: Vec<Source>,
    #[serde(rename = "windows")]
    windows_binary: Option<Vec<Source>>,
    #[serde(deserialize_with = "date_deserializer")]
    pub date: NaiveDate,
    museum: Option<bool>,
    pub version: Option<Version>,
}

// TODO: wait for #[serde(flatten)] for enum variant, or implement custom deserializer
// { "announcement": { English:  "/releases/..." } }
// or
// { "announcement": true }
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Announcement {
    English(English),
    Flag(bool),
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "PascalCase"))]
struct English {
    english: String,
}

#[derive(Serialize, Deserialize, Debug)]
// #[serde(untagged)]
enum Tag {
    #[serde(rename = "security")]
    Security,
    // TOOD: skip deserialize if value is ""
    #[serde(rename = "")]
    None,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Source {
    File(File),
    Link { link: String, name: String },
}
#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    filename: String,
    name: String,
    #[serde(flatten)]
    checksum: Option<Hash>,
    // TODO: Option<NaiveTime>
    date: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "lowercase", serialize = "lowercase"))]
pub enum Hash {
    SHA256(String),
    MD5(String),
}

#[derive(Error, Debug)]
pub enum ChecksumError {
    #[error("Invalid checksum\nexptected: {expected}\ngot: {got}")]
    InvalidChecksum { expected: String, got: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

use sha2::Digest;
impl Hash {
    pub fn hash_type(&self) -> &'static str {
        match self {
            Hash::SHA256(_) => "SHA-256",
            Hash::MD5(_) => "MD5",
        }
    }
    pub fn verify(&self, mut data: impl std::io::Read) -> Result<(), ChecksumError> {
        let (checksum, hash) = match self {
            Hash::SHA256(checksum) => {
                let mut sha256 = sha2::Sha256::new();
                std::io::copy(&mut data, &mut sha256)?;
                let hash = sha256.finalize();
                (checksum, format!("{:x}", hash))
            }
            Hash::MD5(checksum) => {
                let mut md5 = md5::Context::new();
                std::io::copy(&mut data, &mut md5)?;
                let hash = md5.compute();
                (checksum, format!("{:x}", hash))
            }
        };
        if checksum == &hash {
            Ok(())
        } else {
            Err(ChecksumError::InvalidChecksum {
                expected: checksum.clone(),
                got: hash,
            })
        }
    }
}

#[derive(Debug, Clone, Copy, Display, PartialEq)]
pub enum Support {
    #[display(fmt = "Active support")]
    ActiveSupport,
    #[display(fmt = "Security fixes only")]
    SecurityFixesOnly,
    #[display(fmt = "End of life")]
    EndOfLife,
}

impl Release {
    fn source_file(&self, extention: &str) -> Option<&File> {
        self.source.iter().find_map(|source| match source {
            Source::File(file) if file.filename.ends_with(extention) => Some(file),
            _ => None,
        })
    }
    pub fn source_url(&self) -> (String, Option<&Hash>) {
        let source_file = self.source_file(".tar.gz").unwrap();
        let url = if self.museum == Some(true) {
            let major_version = self.version.unwrap().major_version();
            format!(
                "https://museum.php.net/php{}/{}",
                major_version, source_file.filename
            )
        } else {
            format!("https://www.php.net/distributions/{}", source_file.filename)
            // format!("http://jp1.php.net/get/{}/from/this/mirror/", filename)
        };
        (url, source_file.checksum.as_ref())
    }
    pub fn calculate_support(&self) -> Support {
        let release_date = self.date;
        let release_year = release_date.year();
        let active_support_deadline = release_date
            .with_year(release_year + 2)
            .unwrap_or_else(|| NaiveDate::from_yo(release_year + 1, 1).succ());
        let security_support_deadline = release_date
            .with_year(release_year + 3)
            .unwrap_or_else(|| NaiveDate::from_yo(release_year + 2, 1).succ());
        let today = Utc::now().naive_local().date();

        if today < active_support_deadline {
            Support::ActiveSupport
        } else if today < security_support_deadline {
            Support::SecurityFixesOnly
        } else {
            Support::EndOfLife
        }
    }
}

fn date_deserializer<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%d %B %Y").map_err(serde::de::Error::custom)
}

// fn option_date_deserializer<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
// where
//     D: de::Deserializer<'de>,
// {
//     #[derive(Deserialize)]
//     struct Helper(#[serde(deserialize_with = "date_deserializer")] NaiveDate);
//     let helper = Option::deserialize(deserializer)?;
//     Ok(helper.map(|Helper(o)| o))
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize() {
        let json = r#"
            {
                "8.1.1": {
                    "announcement": true,
                    "tags": [],
                    "date": "16 Dec 2021",
                    "source": [
                        {
                            "filename": "php-8.1.1.tar.gz",
                            "name": "PHP 8.1.1 (tar.gz)",
                            "sha256": "4e4cf3f843a5111f6c55cd21de8f26834ea3cd4a5be77c88357cbcec4a2d671d",
                            "date": "16 Dec 2021"
                        },
                        {
                            "filename": "php-8.1.1.tar.bz2",
                            "name": "PHP 8.1.1 (tar.bz2)",
                            "sha256": "8f8bc9cad6cd124edc111f7db0a109745e2f638770a101b3c22a2953f7a9b40e",
                            "date": "16 Dec 2021"
                        },
                        {
                            "filename": "php-8.1.1.tar.xz",
                            "name": "PHP 8.1.1 (tar.xz)",
                            "sha256": "33c09d76d0a8bbb5dd930d9dd32e6bfd44e9efcf867563759eb5492c3aff8856",
                            "date": "16 Dec 2021"
                        }
                    ]
                },
                "8.0.14": {
                    "announcement": true,
                    "tags": [],
                    "date": "16 Dec 2021",
                    "source": [
                        {
                            "filename": "php-8.0.14.tar.gz",
                            "name": "PHP 8.0.14 (tar.gz)",
                            "sha256": "e67ebd8c4c77247ad1fa88829e5b95d51a19edf3d87814434de261e20a63ea20",
                            "date": "16 Dec 2021"
                        },
                        {
                            "filename": "php-8.0.14.tar.bz2",
                            "name": "PHP 8.0.14 (tar.bz2)",
                            "sha256": "bb381fdf4817ad7c24c23ea7f77cad68dceb86eb3ac1a37acedadf8ad0a0cd4b",
                            "date": "16 Dec 2021"
                        },
                        {
                            "filename": "php-8.0.14.tar.xz",
                            "name": "PHP 8.0.14 (tar.xz)",
                            "sha256": "fbde8247ac200e4de73449d9fefc8b495d323b5be9c10cdb645fb431c91156e3",
                        "date": "16 Dec 2021"
                        }
                    ]
                }
            }
        "#;
        let resp: Result<Response, _> = serde_json::from_str(json);
        assert!(resp.is_ok());
        let resp = resp.unwrap();
        assert!(if let Response::Map(map) = resp {
            map.contains_key(&"8.1.1".parse().unwrap())
        } else {
            false
        });
    }
    #[test]
    fn fetch_all_major_test() {
        let releases = fetch_all("3".parse().unwrap());
        assert!(releases.is_ok());
        assert!(!releases.unwrap().is_empty());
        let releases = fetch_all("5".parse().unwrap());
        assert!(releases.is_ok());
        assert!(!releases.unwrap().is_empty());
        let releases = fetch_all("8".parse().unwrap());
        assert!(releases.is_ok());
        assert!(!releases.unwrap().is_empty());
    }
    #[test]
    fn fetch_latest_test() {
        let latest_release = fetch_latest("7.0".parse().unwrap());
        assert!(latest_release.is_ok());
        assert_eq!(
            latest_release.unwrap().version.unwrap(),
            "7.0.33".parse().unwrap()
        );
    }
}

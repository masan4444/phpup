use crate::curl;
use crate::version::Version;
use chrono::{Datelike, NaiveDate, Utc};
use derive_more::Display;
use serde::{de, Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("can't parse json: {0}")]
    JsonParseError(#[from] serde_json::error::Error),
    #[error("{0}")]
    ReceiveErrMsg(String),
}

fn fetch_and_parse(
    version: Option<Version>,
    max: Option<usize>,
) -> Result<BTreeMap<Version, Release>, Error> {
    let base_url = "https://www.php.net/releases/index.php";
    let query = format!(
        "?json=1{}{}",
        version.map_or("".to_owned(), |version| format!("&version={}", version)),
        max.map_or("".to_owned(), |max| format!("&max={}", max)),
    );
    let url = &format!("{}{}", base_url, query);
    // dbg!(url);
    let json = curl::get_as_slice(url);
    // println!("{:?}", std::str::from_utf8(&json));

    let resp: Response = serde_json::from_slice(&json).unwrap();
    match resp {
        Response::Map(releases) => Ok(releases),
        Response::One(release) => Ok([(release.version.unwrap(), release)].into_iter().collect()),
        Response::Error { msg } => Err(Error::ReceiveErrMsg(format!(
            "{}{}",
            msg,
            version.map_or("".to_owned(), |v| format!(": \"{}\"", v))
        ))),
    }
}

pub fn fetch_all(version: Version) -> Result<BTreeMap<Version, Release>, Error> {
    fetch_and_parse(Some(version), Some(1000))
}

pub fn fetch_latest(version: Version) -> Result<Release, Error> {
    let mut latest = fetch_and_parse(Some(version), None)?;
    let version = *latest.keys().next().unwrap();
    Ok(latest.remove(&version).unwrap())
}

pub fn fetch_oldest_patch(version: Version) -> Result<Release, Error> {
    let oldest_minor_version =
        Version::from_numbers(version.major_version(), version.minor_version(), Some(0));
    fetch_latest(oldest_minor_version)
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
    File {
        filename: String,
        name: String,
        #[serde(flatten)]
        checksum: Option<Hash>,
        // TODO: Option<NaiveTime>
        date: Option<String>,
    },
    Link {
        link: String,
        name: String,
    },
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "lowercase", serialize = "lowercase"))]
enum Hash {
    SHA256(String),
    MD5(String),
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
    fn source_filename(&self, extention: &str) -> String {
        self.source
            .iter()
            .find_map(|source| match source {
                Source::File { filename, .. } if filename.ends_with(extention) => Some(filename),
                _ => None,
            })
            .unwrap()
            .to_string()
    }
    pub fn source_url(&self) -> String {
        let filename = self.source_filename(".tar.gz");
        if self.museum == Some(true) {
            let major_version = self.version.unwrap().major_version();
            format!("https://museum.php.net/php{}/{}", major_version, filename)
        } else {
            format!("https://www.php.net/distributions/{}", filename)
        }
        // format!("http://jp1.php.net/get/{}/from/this/mirror/", filename)
    }
    pub fn calculate_support(&self) -> Support {
        let release_date = self.date;
        let release_year = release_date.year();
        let active_support_deadline = release_date
            .with_year(release_year + 2)
            .unwrap_or(NaiveDate::from_yo(release_year + 1, 1).succ());
        let security_support_deadline = release_date
            .with_year(release_year + 3)
            .unwrap_or(NaiveDate::from_yo(release_year + 2, 1).succ());
        let today = Utc::now().naive_local().date();

        if today < active_support_deadline {
            return Support::ActiveSupport;
        } else if today < security_support_deadline {
            return Support::SecurityFixesOnly;
        } else {
            return Support::EndOfLife;
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
    fn test() {
        use chrono::NaiveDate;
        let file = Source::File {
            filename: "ffff".to_owned(),
            name: "nn".to_owned(),
            checksum: Some(Hash::SHA256("33333333".to_owned())),
            date: None,
        };
        let announcement = Announcement::English(English {
            english: "aaaa".to_owned(),
        });
        let announcement2 = Announcement::Flag(true);
        let release = Release {
            announcement: Some(announcement2),
            tags: Some(vec![Tag::Security]),
            source: vec![],
            windows_binary: None,
            date: NaiveDate::from_ymd(2021, 9, 3),
            museum: None,
            version: None,
        };
        println!("{:#?}", serde_json::to_string(&file).unwrap());

        let json = serde_json::to_string(&release).unwrap();
        println!("{:#?}", json);

        let json = r#"{

          }"#;

        let release: Release = serde_json::from_str(&json).unwrap();
        println!("{:#?}", release);

        // let base_url = "https://www.php.net/releases/index.php";
        // let query = format!("?json=1&version=1",);
        // let url = &format!("{}{}", base_url, query);
        // // println!("{}", url);
        // let output = curl::get_as_slice(url);
        // let resp: Result<Response, _> = serde_json::from_slice(&output);
        // println!("{:?}", resp);
    }
}

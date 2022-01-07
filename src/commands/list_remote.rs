use crate::commands::list_remote::version::Version;

use super::Command;
use octocrab;
use std::cmp::Reverse;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ListRemote {
    version: Option<String>,
}

impl Command for ListRemote {
    fn run(&self) {
        // use tokio::runtime::Runtime;
        // Runtime::new()
        //     .unwrap()
        //     .block_on(fetch_versions_from_github());

        match &self.version {
            Some(version) => {
                let versions = php_net_release::fetch_all_versions(version);
                for v in versions {
                    println!("{}", v);
                }
            }
            None => {
                let mut versions = php_net_release::fetch_every_major_latest_version();
                versions.sort_by_key(|v| Reverse(*v));

                let column = 5;
                let row = 5;
                let mut iter = versions.iter();
                let mut major = 0;
                let mut minor = -1;

                for _ in 0..column {
                    if minor == -1 {
                        let minor_version = if let Some(minor_version) = iter.next() {
                            minor_version
                        } else {
                            break;
                        };
                        major = minor_version.major_version();
                        minor = minor_version.minor_version().unwrap() as isize;
                        if major == 3 {
                            println!("{}.{}: {:7}", major, minor, minor_version.to_string());
                            break;
                        }
                    }

                    let v = Version::from_numbers(major, Some(minor as usize), None);
                    print!("{}: ", v);
                    let patch_versions = php_net_release::fetch_all_versions(&format!("{}", v));
                    let len = patch_versions.len();
                    for v in &patch_versions[..row.min(len)] {
                        print!("{:7} ", v.to_string());
                    }
                    if row < len {
                        print!("...")
                    }
                    println!("");
                    minor -= 1;
                }
                if iter.next().is_some() {
                    println!(" .");
                    println!(" .");
                }
            }
        }
    }
}

pub mod php_net_release {
    use crate::commands::list_remote::http;
    use crate::commands::list_remote::version;
    use serde::{Deserialize, Serialize};
    use std::cmp::Reverse;
    use std::collections::HashMap;

    pub fn fetch_releases(version: Option<&str>, max: Option<usize>) -> HashMap<String, Response> {
        let base_url = "https://www.php.net/releases/index.php";
        let query = format!(
            "?json=1{}{}",
            version.map_or("".to_string(), |version| format!("&version={}", version)),
            max.map_or("".to_string(), |max| format!("&max={}", max)),
        );
        let url = &format!("{}{}", base_url, query);

        // println!("{}", url);

        let output = http::get_as_slice(url);
        // println!("{}", std::str::from_utf8(&output).unwrap());

        if version.is_some() && max.is_none() {
            let mut releases: HashMap<String, Response> = HashMap::new();
            releases.insert(
                version.unwrap().to_string(),
                serde_json::from_slice(&output).unwrap(),
            );
            releases
        } else {
            serde_json::from_slice(&output).unwrap()
        }
    }

    pub fn fetch_every_major_latest_version() -> Vec<version::Version> {
        let releases = fetch_releases(None, None);
        releases
            .values()
            .map(|release| (release.version.as_ref().unwrap()).try_into().unwrap())
            .collect()
    }

    pub fn fetch_all_versions(version: &str) -> Vec<version::Version> {
        let releases = fetch_releases(Some(version), Some(1000));
        let mut versions: Vec<version::Version> =
            releases.keys().map(|key| key.try_into().unwrap()).collect();
        versions.sort_by_key(|v| Reverse(*v));
        versions
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Response {
        announcement: Option<Announcement>,
        tags: Option<Vec<Tag>>,
        source: Vec<Source>,
        #[serde(rename = "windows")]
        windows_binary: Option<Vec<Source>>,
        date: String,
        museum: Option<bool>,
        version: Option<String>,
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
            hash: Option<Hash>,
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

    impl Response {
        // TODO: windows php-3
        pub fn source_version(&self) -> version::Version {
            self.source
                .iter()
                .find_map(|src| match src {
                    Source::File { filename, .. } => Some(filename),
                    _ => None,
                })
                .unwrap()
                .try_into()
                .unwrap()
        }
    }

    pub fn test() {
        let file = Source::File {
            filename: "ffff".to_string(),
            name: "nn".to_string(),
            hash: Some(Hash::SHA256("33333333".to_string())),
            date: Some("ddddd".to_string()),
        };
        let announcement = Announcement::English(English {
            english: "aaaa".to_string(),
        });
        let announcement2 = Announcement::Flag(true);
        let response = Response {
            announcement: Some(announcement2),
            tags: None,
            source: vec![],
            windows_binary: None,
            date: "dddddd".to_string(),
            museum: None,
            version: None,
        };
        println!("{:#?}", serde_json::to_string(&file).unwrap());

        let json = serde_json::to_string(&response).unwrap();
        println!("{:#?}", json);

        let json = r#"{
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
            ],
            "version": "8.1.1"
          }"#;

        let resp: Response = serde_json::from_str(&json).unwrap();
        println!("{:#?}", resp);
    }
}

async fn fetch_versions_from_github() {
    let instance = octocrab::instance();

    let page = instance
        .repos("php", "php-src")
        .list_tags()
        .per_page(100u8)
        .send()
        .await
        .unwrap();
    println!("number of pages: {}", page.number_of_pages().unwrap());

    let tags = instance
        .all_pages::<octocrab::models::repos::Tag>(page)
        .await
        .unwrap();
    println!("number of tags: {}", tags.len());

    let tag_names = tags
        .iter()
        .map(|tag| tag.name.clone())
        .collect::<Vec<String>>();

    for tag_name in &tag_names {
        println!("{}", tag_name)
    }
}

pub mod version {
    use regex::Regex;
    use std::fmt;

    pub type Version = Major;
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Major {
        version: usize,
        minor: Option<Minor>,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Minor {
        version: usize,
        patch: Option<Patch>,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Patch {
        version: usize,
        pre: Option<Pre>,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Pre {
        version: usize,
        pre_type: PreType,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum PreType {
        Alpha,
        Beta,
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

        fn major(self) -> Major {
            self
        }
        fn minor(self) -> Option<Minor> {
            self.minor
        }
        fn patch(self) -> Option<Patch> {
            self.minor().map(|minor| minor.patch).flatten()
        }
        fn pre(self) -> Option<Pre> {
            self.patch().map(|patch| patch.pre).flatten()
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
    }

    impl TryFrom<&str> for Version {
        type Error = ();
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            if value == "3.0.x (latest)" {
                return Ok(Version::from_numbers(
                    3,
                    Some(9),
                    Some(if cfg!(target_os = "windows") { 17 } else { 18 }),
                ));
            }
            let re: Regex = Regex::new(
                r"(?x)
                ^ # start
                (\d+) # major
                    (?: \.(\d+) # minor
                        (?: \.(\d+) # patch
                            (?: (alpha|beta|RC)(\d+) # pre
                            )?
                        )?
                    )?
                $ # end
                ",
            )
            .unwrap();

            let cap = re.captures(value).ok_or(())?;
            let to_num = |m: regex::Match| m.as_str().parse().unwrap();
            let major = Major {
                version: to_num(cap.get(1).unwrap()),
                minor: cap.get(2).map(to_num).map(|version| Minor {
                    version,
                    patch: cap.get(3).map(to_num).map(|version| Patch {
                        version,
                        pre: cap.get(5).map(to_num).map(|version| Pre {
                            version,
                            pre_type: match &cap[4] {
                                "alpha" => PreType::Alpha,
                                "beta" => PreType::Beta,
                                _ => PreType::Rc,
                            },
                        }),
                    }),
                }),
            };

            Ok(major)
        }
    }

    impl TryFrom<&String> for Version {
        type Error = ();
        fn try_from(value: &String) -> Result<Self, Self::Error> {
            Version::try_from(value as &str)
        }
    }

    impl Into<String> for &Version {
        fn into(self) -> String {
            format!(
                "{}{}{}{}{}",
                self.major_version(),
                self.minor_version()
                    .map_or("".to_string(), |v| format!(".{}", v)),
                self.patch_version()
                    .map_or("".to_string(), |v| format!(".{}", v)),
                self.pre_type().map_or("".to_string(), |t| format!(
                    "{}",
                    match t {
                        PreType::Alpha => "alpha",
                        PreType::Beta => "beta",
                        PreType::Rc => "RC",
                    }
                )),
                self.pre_version()
                    .map_or("".to_string(), |v| format!(".{}", v)),
            )
        }
    }

    impl fmt::Display for Version {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let s: String = self.into();
            write!(f, "{}", s)
        }
    }

    pub fn test(version: &str) {
        let version: Result<Version, _> = version.try_into();
        println!("{:?}", version.unwrap());
    }
}

pub mod http {
    use std::process::Command;

    const CURL_PATH: &str = if cfg!(target_os = "windows") {
        "curl.exe"
    } else {
        "curl"
    };

    pub fn get_as_slice(url: &str) -> Vec<u8> {
        Command::new(CURL_PATH).arg(url).output().unwrap().stdout
    }
}

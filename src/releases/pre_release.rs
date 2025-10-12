use super::{FetchError, Hash};
use crate::curl;
use crate::version::Version;
use serde::{de, Deserialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct PreRelease {
    pub version: Version,
    baseurl: String,
    #[serde(deserialize_with = "hash_de", alias = "sha256_gz")]
    checksum: Hash,
}

fn hash_de<'de, D>(deserializer: D) -> Result<Hash, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Hash::SHA256(s))
}

#[derive(Deserialize, Debug)]
pub struct PreReleaseMap(HashMap<Version, PreRelease>);

impl PreReleaseMap {
    pub fn get_versions_included_by<'a>(&'a self, version: &'a Version) -> impl Iterator<Item = &'a Version> {
        self.0.values().filter_map(|v| {
            if version.includes(&v.version) {
                Some(&v.version)
            } else {
                None
            }
        })
    }

    pub fn get(&self, version: &Version) -> Option<&PreRelease> {
        let key = Self::build_key_from_pre_release_version(version);

        match self.0.get(&key) {
            Some(pr) if &pr.version == version => Some(pr),
            _ => None
        }
    }

    pub fn remove(&mut self, version: &Version) -> Option<PreRelease> {
        let key = Self::build_key_from_pre_release_version(&version);

        match self.0.get(&key) {
            Some(pr) if &pr.version == version => self.0.remove(&key),
            _ => None
        }
    }

    fn build_key_from_pre_release_version(version: &Version) -> Version {
        assert!(
            version.pre_type().is_some(),
            "Version {} is not pre-release",
            version
        );

        Version::from_numbers(
            version.major_version(),
            version.minor_version(),
            version.patch_version(),
            None,
        )
    }
}

#[derive(Deserialize, Debug)]
struct Response {
    releases: PreReleaseMap,
}

pub fn fetch_all() -> Result<PreReleaseMap, FetchError> {
    let url = "https://www.php.net/release-candidates.php?format=json";
    let json = curl::get_as_slice(url)?;

    let resp: Response =
        serde_json::from_slice(&json).unwrap_or_else(|_| panic!("Can't parse json from {}", url));

    Ok(resp.releases)
}

pub fn fetch(version: Version) -> Result<PreRelease, FetchError> {
    let mut releases = fetch_all()?;

    releases.remove(&version).ok_or(FetchError::NotFoundRelease(version))
}

impl PreRelease {
    pub fn source_url(&self) -> (String, Option<&Hash>) {
        (
            format!("{}php-{}.tar.gz", self.baseurl, self.version),
            Some(&self.checksum),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::semantic::PreType;

    #[test]
    fn deserialize() {
        let json = r#"{
            "reported": [
                "8.1.27-dev",
                "8.2.27-dev",
                "8.3.26-dev",
                "8.4.13-dev",
                "8.5.0-dev",
                "8.5.0RC1"
            ],
            "releases": {
                "8.5.0": {
                    "type": "RC",
                    "number": 1,
                    "sha256_bz2": "8365ae9263cc160e6182302f0bdcc80edf1806029556e6870beb3078a625389c",
                    "sha256_gz": "0ea5059a387117fe6ed9a72cdc20945dbff6acc072df936e97d35a9cb26420e0",
                    "sha256_xz": "96f064b5d604e00e5fe1c993d4881da659a99d6d1d8ac0b1df8fec6406e34a9d",
                    "date": "25 Sep 2025",
                    "baseurl": "https://downloads.php.net/~daniels/",
                    "version": "8.5.0RC1",
                    "files": {
                        "bz2": {
                            "sha256": "8365ae9263cc160e6182302f0bdcc80edf1806029556e6870beb3078a625389c",
                            "path": "https://downloads.php.net/~daniels/php-8.5.0RC1.tar.bz2"
                        },
                        "gz": {
                            "sha256": "0ea5059a387117fe6ed9a72cdc20945dbff6acc072df936e97d35a9cb26420e0",
                            "path": "https://downloads.php.net/~daniels/php-8.5.0RC1.tar.gz"
                        },
                        "xz": {
                            "sha256": "96f064b5d604e00e5fe1c993d4881da659a99d6d1d8ac0b1df8fec6406e34a9d",
                            "path": "https://downloads.php.net/~daniels/php-8.5.0RC1.tar.xz"
                        }
                    }
                }
            }
        }"#;

        let resp: Result<Response, _> = serde_json::from_str(json);
        assert!(resp.is_ok());

        let resp = resp.unwrap();
        let key = "8.5.0RC1".parse().unwrap();

        let pre_release = resp.releases.get(&key).unwrap();
        assert_eq!(
            pre_release.version,
            Version::from_numbers(8, Some(5), Some(0), Some((PreType::Rc, 1)))
        );
        assert_eq!(
            pre_release.source_url().0,
            "https://downloads.php.net/~daniels/php-8.5.0RC1.tar.gz"
        );

        assert!(match pre_release.source_url().1 {
            Some(Hash::SHA256(val)) =>
                val == "0ea5059a387117fe6ed9a72cdc20945dbff6acc072df936e97d35a9cb26420e0",
            _ => false,
        })
    }
}

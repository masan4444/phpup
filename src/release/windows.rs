#![cfg(windows)]

use regex::{Captures, Regex};
use std::collections::BTreeMap;

use crate::curl;
use crate::version::Version;

const ARCH: &str = if cfg!(target_arch = "x86") {
    "x86"
} else if cfg!(target_arch = "x86_64") {
    "x64"
} else {
    panic!()
};
const BASE_URL: &str = "https://windows.php.net/downloads/releases";
const NUMBER_REGEX: &str = r"[1-9]+\d*|0";

fn build_regex(version: Option<Version>) -> Regex {
    let major_regex = version
        .map(|v| v.major_version().to_string())
        .unwrap_or(NUMBER_REGEX.to_string());
    let minor_regex = version
        .and_then(|v| v.minor_version())
        .map(|minor| minor.to_string())
        .unwrap_or(NUMBER_REGEX.to_string());
    let patch_regex = version
        .and_then(|v| v.patch_version())
        .map(|patch| patch.to_string())
        .unwrap_or(NUMBER_REGEX.to_string());
    Regex::new(&format!(
        r"(?x)
            (
                php-((?:{})\.(?:{})\.(?:{}))
                (?:\-(nts))?
                \-Win32
                \-(VC|vs)(\d+)
                \-({})
                \.zip
            )</A>
        ",
        major_regex, minor_regex, patch_regex, ARCH
    ))
    .unwrap()
}

pub fn fetch(version: Option<Version>) -> Result<BTreeMap<Version, Release>, super::FetchError> {
    let regex = build_regex(version);

    let past_url = &format!("{}{}", BASE_URL, "/archives/");
    let utf8 = curl::get_as_slice(past_url)?;
    let html = std::str::from_utf8(&utf8).unwrap();
    let past_releases = regex.captures_iter(html);

    let latest_url = &format!("{}{}", BASE_URL, "/");
    let utf8 = curl::get_as_slice(latest_url)?;
    let html = std::str::from_utf8(&utf8).unwrap();
    let latest_releases = regex.captures_iter(html);

    let releases: BTreeMap<Version, Release> = past_releases
        .chain(latest_releases)
        .map(|cap| Release::from(cap))
        .map(|release| (release.version, release))
        .collect();

    if let Some(version) = version {
        if releases.len() == 0 {
            return Err(super::FetchError::NotFoundRelease(version));
        }
    }

    Ok(releases)
}

#[derive(Debug)]
pub struct Release {
    pub version: Version,
    pub thread_safe: bool,
    pub compiler_version: CompilerVersion,
    pub arch: Arch,
    pub filename: String,
}

#[derive(Debug)]
pub enum CompilerVersion {
    VisualCpp(usize),
    VisualStudio(usize),
}

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Arch {
    x86,
    x64,
}

impl From<Captures<'_>> for Release {
    fn from(cap: Captures) -> Self {
        let compiler_version = cap.get(5).unwrap().as_str().parse().unwrap();
        let compiler_version = match cap.get(4).unwrap().as_str() {
            "VC" => CompilerVersion::VisualCpp(compiler_version),
            "vs" => CompilerVersion::VisualStudio(compiler_version),
            _ => panic!(),
        };
        let arch = match cap.get(6).unwrap().as_str() {
            "x86" => Arch::x86,
            "x64" => Arch::x64,
            _ => panic!(),
        };
        Release {
            version: cap.get(2).unwrap().as_str().parse().unwrap(),
            thread_safe: cap.get(3).is_none(),
            compiler_version,
            arch,
            filename: cap.get(1).unwrap().as_str().to_owned(),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test() {
        let query_version: Version = "8.0.0".parse().unwrap();
        let release = fetch(Some(query_version)).unwrap();

        assert_eq!(release.keys().into_iter().next(), Some(&query_version))
    }
}

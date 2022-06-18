pub mod unix;
mod windows;

use crate::curl;
use crate::version::Version;
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("Can't find releases that matches {0}")]
    NotFoundRelease(Version),

    #[error(transparent)]
    CurlError(#[from] curl::Error),
}

#[cfg(windows)]
pub fn fetch_all() -> Result<BTreeMap<Version, windows::Release>, FetchError> {
    windows::fetch(None)
}
#[cfg(windows)]
pub fn fetch(version: Version) -> Result<BTreeMap<Version, windows::Release>, FetchError> {
    windows::fetch(Some(version))
}
#[cfg(windows)]
pub fn fetch_latest(version: Version) -> Result<windows::Release, FetchError> {
    windows::fetch(Some(version)).map(|rs| rs.into_iter().last().unwrap().1)
}

#[cfg(unix)]
pub fn fetch_all() -> Result<BTreeMap<Version, windows::Release>, FetchError> {
    let versions = vec![
        Version::from_major(3),
        Version::from_major(4),
        Version::from_major(5),
        Version::from_major(7),
        Version::from_major(8),
    ];
    versions.map(|version| unix::fetch_all(version))
}
#[cfg(unix)]
pub fn fetch(version: Version) -> Result<BTreeMap<Version, windows::Release>, FetchError> {
    windows::fetch(Some(version))
}
#[cfg(unix)]
pub fn fetch_latest(version: Version) -> Result<windows::Release, FetchError> {
    windows::fetch(Some(version)).map(|rs| rs.into_iter().last().unwrap().1)
}

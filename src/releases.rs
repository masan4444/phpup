use self::{hash::Hash, pre_release::PreRelease, release::Release};
use crate::curl;
use crate::version::Version;
use thiserror::Error;

pub mod hash;
pub mod pre_release;
pub mod release;

pub enum Installable {
    Stable(Release),
    Pre(PreRelease),
}

impl Installable {
    pub fn fetch(version: Version) -> Result<Self, FetchError> {
        match version.pre_type() {
            Some(_) => {
                let pre_release = pre_release::fetch(version)?;
                Ok(Self::Pre(pre_release))
            }
            None => {
                let release = release::fetch_latest(version)?;
                Ok(Self::Stable(release))
            }
        }
    }

    pub fn version(&self) -> Version {
        match self {
            Self::Stable(release) => release.version.unwrap(),
            Self::Pre(pre_release) => pre_release.version,
        }
    }

    pub fn source_url(&self) -> (String, Option<&Hash>) {
        match self {
            Self::Stable(release) => release.source_url(),
            Self::Pre(pre_release) => pre_release.source_url(),
        }
    }
}

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("Can't find releases that matches {0}")]
    NotFoundRelease(Version),

    #[error(transparent)]
    CurlError(#[from] curl::Error),

    #[error("Receive error message from release site: {0}")]
    Other(String),
}

#[derive(Error, Debug)]
pub enum ChecksumError {
    #[error("Invalid checksum\nexptected: {expected}\ngot: {got}")]
    InvalidChecksum { expected: String, got: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

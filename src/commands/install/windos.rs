#![cfg(windows)]

use crate::commands::{Command, Config};
use crate::curl;
use crate::release;
use crate::version::{self, Version};
use clap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Install {
    version: Option<Version>,

    #[clap(flatten)]
    version_file: version::File,

    /// Specify configure options used by the PHP configure scripts.
    /// To specify two or more options, enclose them with quotation marks.
    #[clap(long, env = "PHPUP_CONFIGURE_OPTS", allow_hyphen_values = true)]
    configure_opts: Option<String>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't detect a version: {0}")]
    NoVersionFromFile(#[from] version::file::Error),

    #[error("Can't specify the system version by {0}")]
    SpecifiedSystemVersion(PathBuf),

    #[error(transparent)]
    FailedFetchRelease(#[from] release::FetchError),

    #[error(transparent)]
    FailedDownload(#[from] curl::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Command for Install {
    type Error = Error;

    fn run(&self, _config: &Config) -> Result<(), Self::Error> {
        todo!()
    }
}

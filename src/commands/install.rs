mod make;
mod progress_reader;

use super::{Command, Config};
use crate::curl;
use crate::decorized::Decorized;
use crate::release;
use crate::version::{self, Version};
use clap;
use colored::Colorize;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;
use progress_reader::ProgressReader;
use std::fs;
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};
use tar::Archive;
use thiserror::Error;

static PROGRESS_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    let progress_template =
        "{prefix:>12.cyan.bold} [{bar:25}] {bytes}/{total_bytes} ({eta}) {wide_msg}";
    ProgressStyle::default_bar()
        .template(progress_template)
        .progress_chars("=> ")
});

#[derive(clap::Parser, Debug)]
pub struct Install {
    version: Option<Version>,

    #[clap(flatten)]
    version_file: version::File,
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
    FailedMake(#[from] make::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Command for Install {
    type Error = Error;

    fn run(&self, config: &Config) -> Result<(), Error> {
        let request_version = self
            .version
            .map_or_else(|| self.get_version_from_version_file(), Ok)?;

        let release = release::fetch_latest(request_version)?;
        let install_version = release.version.unwrap();

        if version::latest_installed_by(&request_version, config) == Some(install_version) {
            println!(
                "{}: Already installed {}",
                "warning".yellow().bold(),
                install_version.decorized_with_prefix()
            );
            return Ok(());
        }
        println!(
            "{:>12} {}",
            "Installing".green().bold(),
            install_version.decorized_with_prefix()
        );

        let install_dir = config.versions_dir().join(install_version.to_string());
        let download_dir = tempfile::Builder::new()
            .prefix(".downloads-")
            .tempdir_in(&config.base_dir())?;

        let (tar_gz, file_size) = download(&release.source_url(), &download_dir)?;
        let source_dir = unpack(&tar_gz, file_size, &download_dir)?;
        build(&source_dir, &install_dir).unwrap();
        println!(
            "{:>12} {}",
            "Installed".green().bold(),
            install_dir.display().decorized()
        );
        Ok(())
    }
}

impl Install {
    fn get_version_from_version_file(&self) -> Result<Version, Error> {
        let version_info = self.version_file.get_version_info()?;
        if let Some(version) = version_info.version.as_version() {
            println!(
                "{} has been specified from {}",
                version.decorized(),
                version_info.filepath.display().decorized()
            );
            Ok(version)
        } else {
            Err(Error::SpecifiedSystemVersion(version_info.filepath))
        }
    }
}

// TODO: checksum
fn download(url: &str, dir: impl AsRef<Path>) -> Result<(PathBuf, u64), Error> {
    let curl::Header { content_length } = curl::get_header(url)?;
    let progress_bar = ProgressBar::new(content_length.unwrap() as u64)
        .with_style(PROGRESS_STYLE.clone())
        .with_prefix("Downloading")
        .with_message(url.to_owned());

    let (stdout, mut stderr) = curl::get_as_reader(url)?;
    let mut progress_reader = ProgressReader::new(stdout, &progress_bar);

    let download_file_path = dir.as_ref().join(url.rsplit('/').next().unwrap());
    let download_file = fs::File::create(&download_file_path)?;
    let mut file_writer = BufWriter::new(&download_file);

    std::io::copy(&mut progress_reader, &mut file_writer)?;

    let download_size = download_file.metadata()?.len();
    if download_size > 0 {
        progress_bar.finish_and_clear();
        println!("{:>12} {}", "Downloaded".green().bold(), url);
        Ok((download_file_path, download_size))
    } else {
        let mut err_msg = String::new();
        stderr.read_to_string(&mut err_msg)?;
        Err(Error::FailedDownload(curl::Error::ExitFailed(
            "curl".to_owned(),
            err_msg,
        )))
    }
}

fn unpack(
    tar_gz: impl AsRef<Path>,
    file_size: u64,
    dst_dir: impl AsRef<Path>,
) -> Result<PathBuf, Error> {
    let progress_bar = ProgressBar::new(file_size)
        .with_style(PROGRESS_STYLE.clone())
        .with_prefix("Unpacking")
        .with_message(tar_gz.as_ref().to_str().unwrap().to_owned());

    let file_reader = BufReader::new(fs::File::open(&tar_gz)?);
    let progress_reader = ProgressReader::new(file_reader, &progress_bar);
    let gz_decoder = GzDecoder::new(progress_reader);
    let mut tar_archive = Archive::new(gz_decoder);

    tar_archive.unpack(&dst_dir)?;
    progress_bar.finish_and_clear();

    println!(
        "{:>12} {}",
        "Unpacked".green().bold(),
        tar_gz.as_ref().display()
    );
    let tar_gz_filename = tar_gz.as_ref().file_name().unwrap().to_str().unwrap();
    let unpaked_dirname = &tar_gz_filename[..tar_gz_filename.len() - ".tar.gz".len()];
    Ok(dst_dir.as_ref().join(unpaked_dirname))
}

#[cfg(unix)]
fn build(src_dir: impl AsRef<Path>, dist_dir: impl AsRef<Path>) -> Result<(), Error> {
    use make::Command;

    println!(
        "{:>12} {}",
        "Building".cyan().bold(),
        src_dir.as_ref().display()
    );
    let current_dir = src_dir.as_ref();

    make::Configure {
        current_dir,
        dist_dir: dist_dir.as_ref(),
    }
    .run(1)?;
    make::Make { current_dir }.run(2)?;
    make::Install { current_dir }.run(3)?;
    Ok(())
}

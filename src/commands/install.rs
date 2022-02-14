use super::{Command, Config};
use crate::curl;
use crate::decorized::Decorized;
use crate::release;
use crate::version::{self, Version};
use clap;
use colored::Colorize;
use flate2::read::GzDecoder;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use tar::Archive;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Install {
    version: Option<Version>,

    #[clap(flatten)]
    version_file: version::File,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    FailedFetchRelease(#[from] release::FetchError),

    #[error("Can't detect a version: {0}")]
    NoVersionFromFile(#[from] version::file::Error),

    #[error("Can't specify the system version by {0}")]
    SpecifiedSystemVersion(PathBuf),
}

impl Command for Install {
    type Error = Error;

    fn run(&self, config: &Config) -> Result<(), Error> {
        let request_version = self
            .version
            .map_or_else(|| self.get_version_from_version_file(), Ok)?;

        let release = release::fetch_latest(request_version)?;
        let install_version = release.version.unwrap();
        let url = release.source_url();

        if version::latest_installed_by(&request_version, config) == Some(install_version) {
            println!(
                "{}: Already installed {}",
                "warning".yellow().bold(),
                install_version.decorized_with_prefix()
            );
            return Ok(());
        }

        // .phpup/versions/php/3.1.4/.downloads-aaa/php-3.1.4
        //                    |     |              |        |
        //         versions_dir     |              |        |
        //                install_dir              |        |
        //                              download_dir        |
        //                                         source_dir
        //
        // .phpup/versions/php/3.1.4/{bin,include,lib,php,var}

        let install_dir = config.versions_dir().join(install_version.to_string());
        fs::create_dir_all(&install_dir).unwrap();
        println!(
            "{} {}",
            "Installing".green().bold(),
            install_version.decorized_with_prefix()
        );

        let download_dir = tempfile::Builder::new()
            .prefix(".download-")
            .tempdir_in(&install_dir)
            .expect("Can't create a temporary directory to download to");
        println!("{} {}", "Downloading".green().bold(), url);
        Self::download_and_unpack(&url, &download_dir);

        let source_dir = fs::read_dir(&download_dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        println!(
            "{} {}",
            "Building from".green().bold(),
            source_dir.display().decorized()
        );
        Self::build(&source_dir, &install_dir).unwrap();
        println!(
            "{} {}",
            "Installed to".green().bold(),
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

    // TODO: checksum
    fn download_and_unpack(url: &str, path: impl AsRef<Path>) {
        let response = curl::get_as_reader(url);
        let gz_decoder = GzDecoder::new(response);
        let mut tar_archive = Archive::new(gz_decoder);
        tar_archive.unpack(path).unwrap();
    }

    #[cfg(unix)]
    fn build(src_dir: impl AsRef<Path>, dist_dir: impl AsRef<Path>) -> Result<(), ()> {
        let configure = vec![
            "./configure".to_owned(),
            format!("--prefix={}", dist_dir.as_ref().display()),
        ];
        println!("[1/3] {}", configure.join(" "));
        let configure_output = process::Command::new(&configure[0])
            .args(&configure[1..])
            .current_dir(&src_dir)
            .output()
            .unwrap();
        if !configure_output.status.success() {
            println!(
                "configure failed: {}",
                String::from_utf8_lossy(&configure_output.stderr)
            );
            return Err(());
        };

        let make = vec![
            "make".to_owned(),
            "-j".to_owned(),
            num_cpus::get().to_string(),
        ];
        println!("[2/3] {}", make.join(" "));
        let make_output = process::Command::new(&make[0])
            .args(&make[1..])
            .current_dir(&src_dir)
            .output()
            .unwrap();
        if !make_output.status.success() {
            println!(
                "make failed: {}",
                String::from_utf8_lossy(&make_output.stderr)
            );
            return Err(());
        };

        let install = vec!["make".to_owned(), "install".to_owned()];
        println!("[3/3] {}", install.join(" "));
        let install_output = process::Command::new(&install[0])
            .args(&install[1..])
            .current_dir(&src_dir)
            .output()
            .unwrap();
        if !install_output.status.success() {
            println!(
                "make install: {}",
                String::from_utf8_lossy(&install_output.stderr)
            );
            return Err(());
        };
        Ok(())
    }
}

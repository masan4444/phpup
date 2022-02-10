use super::{Command, Config};
use crate::curl;
use crate::decorized::Decorized;
use crate::release;
use crate::version::Version;
use crate::version_file::{self, VersionFile};
use clap;
use colored::Colorize;
use flate2::read::GzDecoder;
use std::fs;
use std::path::Path;
use std::process;
use tar::Archive;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Install {
    version: Option<Version>,

    #[clap(flatten)]
    version_file: VersionFile,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    CantFetchReleaseError(#[from] release::Error),

    #[error("Can't detect a version: {0}")]
    NoVersionFromFileError(#[from] version_file::Error),
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

        if config.latest_local_version_included_in(&request_version) == Some(install_version) {
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
        let (version, version_file_path) = self.version_file.get_version()?;
        println!(
            "{} has been specified from {}",
            version.decorized(),
            version_file_path.display().decorized()
        );
        Ok(version)
    }

    // TODO: checksum
    fn download_and_unpack(url: &str, path: impl AsRef<Path>) {
        let response = curl::get_as_reader(url);
        let gz_decoder = GzDecoder::new(response);
        let mut tar_archive = Archive::new(gz_decoder);
        tar_archive.unpack(path).unwrap();
    }

    fn build(src_dir: impl AsRef<Path>, dist_dir: impl AsRef<Path>) -> Result<(), ()> {
        let mut command = process::Command::new("sh");

        println!("./configure");
        command
            .arg("configure")
            .arg(format!("--prefix={}", dist_dir.as_ref().to_str().unwrap()));
        // .args(configure_opts);

        let configure = command.current_dir(&src_dir).output().unwrap();
        if !configure.status.success() {
            println!(
                "configure failed: {}",
                String::from_utf8_lossy(&configure.stderr).to_string()
            );
            return Err(());
        };

        println!("./make");
        let make = process::Command::new("make")
            .arg("-j")
            .arg(num_cpus::get().to_string())
            .current_dir(&src_dir)
            .output()
            .unwrap();
        if !make.status.success() {
            println!(
                "make failed: {}",
                String::from_utf8_lossy(&make.stderr).to_string()
            );
            return Err(());
        };

        println!("./make install");
        let make_install = process::Command::new("make")
            .arg("install")
            .current_dir(&src_dir)
            .output()
            .unwrap();
        if !make_install.status.success() {
            println!(
                "make install: {}",
                String::from_utf8_lossy(&make_install.stderr).to_string()
            );
            return Err(());
        };
        Ok(())
    }
}

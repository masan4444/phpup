use super::{Command, Config};
use crate::curl;
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
        let versions_dir = config.versions_dir();

        let request_version = self
            .version
            .unwrap_or(self.get_version_from_version_file()?);

        let release = release::fetch_latest(request_version)?;
        let install_version = release.version.unwrap();
        let url = release.source_url();

        if let Some(installed_version) = config.latest_local_version_included_in(&request_version) {
            if installed_version == install_version {
                println!(
                    "{}: Already installed PHP {}",
                    "warning".yellow().bold(),
                    installed_version.to_string().cyan()
                );
                return Ok(());
            }
        }

        // .phpup/versions/php/3.1.4/.downloads-aaa/php-3.1.4
        //                    |     |              |        |
        //         versions_dir     |              |        |
        //                install_dir              |        |
        //                              download_dir        |
        //                                         source_dir
        //
        // .phpup/versions/php/3.1.4/{bin,include,lib,php,var}

        let install_dir = versions_dir.join(install_version.to_string());
        fs::create_dir_all(&install_dir).unwrap();
        println!(
            "{} {}",
            "Installing".green().bold(),
            install_version.to_string().cyan()
        );

        let download_dir = tempfile::Builder::new()
            .prefix(".download-")
            .tempdir_in(&install_dir)
            .expect("Can't create a temporary directory to download to");
        println!("{} {}", "Downloading".green().bold(), url);
        println!("  to {} ...", download_dir.path().to_string_lossy());
        Self::download_and_unpack(&url, &download_dir);

        let source_dir = fs::read_dir(&download_dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        println!(
            "{} {} ...",
            "Buiding from".green().bold(),
            source_dir.to_string_lossy()
        );
        Self::build(&source_dir, &install_dir).unwrap();
        println!(
            "{} {}",
            "Installed to".green().bold(),
            install_dir.to_string_lossy()
        );
        Ok(())
    }
}

impl Install {
    fn get_version_from_version_file(&self) -> Result<Version, Error> {
        let (version, version_file_path) = self.version_file.get_version()?;
        println!(
            "Detected {} from {:?}",
            version.to_string().cyan(),
            version_file_path
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

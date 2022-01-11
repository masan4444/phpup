use super::{Command, Config};
use crate::curl;
use crate::release;
use crate::version::Version;
use colored::Colorize;
use flate2::read::GzDecoder;
use std::fs;
use std::path::Path;
use std::process;
use structopt::StructOpt;
use tar::Archive;

#[derive(StructOpt, Debug)]
pub struct Install {
    version: Option<Version>,
}

impl Command for Install {
    fn run(&self, config: &Config) -> anyhow::Result<()> {
        let versions_dir = config.versions_dir();
        let local_versions = config.local_versions();

        match &self.version {
            Some(version) => {
                if local_versions.contains(&version) {
                    println!(
                        "{}: Already installed {}",
                        "warning".yellow().bold(),
                        version.to_string()
                    );
                    return Ok(());
                }

                let release = release::fetch_latest(*version)?;
                let install_version = release.version.unwrap();
                println!("Installing {}...", install_version);
                let url = release.source_url();

                let install_dir = versions_dir.join(install_version.to_string());
                fs::create_dir_all(&install_dir).unwrap();
                let download_dir = tempfile::Builder::new()
                    .prefix(".download-")
                    .tempdir_in(&install_dir)?;
                println!("Downloading {}...", url);
                Self::download_and_unpack(&url, &download_dir);

                let source_dir = fs::read_dir(&download_dir.path())
                    .unwrap()
                    .next()
                    .unwrap()
                    .unwrap()
                    .path();
                println!("Buiding...");
                Self::build(source_dir, install_dir).unwrap();
            }
            None => {}
        };
        Ok(())
    }
}

impl Install {
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

use super::{Command, Config};
use crate::curl;
use crate::decorized::Decorized;
use crate::release;
use crate::version::{self, Version};
use clap;
use colored::Colorize;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::{Path, PathBuf};
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

    #[error(transparent)]
    FailedDownload(#[from] curl::Error),
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
            "{:>12} {}",
            "Installing".green().bold(),
            install_version.decorized_with_prefix()
        );

        let download_dir = tempfile::Builder::new()
            .prefix(".download-")
            .tempdir_in(&install_dir)
            .expect("Can't create a temporary directory to download to");
        download_and_unpack(&url, &download_dir)?;

        let source_dir = fs::read_dir(&download_dir.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();

        build(&source_dir, &install_dir).unwrap();
        println!(
            "{:>12} {}",
            "Installed".green().bold(),
            install_dir.display().decorized()
        );
        Ok(())
    }
}

use std::io::Read;
pub struct ProgressReader<R: Read, C: FnMut(usize)> {
    reader: R,
    callback: C,
}
impl<R: Read, C: FnMut(usize)> ProgressReader<R, C> {
    pub fn new(reader: R, callback: C) -> Self {
        Self { reader, callback }
    }
}
impl<R: Read, C: FnMut(usize)> Read for ProgressReader<R, C> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.reader.read(buf)?;
        (self.callback)(read);
        Ok(read)
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
fn download_and_unpack(url: &str, path: impl AsRef<Path>) -> Result<(), Error> {
    let (stdout, _) = curl::get_as_reader(url)?;
    let pb = ProgressBar::new(17477521);
    let template = format!(
        "{{msg:>12.cyan.bold}} [{{bar:25}}] {{bytes}}/{{total_bytes}} ({{eta}}) {}",
        url
    );
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&template)
            .progress_chars("=> "),
    );
    pb.set_message("Downloading");
    let p_reader = ProgressReader::new(stdout, |i| pb.inc(i as u64));
    let gz_decoder = GzDecoder::new(p_reader);
    let mut tar_archive = Archive::new(gz_decoder);
    tar_archive.unpack(path).unwrap();
    pb.finish_and_clear();
    println!("{:>12} {}", "Downloaded".green().bold(), url);
    Ok(())
}

#[cfg(unix)]
fn build(src_dir: impl AsRef<Path>, dist_dir: impl AsRef<Path>) -> Result<(), ()> {
    use make::Command;

    println!(
        "{:>12} {}",
        "Building".cyan().bold(),
        src_dir.as_ref().display().decorized()
    );

    let configure = make::Configure {
        dist_dir: dist_dir.as_ref(),
        working_dir: src_dir.as_ref(),
    };
    configure.run(1)?;

    let make = make::Make {
        working_dir: src_dir.as_ref(),
    };
    make.run(2)?;

    let make_install = make::MakeInstall {
        working_dir: src_dir.as_ref(),
    };
    make_install.run(3)?;
    Ok(())
}

mod make {
    use indicatif::{ProgressBar, ProgressStyle};
    use once_cell::sync::Lazy;
    use std::path::Path;
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::Duration;

    static SPINNER_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{prefix:>12.bold.dim} {spinner} {wide_msg}")
    });

    pub trait Command {
        fn command(&self) -> &'static str;
        fn args(&self) -> Vec<String>;
        fn working_dir(&self) -> &Path;
        fn command_line(&self) -> String {
            format!("{} {}", self.command(), self.args().join(" "))
        }
        fn wait(&self, handle_wait: impl Fn()) -> std::process::Output {
            let command = self.command();
            let args = self.args();
            let working_dir = self.working_dir().to_owned();

            let (tx, rx) = channel();
            thread::spawn(move || {
                tx.send(
                    std::process::Command::new(command)
                        .args(args)
                        .current_dir(working_dir)
                        .output()
                        .unwrap(),
                )
            });
            loop {
                if let Ok(output) = rx.try_recv() {
                    break output;
                }
                handle_wait();
                thread::sleep(Duration::from_millis(50));
            }
        }
        fn run(&self, prefix: usize) -> Result<(), ()> {
            let pb = ProgressBar::new(0);
            pb.set_style(SPINNER_STYLE.clone());
            pb.set_prefix(format!("[{}/3]", prefix));
            pb.set_message(self.command_line());

            let output = self.wait(|| pb.inc(1));
            pb.finish();
            if !output.status.success() {
                println!(
                    "{} failed: {}",
                    self.command(),
                    String::from_utf8_lossy(&output.stderr)
                );
                return Err(());
            };
            Ok(())
        }
    }

    pub struct Configure<'a> {
        pub dist_dir: &'a Path,
        pub working_dir: &'a Path,
    }
    impl Command for Configure<'_> {
        fn command(&self) -> &'static str {
            "./configure"
        }
        fn args(&self) -> Vec<String> {
            vec![format!("--prefix={}", self.dist_dir.display())]
        }
        fn working_dir(&self) -> &Path {
            self.working_dir
        }
    }

    pub struct Make<'a> {
        pub working_dir: &'a Path,
    }
    impl Command for Make<'_> {
        fn command(&self) -> &'static str {
            "make"
        }
        fn args(&self) -> Vec<String> {
            vec!["-j".to_owned(), num_cpus::get().to_string()]
        }
        fn working_dir(&self) -> &Path {
            self.working_dir
        }
    }

    pub struct MakeInstall<'a> {
        pub working_dir: &'a Path,
    }
    impl Command for MakeInstall<'_> {
        fn command(&self) -> &'static str {
            "make"
        }
        fn args(&self) -> Vec<String> {
            vec!["install".to_owned()]
        }
        fn working_dir(&self) -> &Path {
            self.working_dir
        }
    }
}

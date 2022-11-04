use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use thiserror::Error;

static PROGRESS_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::default_spinner()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
        .template("{prefix:>12.bold.dim} {spinner} {wide_msg}")
        .unwrap()
});

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't execute `{command}` because {source}")]
    FailedExecute {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error(
        "build error\n=== Please follow the messages below to resolve dependencies, etc. ===\n\n{0}"
    )]
    ExitFailed(String),
}

pub trait Command {
    fn command(&self) -> &'static str;
    fn args(&self) -> Vec<String>;
    fn order(&self) -> usize;
    fn command_line(&self) -> String {
        format!("{} {}", self.command(), self.args().join(" "))
    }
    fn wait(
        &self,
        current_dir: impl AsRef<Path>,
        handle_wait: impl Fn(),
    ) -> Result<std::process::Output, Error> {
        let command = self.command();
        let args = self.args();
        let current_dir = current_dir.as_ref().to_path_buf();

        let (tx, rx) = channel();
        thread::spawn(move || {
            tx.send(
                std::process::Command::new(command)
                    .args(args)
                    .current_dir(current_dir)
                    .output(),
            )
        });
        loop {
            if let Ok(output) = rx.try_recv() {
                break output.map_err(|source| Error::FailedExecute {
                    command: self.command_line(),
                    source,
                });
            }
            handle_wait();
            thread::sleep(Duration::from_millis(50));
        }
    }
    fn run(&self, current_dir: impl AsRef<Path>) -> Result<(), Error> {
        let pb = ProgressBar::new(0)
            .with_style(PROGRESS_STYLE.clone())
            .with_prefix(format!("[{}/3]", self.order()))
            .with_message(self.command_line());

        let output = self.wait(current_dir, || pb.inc(1))?;
        pb.finish();
        if output.status.success() {
            Ok(())
        } else {
            Err(Error::ExitFailed(String::from_utf8(output.stderr).unwrap()))
        }
    }
}

pub struct Configure<'a> {
    pub prefix: &'a Path,
    pub opts: Vec<&'a str>,
}
impl Command for Configure<'_> {
    fn command(&self) -> &'static str {
        "./configure"
    }
    fn args(&self) -> Vec<String> {
        [format!("--prefix={}", self.prefix.display()).as_str()]
            .iter()
            .chain(self.opts.iter())
            .map(|&s| s.to_owned())
            .collect_vec()
    }
    fn order(&self) -> usize {
        1
    }
}

pub struct Make {}
impl Command for Make {
    fn command(&self) -> &'static str {
        "make"
    }
    fn args(&self) -> Vec<String> {
        vec!["-j".to_owned(), num_cpus::get().to_string()]
    }
    fn order(&self) -> usize {
        2
    }
}

pub struct Install {}
impl Command for Install {
    fn command(&self) -> &'static str {
        "make"
    }
    fn args(&self) -> Vec<String> {
        vec!["install".to_owned()]
    }
    fn order(&self) -> usize {
        3
    }
}

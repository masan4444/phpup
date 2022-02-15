use indicatif::{ProgressBar, ProgressStyle};
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
});

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Other(String),
}

pub trait Command {
    fn command(&self) -> &'static str;
    fn args(&self) -> Vec<String>;
    fn current_dir(&self) -> &Path;
    fn command_line(&self) -> String {
        format!("{} {}", self.command(), self.args().join(" "))
    }
    fn wait(&self, handle_wait: impl Fn()) -> std::process::Output {
        let command = self.command();
        let args = self.args();
        let current_dir = self.current_dir().to_owned();

        let (tx, rx) = channel();
        thread::spawn(move || {
            tx.send(
                std::process::Command::new(command)
                    .args(args)
                    .current_dir(current_dir)
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
    fn run(&self, prefix: usize) -> Result<(), Error> {
        let pb = ProgressBar::new(0)
            .with_style(PROGRESS_STYLE.clone())
            .with_prefix(format!("[{}/3]", prefix))
            .with_message(self.command_line());

        let output = self.wait(|| pb.inc(1));
        pb.finish();
        if output.status.success() {
            Ok(())
        } else {
            Err(Error::Other(String::from_utf8(output.stderr).unwrap()))
        }
    }
}

pub struct Configure<'a> {
    pub current_dir: &'a Path,
    pub dist_dir: &'a Path,
}
impl Command for Configure<'_> {
    fn command(&self) -> &'static str {
        "./configure"
    }
    fn args(&self) -> Vec<String> {
        vec![format!("--prefix={}", self.dist_dir.display())]
    }
    fn current_dir(&self) -> &Path {
        self.current_dir
    }
}

pub struct Make<'a> {
    pub current_dir: &'a Path,
}
impl Command for Make<'_> {
    fn command(&self) -> &'static str {
        "make"
    }
    fn args(&self) -> Vec<String> {
        vec!["-j".to_owned(), num_cpus::get().to_string()]
    }
    fn current_dir(&self) -> &Path {
        self.current_dir
    }
}

pub struct Install<'a> {
    pub current_dir: &'a Path,
}
impl Command for Install<'_> {
    fn command(&self) -> &'static str {
        "make"
    }
    fn args(&self) -> Vec<String> {
        vec!["install".to_owned()]
    }
    fn current_dir(&self) -> &Path {
        self.current_dir
    }
}

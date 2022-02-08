use crate::version::{ParseError, Version};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const DEFAULT_VERSION_FILE_NAME: &str = ".php-version";

#[derive(clap::Parser, Debug)]
pub struct VersionFile {
    /// Spacify a custom version file name
    #[clap(long = "version-file-name", env = "PHPUP_VERSION_FILE_NAME", default_value = DEFAULT_VERSION_FILE_NAME)]
    filename: PathBuf,

    /// Enable recursive search in a parent dirctory for a version file
    #[clap(
        long = "recursive-version-file",
        visible_alias = "recursive",
        env = "PHPUP_RECURSIVE_VERSION_FILE"
    )]
    is_recursive: bool,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't parse string written in {filepath}: {source}")]
    VersionParseError {
        filepath: PathBuf,
        #[source]
        source: ParseError,
    },
    #[error("Can't find a version file: \"{0}\"")]
    NoVersionFileError(PathBuf),
}

impl VersionFile {
    pub fn is_recursive(&self) -> bool {
        self.is_recursive
    }
    pub fn filename(&self) -> &Path {
        self.filename.as_path()
    }
    pub fn get_version(&self) -> Result<(Version, PathBuf), Error> {
        let current_dir = std::env::current_dir().expect("Can't get a current directory");
        if self.is_recursive {
            self.search_recursively(current_dir)
        } else {
            self.search_current(current_dir)?
                .ok_or(Error::NoVersionFileError(self.filename.clone()))
        }
    }

    fn search_current(
        &self,
        current_dir: impl AsRef<Path>,
    ) -> Result<Option<(Version, PathBuf)>, Error> {
        let filepath = current_dir.as_ref().join(self.filename.as_path());
        fs::read_to_string(&filepath)
            .ok()
            .map(|string| {
                string
                    .trim()
                    .parse::<Version>()
                    .or_else(|source| {
                        Err(Error::VersionParseError {
                            filepath: filepath.clone(),
                            source,
                        })
                    })
                    .map(|v| (v, filepath))
            })
            .transpose()
    }

    fn search_recursively(
        &self,
        current_dir: impl AsRef<Path>,
    ) -> Result<(Version, PathBuf), Error> {
        let mut searching_dir = Some(current_dir.as_ref());
        while let Some(dir) = searching_dir {
            if let Some(version_info) = self.search_current(dir)? {
                return Ok(version_info);
            }
            searching_dir = dir.parent()
        }
        Err(Error::NoVersionFileError(self.filename.clone()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{fs, io::Write};

    #[test]
    fn search_current_success() {
        let module = VersionFile {
            filename: PathBuf::from(".php-version"),
            is_recursive: false,
        };

        let current_dir = tempfile::tempdir().unwrap();
        let version_file_path = current_dir.path().join(".php-version");
        fs::File::create(&version_file_path)
            .unwrap()
            .write_all(b"8.1.1")
            .unwrap();

        let version_info = module.search_current(current_dir);

        assert!(version_info.is_ok());
        assert_eq!(
            version_info.unwrap(),
            Some(("8.1.1".parse().unwrap(), version_file_path))
        );
    }

    #[test]
    fn search_recursively_success() {
        let module = VersionFile {
            filename: PathBuf::from(".php-version"),
            is_recursive: true,
        };

        let root_dir = tempfile::tempdir().unwrap();
        let version_file_path = root_dir.path().join(".php-version");
        fs::File::create(&version_file_path)
            .unwrap()
            .write_all(b"8.1.1")
            .unwrap();

        let current_dir = root_dir.path().join("child").join("grand_child");
        let version_info = module.search_recursively(current_dir);

        assert!(version_info.is_ok());
        assert_eq!(
            version_info.unwrap(),
            ("8.1.1".parse().unwrap(), version_file_path)
        );
    }
}

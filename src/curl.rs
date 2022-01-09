use std::{
    ffi::OsStr,
    process::{Command, Stdio},
};

const CURL_PATH: &str = if cfg!(target_os = "windows") {
    "curl.exe"
} else {
    "curl"
};

pub fn get_as_slice(url: impl AsRef<OsStr>) -> Vec<u8> {
    Command::new(CURL_PATH).arg(url).output().unwrap().stdout
}

pub fn get_as_reader(url: impl AsRef<OsStr>) -> impl std::io::Read {
    let cmd = Command::new(CURL_PATH)
        .arg(url)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    cmd.stdout.unwrap()
}

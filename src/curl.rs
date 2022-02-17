use std::process::ChildStderr;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Can't execute `{command}` because {source}")]
    FailedExecute {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to execute `{0}` because {1}")]
    ExitFailed(String, String),
}

const CURL_PATH: &str = if cfg!(target_os = "windows") {
    "curl.exe"
} else {
    "curl"
};

pub struct Header {
    pub content_length: Option<usize>,
}

pub fn get_as_slice(url: &str) -> Result<Vec<u8>, Error> {
    let command = [CURL_PATH, url, "-sS"];
    let output = Command::new(command[0])
        .args(&command[1..])
        .output()
        .map_err(|source| Error::FailedExecute {
            command: command.join(" "),
            source,
        })?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        let err_msg = String::from_utf8(output.stderr).unwrap();
        Err(Error::ExitFailed(command.join(" "), err_msg))
    }
}

pub fn get_as_reader(url: &str) -> Result<(ChildStdout, ChildStderr), Error> {
    let command = [CURL_PATH, url, "-sS"];
    let process = Command::new(command[0])
        .args(&command[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| Error::FailedExecute {
            command: command.join(" "),
            source,
        })?;
    Ok((process.stdout.unwrap(), process.stderr.unwrap()))
}

pub fn get_header(url: &str) -> Result<Header, Error> {
    let command = [CURL_PATH, url, "-sSI"];
    let output = Command::new(command[0])
        .args(&command[1..])
        .output()
        .map_err(|source| Error::FailedExecute {
            command: command.join(" "),
            source,
        })?;
    let output = if output.status.success() {
        String::from_utf8(output.stdout).unwrap()
    } else {
        return Err(Error::ExitFailed(
            command.join(" "),
            String::from_utf8(output.stderr).unwrap(),
        ));
    };

    let mut lines = output.lines();
    let content_length = lines
        .find(|line| line.starts_with("content-length"))
        .and_then(|line| line.split(": ").last())
        .and_then(|length| length.parse::<usize>().ok());
    Ok(Header { content_length })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let output = get_as_slice("http://examsssssssple.com/");
        match output {
            Ok(v) => println!("{:?}", v),
            Err(e) => println!("{}", e),
        }
    }
}

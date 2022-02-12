use std::path::PathBuf;
use which::which;

pub fn path() -> Option<PathBuf> {
    which("php")
        .ok()
        .and_then(|bin_path| bin_path.parent().map(|p| p.to_path_buf()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let system_path = path().unwrap();
        println!("{:?}", system_path);
    }
}

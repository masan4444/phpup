use std::path::PathBuf;
use which::which_all;

pub fn path() -> Option<PathBuf> {
    let multishell_path_dir = std::env::temp_dir().join("phpup");
    which_all("php")
        .ok()
        .into_iter()
        .flatten()
        .find(|bin_path| !bin_path.starts_with(&multishell_path_dir))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let system_path = path();
        println!("{:?}", system_path);
    }
}

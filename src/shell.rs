use crate::version;
use indoc::formatdoc;
use std::fmt::Display;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

pub const fn available_shells() -> &'static [&'static str] {
    &["bash", "zsh", "fish", "powershell"]
}

use Shell::*;

impl Shell {
    pub fn set_path(&self, path: impl AsRef<Path>) -> String {
        match &self {
            Bash | Zsh => {
                format!("export PATH={}:$PATH", path.as_ref().display())
            }
            Fish => format!("set -gx PATH {} $PATH;", path.as_ref().display()),
            PowerShell => unimplemented!(),
        }
    }
    pub fn set_env(&self, name: impl Display, value: impl Display) -> String {
        match &self {
            Bash | Zsh => {
                format!("export {}={}", name, value)
            }
            Fish => format!("set -gx {} {};", name, value),
            PowerShell => unimplemented!(),
        }
    }
    pub fn auto_switch_hook(&self, version_file: &version::File) -> String {
        let version_file_name = version_file.filename().display();
        let is_recursive_version_file = if version_file.is_recursive() {
            "--recursive-version-file"
        } else {
            ""
        };
        let phpup_use = format!(
            "phpup use --quiet --version-file-name {} {}",
            version_file_name, is_recursive_version_file
        );

        match &self {
            Bash => {
                formatdoc! {
                    r#"
                    __phpup_use() {{
                        {phpup_use}
                    }}
                    __phpupcd() {{
                        \cd "$@" || return $?
                        __phpup_use
                    }}
                    alias cd=__phpupcd
                    __phpup_use"#,
                    phpup_use = phpup_use
                }
            }
            Zsh => {
                formatdoc! {
                    r#"
                    autoload -U add-zsh-hook
                    _phpup_autoload_hook () {{
                        {phpup_use}
                    }}
                    add-zsh-hook chpwd _phpup_autoload_hook \
                        && _phpup_autoload_hook"#,
                    phpup_use = phpup_use
                }
            }
            Fish => {
                formatdoc!(
                    r#"
                    function _phpup_autoload_hook --on-variable PWD --description 'Change PHP version on directory change'
                        status --is-command-substitution; and return
                        {phpup_use}
                    end
                    _phpup_autoload_hook"#,
                    phpup_use = phpup_use
                )
            }
            PowerShell => {
                unimplemented!()
            }
        }
    }
    pub fn rehash(&self) -> Option<String> {
        match &self {
            Bash | Fish => None,
            Zsh => Some("rehash".to_string()),
            PowerShell => unimplemented!(),
        }
    }
    pub fn to_clap_shell(&self) -> clap_complete::Shell {
        match &self {
            Bash => clap_complete::Shell::Bash,
            Zsh => clap_complete::Shell::Zsh,
            Fish => clap_complete::Shell::Fish,
            PowerShell => clap_complete::Shell::PowerShell,
        }
    }
}

mod detect;
pub use detect::detect;
pub use detect::ShellDetectError;

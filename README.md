# PHP-UP - Cross-Platform PHP version manager

:zap: PHP version manager written in rust

<div align="center">
  <img src="https://raw.githubusercontent.com/wiki/masan4444/phpup/blob/phpup.gif" alt="Blazing fast!">
</div>

## Features

- No requirements for system PHP installation
- Cross-platform support (Linux, macOS, Windows[WIP])
- Automatically version switching via `.php-version`

## Quick Start (Linux, macOS)

```sh
curl https://raw.githubusercontent.com/masan4444/phpup/master/.ci/install.sh | bash
```

#### Upgrade

To prevent duplication in your shell config file, add `--skip-shell` option to install command.

```sh
curl https://raw.githubusercontent.com/masan4444/phpup/master/.ci/install.sh --skip-shell | bash
```

#### Uninstall

To remove PHP-UP, just delete the `.phpup` folder in your home directory.
You should also edit your shell configuration to remove any references to phpup.

## Installation

### Requirements

- OS: Linux, macOS, Windows[WIP]
- shell: bash, zsh, fish[WIP], powershell[WIP]
- `curl`/`ps` installation

### Installation

#### using a release binary

1. Download the [latest release binary](https://github.com/masan4444/phpup/releases) for your system
2. Make it available globally on `PATH` environment variable

#### using cargo

```
cargo install phpup
```

### Shell setup

Add the following to your `.bashrc` or `.zshrc`

```bash
eval "$(phpup init --auto --recursive)"
```

- To automatically run `phpup use` when a directory contains a `.php-version` file, add the `--auto` (long: `--auto-switch`) option.
- To search recursively for a `.php-version` file in a parent directory when running `phpup use` automatically, add the `--recursive` (long: `--recursive-version-file`) option.
- For more options, run `phpup init --help`.

## Usage

```
phpup help
```

## Contribution

PRs Welcome :tada:

- [TODO.md](TODO.md)

## Inspired

- [Schniz/fnm](https://github.com/Schniz/fnm)
- [TaKO8Ki/frum](https://github.com/TaKO8Ki/frum)

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

For bash, zsh, there's a [installation script](./.ci/install.sh)

```sh
curl -fsSL https://phpup.vercel.app/install | bash
```

#### Upgrade

To prevent duplication in your shell config file, add `--skip-shell` option to install command.

```sh
curl -fsSL https://phpup.vercel.app/install  | bash -s -- --skip-shell
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

### PHP Installation

#### How to specify configure options

##### using the command option: `--configure-opts`

```sh
PKG_CONFIG_PATH="$(brew --prefix libxml2)/lib/pkgconfig" \
phpup install 8.1 \
  --configure-opts="--with-iconv=$(brew --prefix libiconv)"
```

##### using the shell variable: `PHPUP_CONFIGURE_OPTS`

```sh
PKG_CONFIG_PATH="$(brew --prefix libxml2)/lib/pkgconfig" \
PHPUP_CONFIGURE_OPTS="--with-iconv=$(brew --prefix libiconv)" \
phpup install 8.1
```

##### using the env variable: `PHPUP_CONFIGURE_OPTS`

```sh
export PKG_CONFIG_PATH="$(brew --prefix libxml2)/lib/pkgconfig"
export PHPUP_CONFIGURE_OPTS="--with-iconv=$(brew --prefix libiconv)"
phpup install 8.1
```

See [List of core configure options](https://www.php.net/manual/en/configure.about.php) for more configre options.

### For more details

```
phpup help
```

## Contribution

PRs Welcome :tada:

- [TODO.md](TODO.md)

## Inspired

- [Schniz/fnm](https://github.com/Schniz/fnm)
- [TaKO8Ki/frum](https://github.com/TaKO8Ki/frum)

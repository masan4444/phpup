# TODO

## New feature

- [x] alias
- [x] defaut alias
- [x] auto version switching via `.php-version`
- [x] colorize output
- [x] embed system php version
- [x] shell completion
- [ ] pre-release install (`8.1.0alpha2` etc..)
- [ ] install with extention (`+mysql` etc..) + configure option

## Config

- [x] `.phpup` dirctory path (default: `$HOME/.phpup`)
- [ ] source mirror url (default: `https://www.php.net/distributions` and `https://museum.php.net`)

## Error handling

- [ ] `install`
- [x] curl.rs
- [x] if `$MULTISHELL_PATH` is unset, return message: Need to run `eval $(phpup init)`

## Performance

- [ ] `list-remote`: parallel download
- [ ] `install`: parallel download

## Refactor

- [ ] logger

## Cross-platform support

- [x] bash
- [x] fish
- [ ] windows

## Test

## Installation support

- [ ] cargo
- [ ] brew
- [ ] curl
- [ ] windows package manager

## Document

- [ ] command description
- [ ] Cargo.toml
- [ ] README.md
- [ ] cargo doc

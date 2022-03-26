#!/bin/bash

# Based on https://github.com/Schniz/fnm/blob/0fc14222846161ec120c992206215556173b59ea/.ci/install.sh

set -e

INSTALL_DIR="$HOME/.phpup"
RELEASE="latest"
OS="$(uname -s)"
REPOSITORY="https://github.com/masan4444/phpup"

# Parse Flags
parse_args() {
  while [[ $# -gt 0 ]]; do
    key="$1"

    case $key in
    -d | --install-dir)
      INSTALL_DIR="$2"
      shift # past argument
      shift # past value
      ;;
    -s | --skip-shell)
      SKIP_SHELL="true"
      shift # past argument
      ;;
    # --force-install | --force-no-brew)
    #   echo "\`--force-install\`: I hope you know what you're doing." >&2
    #   FORCE_INSTALL="true"
    #   shift
    #   ;;
    -r | --release)
      RELEASE="$2"
      shift # past release argument
      shift # past release value
      ;;
    *)
      echo "Unrecognized argument $key"
      exit 1
      ;;
    esac
  done
}

set_filename() {
  if [ "$OS" = "Linux" ]; then
    # Based on https://stackoverflow.com/a/45125525
    case "$(uname -m)" in
      arm | armv7*)
        FILENAME="phpup-linux-armv7"
        ;;
      aarch* | armv8*)
        FILENAME="phpup-linux-aarch64"
        ;;
      *)
        FILENAME="phpup-linux"
    esac
  # elif [ "$OS" = "Darwin" ] && [ "$FORCE_INSTALL" = "true" ]; then
  elif [ "$OS" = "Darwin" ]; then
    FILENAME="phpup-macos"
    USE_HOMEBREW="false"
    # echo "Downloading the latest phpup binary from GitHub..."
    # echo "  Pro tip: it's easier to use Homebrew for managing phpup in macOS."
    # echo "           Remove the \`--force-no-brew\` so it will be easy to upgrade."
  # elif [ "$OS" = "Darwin" ]; then
  #   USE_HOMEBREW="true"
  #   echo "Downloading phpup using Homebrew..."
  else
    echo "OS $OS is not supported."
    echo "If you think that's a bug - please file an issue to $REPOSITORY/issues"
    exit 1
  fi
}

download_phpup() {
  if [ "$USE_HOMEBREW" = "true" ]; then
    brew install phpup
  else
    if [ "$RELEASE" = "latest" ]; then
      URL="$REPOSITORY/releases/latest/download/$FILENAME.zip"
    else
      URL="$REPOSITORY/releases/download/$RELEASE/$FILENAME.zip"
    fi

    DOWNLOAD_DIR=$(mktemp -d)

    echo "Downloading $URL..."

    mkdir -p "$INSTALL_DIR/bin" &>/dev/null

    if ! curl --progress-bar --fail -L "$URL" -o "$DOWNLOAD_DIR/$FILENAME.zip"; then
      echo "Download failed.  Check that the release/filename are correct."
      exit 1
    fi

    unzip -q "$DOWNLOAD_DIR/$FILENAME.zip" -d "$DOWNLOAD_DIR"

    mv "$DOWNLOAD_DIR/$FILENAME/phpup" "$INSTALL_DIR/bin/phpup"
    cp -r "$DOWNLOAD_DIR/$FILENAME/completions" "$INSTALL_DIR"

    chmod u+x "$INSTALL_DIR/bin/phpup"
  fi
}

check_dependencies() {
  echo "Checking dependencies for the installation script..."

  echo -n "Checking availability of curl... "
  if hash curl 2>/dev/null; then
    echo "OK!"
  else
    echo "Missing!"
    SHOULD_EXIT="true"
  fi

  echo -n "Checking availability of unzip... "
  if hash unzip 2>/dev/null; then
    echo "OK!"
  else
    echo "Missing!"
    SHOULD_EXIT="true"
  fi

  if [ "$USE_HOMEBREW" = "true" ]; then
    echo -n "Checking availability of Homebrew (brew)... "
    if hash brew 2>/dev/null; then
      echo "OK!"
    else
      echo "Missing!"
      SHOULD_EXIT="true"
    fi
  fi

  if [ "$SHOULD_EXIT" = "true" ]; then
    echo "Not installing phpup due to missing dependencies."
    exit 1
  fi
}

ensure_containing_dir_exists() {
  local CONTAINING_DIR
  CONTAINING_DIR="$(dirname "$1")"
  if [ ! -d "$CONTAINING_DIR" ]; then
    echo " >> Creating directory $CONTAINING_DIR"
    mkdir -p "$CONTAINING_DIR"
  fi
}

setup_shell() {
  CURRENT_SHELL="$(basename "$SHELL")"

  if [ "$CURRENT_SHELL" = "zsh" ]; then
    CONF_FILE=${ZDOTDIR:-$HOME}/.zshrc
    ensure_containing_dir_exists "$CONF_FILE"
    echo "Installing for Zsh. Appending the following to $CONF_FILE:"
    echo ""
    echo '  # PHP-UP'
    echo '  export PATH='"$INSTALL_DIR/bin"':$PATH'
    echo '  eval "$(phpup init --auto --recursive)"'
    echo '  fpath=('$INSTALL_DIR/completions/zsh' $fpath)'
    echo '  # To use completion, run `compinit` after adding $fpath'
    echo '  # compinit'

    echo '' >>$CONF_FILE
    echo '# PHP-UP' >>$CONF_FILE
    echo 'export PATH='$INSTALL_DIR/bin':$PATH' >>$CONF_FILE
    echo 'eval "$(phpup init --auto --recursive)"' >>$CONF_FILE
    echo 'fpath=('$INSTALL_DIR/completions/zsh' $fpath)' >>$CONF_FILE
    echo '# To use completion, run `compinit` after adding $fpath' >>$CONF_FILE
    echo '# compinit' >>$CONF_FILE

  elif [ "$CURRENT_SHELL" = "fish" ]; then
    CONF_FILE=$HOME/.config/fish/conf.d/phpup.fish
    ensure_containing_dir_exists "$CONF_FILE"
    echo "Installing for Fish. Appending the following to $CONF_FILE:"
    echo ""
    echo '  # PHP-UP'
    echo '  set PATH '$INSTALL_DIR/bin' $PATH'
    echo '  phpup init --auto --recursive | source'
    echo '  set -gx fish_complete_path '$INSTALL_DIR/completions/fish' $fish_complete_path'

    echo '# PHP-UP' >>$CONF_FILE
    echo 'set PATH '$INSTALL_DIR/bin' $PATH' >>$CONF_FILE
    echo 'phpup init --auto --recursive | source' >>$CONF_FILE
    echo 'set -gx fish_complete_path '$INSTALL_DIR/completions/fish' $fish_complete_path' >>$CONF_FILE

  elif [ "$CURRENT_SHELL" = "bash" ]; then
    if [ "$OS" = "Darwin" ]; then
      CONF_FILE=$HOME/.profile
    else
      CONF_FILE=$HOME/.bashrc
    fi
    ensure_containing_dir_exists "$CONF_FILE"
    echo "Installing for Bash. Appending the following to $CONF_FILE:"
    echo ""
    echo '  # PHP-UP'
    echo '  export PATH='"$INSTALL_DIR/bin"':$PATH'
    echo '  eval "$(phpup init --auto --recursive)"'
    echo '  [ -s '"$INSTALL_DIR/completions/bash/_phpup"' ] && \. '"$INSTALL_DIR/completions/bash/_phpup"''

    echo '' >>$CONF_FILE
    echo '# PHP-UP' >>$CONF_FILE
    echo 'export PATH='"$INSTALL_DIR/bin"':$PATH' >>$CONF_FILE
    echo 'eval "$(phpup init --auto --recursive)"' >>$CONF_FILE
    echo '[ -s '"$INSTALL_DIR/completions/bash/_phpup"' ] && \. '"$INSTALL_DIR/completions/bash/_phpup"'' >>$CONF_FILE

  else
    echo "Could not infer shell type. Please set up manually."
    exit 1
  fi

  echo ""
  echo "In order to apply the changes, open a new terminal or run the following command:"
  echo ""
  echo "  source $CONF_FILE"
}

parse_args "$@"
set_filename
check_dependencies
download_phpup
if [ "$SKIP_SHELL" != "true" ]; then
  setup_shell
fi

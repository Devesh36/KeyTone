#!/bin/sh

set -eu

repository="Devesh36/KeyTone"
release_api="https://api.github.com/repos/${repository}/releases/latest"
temporary_directory="$(mktemp -d)"

cleanup() {
  rm -rf "$temporary_directory"
}

trap cleanup EXIT HUP INT TERM

say() {
  printf 'Keytone: %s\n' "$1"
}

download_asset() {
  suffix="$1"
  destination="$2"
  asset_url="$(
    curl --proto '=https' --tlsv1.2 -fsSL "$release_api" |
      sed -n 's/.*"browser_download_url": "\([^"]*\)".*/\1/p' |
      grep -i "${suffix}$" |
      head -n 1
  )"

  if [ -z "$asset_url" ]; then
    say "No ${suffix} installer was found in the latest release."
    exit 1
  fi

  say "Downloading $(basename "$asset_url")..."
  curl --proto '=https' --tlsv1.2 -fL "$asset_url" -o "$destination"
}

install_macos() {
  image="$temporary_directory/keytone.dmg"
  download_asset '.dmg' "$image"

  say "Mounting installer..."
  mount_point="$(
    hdiutil attach -nobrowse -readonly "$image" |
      sed -n 's|^.*\(/Volumes/.*\)$|\1|p' |
      tail -n 1
  )"

  if [ -z "$mount_point" ] || [ ! -d "$mount_point/Keytone.app" ]; then
    say "The disk image did not contain Keytone.app."
    exit 1
  fi

  trap 'hdiutil detach "$mount_point" >/dev/null 2>&1 || true; cleanup' EXIT HUP INT TERM

  if [ -w /Applications ]; then
    ditto "$mount_point/Keytone.app" /Applications/Keytone.app
  else
    say "Administrator access is needed to copy Keytone to Applications."
    sudo ditto "$mount_point/Keytone.app" /Applications/Keytone.app
  fi

  hdiutil detach "$mount_point" >/dev/null
  trap cleanup EXIT HUP INT TERM
  say "Installed Keytone in /Applications."
  say "Open it once, then approve it in Privacy & Security if macOS asks."
}

install_linux() {
  if command -v apt-get >/dev/null 2>&1; then
    package="$temporary_directory/keytone.deb"
    download_asset '.deb' "$package"
    say "Installing the Debian package..."
    sudo apt-get install -y "$package"
  elif command -v dnf >/dev/null 2>&1; then
    package="$temporary_directory/keytone.rpm"
    download_asset '.rpm' "$package"
    say "Installing the RPM package..."
    sudo dnf install -y "$package"
  else
    install_directory="${XDG_BIN_HOME:-${HOME}/.local/bin}"
    appimage="$install_directory/keytone"
    mkdir -p "$install_directory"
    download_asset '.appimage' "$appimage"
    chmod 755 "$appimage"
    say "Installed Keytone at ${appimage}."
    case ":${PATH}:" in
      *":${install_directory}:"*) ;;
      *) say "Add ${install_directory} to PATH to run 'keytone' from any shell." ;;
    esac
  fi

  say "Installation complete."
}

if ! command -v curl >/dev/null 2>&1; then
  say "curl is required to install Keytone."
  exit 1
fi

case "$(uname -s)" in
  Darwin) install_macos ;;
  Linux) install_linux ;;
  *)
    say "This installer supports macOS and Linux. On Windows, use install.ps1."
    exit 1
    ;;
esac

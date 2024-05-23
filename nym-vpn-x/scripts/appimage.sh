#!/bin/bash

# NymVPN AppImage installer

set -E
set -o pipefail
# catch errors
trap 'catch $? ${FUNCNAME[0]:-main} $LINENO' ERR
cwd=$(pwd)

# ANSI style codes
RED="\e[38;5;1m" # red
GRN="\e[38;5;2m" # green
YLW="\e[38;5;3m" # yellow
BLU="\e[38;5;4m" # blue
BLD="\e[1m"      # bold
RS="\e[0m"       # style reset
B_RED="$BLD$RED"
B_GRN="$BLD$GRN"
B_YLW="$BLD$YLW"
B_BLU="$BLD$BLU"
####

# TODO set/replace with the target tag and version
tag=
version=
appimage_url="https://github.com/nymtech/nym-vpn-client/releases/download/$tag/nymvpn-x_${version}.AppImage"

# function called when an error occurs, will print the exit
# status, function name and line number
catch() {
  log_e "$B_RED✗$RS unexpected error, [$BLD$1$RS] $BLD$2$RS L#$BLD$3$RS"
  cleanup
  cd "$cwd" || true
  exit 1
}

log() {
  echo -e "$1"
}

# log to stderr
log_e() {
  echo >&2 -e "$1"
}

# silent pushd that don't print the directory change
_pushd() {
  command pushd "$@" >/dev/null || exit 1
}

# silent popd that don't print the directory change
_popd() {
  command popd >/dev/null || exit 1
}

# check if a command exists
need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    log_e " $B_RED⚠$RS need$BLD $1$RS (command not found)"
    exit 1
  fi
}

# replace the HOME directory with '~' in the given path
tilded() {
  echo "${1/#$HOME/\~}"
}

need_cmd mktemp
temp_dir=$(mktemp -d)
data_home=${XDG_DATA_HOME:-$HOME/.local/share}
state_home=${XDG_STATE_HOME:-$HOME/.local/state}
xdg_bin_home="$HOME/.local/bin"
app_dir="nymvpn-x"
usr_bin_dir="/usr/bin"
# default installation directory
install_dir="$xdg_bin_home"
icons_dir="$data_home/icons"
appimage="nymvpn-x_${version}.AppImage"
target_appimage="nymvpn-x.appimage"
desktop_dir="$data_home/applications"

### desktop entry ###
desktop_entry="[Desktop Entry]
Name=NymVPN-x
Type=Application
Version=1.0
Comment=Decentralized, mixnet, and zero-knowledge VPN
Exec=INSTALL_DIR/$target_appimage %U
Icon=$icons_dir/nym-vpn.svg
Terminal=false
Categories=Network;"
###

### app icon ###
icon='<svg width="32" height="32" viewBox="0 0 32 32" fill="#FB6E4E" xmlns="http://www.w3.org/2000/svg"><path d="M3.7229 29.9997C-0.460546 26.7617 -1.23766 20.7391 2.0003 16.5557C5.23826 12.3722 11.2609 11.5951 15.4443 14.8331C19.6278 18.0711 20.4049 24.0937 17.1669 28.2771C13.9289 32.4605 7.90634 33.2377 3.7229 29.9997ZM28.0076 23.2647C33.3308 17.9415 33.3308 9.31561 28.0076 3.9924C22.6844 -1.3308 14.0455 -1.3308 8.73526 3.9924C3.42501 9.31561 3.41205 17.9415 8.73526 23.2647C14.0585 28.5879 22.6844 28.5879 28.0076 23.2647Z" fill="#FB6E4E"/></svg>'
###

pre_check() {
  if [ -z "$tag" ] || [ -z "$version" ]; then
    log "${B_RED}✗$RS \`tag\` and \`version\` must be set"
    exit 1
  fi
}

download() {
  need_cmd curl
  need_cmd sha256sum

  _pushd "$temp_dir"
  log "  ${B_GRN}Downloading$RS $appimage"
  curl -fL -# "$appimage_url" -o $appimage
  log "  ${B_GRN}Downloading$RS $appimage.sha256sum"
  curl -fL -# "$appimage_url.sha256sum" -o "$appimage.sha256sum"
  log "  ${B_GRN}Checking$RS sha256sum"
  sha256sum --check --status "$appimage.sha256sum"
  _popd
}

select_install_dir() {
  # read user input for the installation directory
  # select between these 2 options
  # 1. ~/.local/bin (default)
  # 2. /usr/local/bin
  choice=""
  log "  ${B_GRN}Select$RS the directory where the AppImage will be moved to"
  prompt="    [${B_BLU}H$RS ~/.local/bin (default)] [${B_BLU}U$RS /usr/bin] (${B_BLU}h$RS/${B_BLU}u$RS) "
  read -r -p "$(echo -e "$prompt")" choice
  case $choice in
  "h" | "H")
    install_dir="$xdg_bin_home"
    ;;
  "u" | "U")
    install_dir="$usr_bin_dir"
    ;;
  *)
    install_dir="$xdg_bin_home"
    ;;
  esac
}

# checking if a directory is in the PATH
in_path() {
  if [[ ":$PATH:" == *":$1:"* ]]; then
    return 0
  fi
  return 1
}

_install() {
  need_cmd install

  log "  ${B_GRN}Installing$RS AppImage"
  if ! in_path "$install_dir"; then
    install_dir="$usr_bin_dir"
  else
    select_install_dir
  fi
  log "  ${B_GRN}Selected$RS installation directory $install_dir"

  if [ "$install_dir" = "$usr_bin_dir" ]; then
    need_cmd sudo
    log "  ${B_YLW}Need$RS sudo to install AppImage in $install_dir"
    sudo install -o "$(id -u)" -g "$(id -g)" -Dm755 "$temp_dir/$appimage" "$install_dir/$target_appimage"
  else
    install -Dm755 "$temp_dir/$appimage" "$install_dir/$target_appimage"
  fi

  path=$(tilded "$install_dir/$target_appimage")
  log "   ${B_GRN}Installed$RS $path"

  log "  ${B_GRN}Installing$RS desktop entry"
  _pushd "$temp_dir"
  echo "${desktop_entry/INSTALL_DIR/$install_dir}" >"nymvpn-x.desktop"
  echo "$icon" >"nym-vpn.svg"
  _popd
  install -Dm644 "$temp_dir/nymvpn-x.desktop" "$desktop_dir/nymvpn-x.desktop"
  install -Dm644 "$temp_dir/nym-vpn.svg" "$icons_dir/nym-vpn.svg"
  install -Dm755 -d "$state_home/$app_dir"
  path=$(tilded "$desktop_dir/nymvpn-x.desktop")
  log "   ${B_GRN}Installed$RS $path"
}

post_install() {
  if ! in_path "$install_dir"; then
    log "${B_YLW}⚠$RS $install_dir is not in the ${BLD}PATH$RS
  please add it using your shell configuration"
  fi
}

cleanup() {
  rm -rf "$temp_dir"
}

pre_check
download
_install
post_install
cleanup

log "   ${B_GRN}✓$RS Done"

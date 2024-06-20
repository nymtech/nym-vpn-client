#!/bin/bash

# All-in-one NymVPN installer
# It installs the daemon (nym-vpnd) from prebuilt binary
# and the client (nymvpn-x) from AppImage

set -E
set -o pipefail
# catch errors
trap 'catch $? ${FUNCNAME[0]:-main} $LINENO' ERR
cwd=$(pwd)

# ANSI style codes
RED="\e[38;5;1m" # red
GRN="\e[38;5;2m" # green
YLW="\e[38;5;3m" # yellow
GRY="\e[38;5;8m" # gray
BLD="\e[1m"      # bold
ITL="\e[3m"      # italic
RS="\e[0m"       # style reset
B_RED="$BLD$RED"
B_GRN="$BLD$GRN"
B_YLW="$BLD$YLW"
I_YLW="$ITL$YLW"
B_GRY="$BLD$GRY"
####

# nymvpn-x AppImage
vpnx_tag=nightly-x
vpnx_version=0.1.2-dev
appimage_url="https://github.com/nymtech/nym-vpn-client/releases/download/$vpnx_tag/nymvpn-x_${vpnx_version}.AppImage"

# nym-vpnd prebuilt binary
vpnd_tag=nightly
vpnd_version=0.1.7-dev
vpnd_url="https://github.com/nymtech/nym-vpn-client/releases/download/$vpnd_tag/nym-vpn-core-v${vpnd_version}_linux_x86_64.tar.gz"

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

bin_in_path() {
  if which "$1" &>/dev/null; then
    log "${B_YLW}⚠$RS $1 is present in the system"
    return 0
  fi
  return 1
}

user_prompt() {
  # check if the script is running in a terminal, if not,
  # ie. piped into bash read from /dev/tty to get user input
  if [ -t 0 ]; then
    read -r -p "$(echo -e "$2")" "$1"
  else
    read -r -p "$(echo -e "$2")" "$1" </dev/tty
  fi
}

need_cmd mktemp
temp_dir=$(mktemp -d)

data_home=${XDG_DATA_HOME:-$HOME/.local/share}
state_home=${XDG_STATE_HOME:-$HOME/.local/state}
xdg_bin_home="$HOME/.local/bin"
app_dir="nymvpn-x"
usr_bin_dir="/usr/bin"
# default installation directory
prefix_install="$HOME/.local"
install_dir="$xdg_bin_home"
icon_name="nymvpn-x.svg"
icons_dir="$data_home/icons"
desktop_dir="$data_home/applications"
appimage="nymvpn-x_${vpnx_version}.AppImage"
wrapper_sh="nymvpn-x-wrapper.sh"
target_appimage="nymvpn-x.appimage"
core_archive="nym-vpn-core-v${vpnd_version}_linux_x86_64.tar.gz"
vpnd_bin="nym-vpnd"
vpnd_service="nym-vpnd.service"
units_dir="/usr/lib/systemd/system"

### desktop entry wrapper script ###
wrapper="#! /bin/bash

# fix an issue with NVIDIA gpu
# https://github.com/nymtech/nym-vpn-client/issues/305
export WEBKIT_DISABLE_DMABUF_RENDERER=1

RUST_LOG=info,nym_vpn_x=debug INSTALL_DIR/$target_appimage"
###

### desktop entry ###
desktop_entry="[Desktop Entry]
Name=NymVPN-x
Type=Application
Version=1.0
Comment=Decentralized, mixnet, and zero-knowledge VPN
Exec=INSTALL_DIR/$wrapper_sh %U
Icon=ICONS_DIR/$icon_name
Terminal=false
Categories=Network;"
###

### app icon ###
icon='<svg width="32" height="32" viewBox="0 0 32 32" fill="#FB6E4E" xmlns="http://www.w3.org/2000/svg"><path d="M3.7229 29.9997C-0.460546 26.7617 -1.23766 20.7391 2.0003 16.5557C5.23826 12.3722 11.2609 11.5951 15.4443 14.8331C19.6278 18.0711 20.4049 24.0937 17.1669 28.2771C13.9289 32.4605 7.90634 33.2377 3.7229 29.9997ZM28.0076 23.2647C33.3308 17.9415 33.3308 9.31561 28.0076 3.9924C22.6844 -1.3308 14.0455 -1.3308 8.73526 3.9924C3.42501 9.31561 3.41205 17.9415 8.73526 23.2647C14.0585 28.5879 22.6844 28.5879 28.0076 23.2647Z" fill="#FB6E4E"/></svg>'
###

### daemon service ###
service="[Unit]
Description=NymVPN daemon
StartLimitBurst=6
StartLimitIntervalSec=24
Wants=network-pre.target
After=network-pre.target NetworkManager.service systemd-resolved.service

[Service]
ExecStart=INSTALL_DIR/$vpnd_bin
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target"
###

pre_check() {
  if [ -z "$vpnx_tag" ] || [ -z "$vpnx_version" ]; then
    log "${B_RED}✗$RS \`vpnx_tag\` and \`vpnx_version\` must be set"
    exit 1
  fi
  if [ -z "$vpnd_tag" ] || [ -z "$vpnd_version" ]; then
    log "${B_RED}✗$RS \`vpnd_tag\` and \`vpnd_version\` must be set"
    exit 1
  fi
}

# Download `nymvpn-x` appimage
download_client() {
  _pushd "$temp_dir"
  log "  ${B_GRN}Downloading$RS $appimage"
  curl -fL -# "$appimage_url" -o $appimage
  log "  ${B_GRN}Downloading$RS $appimage.sha256sum"
  curl -fL -# "$appimage_url.sha256sum" -o "$appimage.sha256sum"
  log "  ${B_GRN}Checking$RS sha256sum"
  sha256sum --check --status "$appimage.sha256sum"
  _popd
}

# Download `nym-vpnd` prebuilt binary
download_daemon() {
  _pushd "$temp_dir"
  log "  ${B_GRN}Downloading$RS nym-vpnd archive"
  curl -fL -# "$vpnd_url" -o $core_archive
  log "  ${B_GRN}Downloading$RS archive sha256sum"
  curl -fL -# "$vpnd_url.sha256sum" -o "$core_archive.sha256sum"
  log "  ${B_GRN}Checking$RS sha256sum"
  sha256sum --check --status "$core_archive.sha256sum"
  log "  ${B_GRN}Unarchiving$RS nym-vpnd"
  tar -xzf "$core_archive"
  mv "${core_archive%.tar.gz}/$vpnd_bin" $vpnd_bin
  _popd
}

select_install_dir() {
  # prompt user for the installation directory
  # select between these 2 options
  # 1. ~/.local (default)
  # 2. /usr
  choice=""
  log "  ${B_GRN}Select$RS the install directory"
  prompt="    ${B_YLW}H$RS ~/.local (default) or ${B_YLW}U$RS /usr (${B_YLW}h$RS/${B_YLW}u$RS) "
  user_prompt choice "$prompt"

  if [ "$choice" = "u" ] || [ "$choice" = "U" ]; then
    prefix_install="/usr"
    install_dir="$usr_bin_dir"
    desktop_dir="/usr/share/applications"
    icons_dir="/usr/share/icons"
  else
    prefix_install="~/.local"
    install_dir="$xdg_bin_home"
    desktop_dir="$data_home/applications"
    icons_dir="$data_home/icons"
  fi
}

# checking if a directory is in the PATH
dir_in_path() {
  if [[ ":$PATH:" == *":$1:"* ]]; then
    return 0
  fi
  return 1
}

check_install_dir() {
  if ! dir_in_path "$xdg_bin_home"; then
    prefix_install="/usr"
    install_dir="$usr_bin_dir"
    desktop_dir="/usr/share/applications"
    icons_dir="/usr/share/icons"
    log "  ${B_GRN}Install$RS directory set to $prefix_install"
  else
    select_install_dir
    log "  ${B_GRN}Selected$RS install directory $prefix_install"
  fi
}

# check if a unit exists
# return 0 if found, 1 if not found
check_unit() {
  if systemctl status nym-vpnd &>/dev/null; then
    return 0
  else
    status=$?
    if [ $status -eq 4 ]; then
      # exit code 4 means the service is not found
      return 1
    fi
  fi
  # other exit code mean the service exists
  return 0
}

sanity_check() {
  log "  ${B_GRN}Checking$RS for existing installation"
  # check for any existing installation, if found cancel the script
  if bin_in_path $vpnd_bin || bin_in_path nymvpn-x || bin_in_path $target_appimage; then
    log "  ${I_YLW}Please remove or cleanup any existing installation before running this script$RS"
    exit 1
  fi

  files_check=("$install_dir/$target_appimage" "$install_dir/$wrapper_sh" "$desktop_dir/nymvpn-x.desktop" "$icons_dir/$icon_name" "$install_dir/$vpnd_bin" "$units_dir/$vpnd_service")

  for file in "${files_check[@]}"; do
    if [ -a "$file" ]; then
      log "${B_YLW}⚠$RS $file already exists"
      log "  ${I_YLW}Please remove or cleanup any existing installation before running this script$RS"
      exit 1
    fi
  done

  if check_unit "nym_vpnd"; then
    log "  ${I_YLW}⚠$RS nym-vpnd unit service found on the system$RS"
    log "  ${I_YLW}Please remove or cleanup any existing installation before running this script$RS"
    exit 1
  fi
}

# prompt user to enable and start the service
start_service() {
  choice=""
  log "  ${B_GRN}Enable$RS and ${B_GRN}start$RS nym-vpnd service?"
  prompt="    ${B_YLW}Y${RS}es (recommended) ${B_YLW}N${RS}o "
  user_prompt choice "$prompt"

  if [ "$choice" = "y" ] || [ "$choice" = "Y" ]; then
    sudo systemctl enable $vpnd_service
    sudo systemctl start $vpnd_service
    log "   ${B_GRN}Enabled$RS and started nym-vpnd service"
  else
    log "   Run the following commands to enable and start the VPN service:
    ${I_YLW}sudo systemctl enable $vpnd_service$RS
    ${I_YLW}sudo systemctl start $vpnd_service$RS"
  fi
}

install_client() {
  log "  ${B_GRN}Installing$RS nymvpn-x.AppImage"
  _pushd "$temp_dir"
  echo "${wrapper/INSTALL_DIR/$install_dir}" >"$wrapper_sh"
  _popd

  if [ "$install_dir" = "$usr_bin_dir" ]; then
    log "  ${B_YLW}sudo$RS needed to install AppImage in $install_dir"
    sudo install -o "$(id -u)" -g "$(id -g)" -Dm755 "$temp_dir/$appimage" "$install_dir/$target_appimage"
    sudo install -o "$(id -u)" -g "$(id -g)" -Dm755 "$temp_dir/$wrapper_sh" "$install_dir/$wrapper_sh"
  else
    install -Dm755 "$temp_dir/$appimage" "$install_dir/$target_appimage"
    install -Dm755 "$temp_dir/$wrapper_sh" "$install_dir/$wrapper_sh"
  fi

  log "   ${B_GRN}Installed$RS $(tilded "$install_dir/$target_appimage")"
  log "   ${B_GRN}Installed$RS $(tilded "$install_dir/$wrapper_sh")"

  log "  ${B_GRN}Installing$RS desktop entry"
  _pushd "$temp_dir"
  echo "${desktop_entry/INSTALL_DIR/$install_dir}" >"nymvpn-x.desktop"
  sed -i "s|ICONS_DIR|$icons_dir|" "nymvpn-x.desktop"
  echo "$icon" >"$icon_name"
  _popd
  if [ "$install_dir" = "$usr_bin_dir" ]; then
    log "  ${B_YLW}sudo$RS needed to install desktop entry in $desktop_dir"
    sudo install -Dm644 "$temp_dir/nymvpn-x.desktop" "$desktop_dir/nymvpn-x.desktop"
    sudo install -Dm644 "$temp_dir/$icon_name" "$icons_dir/$icon_name"
  else
    install -Dm644 "$temp_dir/nymvpn-x.desktop" "$desktop_dir/nymvpn-x.desktop"
    install -Dm644 "$temp_dir/$icon_name" "$icons_dir/$icon_name"
  fi
  log "   ${B_GRN}Installed$RS $(tilded "$desktop_dir/nymvpn-x.desktop")"
  log "   ${B_GRN}Installed$RS $(tilded "$icons_dir/$icon_name")"
  install -Dm755 -d "$state_home/$app_dir"
}

install_daemon() {
  log "  ${B_GRN}Installing$RS nym-vpnd"
  if [ "$install_dir" = "$usr_bin_dir" ]; then
    log "  ${B_YLW}sudo$RS needed to install nym-vpnd in $install_dir"
    sudo install -o "$(id -u)" -g "$(id -g)" -Dm755 "$temp_dir/$vpnd_bin" "$install_dir/$vpnd_bin"
  else
    install -Dm755 "$temp_dir/$vpnd_bin" "$install_dir/$vpnd_bin"
  fi
  log "   ${B_GRN}Installed$RS $(tilded "$install_dir/$vpnd_bin")"

  log "  ${B_GRN}Installing$RS systemd service"
  _pushd "$temp_dir"
  echo "${service/INSTALL_DIR/$install_dir}" >"$vpnd_service"
  _popd
  log "  ${B_YLW}sudo$RS needed to install nym-vpnd.service in $units_dir"
  sudo install -Dm644 "$temp_dir/$vpnd_service" "$units_dir/$vpnd_service"
  log "   ${B_GRN}Installed$RS $(tilded "$units_dir/$vpnd_service")"
}

post_install() {
  if ! dir_in_path "$install_dir"; then
    log "${B_YLW}⚠$RS $install_dir is not in the ${BLD}PATH$RS
  please add it using your shell configuration"
  fi
}

cleanup() {
  rm -rf "$temp_dir"
}

need_cmd install
need_cmd sudo
need_cmd sed
need_cmd curl
need_cmd tar
need_cmd sha256sum
need_cmd which

log "$ITL${B_GRN}nym$RS$ITL${B_GRY}VPN$RS\n"
log "  nymvpn-x $ITL${B_YLW}$vpnx_version$RS"
log "  nym-vpnd $ITL${B_YLW}$vpnd_version$RS\n"

pre_check
check_install_dir
sanity_check
download_client
download_daemon
install_client
install_daemon
start_service
post_install
cleanup

log "\n${B_GRN}✓$RS"

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
BLD="\e[1m"      # bold
RS="\e[0m"       # style reset
B_RED="$BLD$RED"
B_GRN="$BLD$GRN"
B_YLW="$BLD$YLW"
####

# TODO set/replace with the target tag and version
tag=
version=
appimage_url="https://github.com/nymtech/nym-vpn-client/releases/download/$tag/nym-vpn_${version}.AppImage"

# disable the desktop entry install if needed
desktop_entry_disabled=false

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
install_dir="$HOME/.local/bin"
icons_dir="$data_home/icons"
appimage="nym-vpn_${version}.AppImage"
target_appimage="nym-vpn.appimage"
desktop_dir="$data_home/applications"
wrapper="nym-vpn-wrapper.sh"
policy="net.nymtech.nymvpn.policy"

### desktop entry ###
desktop_entry="[Desktop Entry]
Name=NymVPN
Type=Application
Version=1.0
Comment=Decentralized, mixnet, and zero-knowledge VPN
Exec=$install_dir/$wrapper %U
Icon=$icons_dir/nym-vpn.svg
Terminal=false
Categories=Network;"
###

### app icon ###
icon='<svg width="32" height="32" viewBox="0 0 32 32" fill="#FB6E4E" xmlns="http://www.w3.org/2000/svg"><path d="M3.7229 29.9997C-0.460546 26.7617 -1.23766 20.7391 2.0003 16.5557C5.23826 12.3722 11.2609 11.5951 15.4443 14.8331C19.6278 18.0711 20.4049 24.0937 17.1669 28.2771C13.9289 32.4605 7.90634 33.2377 3.7229 29.9997ZM28.0076 23.2647C33.3308 17.9415 33.3308 9.31561 28.0076 3.9924C22.6844 -1.3308 14.0455 -1.3308 8.73526 3.9924C3.42501 9.31561 3.41205 17.9415 8.73526 23.2647C14.0585 28.5879 22.6844 28.5879 28.0076 23.2647Z" fill="#FB6E4E"/></svg>'
###

### wrapper script ###
wrapper_script="#!/bin/bash

pkexec $install_dir/$target_appimage >$state_home/nym-vpn/vpn.log 2>&1"
###

### polkit action ###
polkit_action="<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE policyconfig PUBLIC
 \"-//freedesktop//DTD PolicyKit Policy Configuration 1.0//EN\"
 \"http://www.freedesktop.org/standards/PolicyKit/1/policyconfig.dtd\">
<policyconfig>
 <vendor>Nym</vendor>
 <vendor_url>https://nymvpn.com</vendor_url>
 <icon_name>nym-vpn</icon_name>
 <action id=\"net.nymtech.nymvpn\">
 <description>NymVPN</description>
 <message>NymVPN requires root privileges to create a local virtual network device and set system routing rules</message>
 <defaults>
 <allow_any>no</allow_any>
 <allow_inactive>auth_admin</allow_inactive>
 <allow_active>auth_admin_keep</allow_active>
 </defaults>
 <annotate key=\"org.freedesktop.policykit.exec.path\">$install_dir/$target_appimage</annotate>
 <annotate key=\"org.freedesktop.policykit.exec.allow_gui\">true</annotate>
 </action>
</policyconfig>"
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

_install() {
  need_cmd install

  log "  ${B_GRN}Installing$RS AppImage"
  install -Dm755 "$temp_dir/$appimage" "$install_dir/$target_appimage"
  path=$(tilded "$install_dir/$target_appimage")
  log "   ${B_GRN}Installed$RS $path"

  if [ $desktop_entry_disabled == true ] || ! command -v "pkexec" >/dev/null 2>&1; then
    return
  fi

  log "  ${B_GRN}Installing$RS desktop entry"
  _pushd "$temp_dir"
  echo "$desktop_entry" >"nym-vpn.desktop"
  echo "$icon" >"nym-vpn.svg"
  echo "$wrapper_script" >"$wrapper"
  echo "$polkit_action" >"$policy"
  _popd
  install -Dm644 "$temp_dir/nym-vpn.svg" "$icons_dir/nym-vpn.svg"
  install -Dm644 "$temp_dir/nym-vpn.desktop" "$desktop_dir/nym-vpn.desktop"
  install -Dm755 -d "$state_home/nym-vpn"
  install -Dm755 "$temp_dir/$wrapper" "$install_dir/$wrapper"
  path=$(tilded "$desktop_dir/nym-vpn.desktop")
  log "   ${B_GRN}Installed$RS $path"

  log "  ${B_GRN}Installing$RS polkit policy"
  sudo install -Dm644 "$temp_dir/$policy" "/usr/share/polkit-1/actions/$policy"
  log "   ${B_GRN}Installed$RS /usr/share/polkit-1/actions/$policy"
}

post_install() {
  # checking if ~/.local/bin is in the PATH
  if ! [[ ":$PATH:" == *":$install_dir:"* ]]; then
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

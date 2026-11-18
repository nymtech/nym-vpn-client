#!/bin/bash
#
# Tauri also invokes this from the NSIS output directory to sign the
# uninstaller, so always run relative to src-tauri/.
#

cd "$(dirname "$0")/.." || exit 1

code_sign_tool="./CodeSignTool.sh"

# If CodeSignTool.sh then we aren't code-signing this build.
if [[ ! -f "$code_sign_tool" ]]; then
	echo "Not code signing $1"
	exit 0
fi

function sign {
	local exe="$1"
	if [[ ! -f "$exe" ]]; then
		echo "Cannot code sign $exe: file does not exist" >&2
		exit 1
	fi

	echo "Code signing $exe"

	local name="${exe##*[\\/]}"
	local program_name="${2:-${name%.*}}"

	if ! "$code_sign_tool" \
		sign \
		-username "$SSL_COM_USERNAME" \
		-password "$SSL_COM_PASSWORD" \
		-credential_id "$SSL_COM_CREDENTIAL_ID" \
		-totp_secret "$SSL_COM_TOTP_SECRET" \
		-program_name "$program_name" \
		-input_file_path "$exe" \
		-override; then
			echo "Failed to sign $exe" >&2
			exit 2
	fi
}

file="$1"
dir="${file%[\\/]*}"

if [[ "$file" == *.tmp ]]; then
	signable="${file%.tmp}.exe"
	cp -f "$file" "$signable" || exit 1
	sign "$signable" uninstall
	mv -f "$signable" "$file" || exit 1
else
	sign "$file"
fi

# Remove once Tauri signs nsExec.dll: https://github.com/tauri-apps/tauri/pull/16572
if [[ "${file##*[\\/]}" == "System.dll" && "${dir##*[\\/]}" == "x86-unicode" ]]; then
	sign "$dir/nsExec.dll"
fi

exit 0

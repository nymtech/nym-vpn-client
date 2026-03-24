#!/bin/bash
#
# This custom code signing script is used to sign more than just the bundle (NymVPN.exe),
# but when it is used to sign that, it will also sign the daemon and the split-tunnel driver
# artifacts.
#
# This assumes the cwd is src-tauri/.
#

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

	if ! "$code_sign_tool" \
		sign \
		-username "$SSL_COM_USERNAME" \
		-password "$SSL_COM_PASSWORD" \
		-credential_id "$SSL_COM_CREDENTIAL_ID" \
		-totp_secret "$SSL_COM_TOTP_SECRET" \
		-program_name "${exe%.*}" \
		-input_file_path "$exe" \
		-override; then
			echo "Failed to sign $exe" >&2
			exit 2
	fi
}

exe=$1

# If the exe is NymVPN.exe then sign the daemon and split-tunnel driver first
if [[ "$exe" == *NymVPN.exe ]]; then
	sign "nym-vpnd.exe"
	sign "nymvpn-split-tunnel.sys"
fi

sign "$exe"

exit 0

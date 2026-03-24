NYM_VPN_EXE_PATH=$1
SSL_COM_USERNAME=$2
SSL_COM_PASSWORD=$3
SSL_COM_CREDENTIAL_ID=$4
SSL_COM_TOTP_SECRET=$5
CODE_SIGNING_TOOL="/c/actions-runner/_work/nym-vpn-client/nym-vpn-client/nym-vpn-app/src-tauri/CodeSignTool.sh"

echo "Code signing in directory $(pwd)"

for exe in nym-vpnd.exe nymvpn-split-tunnel.sys $NYM_VPN_EXE_PATH; do
	if [ ! -f "$exe" ]; then
		echo "Cannot code sign $exe: file does not exist" >&2
		exit 1
	fi

	echo "Code signing $exe"

	if ! $CODE_SIGNING_TOOL \
		sign \
		-username $SSL_COM_USERNAME \
		-password $SSL_COM_PASSWORD \
		-credential_id $SSL_COM_CREDENTIAL_ID \
		-totp_secret $SSL_COM_TOTP_SECRET \
		-program_name "${exe%.*}" \
		-input_file_path $exe \
		-override; then
			echo "Failed to sign $exe" >&2
			exit 2
		fi
done

exit 0

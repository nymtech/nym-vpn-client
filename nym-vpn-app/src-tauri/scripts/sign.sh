NYM_VPN_EXE_PATH=$1
SSL_COM_USERNAME=$2
SSL_COM_PASSWORD=$3
SSL_COM_CREDENTIAL_ID=$4
SSL_COM_TOTP_SECRET=$5
CODE_SIGNING_TOOL="/c/actions-runner/_work/nym-vpn-client/nym-vpn-client/nym-vpn-app/src-tauri/CodeSignTool.sh"

# TEMP: Find the driver file
find /c/actions-runner/_work/nym-vpn-client/nym-vpn-client -name \*.sys -print

if ! $CODE_SIGNING_TOOL_PATH \
sign \
-username $SSL_COM_USERNAME \
-password $SSL_COM_PASSWORD \
-credential_id $SSL_COM_CREDENTIAL_ID \
-totp_secret $SSL_COM_TOTP_SECRET \
-program_name nym-vpnd \
-input_file_path /c/actions-runner/_work/nym-vpn-client/nym-vpn-client/nym-vpn-app/src-tauri/nym-vpnd.exe \
-override; then
	echo "Failed to sign nym-vpnd.exe" >&2
	exit 1
fi

if ! $CODE_SIGNING_TOOL \
sign \
-username $SSL_COM_USERNAME \
-password $SSL_COM_PASSWORD \
-credential_id $SSL_COM_CREDENTIAL_ID \
-totp_secret $SSL_COM_TOTP_SECRET \
-program_name NymVPN \
-input_file_path $NYM_VPN_EXE_PATH \
-override; then
	echo "Failed to sign $NYM_VPN_EXE_PATH" >&2
	exit 1
fi
SSL_COM_USERNAME=$2
SSL_COM_PASSWORD=$3
SSL_COM_CREDENTIAL_ID=$4
SSL_COM_TOTP_SECRET=$5

/c/actions-runner/_work/nym-vpn-client/nym-vpn-client/nym-vpn-app/src-tauri/CodeSignTool.sh \
sign \
-username $SSL_COM_USERNAME \
-password $SSL_COM_PASSWORD \
-credential_id $SSL_COM_CREDENTIAL_ID \
-totp_secret $SSL_COM_TOTP_SECRET \
-program_name nym-vpnd \
-input_file_path /c/actions-runner/_work/nym-vpn-client/nym-vpn-client/nym-vpn-app/src-tauri/nym-vpnd.exe \
-override && \
/c/actions-runner/_work/nym-vpn-client/nym-vpn-client/nym-vpn-app/src-tauri/CodeSignTool.sh \
sign \
-username $SSL_COM_USERNAME \
-password $SSL_COM_PASSWORD \
-credential_id $SSL_COM_CREDENTIAL_ID \
-totp_secret $SSL_COM_TOTP_SECRET \
-program_name NymVPN \
-input_file_path $1 \
-override
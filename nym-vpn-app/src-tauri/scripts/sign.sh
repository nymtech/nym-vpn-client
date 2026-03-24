# This assumes the cwd is src-tauri/.

echo "Code signing in directory $(pwd)"

for exe in nym-vpnd.exe nymvpn-split-tunnel.sys target/release/NymVPN.exe; do
	if [ ! -f "$exe" ]; then
		echo "Cannot code sign $exe: file does not exist" >&2
		exit 1
	fi

	echo "Code signing $exe"

	if ! ./CodeSignTool.sh \
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
done

exit 0

package net.nymtech.nymvpn.util

object Constants {

	const val APP_PROJECT_NAME = "nym-vpn-android"

	const val SUBSCRIPTION_TIMEOUT = 5_000L
	const val LOG_BUFFER_SIZE = 5_000L

	const val TEXT_MIME_TYPE = "text/plain"
	const val BASE_LOG_FILE_NAME = "nym_vpn_logs"

	// testing stuff
	const val CONNECT_TEST_TAG = "connectTag"
	const val LOGIN_TEST_TAG = "loginTag"
	const val DISCONNECT_TEST_TAG = "disconnectTag"

	const val VPN_SETTINGS_PACKAGE = "android.net.vpn.SETTINGS"

	const val KOTLIN_LICENSES_ASSET_FILE_NAME = "artifacts.json"
	const val RUST_LICENSES_ASSET_FILE_NAME = "licenses_rust.json"

	// error message ids for vpn api errors
	const val MAX_DEVICES_REACHED_ID = "nym-vpn-website.public-api.register-device.max-devices-exceeded"
	const val MAX_BANDWIDTH_REACHED_ID = "nym-vpn-website.public-api.device.zk-nym.request_failed.fair_usage_used_for_month"
	const val SUBSCRIPTION_EXPIRED_ID = "nym-vpn-website.public-api.device.zk-nym.request_failed.no_active_subscription"
}

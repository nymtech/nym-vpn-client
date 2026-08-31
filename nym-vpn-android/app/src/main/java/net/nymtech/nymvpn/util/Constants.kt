package net.nymtech.nymvpn.util

object Constants {

	const val APP_PROJECT_NAME = "nym-vpn-android"

	const val APP_ID = "net.nymtech.nymvpn"

	const val SUBSCRIPTION_TIMEOUT = 5_000L
	const val LOG_BUFFER_SIZE = 5_000L

	const val AUTO_START_NETWORK_WAIT_MS = 2_000L
	const val AUTO_START_INIT_WAIT_MS = 15_000L
	const val AUTO_START_STUCK_STATE_TIMEOUT_MS = 45_000L

	const val BASE_LOG_FILE_NAME = "nym_vpn_logs"

	// testing stuff
	const val CONNECT_TEST_TAG = "connectTag"
	const val LOGIN_TEST_TAG = "loginTag"
	const val DISCONNECT_TEST_TAG = "disconnectTag"

	const val VPN_SETTINGS_PACKAGE = "android.net.vpn.SETTINGS"

	const val KOTLIN_LICENSES_ASSET_FILE_NAME = "artifacts.json"
	const val RUST_LICENSES_ASSET_FILE_NAME = "licenses_rust.json"

	const val URL_STREAMING_SERVICES_ARTICLE = "https://support.nym.com/hc/en-us/articles/35279486714641" +
		"-Why-can-t-I-access-streaming-services-while-using-NymVPN"

	const val URL_GATEWAYS_LOCATION = "https://support.nymvpn.com/hc/en-us/articles/26448676449297" +
		"-How-is-server-location-determined-by-NymVPN"

	val countryCodesForRegionSupport = listOf("us", "ca", "au", "mx", "br", "in", "cn")
}

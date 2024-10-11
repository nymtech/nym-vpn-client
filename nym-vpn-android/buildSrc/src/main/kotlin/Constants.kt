object Constants {
	const val VERSION_NAME = "v1.1.1"
    const val VERSION_CODE = 11100
    const val TARGET_SDK = 34
    const val COMPILE_SDK = 34
    const val MIN_SDK = 24

	const val NIGHTLY_CODE = 42
	const val PRERELEASE_CODE = 99

    const val JVM_TARGET = "17"

	const val APP_NAME = "nymvpn"
	const val NAMESPACE_ROOT = "net.nymtech"
    const val APP_ID = "${NAMESPACE_ROOT}.${APP_NAME}"

    const val VPN_LIB_NAME = "vpn"

    const val RELEASE = "release"
	const val PRERELEASE = "prerelease"
	const val NIGHTLY = "nightly"
    const val TYPE = "type"

	const val FLAVOR = "FLAVOR"

    const val STORE_PASS_VAR = "SIGNING_STORE_PASSWORD"
    const val KEY_ALIAS_VAR = "SIGNING_KEY_ALIAS"
    const val KEY_PASS_VAR = "SIGNING_KEY_PASSWORD"
    const val KEY_STORE_PATH_VAR = "KEY_STORE_PATH"

    const val FDROID = "fdroid"
    const val GENERAL = "general"
    const val BUILD_LIB_TASK = "buildDeps"

    const val ANDROID_TERMS_URL = "https://developer.android.com/studio/terms.html"
	const val XZING_LICENSE_URL: String = "https://github.com/journeyapps/zxing-android-embedded/blob/master/COPYING"

	//build config
    const val SENTRY_DSN = "SENTRY_DSN"

	const val NYM_SHARED_LIB = "libnym_vpn_lib.so"
	const val WG_SHARED_LIB = "libwg.so"

	const val CLEAN_TASK = "clean"
	const val BUILD_SOURCE_TASK = "buildSource"
	const val DOWNLOAD_LIB_TASK = "downloadLib"

	const val CORE_BUILD_PROP = "libVersion"

}

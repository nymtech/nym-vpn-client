import org.gradle.api.JavaVersion

object Constants {
	const val VERSION_NAME = "2026.12.0-beta.9"
    const val VERSION_CODE = 20261200
    const val TARGET_SDK = 36
    const val COMPILE_SDK = 37
    const val MIN_SDK = 24

	const val JVM_TARGET = "21"
	val JAVA_VERSION = JavaVersion.VERSION_21

	const val NDK_VERSION = "28.2.13676358"

	const val APP_NAME = "nymvpn"
	const val NAMESPACE = "net.nymtech"
	const val APP_ID = "${NAMESPACE}.${APP_NAME}"

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

	//licensee
	val allowedLicenses = listOf("MIT", "Apache-2.0", "BSD-3-Clause")
	const val ANDROID_TERMS_URL = "https://developer.android.com/studio/terms.html"
	const val XZING_LICENSE_URL: String = "https://github.com/journeyapps/zxing-android-embedded/blob/master/COPYING"
}

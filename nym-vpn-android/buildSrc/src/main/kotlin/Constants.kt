import org.gradle.api.JavaVersion

object Constants {
    const val VERSION_NAME = "v0.1.1"
    const val VERSION_CODE = 11
    const val TARGET_SDK = 34
    const val COMPILE_SDK = 34
    const val MIN_SDK = 24

    const val JVM_TARGET = "17"
    val JAVA_VERSION = JavaVersion.VERSION_17

    const val COMPOSE_COMPILER_EXTENSION_VERSION = "1.5.11"
    const val NAMESPACE = "net.nymtech"

    const val APP_NAME = "nymvpn"
    const val VPN_LIB_NAME = "vpn_client"

    const val RELEASE = "release"
    const val TYPE = "type"

    const val STORE_PASS_VAR = "SIGNING_STORE_PASSWORD"
    const val KEY_ALIAS_VAR = "SIGNING_KEY_ALIAS"
    const val KEY_PASS_VAR = "SIGNING_KEY_PASSWORD"
    const val KEY_STORE_PATH_VAR = "KEY_STORE_PATH"

    const val FDROID = "fdroid"
    const val GENERAL = "general"
    const val BUILD_LIB_TASK = "buildDeps"

    const val SANDBOX_API_URL = "https://sandbox-nym-api1.nymtech.net/api"
    const val SANDBOX_EXPLORER_URL = "https://sandbox-explorer.nymtech.net/api"

    const val MAINNET_API_URL = "https://validator.nymtech.net/api/"
    const val MAINNET_EXPLORER_URL = "https://explorer.nymtech.net/api/"

    //licensee
    val allowedLicenses = listOf("MIT", "Apache-2.0", "BSD-3-Clause")
    const val ANDROID_TERMS_URL = "https://developer.android.com/studio/terms.html"

    //build config
    const val OPT_IN_REPORTING = "OPT_IN_REPORTING"
    const val SENTRY_DSN = "SENTRY_DSN"
}
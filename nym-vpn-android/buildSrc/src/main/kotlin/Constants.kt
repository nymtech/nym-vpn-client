import org.gradle.api.JavaVersion

object Constants {
    const val VERSION_NAME = "v0.0.3-test"
    const val VERSION_CODE = 3
    const val TARGET_SDK = 34
    const val COMPILE_SDK = 34
    const val MIN_SDK = 24
    const val NDK_VERSION = "26.1.10909125"

    const val JVM_TARGET = "17"
    val JAVA_VERSION = JavaVersion.VERSION_17

    const val COMPOSE_COMPILER_EXTENSION_VERSION = "1.5.8"
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
}
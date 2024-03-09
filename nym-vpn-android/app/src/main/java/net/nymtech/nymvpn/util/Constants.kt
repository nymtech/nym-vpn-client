package net.nymtech.nymvpn.util

import android.system.Os

object Constants {

    const val DEFAULT_COUNTRY_ISO = "DE"

    const val SUBSCRIPTION_TIMEOUT = 5_000L
    const val LOG_BUFFER_DELAY = 3_000L
    const val LOG_BUFFER_SIZE = 5_000L

    const val EMAIL_MIME_TYPE = "message/rfc822"
    const val TEXT_MIME_TYPE = "text/plain"
    const val BASE_LOG_FILE_NAME = "nym_vpn_logs"

    //must end in /
    const val SANDBOX_URL = "https://sandbox-nym-api1.nymtech.net/api/v1/"

    const val SENTRY_DEV_ENV = "development"
    const val SENTRY_PROD_ENV = "production"

    const val NYM_VPN_LIB_TAG = "libnymvpn"
}
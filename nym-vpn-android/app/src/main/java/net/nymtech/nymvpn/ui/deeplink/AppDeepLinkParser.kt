package net.nymtech.nymvpn.ui.deeplink

import net.nymtech.nymvpn.util.Constants
import java.net.URI
import java.net.URLDecoder

sealed interface AppDeepLink {
	data class PrivyReturn(val callbackUrl: String) : AppDeepLink

	// callbackUrl is null when the return carries no account payload.
	data class PaymentReturn(val callbackUrl: String?) : AppDeepLink
}

// Allowlist parser for website-to-app return links; anything not matching exactly returns null.
object AppDeepLinkParser {

	private const val HTTPS = "https"
	private const val LEGACY_SCHEME = "nymvpn"
	private const val LEGACY_PRIVY_HOST = "auth"
	private const val LEGACY_PRIVY_PATH = "/privy/privateKey"
	private const val LEGACY_PAYMENT_HOST = "account"
	private const val LEGACY_PAYMENT_PATH = "/response"

	private const val PARAM_DEEPLINK_ID = "deeplink_id"
	private const val PARAM_PAYLOAD = "payload"
	private val CALLBACK_PARAMS = setOf(PARAM_DEEPLINK_ID, PARAM_PAYLOAD)

	private const val MAX_RAW_LENGTH = 2048
	private const val MAX_PARAMS_LENGTH = 512

	private val DEEPLINK_ID = Regex("^[0-9]{1,20}$")

	// Encrypted payload is ~108 bytes, ~148 base58 chars.
	private val BASE58_PAYLOAD = Regex("^[1-9A-HJ-NP-Za-km-z]{100,200}$")

	fun parse(raw: String?): AppDeepLink? {
		if (raw.isNullOrEmpty() || raw.length > MAX_RAW_LENGTH) return null
		val uri = runCatching { URI(raw) }.getOrNull() ?: return null
		if (uri.rawUserInfo != null || uri.port != -1) return null

		val scheme = uri.scheme
		val host = uri.host
		val path = uri.rawPath
		val isHttps = scheme == HTTPS && host == Constants.APP_LINK_HOST
		val isLegacy = scheme == LEGACY_SCHEME

		return when {
			isHttps && path == Constants.APP_LINK_PRIVY_PATH -> {
				if (uri.rawFragment != null && uri.rawQuery != null) return null
				parseCallback(uri.rawFragment ?: uri.rawQuery, LEGACY_PRIVY_HOST, LEGACY_PRIVY_PATH)
					?.let(AppDeepLink::PrivyReturn)
			}

			isLegacy && host == LEGACY_PRIVY_HOST && path.isLegacyPath(LEGACY_PRIVY_PATH) -> {
				if (uri.rawFragment != null) return null
				parseCallback(uri.rawQuery, LEGACY_PRIVY_HOST, LEGACY_PRIVY_PATH)
					?.let(AppDeepLink::PrivyReturn)
			}

			isHttps && path == Constants.APP_LINK_PAYMENT_PATH -> {
				if (uri.rawFragment != null && uri.rawQuery != null) return null
				parsePayment(uri.rawFragment ?: uri.rawQuery)
			}

			isLegacy && host == LEGACY_PAYMENT_HOST && path.isLegacyPath(LEGACY_PAYMENT_PATH) -> {
				if (uri.rawFragment != null) return null
				parsePayment(uri.rawQuery)
			}

			else -> null
		}
	}

	// Tolerate a trailing slash on legacy links the live website already sends.
	private fun String?.isLegacyPath(expected: String): Boolean = this == expected || this == "$expected/"

	private fun parsePayment(params: String?): AppDeepLink.PaymentReturn? {
		if (params.isNullOrEmpty()) return AppDeepLink.PaymentReturn(null)
		val callback = parseCallback(params, LEGACY_PAYMENT_HOST, LEGACY_PAYMENT_PATH) ?: return null
		return AppDeepLink.PaymentReturn(callback)
	}

	private fun parseCallback(params: String?, host: String, path: String): String? {
		if (params.isNullOrEmpty() || params.length > MAX_PARAMS_LENGTH) return null

		val pairs = params.split('&').map { it.split('=', limit = 2) }
		if (pairs.any { it.size != 2 }) return null

		val decoded = pairs.map { (k, v) -> (decode(k) ?: return null) to (decode(v) ?: return null) }
		val keys = decoded.map { it.first }
		if (keys.size != CALLBACK_PARAMS.size || keys.toSet() != CALLBACK_PARAMS) return null

		val values = decoded.toMap()
		val deeplinkId = values.getValue(PARAM_DEEPLINK_ID).takeIf(DEEPLINK_ID::matches) ?: return null
		val payload = values.getValue(PARAM_PAYLOAD).takeIf(BASE58_PAYLOAD::matches) ?: return null

		return "$LEGACY_SCHEME://$host$path?$PARAM_DEEPLINK_ID=$deeplinkId&$PARAM_PAYLOAD=$payload"
	}

	private fun decode(value: String): String? = runCatching { URLDecoder.decode(value, "UTF-8") }.getOrNull()
}

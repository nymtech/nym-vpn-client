package net.nymtech.nymvpn.manager.freepass

import java.net.URI
import java.net.URISyntaxException

// Shape check only, matching isValidRedemptionCode and redemption_link in websites www/vpn-api.
object FreepassCode {
	private const val SCHEME = "https"
	private val HOSTS = setOf("nym.com", "www.nym.com")
	private const val PATH = "/account/freepass"
	private const val CODE_PARAM = "code="
	private const val MAX_LENGTH = 512

	private val CODE_PATTERN = Regex("\\w{4,30}")

	fun parse(contents: String): String? {
		val trimmed = contents.trim()
		if (trimmed.length > MAX_LENGTH) return null
		if (CODE_PATTERN.matches(trimmed)) return trimmed
		val uri = try {
			URI(trimmed)
		} catch (_: URISyntaxException) {
			return null
		}
		val valid = uri.scheme == SCHEME &&
			uri.rawUserInfo == null &&
			uri.host in HOSTS &&
			uri.port == -1 &&
			uri.rawPath == PATH &&
			uri.rawFragment == null
		if (!valid) return null
		val code = uri.rawQuery?.split('&')?.firstOrNull { it.startsWith(CODE_PARAM) }?.removePrefix(CODE_PARAM) ?: return null
		return code.takeIf { CODE_PATTERN.matches(it) }
	}
}

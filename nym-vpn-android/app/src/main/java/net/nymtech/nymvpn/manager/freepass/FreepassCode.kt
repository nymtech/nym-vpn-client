package net.nymtech.nymvpn.manager.freepass

import java.net.URI
import java.net.URISyntaxException

// Shape check only, matching isValidRedemptionCode and redemption_link in websites www/vpn-api.
// Accepts a bare code, the legacy /account/freepass?code= link and the /download#code= link, which can be scanned before the app is installed.
object FreepassCode {
	private const val SCHEME = "https"
	private val HOSTS = setOf("nym.com", "www.nym.com")
	private const val FREEPASS_PATH = "/account/freepass"
	private const val DOWNLOAD_PATH = "/download"
	private const val CODE_PARAM = "code="
	private const val MAX_LENGTH = 512

	private val CODE_PATTERN = Regex("\\w{4,30}")

	fun parse(contents: String): String? {
		val trimmed = contents.trim()
		if (trimmed.length > MAX_LENGTH) return null
		if (CODE_PATTERN.matches(trimmed)) return trimmed
		// Typed links often lack the scheme.
		val uri = try {
			URI(if ("://" in trimmed) trimmed else "$SCHEME://$trimmed")
		} catch (_: URISyntaxException) {
			return null
		}
		val valid = uri.scheme == SCHEME &&
			uri.rawUserInfo == null &&
			uri.host in HOSTS &&
			uri.port == -1
		if (!valid) return null
		val params = when (uri.rawPath) {
			FREEPASS_PATH -> uri.rawQuery.takeIf { uri.rawFragment == null }
			DOWNLOAD_PATH -> uri.rawFragment
			else -> null
		} ?: return null
		val code = params.split('&').firstOrNull { it.startsWith(CODE_PARAM) }?.removePrefix(CODE_PARAM) ?: return null
		return code.takeIf { CODE_PATTERN.matches(it) }
	}
}

package net.nymtech.nymvpn.manager.pairing

import java.net.URI
import java.net.URISyntaxException

// Shape check only; the core verifies the key. Spec and shared test vectors: docs/qr-pairing-format.md.
object PairingQrCode {
	private const val SCHEME = "https"
	private const val HOST = "nym.com"
	private const val PATH = "/qr-pairing"
	private const val KEY_PARAM = "pubkey="
	internal const val MAX_LENGTH = 512

	// Base58 of a 32-byte key.
	private val KEY_PATTERN = Regex("[1-9A-HJ-NP-Za-km-z]{32,44}")

	fun parsePublicKey(contents: String): String? {
		if (contents.length > MAX_LENGTH) return null
		val uri = try {
			URI(contents)
		} catch (_: URISyntaxException) {
			return null
		}
		val valid = uri.scheme == SCHEME &&
			uri.rawUserInfo == null &&
			uri.host == HOST &&
			uri.port == -1 &&
			uri.rawPath == PATH &&
			uri.rawFragment == null
		if (!valid) return null
		val query = uri.rawQuery ?: return null
		if (!query.startsWith(KEY_PARAM)) return null
		return query.removePrefix(KEY_PARAM).takeIf { KEY_PATTERN.matches(it) }
	}
}

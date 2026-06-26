package net.nymtech.nymvpn.util

sealed interface FreepassParseResult {
    data class Valid(val code: String) : FreepassParseResult
    data object Invalid : FreepassParseResult
}

private val BASE58_CODE = Regex("^[1-9A-HJ-NP-Za-km-z]{4,128}$")
private const val MAX_RAW_LEN = 4096

private fun isTrustedHost(host: String?): Boolean {
    if (host == null) return false
    val h = host.lowercase()
    return h == "nym.com" || h.endsWith(".nym.com")
}

private fun validateCode(candidate: String): FreepassParseResult =
    if (BASE58_CODE.matches(candidate)) FreepassParseResult.Valid(candidate) else FreepassParseResult.Invalid

private fun queryParam(uri: java.net.URI, name: String): String? {
    val query = uri.query ?: return null
    val prefix = "$name="
    return query.split("&")
        .firstOrNull { it.startsWith(prefix) }
        ?.removePrefix(prefix)
        ?.let { java.net.URLDecoder.decode(it, "UTF-8") }
}

fun parseFreepassCode(raw: String): FreepassParseResult {
    val trimmed = raw.trim()
    if (trimmed.isEmpty() || trimmed.length > MAX_RAW_LEN) return FreepassParseResult.Invalid
    if (trimmed.any { it.isWhitespace() || it.isISOControl() }) return FreepassParseResult.Invalid

    val uri = runCatching { java.net.URI(trimmed) }.getOrNull()
    if (uri?.scheme != null) {
        // Has a URI scheme → must be a trusted https nym.com URL, else reject.
        if (!uri.scheme.equals("https", ignoreCase = true)) return FreepassParseResult.Invalid
        if (!isTrustedHost(uri.host)) return FreepassParseResult.Invalid
        val code = queryParam(uri, "code") ?: return FreepassParseResult.Invalid
        return validateCode(code)
    }
    return validateCode(trimmed)
}

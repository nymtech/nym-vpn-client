package net.nymtech.vpn.util

// Core errors arrive as text: API messageIds match first, then AccountCommandError prefixes from nym-vpn-lib-types.
abstract class CommandErrorMapper<E>(val unknown: E, val server: E, offline: E, private val messageIds: List<Pair<String, E>> = emptyList(), prefixes: List<Pair<String, E>> = emptyList()) {
	private val prefixes = prefixes + listOf(
		"Error: no connectivity" to offline,
		"Error: vpn-api error" to server,
		"Error: unexpected vpn-api response" to server,
	)

	fun fromErrorChain(errorChain: String): E = messageIds.firstOrNull { (id, _) -> errorChain.contains(id) }?.second
		?: prefixes.firstOrNull { (prefix, _) -> errorChain.startsWith(prefix) }?.second
		?: unknown
}

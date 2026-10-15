package net.nymtech.nymvpn.ui.screens.server

enum class ServerListFilter {
	FAVORITES,
	RECENT,
	ALL_SERVERS,
}

data class ServerUiState(
	val error: Boolean = false,
	val query: String = "",
	val rows: List<ServerRow> = emptyList(),
	val filter: ServerListFilter = ServerListFilter.ALL_SERVERS,
	val countryCount: Int = 0,
	val nodeCount: Int = 0,
	val isEmpty: Boolean = true,
	val isLoading: Boolean = false,
	val favoriteGatewayIds: Set<String> = emptySet(),
	/** Index in [rows] of the row holding the current selection, or -1. */
	val scrollTargetIndex: Int = -1,
)

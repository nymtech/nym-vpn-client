package net.nymtech.nymvpn.ui.screens.server

import net.nymtech.vpn.model.NymGateway

/** One LazyColumn row of the server list. Country and region groups are flattened so only visible rows compose. */
sealed interface ServerRow {
	val key: String

	data class Country(
		val code: String,
		val displayName: String,
		val nodeCount: Int,
		val isFavorite: Boolean,
		val isSelected: Boolean,
		/** The current selection is a gateway or region inside this country. */
		val containsSelection: Boolean,
		val expanded: Boolean,
	) : ServerRow {
		override val key: String get() = "country:$code"
	}

	data class Region(
		val countryCode: String,
		val name: String,
		val nodeCount: Int,
		val isFavorite: Boolean,
		val isSelected: Boolean,
		val expanded: Boolean,
		/** Last visible row of its country block, so the bottom corners are rounded. */
		val roundBottom: Boolean,
	) : ServerRow {
		override val key: String get() = "region:$countryCode/$name"
	}

	data class Gateway(
		val gateway: NymGateway,
		val isFavorite: Boolean,
		val isSelected: Boolean,
		val roundBottom: Boolean,
		/** Rendered as its own card (search hit or recent) rather than inside a country block. */
		val standalone: Boolean,
		/** "City, Region, Country" text for standalone rows; null inside a country block. */
		val location: String?,
	) : ServerRow {
		override val key: String get() = "gateway:${gateway.identity}"
	}
}

data class ServerListResult(
	val rows: List<ServerRow>,
	val countryCount: Int,
	val nodeCount: Int,
	val isEmpty: Boolean,
	val favoriteGatewayIds: Set<String>,
	/** Index in [rows] of the row holding the current selection, or -1. */
	val scrollTargetIndex: Int,
)

/** True when this row closes its visual block, i.e. the next row starts a new card. */
val ServerRow.roundBottomEdge: Boolean
	get() = when (this) {
		is ServerRow.Country -> !expanded
		is ServerRow.Region -> roundBottom
		is ServerRow.Gateway -> roundBottom
	}

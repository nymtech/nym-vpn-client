package net.nymtech.nymvpn.ui.screens.server

import net.nymtech.nymvpn.util.Constants.countryCodesForRegionSupport
import net.nymtech.vpn.model.NymGateway
import java.util.Locale

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
) {
	val hasLocations: Boolean get() = rows.any { it !is ServerRow.Safest && it !is ServerRow.Random }
}

sealed interface ItemType {
	data class CountryItem(val locale: Locale, val gateways: List<NymGateway>, val regions: List<Region>? = null, val isFavorite: Boolean = false) : ItemType {
		data class Region(val region: String, val gateways: List<NymGateway>, val isFavorite: Boolean = false)
	}
	data class GatewayItem(val gateway: NymGateway, val isFavorite: Boolean = false) : ItemType
}

sealed interface ServerRow {
	val key: String

	sealed interface Expandable : ServerRow {
		val isExpanded: Boolean
		val expandStateKey: String
	}

	data object Safest : ServerRow {
		override val key = "safest"
	}

	data object Random : ServerRow {
		override val key = "random"
	}

	data class CountryHeader(val locale: Locale, val gatewayCount: Int, val isFavorite: Boolean, override val isExpanded: Boolean, override val expandStateKey: String) : Expandable {
		override val key = "country_${locale.country}"
		val countryCode: String get() = locale.country.lowercase()
	}

	data class RegionHeader(
		val locale: Locale,
		val region: String,
		val gatewayCount: Int,
		val isFavorite: Boolean,
		override val isExpanded: Boolean,
		override val expandStateKey: String,
		val isLastInGroup: Boolean,
	) : Expandable {
		override val key = "region_${locale.country}_$region"
	}

	data class Gateway(override val key: String, val gateway: NymGateway, val isFavorite: Boolean, val isLastInGroup: Boolean) : ServerRow

	data class SearchResult(val gateway: NymGateway, val isFavorite: Boolean) : ServerRow {
		override val key: String get() = gateway.identity
	}
}

internal fun ServerRow.represents(selectedKey: String?): Boolean = when (this) {
	is ServerRow.CountryHeader -> countryCode == selectedKey
	is ServerRow.RegionHeader -> region.equals(selectedKey, true)
	is ServerRow.Gateway -> gateway.identity == selectedKey
	is ServerRow.SearchResult -> gateway.identity == selectedKey
	ServerRow.Safest, ServerRow.Random -> false
}

internal fun NymGateway.serverLocation(countryName: String?): String {
	val region = this.region.takeIf { countryCodesForRegionSupport.contains(this.twoLetterCountryISO) }
	return listOfNotNull(this.city, region, countryName).joinToString(", ")
}

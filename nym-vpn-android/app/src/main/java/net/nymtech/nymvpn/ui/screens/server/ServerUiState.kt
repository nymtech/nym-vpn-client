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
	val items: List<ItemType> = emptyList(),
	val filter: ServerListFilter = ServerListFilter.ALL_SERVERS,
	val countryCount: Int = 0,
	val nodeCount: Int = 0,
	val isEmpty: Boolean = true,
	val isLoading: Boolean = false,
	val favoriteGatewayIds: Set<String> = emptySet(),
)

sealed interface ItemType {
	data class CountryItem(val locale: Locale, val gateways: List<NymGateway>, val regions: List<Region>? = null, val isFavorite: Boolean = false) : ItemType {
		data class Region(val region: String, val gateways: List<NymGateway>, val isFavorite: Boolean = false)
	}
	data class GatewayItem(val gateway: NymGateway, val isFavorite: Boolean = false) : ItemType
}

internal fun NymGateway.serverLocation(countryName: String?): String {
	val region = this.region.takeIf { countryCodesForRegionSupport.contains(this.twoLetterCountryISO) }
	return listOfNotNull(this.city, region, countryName).joinToString(", ")
}

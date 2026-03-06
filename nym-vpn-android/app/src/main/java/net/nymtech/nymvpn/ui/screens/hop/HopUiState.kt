package net.nymtech.nymvpn.ui.screens.hop

import net.nymtech.nymvpn.util.Constants.countryCodesForRegionSupport
import net.nymtech.vpn.model.NymGateway
import java.util.Locale

data class HopUiState(val error: Boolean = false, val query: String = "", val items: List<ItemType> = emptyList())

sealed interface ItemType {
	data class CountryItem(val locale: Locale, val gateways: List<NymGateway>, val regions: List<Region>? = null) : ItemType {
		data class Region(val region: String, val gateways: List<NymGateway>)
	}
	data class GatewayItem(val gateway: NymGateway) : ItemType
}

internal fun NymGateway.serverLocation(countryName: String?): String {
	val region = this.region.takeIf { countryCodesForRegionSupport.contains(this.twoLetterCountryISO) }
	return listOfNotNull(this.city, region, countryName).joinToString(", ")
}

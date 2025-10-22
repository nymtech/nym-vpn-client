package net.nymtech.nymvpn.ui.screens.hop

import net.nymtech.nymvpn.util.Constants.countryCodesForRegionSupport
import net.nymtech.vpn.model.NymGateway
import java.util.Locale

data class HopUiState(
	val error: Boolean = false,
	val query: String = "",
	val countries: List<Locale> = emptyList(),
	val queriedGateways: List<NymGateway> = emptyList(),
)

internal fun NymGateway.serverLocation(countryName: String?): String {
	val region = this.region.takeIf { countryCodesForRegionSupport.contains(this.twoLetterCountryISO) }
	return listOfNotNull(this.city, region, countryName).joinToString(", ")
}

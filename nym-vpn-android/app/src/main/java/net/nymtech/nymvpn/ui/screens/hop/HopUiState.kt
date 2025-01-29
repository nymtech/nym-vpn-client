package net.nymtech.nymvpn.ui.screens.hop

import net.nymtech.vpn.model.Country
import net.nymtech.vpn.model.NymGateway

data class HopUiState(
	val queriedCountries: List<NymGateway> = emptyList(),
	val selected: Country? = null,
	val error: Boolean = false,
	val query: String = "",
)

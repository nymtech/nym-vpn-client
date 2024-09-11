package net.nymtech.nymvpn.ui.screens.hop

import net.nymtech.vpn.model.Country

data class HopUiState(
	val queriedCountries: Set<Country> = emptySet(),
	val selected: Country? = null,
	val query: String = "",
)

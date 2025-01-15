package net.nymtech.nymvpn.ui.screens.hop

import net.nymtech.vpn.model.Country

data class HopUiState(
	val queriedCountries: List<Country> = emptyList(),
	val selected: Country? = null,
	val error: Boolean = false,
	val query: String = "",
)

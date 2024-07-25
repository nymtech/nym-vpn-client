package net.nymtech.nymvpn.ui.screens.hop

import net.nymtech.nymvpn.ui.GatewayLocation
import net.nymtech.vpn.model.Country

data class HopUiState(
	val countries: Set<Country> = emptySet(),
	val lowLatencyCountry: Country? = null,
	val gatewayLocation: GatewayLocation = GatewayLocation.Entry,
	val queriedCountries: Set<Country> = emptySet(),
	val selected: Country? = null,
	val query: String = "",
)

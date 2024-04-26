package net.nymtech.nymvpn.data.model

import net.nymtech.vpn.model.Country

data class Gateways(
	val lowLatencyCountry: Country? = null,
	val entryCountries: Set<Country> = emptySet(),
	val exitCountries: Set<Country> = emptySet(),
)

package net.nymtech.nymvpn.data.domain

import net.nymtech.vpn.model.Country

data class Gateways(
	val lowLatencyEntryCountry: Country? = null,
	val entryCountries: Set<Country> = emptySet(),
	val exitCountries: Set<Country> = emptySet(),
)

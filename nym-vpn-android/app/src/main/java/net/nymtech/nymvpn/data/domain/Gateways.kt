package net.nymtech.nymvpn.data.domain

import net.nymtech.vpn.model.Country

data class Gateways(
	val entryCountries: Set<Country> = emptySet(),
	val exitCountries: Set<Country> = emptySet(),
	val wgCountries: Set<Country> = emptySet(),
)

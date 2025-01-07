package net.nymtech.nymvpn.data.domain

import net.nymtech.vpn.model.Country

data class Gateways(
	val entryCountries: List<Country> = emptyList(),
	val exitCountries: List<Country> = emptyList(),
	val wgCountries: List<Country> = emptyList(),
)

package net.nymtech.nymvpn.data.domain

import net.nymtech.vpn.model.NymGateway

data class Gateways(
	val entryCountries: List<NymGateway> = emptyList(),
	val exitCountries: List<NymGateway> = emptyList(),
	val wgCountries: List<NymGateway> = emptyList(),
)

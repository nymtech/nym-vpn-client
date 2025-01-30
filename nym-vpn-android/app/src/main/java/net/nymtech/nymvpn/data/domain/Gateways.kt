package net.nymtech.nymvpn.data.domain

import net.nymtech.vpn.model.NymGateway

data class Gateways(
	val entryGateways: List<NymGateway> = emptyList(),
	val exitGateways: List<NymGateway> = emptyList(),
	val wgGateways: List<NymGateway> = emptyList(),
)

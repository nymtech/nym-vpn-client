package net.nymtech.nymvpn.data

import kotlinx.coroutines.flow.StateFlow
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.GatewayType

interface GatewayRepository {

	suspend fun setGateways(type: GatewayType, gateways: List<NymGateway>)

	suspend fun clear()

	val gatewayFlow: StateFlow<Gateways>
}

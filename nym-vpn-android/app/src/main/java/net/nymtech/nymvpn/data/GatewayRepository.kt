package net.nymtech.nymvpn.data

import kotlinx.coroutines.flow.Flow
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.vpn.model.NymGateway

interface GatewayRepository {

	suspend fun setEntryGateways(gateways: List<NymGateway>)

	suspend fun setExitGateways(gateways: List<NymGateway>)

	suspend fun setWgGateways(gateways: List<NymGateway>)

	val gatewayFlow: Flow<Gateways>
}

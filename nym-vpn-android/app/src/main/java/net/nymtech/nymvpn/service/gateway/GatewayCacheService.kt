package net.nymtech.nymvpn.service.gateway

import kotlinx.coroutines.flow.StateFlow
import nym_vpn_lib_types.GatewayType

interface GatewayCacheService {
	val syncingTypes: StateFlow<Set<GatewayType>>

	suspend fun refresh(type: GatewayType, force: Boolean = false): Result<Unit>

	suspend fun clear()
}

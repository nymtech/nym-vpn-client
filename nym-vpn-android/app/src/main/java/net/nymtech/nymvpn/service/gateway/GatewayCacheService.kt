package net.nymtech.nymvpn.service.gateway

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.GatewayType

interface GatewayCacheService {
	/** Fetches [type] from the core and stores it, unless it was fetched within the TTL and [force] is false. */
	suspend fun refresh(type: GatewayType, force: Boolean = false): Result<Unit>

	/** Refreshes only the lists [mode] needs (WG for two-hop, entry + exit for five-hop). Fails if any list failed. */
	suspend fun refreshForMode(mode: Tunnel.Mode, force: Boolean = false): Result<Unit>
}

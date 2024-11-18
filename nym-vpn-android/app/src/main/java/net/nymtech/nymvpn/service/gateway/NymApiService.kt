package net.nymtech.nymvpn.service.gateway

import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.Country
import nym_vpn_lib.GatewayType
import nym_vpn_lib.NetworkEnvironment
import nym_vpn_lib.SystemMessage

interface NymApiService {
	suspend fun getCountries(type: GatewayType): Set<Country>
	suspend fun getEnvironment(environment: Tunnel.Environment): NetworkEnvironment
	suspend fun getSystemMessages(environment: Tunnel.Environment): List<SystemMessage>
}

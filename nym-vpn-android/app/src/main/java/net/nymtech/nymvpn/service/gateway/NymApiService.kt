package net.nymtech.nymvpn.service.gateway

import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.Country
import nym_vpn_lib.GatewayType
import nym_vpn_lib.NetworkEnvironment

interface NymApiService {
	suspend fun getCountries(type: GatewayType): Set<Country>
	suspend fun getEnvironment(environment: Tunnel.Environment): NetworkEnvironment
}

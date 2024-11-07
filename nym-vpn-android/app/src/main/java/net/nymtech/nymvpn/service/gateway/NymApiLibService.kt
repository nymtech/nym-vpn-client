package net.nymtech.nymvpn.service.gateway

import net.nymtech.vpn.NymApi
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.Country
import nym_vpn_lib.GatewayType
import nym_vpn_lib.NetworkEnvironment
import javax.inject.Inject

class NymApiLibService @Inject constructor(
	private val nymApi: NymApi,
) : NymApiService {

	override suspend fun getCountries(type: GatewayType): Set<Country> {
		return nymApi.getGatewayCountries(type)
	}

	override suspend fun getEnvironment(environment: Tunnel.Environment): NetworkEnvironment {
		return nymApi.getEnvironment(environment)
	}
}

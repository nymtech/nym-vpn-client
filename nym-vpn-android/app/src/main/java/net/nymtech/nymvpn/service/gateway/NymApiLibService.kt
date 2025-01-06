package net.nymtech.nymvpn.service.gateway

import net.nymtech.vpn.NymApi
import net.nymtech.vpn.model.Country
import nym_vpn_lib.GatewayType
import nym_vpn_lib.SystemMessage
import javax.inject.Inject

class NymApiLibService @Inject constructor(
	private val nymApi: NymApi,
) : NymApiService {

	override suspend fun getCountries(type: GatewayType): Set<Country> {
		return nymApi.getGatewayCountries(type)
	}

	override suspend fun getSystemMessages(): List<SystemMessage> {
		return nymApi.getSystemMessages()
	}
}

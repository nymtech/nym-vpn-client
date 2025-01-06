package net.nymtech.nymvpn.service.gateway

import net.nymtech.vpn.model.Country
import nym_vpn_lib.GatewayType
import nym_vpn_lib.SystemMessage

interface NymApiService {
	suspend fun getCountries(type: GatewayType): Set<Country>
	suspend fun getSystemMessages(): List<SystemMessage>
}

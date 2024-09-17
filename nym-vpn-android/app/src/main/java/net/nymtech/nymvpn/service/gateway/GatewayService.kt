package net.nymtech.nymvpn.service.gateway

import net.nymtech.vpn.model.Country
import nym_vpn_lib.GatewayType

interface GatewayService {
	suspend fun getLowLatencyCountry(): Result<Country>
	suspend fun getCountries(type: GatewayType): Result<Set<Country>>
}

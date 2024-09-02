package net.nymtech.nymvpn.service.gateway

import net.nymtech.vpn.model.Country

interface GatewayService {
	suspend fun getLowLatencyCountry(): Result<Country>
	suspend fun getCountries(exitOnly: Boolean): Result<Set<Country>>
}

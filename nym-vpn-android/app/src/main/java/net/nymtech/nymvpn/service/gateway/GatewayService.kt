package net.nymtech.nymvpn.service.gateway

import net.nymtech.vpnclient.model.Country

interface GatewayService {
	suspend fun getLowLatencyCountry(): Result<Country>
	suspend fun getEntryCountries(): Result<Set<Country>>
	suspend fun getExitCountries(): Result<Set<Country>>
}

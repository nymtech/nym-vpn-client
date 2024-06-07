package net.nymtech.nymvpn.service.gateway

import net.nymtech.vpn.model.Country
import javax.inject.Inject

class GatewayApiService @Inject constructor(
	private val gatewayApi: GatewayApi,
	private val gatewayLibService: GatewayLibService,
) : GatewayService {

	// TODO hopefully we won't have to get this from the lib in the future
	override suspend fun getLowLatencyCountry(): Result<Country> {
		return gatewayLibService.getLowLatencyCountry()
	}

	override suspend fun getEntryCountries(): Result<Set<Country>> {
		return safeApiCall {
			gatewayApi.getAllEntryGatewayTwoCharacterCountryCodes().map {
				Country(it)
			}.toSet()
		}
	}

	override suspend fun getExitCountries(): Result<Set<Country>> {
		return safeApiCall {
			gatewayApi.getAllExitGatewayTwoCharacterCountryCodes().map {
				Country(it)
			}.toSet()
		}
	}
}

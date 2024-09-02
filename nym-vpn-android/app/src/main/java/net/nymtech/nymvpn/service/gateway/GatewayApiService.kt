package net.nymtech.nymvpn.service.gateway

import net.nymtech.vpn.model.Country
import timber.log.Timber
import javax.inject.Inject

class GatewayApiService @Inject constructor(
	private val gatewayApi: GatewayApi,
	private val gatewayLibService: GatewayLibService,
) : GatewayService {

	// TODO hopefully we won't have to get this from the lib in the future
	override suspend fun getLowLatencyCountry(): Result<Country> {
		return gatewayLibService.getLowLatencyCountry()
	}

	override suspend fun getCountries(exitOnly: Boolean): Result<Set<Country>> {
		Timber.d("Getting countries from nym api")
		return safeApiCall {
			val countries = if (exitOnly) gatewayApi.getAllExitGatewayTwoCharacterCountryCodes()
			else gatewayApi.getAllEntryGatewayTwoCharacterCountryCodes()
			countries.map { Country(it) }.toSet()
		}
	}
}

package net.nymtech.nymvpn.service.country

import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.service.gateway.GatewayService
import javax.inject.Inject

class CountryDataStoreCacheService @Inject constructor(
	private val gatewayRepository: GatewayRepository,
	private val gatewayService: GatewayService,
) : CountryCacheService {
	override suspend fun updateExitCountriesCache(): Result<Unit> {
		return gatewayService.getExitCountries().onSuccess {
			gatewayRepository.setExitCountries(it)
		}.mapCatching { }
	}

	override suspend fun updateEntryCountriesCache(): Result<Unit> {
		return gatewayService.getEntryCountries().onSuccess {
			gatewayRepository.setEntryCountries(it)
		}.mapCatching { }
	}

	override suspend fun updateLowLatencyEntryCountryCache(): Result<Unit> {
		return gatewayService.getLowLatencyCountry().onSuccess {
			gatewayRepository.setLowLatencyCountry(it)
		}.mapCatching { }
	}
}

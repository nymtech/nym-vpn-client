package net.nymtech.nymvpn.service.country

import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.module.Android
import net.nymtech.nymvpn.service.gateway.GatewayService
import timber.log.Timber
import javax.inject.Inject

class CountryDataStoreCacheService @Inject constructor(
	private val gatewayRepository: GatewayRepository,
	@Android private val gatewayService: GatewayService,
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
			Timber.d("Updating low latency country cache: $it")
			gatewayRepository.setLowLatencyEntryCountry(it)
		}.mapCatching { }
	}
}

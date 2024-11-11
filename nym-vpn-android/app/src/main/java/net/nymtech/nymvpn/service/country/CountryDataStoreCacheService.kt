package net.nymtech.nymvpn.service.country

import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.service.gateway.NymApiService
import nym_vpn_lib.GatewayType
import timber.log.Timber
import javax.inject.Inject

class CountryDataStoreCacheService @Inject constructor(
	private val gatewayRepository: GatewayRepository,
	private val nymApiService: NymApiService,
) : CountryCacheService {
	override suspend fun updateExitCountriesCache(): Result<Unit> {
		return runCatching {
			val countries = nymApiService.getCountries(GatewayType.MIXNET_EXIT)
			gatewayRepository.setExitCountries(countries)
			Timber.d("Updated mixnet exit countries cache")
		}
	}

	override suspend fun updateEntryCountriesCache(): Result<Unit> {
		return runCatching {
			val countries = nymApiService.getCountries(GatewayType.MIXNET_ENTRY)
			gatewayRepository.setEntryCountries(countries)
			Timber.d("Updated mixnet entry countries cache")
		}
	}

	override suspend fun updateWgCountriesCache(): Result<Unit> {
		return kotlin.runCatching {
			val countries = nymApiService.getCountries(GatewayType.WG)
			gatewayRepository.setWgCountries(countries)
			Timber.d("Updated wg countries cache")
		}
	}
}

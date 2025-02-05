package net.nymtech.nymvpn.service.country

import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import nym_vpn_lib.GatewayType
import timber.log.Timber
import javax.inject.Inject

class CountryDataStoreCacheService @Inject constructor(
	private val gatewayRepository: GatewayRepository,
	private val backend: BackendManager,
) : CountryCacheService {
	override suspend fun updateExitCountriesCache(): Result<Unit> {
		return runCatching {
			val countries = backend.getGatewayCountries(GatewayType.MIXNET_EXIT)
			gatewayRepository.setExitCountries(countries)
			Timber.d("Updated mixnet exit countries cache")
		}.onFailure {
			Timber.w("Failed to get exit countries: ${it.message}")
		}
	}

	override suspend fun updateEntryCountriesCache(): Result<Unit> {
		return runCatching {
			val countries = backend.getGatewayCountries(GatewayType.MIXNET_ENTRY)
			gatewayRepository.setEntryCountries(countries)
			Timber.d("Updated mixnet entry countries cache")
		}.onFailure {
			Timber.w("Failed to get entry countries: ${it.message}")
		}
	}

	override suspend fun updateWgCountriesCache(): Result<Unit> {
		return kotlin.runCatching {
			val countries = backend.getGatewayCountries(GatewayType.WG)
			gatewayRepository.setWgCountries(countries)
			Timber.d("Updated wg countries cache")
		}.onFailure {
			Timber.w("Failed to get wg countries: ${it.message}")
		}
	}
}

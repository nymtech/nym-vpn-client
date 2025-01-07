package net.nymtech.nymvpn.service.country

import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.vpn.backend.Backend
import nym_vpn_lib.GatewayType
import nym_vpn_lib.UserAgent
import timber.log.Timber
import javax.inject.Inject

class CountryDataStoreCacheService @Inject constructor(
	private val gatewayRepository: GatewayRepository,
	private val backend: Backend,
	private val userAgent: UserAgent,
) : CountryCacheService {
	override suspend fun updateExitCountriesCache(): Result<Unit> {
		return runCatching {
			val countries = backend.getGatewayCountries(GatewayType.MIXNET_EXIT, userAgent)
			gatewayRepository.setExitCountries(countries)
			Timber.d("Updated mixnet exit countries cache")
		}
	}

	override suspend fun updateEntryCountriesCache(): Result<Unit> {
		return runCatching {
			val countries = backend.getGatewayCountries(GatewayType.MIXNET_ENTRY, userAgent)
			gatewayRepository.setEntryCountries(countries)
			Timber.d("Updated mixnet entry countries cache")
		}
	}

	override suspend fun updateWgCountriesCache(): Result<Unit> {
		return kotlin.runCatching {
			val countries = backend.getGatewayCountries(GatewayType.WG, userAgent)
			gatewayRepository.setWgCountries(countries)
			Timber.d("Updated wg countries cache")
		}
	}
}

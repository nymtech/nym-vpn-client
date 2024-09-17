package net.nymtech.nymvpn.service.country

import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.module.qualifiers.Native
import net.nymtech.nymvpn.service.gateway.GatewayService
import nym_vpn_lib.GatewayType
import timber.log.Timber
import javax.inject.Inject

class CountryDataStoreCacheService @Inject constructor(
	private val gatewayRepository: GatewayRepository,
	@Native private val gatewayService: GatewayService,
) : CountryCacheService {
	override suspend fun updateExitCountriesCache(): Result<Unit> {
		return runCatching {
			gatewayService.getCountries(GatewayType.MIXNET_EXIT).onSuccess {
				gatewayRepository.setExitCountries(it)
			}
		}
	}

	override suspend fun updateEntryCountriesCache(): Result<Unit> {
		return runCatching {
			gatewayService.getCountries(GatewayType.MIXNET_ENTRY).onSuccess {
				gatewayRepository.setEntryCountries(it)
			}
		}
	}

	override suspend fun updateWgCountriesCache(): Result<Unit> {
		return kotlin.runCatching {
			gatewayService.getCountries(GatewayType.WG).onSuccess {
				gatewayRepository.setWgCountries(it)
			}
		}
	}

	override suspend fun updateLowLatencyEntryCountryCache(): Result<Unit> {
		return runCatching {
			gatewayService.getLowLatencyCountry().onSuccess {
				Timber.d("Updating low latency country cache: $it")
				gatewayRepository.setLowLatencyEntryCountry(it)
			}
		}
	}
}

package net.nymtech.nymvpn.service.gateway

import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.vpn.NymApi
import net.nymtech.vpn.model.Country
import timber.log.Timber
import javax.inject.Inject

class GatewayLibService @Inject constructor(
	private val nymApi: NymApi,
	private val settingsRepository: SettingsRepository
) : GatewayService {

	override suspend fun getLowLatencyCountry(): Result<Country> {
		return runCatching {
			nymApi.getLowLatencyEntryCountry()
		}
	}

	override suspend fun getCountries(exitOnly: Boolean): Result<Set<Country>> {
		return runCatching {
			val env = settingsRepository.getEnvironment()
			Timber.d("Getting countries from lib api")
			nymApi.gateways(exitOnly, env)
		}
	}
}

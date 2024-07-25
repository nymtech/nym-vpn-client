package net.nymtech.nymvpn.service.gateway

import net.nymtech.vpn.NymApi
import net.nymtech.vpn.model.Country
import javax.inject.Inject

class GatewayLibService @Inject constructor(
	private val nymApi: NymApi,
) : GatewayService {
	override suspend fun getLowLatencyCountry(): Result<Country> {
		return runCatching {
			nymApi.getLowLatencyEntryCountry()
		}
	}

	override suspend fun getEntryCountries(): Result<Set<Country>> {
		return runCatching {
			nymApi.gateways(false)
		}
	}

	override suspend fun getExitCountries(): Result<Set<Country>> {
		return runCatching {
			nymApi.gateways(true)
		}
	}
}

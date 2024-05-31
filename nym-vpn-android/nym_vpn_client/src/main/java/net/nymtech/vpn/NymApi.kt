package net.nymtech.vpn

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import net.nymtech.vpn.model.Country
import net.nymtech.vpn.model.Environment
import nym_vpn_lib.getGatewayCountries

class NymApi(
	private val environment: Environment,
	private val ioDispatcher: CoroutineDispatcher,
) {
	suspend fun gateways(exitOnly: Boolean): Set<Country> {
		return withContext(ioDispatcher) {
			getGatewayCountries(environment.apiUrl, environment.explorerUrl, environment.harbourMasterUrl, exitOnly).map {
				Country(isoCode = it.twoLetterIsoCountryCode)
			}.toSet()
		}
	}

	suspend fun getLowLatencyEntryCountry(): Country {
		return withContext(ioDispatcher) {
			Country(
				isoCode =
				nym_vpn_lib.getLowLatencyEntryCountry(
					environment.apiUrl,
					environment.explorerUrl,
					environment.harbourMasterUrl,
				).twoLetterIsoCountryCode,
				isLowLatency = true,
			)
		}
	}
}

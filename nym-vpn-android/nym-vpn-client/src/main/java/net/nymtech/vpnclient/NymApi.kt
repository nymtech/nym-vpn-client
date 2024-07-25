package net.nymtech.vpnclient

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import net.nymtech.vpnclient.model.Country
import net.nymtech.vpnclient.model.Environment
import nym_vpn_lib.UserAgent
import nym_vpn_lib.getGatewayCountries
import nym_vpn_lib.getGatewayCountriesUserAgent

class NymApi(
	private val environment: Environment,
	private val ioDispatcher: CoroutineDispatcher,
	private val userAgent: UserAgent,
) {
	suspend fun gateways(exitOnly: Boolean): Set<Country> {
		return withContext(ioDispatcher) {
			getGatewayCountriesUserAgent(environment.apiUrl, environment.explorerUrl, environment.harbourMasterUrl, exitOnly, userAgent).map {
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

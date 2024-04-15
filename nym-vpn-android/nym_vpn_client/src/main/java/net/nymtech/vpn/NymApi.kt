package net.nymtech.vpn

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import net.nymtech.vpn.model.Country
import net.nymtech.vpn.model.Environment
import nym_vpn_lib.getGatewayCountries

class NymApi(private val environment: Environment) {
	suspend fun gateways(exitOnly: Boolean): Set<Country> {
		return withContext(Dispatchers.IO) {
			getGatewayCountries(environment.apiUrl, environment.explorerUrl, exitOnly).map {
				Country(isoCode = it.twoLetterIsoCountryCode)
			}.toSet()
		}
	}

	suspend fun getLowLatencyEntryCountry(): Country {
		return withContext(Dispatchers.IO) {
			Country(
				isoCode =
				nym_vpn_lib.getLowLatencyEntryCountry(
					environment.apiUrl,
					environment.explorerUrl,
				).twoLetterIsoCountryCode,
				isLowLatency = true,
			)
		}
	}
}

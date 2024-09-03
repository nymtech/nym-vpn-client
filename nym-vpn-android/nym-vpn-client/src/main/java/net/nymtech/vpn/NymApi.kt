package net.nymtech.vpn

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.Country
import nym_vpn_lib.UserAgent
import nym_vpn_lib.getGatewayCountries

class NymApi(
	private val ioDispatcher: CoroutineDispatcher,
	private val userAgent: UserAgent,
) {
	suspend fun gateways(exitOnly: Boolean, environment: Tunnel.Environment): Set<Country> {
		return withContext(ioDispatcher) {
			getGatewayCountries(environment.apiUrl, environment.nymVpnApiUrl, exitOnly, userAgent).map {
				Country(isoCode = it.twoLetterIsoCountryCode)
			}.toSet()
		}
	}

	suspend fun getLowLatencyEntryCountry(): Country {
		// TODO
		return Country()
	}
}

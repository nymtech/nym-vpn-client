package net.nymtech.vpn

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.Country
import nym_vpn_lib.GatewayType
import nym_vpn_lib.UserAgent
import nym_vpn_lib.getGatewayCountries

class NymApi(
	private val ioDispatcher: CoroutineDispatcher,
	private val userAgent: UserAgent,
) {
	suspend fun gateways(type: GatewayType, environment: Tunnel.Environment): Set<Country> {
		return withContext(ioDispatcher) {
			getGatewayCountries(environment.apiUrl, environment.nymVpnApiUrl, type, userAgent, null).map {
				Country(isoCode = it.twoLetterIsoCountryCode)
			}.toSet()
		}
	}

	suspend fun getLowLatencyEntryCountry(environment: Tunnel.Environment): Country {
		return withContext(ioDispatcher) {
			Country(isoCode = nym_vpn_lib.getLowLatencyEntryCountry(environment.apiUrl, environment.nymVpnApiUrl, userAgent).twoLetterIsoCountryCode)
		}
	}
}

package net.nymtech.vpn

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.Country
import nym_vpn_lib.GatewayType
import nym_vpn_lib.UserAgent
import nym_vpn_lib.fetchEnvironment
import nym_vpn_lib.getGatewayCountries
import java.net.URL

class NymApi(
	private val ioDispatcher: CoroutineDispatcher,
	private val userAgent: UserAgent,
) {
	suspend fun gateways(type: GatewayType, environment: Tunnel.Environment): Set<Country> {
		return withContext(ioDispatcher) {
			val environment = fetchEnvironment(environment.name.lowercase())
			getGatewayCountries(URL(environment.nymNetwork.endpoints.first().apiUrl!!), URL(environment.nymVpnNetwork.nymVpnApiUrl), type, userAgent, null).map {
				Country(isoCode = it.twoLetterIsoCountryCode)
			}.toSet()
		}
	}
}

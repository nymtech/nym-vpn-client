package net.nymtech.vpn

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.Country
import nym_vpn_lib.GatewayType
import nym_vpn_lib.NetworkEnvironment
import nym_vpn_lib.SystemMessage
import nym_vpn_lib.UserAgent
import nym_vpn_lib.fetchEnvironment
import nym_vpn_lib.fetchSystemMessages
import nym_vpn_lib.getGatewayCountries

class NymApi(
	private val ioDispatcher: CoroutineDispatcher,
	private val userAgent: UserAgent,
) {

	suspend fun getGatewayCountries(type: GatewayType): Set<Country> {
		return withContext(ioDispatcher) {
			getGatewayCountries(type, userAgent, null).map {
				Country(isoCode = it.twoLetterIsoCountryCode)
			}.toSet()
		}
	}

	suspend fun getEnvironment(environment: Tunnel.Environment): NetworkEnvironment {
		return withContext(ioDispatcher) {
			fetchEnvironment(environment.networkName())
		}
	}

	suspend fun getSystemMessages(environment: Tunnel.Environment): List<SystemMessage> {
		return withContext(ioDispatcher) {
			fetchSystemMessages(environment.networkName())
		}
	}
}

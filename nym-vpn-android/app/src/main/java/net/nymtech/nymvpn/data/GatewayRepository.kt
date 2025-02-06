package net.nymtech.nymvpn.data

import kotlinx.coroutines.flow.Flow
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.vpn.model.NymGateway

interface GatewayRepository {

	suspend fun setEntryCountries(countries: List<NymGateway>)

	suspend fun setExitCountries(countries: List<NymGateway>)

	suspend fun setWgCountries(countries: List<NymGateway>)

	val gatewayFlow: Flow<Gateways>
}

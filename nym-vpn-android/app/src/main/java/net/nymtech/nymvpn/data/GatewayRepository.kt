package net.nymtech.nymvpn.data

import kotlinx.coroutines.flow.Flow
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.vpn.model.Country

interface GatewayRepository {

	suspend fun setEntryCountries(countries: List<Country>)

	suspend fun getEntryCountries(): List<Country>

	suspend fun setExitCountries(countries: List<Country>)

	suspend fun getExitCountries(): List<Country>

	suspend fun setWgCountries(countries: List<Country>)

	val gatewayFlow: Flow<Gateways>
}

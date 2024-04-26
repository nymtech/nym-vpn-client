package net.nymtech.nymvpn.data

import kotlinx.coroutines.flow.Flow
import net.nymtech.nymvpn.data.model.Gateways
import net.nymtech.vpn.model.Country

interface GatewayRepository {

	suspend fun getLowLatencyCountry(): Country?

	suspend fun setLowLatencyCountry(country: Country)

	suspend fun setEntryCountries(countries: Set<Country>)

	suspend fun getEntryCountries(): Set<Country>

	suspend fun setExitCountries(countries: Set<Country>)

	suspend fun getExitCountries(): Set<Country>

	val gatewayFlow: Flow<Gateways>
}

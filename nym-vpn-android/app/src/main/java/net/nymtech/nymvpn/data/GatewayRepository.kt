package net.nymtech.nymvpn.data

import kotlinx.coroutines.flow.Flow
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.vpn.model.Country

interface GatewayRepository {

	suspend fun getLowLatencyEntryCountry(): Country?

	suspend fun setLowLatencyEntryCountry(country: Country)

	suspend fun setEntryCountries(countries: Set<Country>)

	suspend fun getEntryCountries(): Set<Country>

	suspend fun setExitCountries(countries: Set<Country>)

	suspend fun getExitCountries(): Set<Country>

	val gatewayFlow: Flow<Gateways>
}

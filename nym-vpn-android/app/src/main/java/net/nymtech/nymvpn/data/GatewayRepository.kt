package net.nymtech.nymvpn.data

import kotlinx.coroutines.flow.Flow
import net.nymtech.nymvpn.data.model.Gateways
import net.nymtech.vpn.model.Hop
import net.nymtech.vpn.model.HopCountries

interface GatewayRepository {
    suspend fun getFirstHopCountry() : Hop.Country
    suspend fun setFirstHopCountry(country: Hop.Country)

    suspend fun getLowLatencyCountry() : Hop.Country
    suspend fun setLowLatencyCountry(country: Hop.Country)

    suspend fun getLastHopCountry() : Hop.Country
    suspend fun setLastHopCountry(country: Hop.Country)

    suspend fun setEntryCountries(countries: HopCountries)
    suspend fun getEntryCountries() : HopCountries

    suspend fun setExitCountries(countries: HopCountries)
    suspend fun getExitCountries() : HopCountries

    val gatewayFlow : Flow<Gateways>
}
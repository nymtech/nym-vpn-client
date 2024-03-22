package net.nymtech.nymvpn.data.datastore

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.model.Gateways
import net.nymtech.vpn.model.Hop
import net.nymtech.vpn.model.HopCountries
import timber.log.Timber

class DataStoreGatewayRepository(private val dataStoreManager: DataStoreManager) : GatewayRepository {
    override suspend fun getFirstHopCountry(): Hop.Country {
        val country = dataStoreManager.getFromStore(DataStoreManager.FIRST_HOP_COUNTRY)
        return Hop.Country.from(country)
    }

    override suspend fun setFirstHopCountry(country: Hop.Country) {
        dataStoreManager.saveToDataStore(DataStoreManager.FIRST_HOP_COUNTRY, country.toString())
    }

    override suspend fun getLowLatencyCountry(): Hop.Country {
        val country = dataStoreManager.getFromStore(DataStoreManager.LOW_LATENCY_COUNTRY)
        return Hop.Country.from(country)
    }

    override suspend fun setLowLatencyCountry(country: Hop.Country) {
        dataStoreManager.saveToDataStore(DataStoreManager.LOW_LATENCY_COUNTRY, country.toString())
    }

    override suspend fun getLastHopCountry(): Hop.Country {
        val country = dataStoreManager.getFromStore(DataStoreManager.LAST_HOP_COUNTRY)
        return Hop.Country.from(country)
    }

    override suspend fun setLastHopCountry(country: Hop.Country) {
        dataStoreManager.saveToDataStore(DataStoreManager.LAST_HOP_COUNTRY, country.toString())
    }

    override suspend fun setEntryCountries(countries: HopCountries) {
        dataStoreManager.saveToDataStore(DataStoreManager.ENTRY_COUNTRIES, countries.toString())
    }

    override suspend fun getEntryCountries(): HopCountries {
        val countries = dataStoreManager.getFromStore(DataStoreManager.ENTRY_COUNTRIES)
        return Hop.Country.fromCollectionString(countries)
    }

    override suspend fun setExitCountries(countries: HopCountries) {
        dataStoreManager.saveToDataStore(DataStoreManager.EXIT_COUNTRIES, countries.toString())
    }

    override suspend fun getExitCountries(): HopCountries {
        val countries = dataStoreManager.getFromStore(DataStoreManager.EXIT_COUNTRIES)
        return Hop.Country.fromCollectionString(countries)
    }

    override val gatewayFlow: Flow<Gateways> = dataStoreManager.preferencesFlow.map { prefs ->
        prefs?.let { pref ->
            try{
                Gateways(
                    firstHopCountry = Hop.Country.from(pref[DataStoreManager.FIRST_HOP_COUNTRY]),
                    lastHopCountry = Hop.Country.from(pref[DataStoreManager.LAST_HOP_COUNTRY]),
                    lowLatencyCountry = Hop.Country.from(pref[DataStoreManager.LOW_LATENCY_COUNTRY]),
                    exitCountries = Hop.Country.fromCollectionString(pref[DataStoreManager.EXIT_COUNTRIES]),
                    entryCountries = Hop.Country.fromCollectionString(pref[DataStoreManager.ENTRY_COUNTRIES])
                )
            } catch (e : IllegalArgumentException) {
                Timber.e(e)
                Gateways()
            }
        } ?: Gateways()
    }

}
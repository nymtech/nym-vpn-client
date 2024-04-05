package net.nymtech.nymvpn.data.datastore

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.model.Gateways
import net.nymtech.vpn.model.Country
import timber.log.Timber

class DataStoreGatewayRepository(private val dataStoreManager: DataStoreManager) :
    GatewayRepository {
    override suspend fun getFirstHopCountry(): Country {
        val country = dataStoreManager.getFromStore(DataStoreManager.FIRST_HOP_COUNTRY)
        return Country.from(country)
    }

    override suspend fun setFirstHopCountry(country: Country) {
        dataStoreManager.saveToDataStore(DataStoreManager.FIRST_HOP_COUNTRY, country.toString())
    }

    override suspend fun getLowLatencyCountry(): Country {
        val country = dataStoreManager.getFromStore(DataStoreManager.LOW_LATENCY_COUNTRY)
        return Country.from(country)
    }

    override suspend fun setLowLatencyCountry(country: Country) {
        dataStoreManager.saveToDataStore(DataStoreManager.LOW_LATENCY_COUNTRY, country.toString())
    }

    override suspend fun getLastHopCountry(): Country {
        val country = dataStoreManager.getFromStore(DataStoreManager.LAST_HOP_COUNTRY)
        return Country.from(country)
    }

    override suspend fun setLastHopCountry(country: Country) {
        dataStoreManager.saveToDataStore(DataStoreManager.LAST_HOP_COUNTRY, country.toString())
    }

    override suspend fun setEntryCountries(countries: Set<Country>) {
        dataStoreManager.saveToDataStore(DataStoreManager.ENTRY_COUNTRIES, countries.toString())
    }

    override suspend fun getEntryCountries(): Set<Country> {
        val countries = dataStoreManager.getFromStore(DataStoreManager.ENTRY_COUNTRIES)
        return Country.fromCollectionString(countries)
    }

    override suspend fun setExitCountries(countries: Set<Country>) {
        dataStoreManager.saveToDataStore(DataStoreManager.EXIT_COUNTRIES, countries.toString())
    }

    override suspend fun getExitCountries(): Set<Country> {
        val countries = dataStoreManager.getFromStore(DataStoreManager.EXIT_COUNTRIES)
        return Country.fromCollectionString(countries)
    }

    override val gatewayFlow: Flow<Gateways> = dataStoreManager.preferencesFlow.map { prefs ->
        prefs?.let { pref ->
            try {
                Gateways(
                    firstHopCountry = Country.from(pref[DataStoreManager.FIRST_HOP_COUNTRY]),
                    lastHopCountry = Country.from(pref[DataStoreManager.LAST_HOP_COUNTRY]),
                    lowLatencyCountry = Country.from(pref[DataStoreManager.LOW_LATENCY_COUNTRY]),
                    exitCountries = Country.fromCollectionString(pref[DataStoreManager.EXIT_COUNTRIES]),
                    entryCountries = Country.fromCollectionString(pref[DataStoreManager.ENTRY_COUNTRIES])
                )
            } catch (e: IllegalArgumentException) {
                Timber.e(e)
                Gateways()
            }
        } ?: Gateways()
    }

}
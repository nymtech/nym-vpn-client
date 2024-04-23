package net.nymtech.nymvpn.data.datastore

import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.model.Gateways
import net.nymtech.vpn.model.Country
import timber.log.Timber

class DataStoreGatewayRepository(private val dataStoreManager: DataStoreManager) :
	GatewayRepository {
	companion object {
		val LOW_LATENCY_COUNTRY = stringPreferencesKey("LOW_LATENCY_COUNTRY")
		val ENTRY_COUNTRIES = stringPreferencesKey("ENTRY_COUNTRIES")
		val EXIT_COUNTRIES = stringPreferencesKey("EXIT_COUNTRIES")
	}

	override suspend fun getLowLatencyCountry(): Country {
		val country = dataStoreManager.getFromStore(LOW_LATENCY_COUNTRY)
		return Country.from(country)
	}

	override suspend fun setLowLatencyCountry(country: Country) {
		dataStoreManager.saveToDataStore(LOW_LATENCY_COUNTRY, country.toString())
	}

	override suspend fun setEntryCountries(countries: Set<Country>) {
		dataStoreManager.saveToDataStore(ENTRY_COUNTRIES, countries.toString())
	}

	override suspend fun getEntryCountries(): Set<Country> {
		val countries = dataStoreManager.getFromStore(ENTRY_COUNTRIES)
		return Country.fromCollectionString(countries)
	}

	override suspend fun setExitCountries(countries: Set<Country>) {
		dataStoreManager.saveToDataStore(EXIT_COUNTRIES, countries.toString())
	}

	override suspend fun getExitCountries(): Set<Country> {
		val countries = dataStoreManager.getFromStore(EXIT_COUNTRIES)
		return Country.fromCollectionString(countries)
	}

	override val gatewayFlow: Flow<Gateways> =
		dataStoreManager.preferencesFlow.map { prefs ->
			prefs?.let { pref ->
				try {
					Gateways(
						lowLatencyCountry = Country.from(pref[LOW_LATENCY_COUNTRY]),
						exitCountries = Country.fromCollectionString(pref[EXIT_COUNTRIES]),
						entryCountries = Country.fromCollectionString(pref[ENTRY_COUNTRIES]),
					)
				} catch (e: IllegalArgumentException) {
					Timber.e(e)
					Gateways()
				}
			} ?: Gateways()
		}
}

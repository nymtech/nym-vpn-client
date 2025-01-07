package net.nymtech.nymvpn.data.datastore

import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.vpn.model.Country
import timber.log.Timber

class DataStoreGatewayRepository(private val dataStoreManager: DataStoreManager) :
	GatewayRepository {
	companion object {
		val ENTRY_COUNTRIES = stringPreferencesKey("ENTRY_COUNTRIES")
		val EXIT_COUNTRIES = stringPreferencesKey("EXIT_COUNTRIES")
		val WG_COUNTRIES = stringPreferencesKey("WG_COUNTRIES")
	}

	override suspend fun setEntryCountries(countries: List<Country>) {
		dataStoreManager.saveToDataStore(ENTRY_COUNTRIES, countries.toString())
	}

	override suspend fun getEntryCountries(): List<Country> {
		val countries = dataStoreManager.getFromStore(ENTRY_COUNTRIES)
		return Country.fromCollectionString(countries)
	}

	override suspend fun setExitCountries(countries: List<Country>) {
		dataStoreManager.saveToDataStore(EXIT_COUNTRIES, countries.toString())
	}

	override suspend fun getExitCountries(): List<Country> {
		val countries = dataStoreManager.getFromStore(EXIT_COUNTRIES)
		return Country.fromCollectionString(countries)
	}

	override suspend fun setWgCountries(countries: List<Country>) {
		dataStoreManager.saveToDataStore(WG_COUNTRIES, countries.toString())
	}

	override val gatewayFlow: Flow<Gateways> =
		dataStoreManager.preferencesFlow.map { prefs ->
			prefs?.let { pref ->
				try {
					Gateways(
						exitCountries = Country.fromCollectionString(pref[EXIT_COUNTRIES]),
						entryCountries = Country.fromCollectionString(pref[ENTRY_COUNTRIES]),
						wgCountries = Country.fromCollectionString(pref[WG_COUNTRIES]),
					)
				} catch (e: IllegalArgumentException) {
					Timber.e(e)
					Gateways()
				}
			} ?: Gateways()
		}
}

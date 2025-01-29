package net.nymtech.nymvpn.data.datastore

import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.vpn.model.Country
import net.nymtech.vpn.model.NymGateway
import timber.log.Timber

class DataStoreGatewayRepository(private val dataStoreManager: DataStoreManager) :
	GatewayRepository {
	companion object {
		val ENTRY_COUNTRIES = stringPreferencesKey("ENTRY_GATEWAYS")
		val EXIT_COUNTRIES = stringPreferencesKey("EXIT_GATEWAYS")
		val WG_COUNTRIES = stringPreferencesKey("WG_GATEWAYS")
	}

	override suspend fun setEntryCountries(countries: List<NymGateway>) {
		dataStoreManager.saveToDataStore(ENTRY_COUNTRIES, countries.toString())
	}


	override suspend fun setExitCountries(countries: List<NymGateway>) {
		dataStoreManager.saveToDataStore(EXIT_COUNTRIES, countries.toString())
	}

	override suspend fun setWgCountries(countries: List<NymGateway>) {
		dataStoreManager.saveToDataStore(WG_COUNTRIES, countries.toString())
	}

	override val gatewayFlow: Flow<Gateways> =
		dataStoreManager.preferencesFlow.map { prefs ->
			prefs?.let { pref ->
				try {
					Gateways(
						exitCountries = NymGateway.fromCollectionString(pref[EXIT_COUNTRIES]),
						entryCountries = NymGateway.fromCollectionString(pref[ENTRY_COUNTRIES]),
						wgCountries = NymGateway.fromCollectionString(pref[WG_COUNTRIES]),
					)
				} catch (e: IllegalArgumentException) {
					Timber.e(e)
					Gateways()
				}
			} ?: Gateways()
		}
}

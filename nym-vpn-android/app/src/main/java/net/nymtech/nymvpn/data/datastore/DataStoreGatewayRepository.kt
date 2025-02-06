package net.nymtech.nymvpn.data.datastore

import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.vpn.model.NymGateway
import timber.log.Timber

class DataStoreGatewayRepository(private val dataStoreManager: DataStoreManager) :
	GatewayRepository {
	companion object {
		val ENTRY_COUNTRIES = stringPreferencesKey("ENTRY_GATEWAYS")
		val EXIT_COUNTRIES = stringPreferencesKey("EXIT_GATEWAYS")
		val WG_COUNTRIES = stringPreferencesKey("WG_GATEWAYS")
	}

	override suspend fun setEntryGateways(gateways: List<NymGateway>) {
		dataStoreManager.saveToDataStore(ENTRY_COUNTRIES, gateways.toString())
	}

	override suspend fun setExitGateways(gateways: List<NymGateway>) {
		dataStoreManager.saveToDataStore(EXIT_COUNTRIES, gateways.toString())
	}

	override suspend fun setWgGateways(gateways: List<NymGateway>) {
		dataStoreManager.saveToDataStore(WG_COUNTRIES, gateways.toString())
	}

	override val gatewayFlow: Flow<Gateways> =
		dataStoreManager.preferencesFlow.map { prefs ->
			prefs?.let { pref ->
				try {
					Gateways(
						exitGateways = NymGateway.fromCollectionString(pref[EXIT_COUNTRIES]),
						entryGateways = NymGateway.fromCollectionString(pref[ENTRY_COUNTRIES]),
						wgGateways = NymGateway.fromCollectionString(pref[WG_COUNTRIES]),
					)
				} catch (e: IllegalArgumentException) {
					Timber.e(e)
					Gateways()
				}
			} ?: Gateways()
		}
}

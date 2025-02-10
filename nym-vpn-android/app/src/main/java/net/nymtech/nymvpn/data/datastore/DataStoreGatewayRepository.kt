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
		val ENTRY_GATEWAYS = stringPreferencesKey("ENTRY_GATEWAYS")
		val EXIT_GATEWAYS = stringPreferencesKey("EXIT_GATEWAYS")
		val WG_GATEWAYS = stringPreferencesKey("WG_GATEWAYS")
	}

	override suspend fun setEntryGateways(gateways: List<NymGateway>) {
		dataStoreManager.saveToDataStore(ENTRY_GATEWAYS, gateways.toString())
	}

	override suspend fun setExitGateways(gateways: List<NymGateway>) {
		dataStoreManager.saveToDataStore(EXIT_GATEWAYS, gateways.toString())
	}

	override suspend fun setWgGateways(gateways: List<NymGateway>) {
		dataStoreManager.saveToDataStore(WG_GATEWAYS, gateways.toString())
	}

	override val gatewayFlow: Flow<Gateways> =
		dataStoreManager.preferencesFlow.map { prefs ->
			prefs?.let { pref ->
				try {
					Gateways(
						exitGateways = NymGateway.fromCollectionString(pref[EXIT_GATEWAYS]),
						entryGateways = NymGateway.fromCollectionString(pref[ENTRY_GATEWAYS]),
						wgGateways = NymGateway.fromCollectionString(pref[WG_GATEWAYS]),
					)
				} catch (e: IllegalArgumentException) {
					Timber.e(e)
					Gateways()
				}
			} ?: Gateways()
		}
}

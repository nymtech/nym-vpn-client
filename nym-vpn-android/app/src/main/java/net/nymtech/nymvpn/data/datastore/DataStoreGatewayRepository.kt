package net.nymtech.nymvpn.data.datastore

import android.content.Context
import androidx.datastore.core.DataMigration
import androidx.datastore.core.handlers.ReplaceFileCorruptionHandler
import androidx.datastore.preferences.core.PreferenceDataStoreFactory
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.emptyPreferences
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStoreFile
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.GatewayType
import timber.log.Timber

// Persists gateway lists in a dedicated DataStore file and exposes them as an in-memory StateFlow.
class DataStoreGatewayRepository(context: Context, private val legacyDataStoreManager: DataStoreManager, appScope: CoroutineScope, ioDispatcher: CoroutineDispatcher) : GatewayRepository {
	companion object {
		private const val TAG = "gateway-repo"
		private const val FILE_NAME = "gateways"

		val ENTRY_GATEWAYS = stringPreferencesKey("ENTRY_GATEWAYS")
		val EXIT_GATEWAYS = stringPreferencesKey("EXIT_GATEWAYS")
		val WG_GATEWAYS = stringPreferencesKey("WG_GATEWAYS")

		private val LEGACY_KEYS = listOf(ENTRY_GATEWAYS, EXIT_GATEWAYS, WG_GATEWAYS)
	}

	private val legacyMigration = object : DataMigration<Preferences> {
		override suspend fun shouldMigrate(currentData: Preferences): Boolean = LEGACY_KEYS.any { legacyDataStoreManager.getFromStore(it) != null }

		override suspend fun migrate(currentData: Preferences): Preferences {
			val prefs = currentData.toMutablePreferences()
			LEGACY_KEYS.forEach { key ->
				if (prefs[key] == null) legacyDataStoreManager.getFromStore(key)?.let { prefs[key] = it }
			}
			return prefs.toPreferences()
		}

		override suspend fun cleanUp() {
			LEGACY_KEYS.forEach { legacyDataStoreManager.clear(it) }
		}
	}

	private val dataStore = PreferenceDataStoreFactory.create(
		corruptionHandler = ReplaceFileCorruptionHandler { e ->
			Timber.tag(TAG).e(e, "GatewayCacheCorrupted")
			emptyPreferences()
		},
		migrations = listOf(legacyMigration),
		scope = CoroutineScope(ioDispatcher + SupervisorJob()),
		produceFile = { context.preferencesDataStoreFile(FILE_NAME) },
	)

	override val gatewayFlow: StateFlow<Gateways> = dataStore.data
		.catch { e ->
			Timber.tag(TAG).e(e, "GatewayCacheReadFailed")
			emit(emptyPreferences())
		}
		.map { prefs -> Triple(prefs[ENTRY_GATEWAYS], prefs[EXIT_GATEWAYS], prefs[WG_GATEWAYS]) }
		.distinctUntilChanged()
		.map { (entry, exit, wg) ->
			Gateways(
				entryGateways = decode(entry),
				exitGateways = decode(exit),
				wgGateways = decode(wg),
			)
		}
		.flowOn(ioDispatcher)
		.stateIn(appScope, SharingStarted.Eagerly, Gateways())

	override suspend fun setGateways(type: GatewayType, gateways: List<NymGateway>) {
		dataStore.edit { it[type.gatewaysKey()] = gateways.toString() }
	}

	override suspend fun clear() {
		dataStore.edit { it.clear() }
	}

	private fun decode(raw: String?): List<NymGateway> = runCatching { NymGateway.fromCollectionString(raw) }
		.onFailure { Timber.tag(TAG).e(it, "GatewayCacheDecodeFailed") }
		.getOrDefault(emptyList())

	private fun GatewayType.gatewaysKey() = when (this) {
		GatewayType.MIXNET_ENTRY -> ENTRY_GATEWAYS
		GatewayType.MIXNET_EXIT -> EXIT_GATEWAYS
		GatewayType.WG -> WG_GATEWAYS
	}
}

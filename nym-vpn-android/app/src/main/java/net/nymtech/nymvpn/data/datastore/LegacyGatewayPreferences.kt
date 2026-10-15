package net.nymtech.nymvpn.data.datastore

import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import net.nymtech.nymvpn.data.file.FileGatewayRepository
import net.nymtech.vpn.model.NymGateway
import timber.log.Timber
import java.io.IOException

/** Keys under which gateway lists were stored as JSON in the shared preferences file before the file-backed repository. */
object LegacyGatewayPreferences {
	private const val TAG = "gateway-migration"

	private val entryKey = stringPreferencesKey("ENTRY_GATEWAYS")
	private val exitKey = stringPreferencesKey("EXIT_GATEWAYS")
	private val wgKey = stringPreferencesKey("WG_GATEWAYS")
	private val keys = listOf(entryKey, exitKey, wgKey)

	/**
	 * Copies any legacy lists into [repository] (without overwriting lists it already has) and then removes
	 * the legacy keys. The keys are kept when the preferences cannot be read or the lists cannot be persisted,
	 * so an upgrade never loses the cached gateways.
	 */
	suspend fun migrate(dataStoreManager: DataStoreManager, repository: FileGatewayRepository) {
		val prefs = try {
			dataStoreManager.preferencesFlow.filterNotNull().first()
		} catch (e: IOException) {
			Timber.tag(TAG).w(e, "LegacyGatewayPrefsUnreadable")
			return
		}
		if (keys.none { prefs.contains(it) }) return

		fun decode(raw: String?): List<NymGateway> = runCatching { NymGateway.fromCollectionString(raw) }
			.onFailure { Timber.tag(TAG).w(it, "LegacyGatewayListCorrupt") }
			.getOrDefault(emptyList())

		val persisted = repository.importLegacy(entry = decode(prefs[entryKey]), exit = decode(prefs[exitKey]), wg = decode(prefs[wgKey]))
		if (persisted) {
			dataStoreManager.clear(keys)
			Timber.tag(TAG).i("LegacyGatewayPrefsMigrated")
		} else {
			Timber.tag(TAG).w("LegacyGatewayMigrationDeferred")
		}
	}
}

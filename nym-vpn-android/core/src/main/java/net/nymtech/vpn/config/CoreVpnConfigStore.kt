package net.nymtech.vpn.config

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint

import android.content.Context
import androidx.datastore.preferences.core.MutablePreferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.intPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import net.nymtech.vpn.util.extensions.asEntryPoint
import net.nymtech.vpn.util.extensions.asExitPoint
import androidx.datastore.preferences.core.Preferences
import net.nymtech.vpn.model.config.CoreVpnConfig
import net.nymtech.vpn.model.config.LocalVpnPrefs

private const val DS_NAME = "core_vpn_config"

private val Context.coreVpnDataStore by preferencesDataStore(name = DS_NAME)

/**
 * Local-only VPN preference store.
 *
 * Only holds settings the vpn service has no equivalent for (see [LocalVpnPrefs]), plus a
 * one-shot migration flag. The legacy keys below (entry/exit/mode/dns/etc.) are kept read-only
 * so [readLegacyFullConfigForMigration] can migrate a pre-existing install's values into the vpn
 * service's own persisted config exactly once - see [isMigratedToRustConfig].
 */
class CoreVpnConfigStore(private val context: Context) {

	private companion object Companion {
		private val KEY_ENTRY = stringPreferencesKey("ENTRY_POINT")
		private val KEY_EXIT = stringPreferencesKey("EXIT_POINT")
		private val KEY_MODE = stringPreferencesKey("MODE")
		private val KEY_BYPASS_LAN = booleanPreferencesKey("BYPASS_LAN")
		private val KEY_BRIDGES = booleanPreferencesKey("ENABLE_BRIDGES")
		private val KEY_CUSTOM_DNS_ENABLED = booleanPreferencesKey("CUSTOM_DNS_ENABLED")
		private val KEY_CUSTOM_DNS_LIST = stringPreferencesKey("CUSTOM_DNS_LIST")
		private val KEY_RESTRICTED_APPS = stringPreferencesKey("RESTRICTED_APPS")

		private val KEY_ENV_NETWORK = stringPreferencesKey("ENV_NETWORK")
		private val KEY_ENV_DEBUG = booleanPreferencesKey("ENV_DEBUG")
		private val KEY_ENV_SENTRY = booleanPreferencesKey("ENV_SENTRY")
		private val KEY_AD_BLOCKING = booleanPreferencesKey("AD_BLOCKING_ENABLED")
		private val KEY_STEALTH_MODE = booleanPreferencesKey("STEALTH_MODE_ENABLED")
		private val KEY_NODE_FAMILIES_NOTIFICATIONS = booleanPreferencesKey("NODE_FAMILIES_NOTIFICATIONS_ENABLED")
		private val KEY_GEO_EXCLUSION_ENABLED = booleanPreferencesKey("GEO_EXCLUSION_ENABLED")
		private val KEY_GEO_EXCLUSION_PORT = intPreferencesKey("GEO_EXCLUSION_PORT")
		private val KEY_GEO_EXCLUSION_COUNTRIES = stringPreferencesKey("GEO_EXCLUSION_COUNTRIES")

		private val KEY_MIGRATED_TO_RUST_CONFIG = booleanPreferencesKey("MIGRATED_TO_RUST_CONFIG")
		private val KEY_ENSURED_GEO_LOCATION_DEFAULT = booleanPreferencesKey("ENSURED_GEO_LOCATION_DEFAULT")

		private const val SEP = "|"
		private const val MAX_DNS = 5
	}

	val localPrefsFlow: Flow<LocalVpnPrefs> = context.coreVpnDataStore.data.map { prefs ->
		prefs.toLocalPrefs()
	}

	suspend fun getLocalPrefs(): LocalVpnPrefs = localPrefsFlow.first()

	suspend fun updateLocalPrefs(transform: (LocalVpnPrefs) -> LocalVpnPrefs): LocalVpnPrefs {
		var updated = LocalVpnPrefs()
		context.coreVpnDataStore.edit { prefs ->
			val current = prefs.toLocalPrefs()
			updated = transform(current)
			prefs.fromLocalPrefs(updated)
		}
		return updated
	}

	suspend fun isMigratedToRustConfig(): Boolean = context.coreVpnDataStore.data.first()[KEY_MIGRATED_TO_RUST_CONFIG] ?: false

	suspend fun markMigratedToRustConfig() {
		context.coreVpnDataStore.edit { prefs ->
			prefs[KEY_MIGRATED_TO_RUST_CONFIG] = true
		}
	}

	suspend fun hasEnsuredGeoLocationDefault(): Boolean = context.coreVpnDataStore.data.first()[KEY_ENSURED_GEO_LOCATION_DEFAULT] ?: false

	suspend fun markGeoLocationDefaultEnsured() {
		context.coreVpnDataStore.edit { prefs ->
			prefs[KEY_ENSURED_GEO_LOCATION_DEFAULT] = true
		}
	}

	/**
	 * True if any pre-existing (pre-vpn-service-persistence) config value is present. Used to
	 * skip migration on a fresh install, where an empty store would otherwise read back as
	 * Kotlin defaults and overwrite the vpn service's own (possibly different) defaults - see
	 * [readLegacyFullConfigForMigration].
	 */
	suspend fun hasLegacyConfig(): Boolean {
		val prefs = context.coreVpnDataStore.data.first()
		return prefs.contains(KEY_ENTRY) || prefs.contains(KEY_EXIT) || prefs.contains(KEY_MODE)
	}

	/**
	 * Reads the full legacy config, including the fields now owned by the vpn service's
	 * persisted config. Only meant to be called once, to migrate a pre-existing install - see
	 * [isMigratedToRustConfig].
	 */
	suspend fun readLegacyFullConfigForMigration(): CoreVpnConfig = context.coreVpnDataStore.data.first().toLegacyFullConfig()

	private fun Preferences.toLocalPrefs(): LocalVpnPrefs {
		val network = this[KEY_ENV_NETWORK]?.let {
			runCatching { Tunnel.Environment.valueOf(it) }.getOrElse { Tunnel.Environment.MAINNET }
		} ?: Tunnel.Environment.MAINNET

		return LocalVpnPrefs(
			network = network,
			debugLog = this[KEY_ENV_DEBUG] ?: false,
			sentry = this[KEY_ENV_SENTRY] ?: false,
			bypassLan = this[KEY_BYPASS_LAN] ?: false,
			restrictedApps = decodeList(this[KEY_RESTRICTED_APPS]),
		)
	}

	private fun MutablePreferences.fromLocalPrefs(prefs: LocalVpnPrefs) {
		this[KEY_ENV_NETWORK] = prefs.network.networkName().uppercase()
		this[KEY_ENV_DEBUG] = prefs.debugLog
		this[KEY_ENV_SENTRY] = prefs.sentry
		this[KEY_BYPASS_LAN] = prefs.bypassLan
		this[KEY_RESTRICTED_APPS] = encodeList(prefs.restrictedApps)
	}

	private fun Preferences.toLegacyFullConfig(): CoreVpnConfig {
		val entry: EntryPoint = this[KEY_ENTRY]?.asEntryPoint() ?: EntryPoint.Auto(excludeUserCountry = true)
		val exit: ExitPoint = this[KEY_EXIT]?.asExitPoint() ?: ExitPoint.Auto(excludeEntryPointCountry = true, excludeUserCountry = true)

		val mode: Tunnel.Mode =
			this[KEY_MODE]?.let { runCatching { Tunnel.Mode.valueOf(it) }.getOrNull() }
				?: Tunnel.Mode.TWO_HOP_MIXNET

		val localPrefs = toLocalPrefs()

		return CoreVpnConfig(
			entryPoint = entry,
			exitPoint = exit,
			mode = mode,
			bypassLan = localPrefs.bypassLan,
			enableBridges = this[KEY_BRIDGES] ?: false,
			customDnsEnabled = this[KEY_CUSTOM_DNS_ENABLED] ?: false,
			customDns = decodeList(this[KEY_CUSTOM_DNS_LIST]).take(MAX_DNS),
			restrictedApps = localPrefs.restrictedApps,
			network = localPrefs.network,
			debugLog = localPrefs.debugLog,
			sentry = localPrefs.sentry,
			adBlockingEnabled = this[KEY_AD_BLOCKING] ?: false,
			stealthMode = this[KEY_STEALTH_MODE] ?: false,
			nodeFamiliesNotificationsEnabled = this[KEY_NODE_FAMILIES_NOTIFICATIONS] ?: true,
			geoExclusionEnabled = this[KEY_GEO_EXCLUSION_ENABLED] ?: false,
			geoExclusionPort = this[KEY_GEO_EXCLUSION_PORT] ?: 1081,
			geoExclusionCountries = decodeList(this[KEY_GEO_EXCLUSION_COUNTRIES]).ifEmpty { listOf("CN") },
		)
	}

	private fun encodeList(list: List<String>): String = list.asSequence()
		.map { it.trim() }
		.filter { it.isNotEmpty() }
		.joinToString(SEP)

	private fun decodeList(encoded: String?): List<String> {
		val s = encoded.orEmpty().trim()
		if (s.isEmpty()) return emptyList()
		return s.split(SEP).map { it.trim() }.filter { it.isNotEmpty() }
	}
}

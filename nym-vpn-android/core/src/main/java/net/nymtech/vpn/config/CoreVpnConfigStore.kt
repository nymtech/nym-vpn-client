package net.nymtech.vpn.config

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint

import android.content.Context
import androidx.datastore.preferences.core.MutablePreferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.nymtech.vpn.util.extensions.asEntryPoint
import net.nymtech.vpn.util.extensions.asExitPoint
import net.nymtech.vpn.util.extensions.asAlgorithm
import net.nymtech.vpn.util.extensions.asString
import androidx.datastore.preferences.core.Preferences
import net.nymtech.vpn.model.config.CoreVpnConfig
import nym_vpn_lib_types.GatewaySelectionAlgorithm

/**
 * Canonical persisted VPN config store.
 */
class CoreVpnConfigStore(private val context: Context) {

	private companion object Companion {
		private const val DS_NAME = "core_vpn_config"

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
		private val KEY_LEWES = booleanPreferencesKey("ENABLE_LEWES_PROTOCOL")
		private val KEY_AD_BLOCKING = booleanPreferencesKey("AD_BLOCKING_ENABLED")
		private val KEY_STEALTH_MODE = booleanPreferencesKey("STEALTH_MODE_ENABLED")
		private val KEY_ALGORITHM = stringPreferencesKey("ALGORITHM")

		private const val SEP = "|"
		private const val MAX_DNS = 5
	}

	private val Context.dataStore by preferencesDataStore(name = DS_NAME)

	val configFlow: Flow<CoreVpnConfig> = context.dataStore.data.map { prefs ->
		prefs.toCoreConfig()
	}

	suspend fun get(): CoreVpnConfig = throw UnsupportedOperationException("Use CoreVpnConfigRepository.get() instead")

	suspend fun update(transform: (CoreVpnConfig) -> CoreVpnConfig) {
		context.dataStore.edit { prefs ->
			val current = prefs.toCoreConfig()
			val updated = transform(current)
			prefs.fromCoreConfig(updated)
		}
	}

	private fun Preferences.toCoreConfig(): CoreVpnConfig {
		val entry: EntryPoint = this[KEY_ENTRY]?.asEntryPoint() ?: EntryPoint.Random
		val exit: ExitPoint = this[KEY_EXIT]?.asExitPoint() ?: ExitPoint.Random

		val mode: Tunnel.Mode =
			this[KEY_MODE]?.let { runCatching { Tunnel.Mode.valueOf(it) }.getOrNull() }
				?: Tunnel.Mode.TWO_HOP_MIXNET

		val bypassLan = this[KEY_BYPASS_LAN] ?: false
		val enableBridges = this[KEY_BRIDGES] ?: false
		val customDnsEnabled = this[KEY_CUSTOM_DNS_ENABLED] ?: false

		val customDns = decodeList(this[KEY_CUSTOM_DNS_LIST]).take(MAX_DNS)
		val restrictedApps = decodeList(this[KEY_RESTRICTED_APPS])

		val network = this[KEY_ENV_NETWORK] ?.let {
			runCatching { Tunnel.Environment.valueOf(it) }.getOrElse { Tunnel.Environment.MAINNET }
		} ?: Tunnel.Environment.MAINNET

		val debug = this[KEY_ENV_DEBUG] ?: true
		val sentry = this[KEY_ENV_SENTRY] ?: false
		val lewes = this[KEY_LEWES] ?: false
		val adBlockingEnabled = this[KEY_AD_BLOCKING] ?: false
		val stealthMode = this[KEY_STEALTH_MODE] ?: false
		val algorithm: GatewaySelectionAlgorithm = this[KEY_ALGORITHM]?.let { runCatching { it.asAlgorithm() }.getOrNull() } ?: GatewaySelectionAlgorithm.AUTO

		return CoreVpnConfig(
			entryPoint = entry,
			exitPoint = exit,
			mode = mode,
			bypassLan = bypassLan,
			enableBridges = enableBridges,
			customDnsEnabled = customDnsEnabled,
			customDns = customDns,
			restrictedApps = restrictedApps,
			network = network,
			debugLog = debug,
			sentry = sentry,
			lewes = lewes,
			adBlockingEnabled = adBlockingEnabled,
			stealthMode = stealthMode,
			algorithm = algorithm,
		)
	}

	private fun MutablePreferences.fromCoreConfig(cfg: CoreVpnConfig) {
		this[KEY_ENTRY] = cfg.entryPoint.asString()
		this[KEY_EXIT] = cfg.exitPoint.asString()
		this[KEY_MODE] = cfg.mode.name
		this[KEY_BYPASS_LAN] = cfg.bypassLan
		this[KEY_BRIDGES] = cfg.enableBridges
		this[KEY_CUSTOM_DNS_ENABLED] = cfg.customDnsEnabled
		this[KEY_CUSTOM_DNS_LIST] = encodeList(cfg.customDns)
		this[KEY_RESTRICTED_APPS] = encodeList(cfg.restrictedApps)
		this[KEY_ENV_NETWORK] = cfg.network.networkName().uppercase()
		this[KEY_ENV_DEBUG] = cfg.debugLog
		this[KEY_ENV_SENTRY] = cfg.sentry
		this[KEY_LEWES] = cfg.lewes
		this[KEY_AD_BLOCKING] = cfg.adBlockingEnabled
		this[KEY_STEALTH_MODE] = cfg.stealthMode
		this[KEY_ALGORITHM] = cfg.algorithm.name
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

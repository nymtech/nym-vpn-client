package net.nymtech.vpn.config

import android.content.Context
import kotlinx.coroutines.flow.first
import net.nymtech.vpn.model.config.CoreVpnConfig

class CoreVpnConfigRepository(context: Context) {

	private val store = CoreVpnConfigStore(context)

	suspend fun get(): CoreVpnConfig = store.configFlow.first()

	suspend fun applyUpdate(update: CoreVpnConfigUpdate): CoreVpnConfig = applyUpdates(listOf(update))

	suspend fun applyUpdates(updates: List<CoreVpnConfigUpdate>): CoreVpnConfig {
		if (updates.isEmpty()) return get()

		store.update { current ->
			updates.fold(current) { acc, update ->
				when (update) {
					is CoreVpnConfigUpdate.SetEntryPoint -> acc.copy(entryPoint = update.value)
					is CoreVpnConfigUpdate.SetExitPoint -> acc.copy(exitPoint = update.value)
					is CoreVpnConfigUpdate.SetMode -> acc.copy(mode = update.value)
					is CoreVpnConfigUpdate.SetBypassLan -> acc.copy(bypassLan = update.value)
					is CoreVpnConfigUpdate.SetEnableBridges -> acc.copy(enableBridges = update.value)
					is CoreVpnConfigUpdate.SetCustomDnsEnabled -> acc.copy(customDnsEnabled = update.value)
					is CoreVpnConfigUpdate.SetCustomDns -> acc.copy(customDns = update.value)
					is CoreVpnConfigUpdate.SetRestrictedApps -> acc.copy(restrictedApps = update.value)
					is CoreVpnConfigUpdate.SetNetwork -> acc.copy(network = update.value)
					is CoreVpnConfigUpdate.SetDebugLog -> acc.copy(debugLog = update.value)
					is CoreVpnConfigUpdate.SetSentry -> acc.copy(sentry = update.value)
					is CoreVpnConfigUpdate.SetAdBlockingEnabled -> acc.copy(adBlockingEnabled = update.value)
					is CoreVpnConfigUpdate.SetStealthMode -> acc.copy(stealthMode = update.value)
					is CoreVpnConfigUpdate.SetNodeFamiliesNotificationsEnabled -> acc.copy(nodeFamiliesNotificationsEnabled = update.value)
					is CoreVpnConfigUpdate.SetGeoExclusionEnabled -> acc.copy(geoExclusionEnabled = update.value)
					is CoreVpnConfigUpdate.SetGeoExclusionPort -> acc.copy(geoExclusionPort = update.value)
					is CoreVpnConfigUpdate.SetGeoExclusionCountries -> acc.copy(geoExclusionCountries = update.value)
				}
			}
		}

		return get()
	}
}

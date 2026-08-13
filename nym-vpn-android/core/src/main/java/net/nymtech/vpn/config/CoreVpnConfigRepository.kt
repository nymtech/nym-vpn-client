package net.nymtech.vpn.config

import android.content.Context
import net.nymtech.vpn.model.config.CoreVpnConfig
import net.nymtech.vpn.model.config.LocalVpnPrefs

class CoreVpnConfigRepository(context: Context) {

	private val store = CoreVpnConfigStore(context)

	suspend fun getLocalPrefs(): LocalVpnPrefs = store.getLocalPrefs()

	suspend fun updateLocalPrefs(transform: (LocalVpnPrefs) -> LocalVpnPrefs): LocalVpnPrefs =
		store.updateLocalPrefs(transform)

	suspend fun isMigratedToRustConfig(): Boolean = store.isMigratedToRustConfig()

	suspend fun markMigratedToRustConfig() = store.markMigratedToRustConfig()

	suspend fun hasLegacyConfig(): Boolean = store.hasLegacyConfig()

	/** Legacy, pre-vpn-service-persistence config - only used once, to migrate an existing install. */
	suspend fun readLegacyFullConfigForMigration(): CoreVpnConfig = store.readLegacyFullConfigForMigration()
}

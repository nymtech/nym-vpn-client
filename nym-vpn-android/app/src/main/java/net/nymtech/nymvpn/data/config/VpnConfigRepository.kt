package net.nymtech.nymvpn.data.config

import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.model.config.CoreVpnConfig
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import net.nymtech.vpn.model.config.ConfigResult

interface VpnConfigRepository {
	val configFlow: Flow<CoreVpnConfig>

	suspend fun getConfig(): CoreVpnConfig
	suspend fun apply(updates: List<CoreVpnConfigUpdate>): ConfigResult
	suspend fun apply(update: CoreVpnConfigUpdate): ConfigResult
}

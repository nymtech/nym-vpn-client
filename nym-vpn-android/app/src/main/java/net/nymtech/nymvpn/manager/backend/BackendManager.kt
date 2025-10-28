package net.nymtech.nymvpn.manager.backend

import kotlinx.coroutines.flow.Flow
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.vpn.backend.Backend
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.SystemMessage

interface BackendManager {
	suspend fun getBackend(): Backend
	suspend fun stopTunnel()
	suspend fun startTunnel()
	suspend fun storeMnemonic(mnemonic: String)
	suspend fun isMnemonicStored(): Boolean
	suspend fun removeMnemonic()
	suspend fun getAccountLinks(): ParsedAccountLinks?
	suspend fun getSystemMessages(): List<SystemMessage>
	suspend fun getGateways(gatewayType: GatewayType): List<NymGateway>
	suspend fun refreshAccountLinks()
	suspend fun refresh()
	suspend fun createAccount()
	suspend fun registerAccount(purchaseToken: String): String
	suspend fun refreshAccount()
	suspend fun getMnemonic(): List<String>
	val stateFlow: Flow<TunnelManagerState>
	fun getState(): Tunnel.State
	fun initialize()
}

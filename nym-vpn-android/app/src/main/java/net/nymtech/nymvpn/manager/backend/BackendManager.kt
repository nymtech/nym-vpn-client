package net.nymtech.nymvpn.manager.backend

import kotlinx.coroutines.flow.Flow
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib.AccountLinks
import nym_vpn_lib.AccountStateSummary
import nym_vpn_lib.GatewayType
import nym_vpn_lib.SystemMessage

interface BackendManager {
	suspend fun stopTunnel()
	suspend fun startTunnel()
	suspend fun storeMnemonic(mnemonic: String)
	suspend fun isMnemonicStored(): Boolean
	suspend fun removeMnemonic()
	suspend fun getAccountSummary(): AccountStateSummary
	suspend fun getAccountLinks(): AccountLinks?
	suspend fun getSystemMessages(): List<SystemMessage>
	suspend fun getGateways(gatewayType: GatewayType): List<NymGateway>
	suspend fun refreshAccountLinks()
	val stateFlow: Flow<TunnelManagerState>
	fun getState(): Tunnel.State
	fun initialize()
}

package net.nymtech.nymvpn.service.tunnel

import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib.AccountLinks
import nym_vpn_lib.AccountStateSummary

interface TunnelManager {
	suspend fun stop()
	suspend fun start(fromBackground: Boolean = true)
	suspend fun storeMnemonic(mnemonic: String)
	suspend fun isMnemonicStored(): Boolean
	suspend fun removeMnemonic()
	suspend fun getAccountSummary(): AccountStateSummary
	suspend fun getAccountLinks(): AccountLinks?
	suspend fun refreshAccountLinks()
	val stateFlow: Flow<TunnelManagerState>
	fun getState(): Tunnel.State
}

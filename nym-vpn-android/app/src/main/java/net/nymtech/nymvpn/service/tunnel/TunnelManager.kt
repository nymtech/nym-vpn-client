package net.nymtech.nymvpn.service.tunnel

import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.backend.Tunnel

interface TunnelManager {
	suspend fun stop()
	suspend fun start(fromBackground: Boolean = true)
	suspend fun storeMnemonic(credential: String)
	suspend fun isMnemonicStored(): Boolean
	suspend fun removeMnemonic()
	val stateFlow: Flow<TunnelState>
	fun getState(): Tunnel.State
}

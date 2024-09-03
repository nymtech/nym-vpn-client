package net.nymtech.nymvpn.service.tunnel

import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.backend.Tunnel
import java.time.Instant

interface TunnelManager {
	suspend fun stop(): Result<Tunnel.State>
	suspend fun start(): Result<Tunnel.State>
	suspend fun importCredential(credential: String): Result<Instant?>
	val stateFlow: Flow<TunnelState>
	fun getState(): Tunnel.State
}

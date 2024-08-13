package net.nymtech.nymvpn.service.tunnel

import android.content.Context
import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.Tunnel
import java.time.Instant

interface TunnelManager {
	suspend fun stop(context: Context): Result<Tunnel.State>
	suspend fun start(context: Context): Result<Tunnel.State>
	suspend fun importCredential(credential: String): Result<Instant?>
	val stateFlow: Flow<TunnelState>
	fun getState(): Tunnel.State
}

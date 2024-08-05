package net.nymtech.nymvpn.service.tunnel

import android.content.Context
import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.Tunnel

interface TunnelManager {
	suspend fun stopVpn(context: Context): Result<Tunnel.State>
	suspend fun startVpn(context: Context): Result<Tunnel.State>
	val stateFlow: Flow<TunnelState>
}

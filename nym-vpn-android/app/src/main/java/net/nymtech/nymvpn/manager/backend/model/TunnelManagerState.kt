package net.nymtech.nymvpn.manager.backend.model

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib.AccountLinks
import nym_vpn_lib.ConnectionData

data class TunnelManagerState(
	val tunnelState: Tunnel.State = Tunnel.State.Down,
	val backendUiEvent: BackendUiEvent? = null,
	val connectionData: ConnectionData? = null,
	val mixnetConnectionState: MixnetConnectionState? = null,
	val isMnemonicStored: Boolean = false,
	val deviceId: String? = null,
	val accountLinks: AccountLinks? = null,
	val isInitialized: Boolean = false,
	val isNetworkCompatible: Boolean = true,
)

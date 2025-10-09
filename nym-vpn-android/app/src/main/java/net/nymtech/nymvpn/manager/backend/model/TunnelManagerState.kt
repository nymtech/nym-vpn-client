package net.nymtech.nymvpn.manager.backend.model

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.EstablishConnectionState

data class TunnelManagerState(
	val tunnelState: Tunnel.State = Tunnel.State.Down,
	val backendUiEvent: BackendUiEvent? = null,
	val connectionData: ConnectionInfo? = null,
	val establishConnectionState: EstablishConnectionState? = null,
	val mixnetConnectionState: MixnetConnectionState? = null,
	val isMnemonicStored: Boolean = false,
	val deviceId: String? = null,
	val accountId: String? = null,
	val accountLinks: ParsedAccountLinks? = null,
	val isInitialized: Boolean = false,
	val isNetworkCompatible: Boolean = true,
)

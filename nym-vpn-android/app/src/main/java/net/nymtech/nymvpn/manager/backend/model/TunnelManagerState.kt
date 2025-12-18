package net.nymtech.nymvpn.manager.backend.model

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.EstablishConnectionState
import nym_vpn_lib_types.ParsedAccountLinks

data class TunnelManagerState(
	val tunnelState: Tunnel.State = Tunnel.State.Down,
	val accountState: AccountControllerState = AccountControllerState.Offline,
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
	val isRestarting: Boolean = false,
)

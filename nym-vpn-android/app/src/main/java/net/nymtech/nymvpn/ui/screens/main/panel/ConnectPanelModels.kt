package net.nymtech.nymvpn.ui.screens.main.panel

import net.nymtech.nymvpn.ui.model.ConnectionState
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.Score

enum class PanelState { COLLAPSED, FULL }

enum class ConnectMode { FAST, MIXNET }

enum class ConnectAction { CONNECT, DISCONNECT, STOP_KILL_SWITCH, GET_STARTED }

data class ServerNode(val id: String = "", val name: String?, val countryCode: String?, val location: String?, val score: Score)

data class ConnectPanelState(
	val connectionState: ConnectionState,
	val accountState: AccountControllerState,
	val isMnemonicStored: Boolean,
	val connectMode: ConnectMode,
	val exitNode: ServerNode,
	val entryNode: ServerNode,
	val initialPanelState: PanelState,
	val isSubscriptionExpired: Boolean = false,
)

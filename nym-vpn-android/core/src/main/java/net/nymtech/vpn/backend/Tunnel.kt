package net.nymtech.vpn.backend

import nym_vpn_lib_types.ErrorStateReason

/**
 * VPN tunnel interface:
 * - lifecycle state
 * - routing mode
 * - network environment
 */
interface Tunnel {

	sealed class State {
		data object Up : State()
		data object Down : State()
		data object InitializingClient : State()
		data object EstablishingConnection : State()
		data object Disconnecting : State()
		data object Offline : State()
		data class Error(val reason: ErrorStateReason) : State()
	}

	enum class Mode {
		FIVE_HOP_MIXNET,
		TWO_HOP_MIXNET,
		;

		fun isTwoHop(): Boolean = this == TWO_HOP_MIXNET
	}

	enum class Environment {
		CANARY,
		EVIL,
		MAINNET,
		SANDBOX,
		;

		fun networkName(): String = name.lowercase()
	}
}

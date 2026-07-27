package net.nymtech.vpn.backend

import nym_vpn_lib_types.ErrorStateReason
import nym_vpn_lib_types.TunnelType

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

		fun toTunnelType(): TunnelType = when (this) {
			FIVE_HOP_MIXNET -> TunnelType.MIXNET
			TWO_HOP_MIXNET -> TunnelType.WIREGUARD
		}
	}

	enum class Environment {
		CANARY,
		MAINNET,
		SANDBOX,
		;

		fun networkName(): String = name.lowercase()
	}
}

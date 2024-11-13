package net.nymtech.vpn.backend

import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Statistics
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint

interface Tunnel {

	var entryPoint: EntryPoint
	var exitPoint: ExitPoint
	var mode: Mode
	var environment: Environment

	/**
	 * React to a change in state of the tunnel. Should only be directly called by Backend.
	 *
	 * @param newState The new state of the tunnel.
	 */
	fun onStateChange(newState: State)

	/**
	 * React to a change in state of the tunnel statistics. Should only be directly called by Backend.
	 *
	 * @param stats The new state of the tunnel statistics.
	 */
	fun onStatisticChange(stats: Statistics)

	/**
	 * React to a change in state of backend messages. Should only be directly called by Backend.
	 *
	 * @param message The new message from the backend.
	 */
	fun onBackendMessage(message: BackendMessage)

	/**
	 * Sealed class to represent all possible states of a [Tunnel].
	 */
	sealed class State {
		data object Up : State()

		data object Down : State()

		data object Connecting {
			data object InitializingClient : State()

			data object EstablishingConnection : State()
		}

		data object Disconnecting : State()
	}

	/**
	 * Enum class to represent all possible modes of a [Tunnel].
	 */
	enum class Mode {
		FIVE_HOP_MIXNET,
		TWO_HOP_MIXNET,
	}

	/**
	 * Enum class to represent all possible environments of a [Tunnel].
	 */
	enum class Environment {
		CANARY,
		MAINNET,
		SANDBOX,
		QA,
		;

		fun networkName(): String {
			return this.name.lowercase()
		}
	}
}

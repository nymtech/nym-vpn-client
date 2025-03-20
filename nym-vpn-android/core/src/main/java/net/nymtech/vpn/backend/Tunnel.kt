package net.nymtech.vpn.backend

import net.nymtech.vpn.model.BackendEvent
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint

interface Tunnel {

	val entryPoint: EntryPoint
	val exitPoint: ExitPoint
	val mode: Mode
	val environment: Environment
	val credentialMode: Boolean?
	val bypassLan: Boolean

	/**
	 * React to a change in state of the tunnel. Should only be directly called by Backend.
	 *
	 * @param newState The new state of the tunnel.
	 */
	fun onStateChange(newState: State)

	/**
	 * React to a change in state of the tunnel connection information. Should only be directly called by Backend.
	 *
	 * @param event The new state of mixnet or tunnel connection details.
	 */
	fun onBackendEvent(event: BackendEvent)

	/**
	 * Sealed class to represent all possible states of a [Tunnel].
	 */
	sealed class State {
		data object Up : State()

		data object Down : State()

		data object InitializingClient : State()

		data object EstablishingConnection : State()

		data object Disconnecting : State()

		data object Offline : State()
	}

	/**
	 * Enum class to represent all possible modes of a [Tunnel].
	 */
	enum class Mode {
		FIVE_HOP_MIXNET,
		TWO_HOP_MIXNET,
		;

		fun isTwoHop() = this == TWO_HOP_MIXNET
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
	companion object {
		val IPV4_PUBLIC_NETWORKS = listOf(
			"0.0.0.0/5", "8.0.0.0/7", "11.0.0.0/8", "12.0.0.0/6", "16.0.0.0/4", "32.0.0.0/3",
			"64.0.0.0/2", "128.0.0.0/3", "160.0.0.0/5", "168.0.0.0/6", "172.0.0.0/12",
			"172.32.0.0/11", "172.64.0.0/10", "172.128.0.0/9", "173.0.0.0/8", "174.0.0.0/7",
			"176.0.0.0/4", "192.0.0.0/9", "192.128.0.0/11", "192.160.0.0/13", "192.169.0.0/16",
			"192.170.0.0/15", "192.172.0.0/14", "192.176.0.0/12", "192.192.0.0/10",
			"193.0.0.0/8", "194.0.0.0/7", "196.0.0.0/6", "200.0.0.0/5", "208.0.0.0/4",
		)
	}
}

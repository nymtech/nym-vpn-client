package net.nymtech.vpn

import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Statistics
import net.nymtech.vpn.util.Constants
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint
import java.net.URL

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
		MAINNET {
			override val nymVpnApiUrl: URL
				get() = URL("https://nymvpn.com/api")
			override val apiUrl: URL
				get() = URL("https://validator.nymtech.net/api/")
		},
		SANDBOX {
			override val nymVpnApiUrl: URL?
				get() = null
			override val apiUrl: URL
				get() = URL("https://sandbox-nym-api1.nymtech.net/api")
		},
		CANARY {
			override val nymVpnApiUrl: URL?
				get() = null
			override val apiUrl: URL
				get() = URL("https://canary-api.performance.nymte.ch/api")
		}, ;

		/**
		 * Utility function to set all required environment variables for a [Tunnel].
		 */
		fun setup() {
			when (this) {
				MAINNET -> Constants.setupEnvironmentMainnet()
				SANDBOX -> Constants.setupEnvironmentSandbox()
				CANARY -> Constants.setupEnvironmentCanary()
			}
		}

		companion object {
			fun from(flavor: String): Environment {
				return when (flavor) {
					"fdroid", "general" -> MAINNET
					"canary" -> CANARY
					"sandbox" -> SANDBOX
					else -> MAINNET
				}
			}
		}

		abstract val nymVpnApiUrl: URL?
		abstract val apiUrl: URL
	}
}

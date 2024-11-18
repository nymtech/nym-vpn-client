package net.nymtech.vpn.backend

import android.system.Os
import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Statistics
import net.nymtech.vpn.util.Constants.LOG_LEVEL
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

		data object InitializingClient : State()

		data object EstablishingConnection : State()

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

		companion object {
			fun setupMainnet() {
				Os.setenv("CONFIGURED", "true", true)
				Os.setenv("RUST_LOG", LOG_LEVEL, true)
				Os.setenv("RUST_BACKTRACE", "1", true)
				Os.setenv("NETWORK_NAME", "mainnet", true)
				Os.setenv("BECH32_PREFIX", "n", true)
				Os.setenv("MIX_DENOM", "unym", true)
				Os.setenv("MIX_DENOM_DISPLAY", "nym", true)
				Os.setenv("STAKE_DENOM", "unyx", true)
				Os.setenv("STAKE_DENOM_DISPLAY", "nyx", true)
				Os.setenv("DENOMS_EXPONENT", "6", true)
				Os.setenv("REWARDING_VALIDATOR_ADDRESS", "n10yyd98e2tuwu0f7ypz9dy3hhjw7v772q6287gy", true)
				Os.setenv(
					"MIXNET_CONTRACT_ADDRESS",
					"n17srjznxl9dvzdkpwpw24gg668wc73val88a6m5ajg6ankwvz9wtst0cznr",
					true,
				)
				Os.setenv(
					"VESTING_CONTRACT_ADDRESS",
					"n1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrq73f2nw",
					true,
				)
				Os.setenv("STATISTICS_SERVICE_DOMAIN_ADDRESS", "https://mainnet-stats.nymte.ch:8090", true)
				Os.setenv("EXPLORER_API", "https://explorer.nymtech.net/api/", true)
				Os.setenv("NYXD", "https://rpc.nymtech.net", true)
				Os.setenv("NYXD_WS", "wss://rpc.nymtech.net/websocket", true)
				Os.setenv("NYM_API", "https://validator.nymtech.net/api/", true)
				Os.setenv("NYM_VPN_API", "https://nymvpn.com/api", true)
			}
		}
	}
}

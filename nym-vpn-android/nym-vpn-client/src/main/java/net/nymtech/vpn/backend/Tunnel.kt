package net.nymtech.vpn.backend

import android.system.Os
import net.nymtech.vpn.model.BackendMessage
import net.nymtech.vpn.model.Statistics
import net.nymtech.vpn.util.Constants.LOG_LEVEL
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
		CANARY {
			override fun setup() {
				super.setupCommon()
				Os.setenv("REWARDING_VALIDATOR_ADDRESS", "n1duuyj2th2y0z4u4f4wtljpdz9s3pxtu0xx6zdz", true)
				Os.setenv(
					"MIXNET_CONTRACT_ADDRESS",
					"n14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sjyvg3g",
					true,
				)
				Os.setenv(
					"COCONUT_BANDWIDTH_CONTRACT_ADDRESS",
					"n1mf6ptkssddfmxvhdx0ech0k03ktp6kf9yk59renau2gvht3nq2gqt5tdrk",
					true,
				)
				Os.setenv(
					"GROUP_CONTRACT_ADDRESS",
					"n1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sa07c47",
					true,
				)
				Os.setenv(
					"MULTISIG_CONTRACT_ADDRESS",
					"n1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqx5a364",
					true,
				)
				Os.setenv(
					"COCONUT_DKG_CONTRACT_ADDRESS",
					"n1aakfpghcanxtc45gpqlx8j3rq0zcpyf49qmhm9mdjrfx036h4z5sy2vfh9",
					true,
				)

				Os.setenv("EXPLORER_API", "https://canary-explorer.performance.nymte.ch/api", true)
				Os.setenv("NYXD", "https://canary-validator.performance.nymte.ch", true)
				Os.setenv("NYM_API", "https://canary-api.performance.nymte.ch/api", true)
			}

			override val nymVpnApiUrl: URL?
				get() = null
			override val apiUrl: URL
				get() = URL("https://canary-api.performance.nymte.ch/api")
			override val accountUrl: URL
				get() = URL("https://nym-dot-com-git-deploy-canary-nyx-network-staging.vercel.app/")
			override val createAccountUrl: URL
				get() = URL("https://nym-dot-com-git-deploy-canary-nyx-network-staging.vercel.app/account/login")
		},
		MAINNET {
			override fun setup() {
				super.setupCommon()
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
			}

			override val nymVpnApiUrl: URL
				get() = URL("https://nymvpn.com/api")
			override val apiUrl: URL
				get() = URL("https://validator.nymtech.net/api/")
			override val accountUrl: URL
				get() = URL("https://nymvpn.com/account/login")
			override val createAccountUrl: URL
				get() = URL("https://nymvpn.com/account/create")
		},
		SANDBOX {
			override fun setup() {
				super.setupCommon()
				Os.setenv("REWARDING_VALIDATOR_ADDRESS", "n1pefc2utwpy5w78p2kqdsfmpjxfwmn9d39k5mqa", true)
				Os.setenv(
					"MIXNET_CONTRACT_ADDRESS",
					"n1xr3rq8yvd7qplsw5yx90ftsr2zdhg4e9z60h5duusgxpv72hud3sjkxkav",
					true,
				)
				Os.setenv(
					"VESTING_CONTRACT_ADDRESS",
					"n1unyuj8qnmygvzuex3dwmg9yzt9alhvyeat0uu0jedg2wj33efl5qackslz",
					true,
				)
				Os.setenv(
					"COCONUT_BANDWIDTH_CONTRACT_ADDRESS",
					"n13902g92xfefeyzuyed49snlm5fxv5ms6mdq5kvrut27hasdw5a9q9vyw6c",
					true,
				)
				Os.setenv(
					"GROUP_CONTRACT_ADDRESS",
					"n18nczmqw6adwxg2wnlef3hf0etf8anccafp2pjpul5rrtmv96umyq5mv7t5",
					true,
				)
				Os.setenv(
					"MULTISIG_CONTRACT_ADDRESS",
					"n1q3zzxl78rlmxv3vn0uf4vkyz285lk8q2xzne299yt9x6mpfgk90qukuzmv",
					true,
				)
				Os.setenv(
					"COCONUT_DKG_CONTRACT_ADDRESS",
					"n1jsz20ggp5a6v76j060erkzvxmeus8htlpl77yxp878f0gf95cyaq6p2pee",
					true,
				)
				Os.setenv(
					"NAME_SERVICE_CONTRACT_ADDRESS",
					"n12ne7qtmdwd0j03t9t5es8md66wq4e5xg9neladrsag8fx3y89rcs36asfp",
					true,
				)
				Os.setenv(
					"SERVICE_PROVIDER_DIRECTORY_CONTRACT_ADDRESS",
					"n1ps5yutd7sufwg058qd7ac7ldnlazsvmhzqwucsfxmm445d70u8asqxpur4",
					true,
				)
				Os.setenv("EPHEMERA_CONTRACT_ADDRESS", "n19lc9u84cz0yz3fww5283nucc9yvr8gsjmgeul0", true)

				Os.setenv("STATISTICS_SERVICE_DOMAIN_ADDRESS", "http://0.0.0.0", true)
				Os.setenv("EXPLORER_API", "https://sandbox-explorer.nymtech.net/api", true)
				Os.setenv("NYXD", "https://rpc.sandbox.nymtech.net", true)
				Os.setenv("NYXD_WS", "wss://rpc.sandbox.nymtech.net/websocket", true)
				Os.setenv("NYM_API", "https://sandbox-nym-api1.nymtech.net/api", true)
			}

			override val nymVpnApiUrl: URL?
				get() = null
			override val apiUrl: URL
				get() = URL("https://sandbox-nym-api1.nymtech.net/api")
			override val accountUrl: URL
				get() = URL("https://nym-dot-com-git-deploy-canary-nyx-network-staging.vercel.app/")
			override val createAccountUrl: URL
				get() = URL("https://nym-dot-com-git-deploy-canary-nyx-network-staging.vercel.app/account/login")
		},
		QA {
			override fun setup() {
				super.setupCommon()
				Os.setenv("REWARDING_VALIDATOR_ADDRESS", "n1rfvpsynktze6wvn6ldskj8xgwfzzk5v6pnff39", true)
				Os.setenv(
					"MIXNET_CONTRACT_ADDRESS",
					"n1hm4y6fzgxgu688jgf7ek66px6xkrtmn3gyk8fax3eawhp68c2d5qujz296",
					true,
				)
				Os.setenv(
					"GROUP_CONTRACT_ADDRESS",
					"n13l7rwuwktklrwskc7m6lv70zws07en85uma28j7dxwsz9y5hvvhspl7a2t",
					true,
				)
				Os.setenv(
					"MULTISIG_CONTRACT_ADDRESS",
					"n138c9pyf7f3hyx0j3t6vmsz7ultnw2wj0lu6hzndep9z5grgq9haqlc25k0",
					true,
				)
				Os.setenv(
					"COCONUT_DKG_CONTRACT_ADDRESS",
					"n1pk8jgr6y4c5k93gz7qf3xc0hvygmp7csk88c2tf8l39tkq6834wq2a6dtr",
					true,
				)

				Os.setenv("EXPLORER_API", "https://qa-network-explorer.qa.nymte.ch/api", true)
				Os.setenv("NYXD", "https://qa-validator.qa.nymte.ch", true)
				Os.setenv("NYM_API", "https://qa-nym-api.qa.nymte.ch/api", true)
			}

			override val nymVpnApiUrl: URL?
				get() = URL("https://nym-vpn-api-git-deploy-qa-nyx-network-staging.vercel.app/api/")
			override val apiUrl: URL
				get() = URL("https://qa-nym-api.qa.nymte.ch/api")
			override val accountUrl: URL
				get() = URL("https://nym-dot-com-git-deploy-qa-nyx-network-staging.vercel.app/en/account/login")
			override val createAccountUrl: URL
				get() = URL("https://nym-dot-com-git-deploy-qa-nyx-network-staging.vercel.app/en")
		}, ;

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

		abstract fun setup()

		fun setupCommon() {
			Os.setenv("CONFIGURED", "true", true)
			Os.setenv("RUST_LOG", LOG_LEVEL, true)
			Os.setenv("RUST_BACKTRACE", "1", true)
			Os.setenv("NETWORK_NAME", "sandbox", true)
			Os.setenv("BECH32_PREFIX", "n", true)
			Os.setenv("MIX_DENOM", "unym", true)
			Os.setenv("MIX_DENOM_DISPLAY", "nym", true)
			Os.setenv("STAKE_DENOM", "unyx", true)
			Os.setenv("STAKE_DENOM_DISPLAY", "nyx", true)
			Os.setenv("DENOMS_EXPONENT", "6", true)
			nymVpnApiUrl?.let {
				Os.setenv("NYM_VPN_API", it.toString(), true)
			}
		}

		abstract val nymVpnApiUrl: URL?
		abstract val apiUrl: URL
		abstract val accountUrl: URL
		abstract val createAccountUrl: URL
	}
}

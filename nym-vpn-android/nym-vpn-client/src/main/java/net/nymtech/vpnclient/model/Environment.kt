package net.nymtech.vpnclient.model

import java.net.URL
enum class Environment {
	MAINNET {
		override val apiUrl: URL
			get() = URL("https://validator.nymtech.net/api/")
		override val explorerUrl: URL
			get() = URL("https://explorer.nymtech.net/api/")
		override val harbourMasterUrl: URL
			get() = URL("https://harbourmaster.nymtech.net")
	},
	SANDBOX {
		override val apiUrl: URL
			get() = URL("https://sandbox-nym-api1.nymtech.net/api")
		override val explorerUrl: URL
			get() = URL("https://sandbox-explorer.nymtech.net/api")
		override val harbourMasterUrl: URL?
			get() = null
	},
	CANARY {
		override val apiUrl: URL
			get() = URL("https://canary-api.performance.nymte.ch/api")
		override val explorerUrl: URL
			get() = URL("https://canary-explorer.performance.nymte.ch/api")
		override val harbourMasterUrl: URL?
			get() = null
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

	abstract val apiUrl: URL
	abstract val explorerUrl: URL
	abstract val harbourMasterUrl: URL?
}

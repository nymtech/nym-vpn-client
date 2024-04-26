package net.nymtech.vpn.model

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
	}, ;

	abstract val apiUrl: URL
	abstract val explorerUrl: URL
	abstract val harbourMasterUrl: URL?
}

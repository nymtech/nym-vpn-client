package net.nymtech.vpn.backend

import nym_vpn_lib.AccountLinks
import nym_vpn_lib.AccountStateSummary

interface Backend {

	suspend fun init(environment: Tunnel.Environment, credentialMode: Boolean?)

	suspend fun getAccountSummary(): AccountStateSummary

	suspend fun getAccountLinks(environment: Tunnel.Environment): AccountLinks

	suspend fun storeMnemonic(credential: String)

	suspend fun isMnemonicStored(): Boolean

	suspend fun removeMnemonic()

	suspend fun start(tunnel: Tunnel, background: Boolean)

	suspend fun stop()

	fun getState(): Tunnel.State
}

package net.nymtech.vpn.backend

import nym_vpn_lib.AccountStateSummary

interface Backend {

	suspend fun init(environment: Tunnel.Environment) : Boolean

	suspend fun getAccountSummary(): AccountStateSummary

	suspend fun storeMnemonic(credential: String)

	suspend fun isMnemonicStored(): Boolean

	suspend fun removeMnemonic()

	suspend fun start(tunnel: Tunnel, background: Boolean)

	suspend fun stop()

	fun getState(): Tunnel.State
}

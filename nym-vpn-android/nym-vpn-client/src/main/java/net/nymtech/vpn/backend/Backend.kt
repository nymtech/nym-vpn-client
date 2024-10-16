package net.nymtech.vpn.backend

interface Backend {

	suspend fun storeMnemonic(credential: String)

	suspend fun isMnemonicStored(): Boolean

	suspend fun removeMnemonic()

	suspend fun start(tunnel: Tunnel, background: Boolean)

	suspend fun stop()

	fun getState(): Tunnel.State
}

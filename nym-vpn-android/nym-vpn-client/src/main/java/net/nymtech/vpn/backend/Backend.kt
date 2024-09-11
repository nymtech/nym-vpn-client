package net.nymtech.vpn.backend

import java.time.Instant

interface Backend {

	suspend fun validateCredential(credential: String): Instant?

	suspend fun importCredential(credential: String): Instant?

	suspend fun start(tunnel: Tunnel, background: Boolean)

	suspend fun stop()

	fun getState(): Tunnel.State
}

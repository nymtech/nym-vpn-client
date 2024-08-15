package net.nymtech.vpn

import java.time.Instant

interface Backend {

	suspend fun validateCredential(credential: String): Instant?

	suspend fun importCredential(credential: String): Instant?

	suspend fun start(tunnel: Tunnel): Tunnel.State

	suspend fun stop(): Tunnel.State

	fun getState(): Tunnel.State
}

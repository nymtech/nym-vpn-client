package net.nymtech.vpn

import java.time.Instant

interface Backend {

	suspend fun validateCredential(credential: String): Instant?

	suspend fun importCredential(credential: String): Instant?

	fun start(tunnel: Tunnel): Tunnel.State

	fun stop(): Tunnel.State

	fun getState(): Tunnel.State
}

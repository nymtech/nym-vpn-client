package net.nymtech.vpn

import android.content.Context
import java.time.Instant

interface Backend {

	suspend fun validateCredential(credential: String): Instant?

	suspend fun importCredential(credential: String): Instant?

	fun start(context: Context, tunnel: Tunnel): Tunnel.State

	fun stop(context: Context): Tunnel.State

	fun getState(): Tunnel.State
}

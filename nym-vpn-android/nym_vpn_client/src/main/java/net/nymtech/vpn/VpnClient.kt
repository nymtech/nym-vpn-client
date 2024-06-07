package net.nymtech.vpn

import android.content.Context
import android.content.Intent
import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.model.VpnClientState
import net.nymtech.vpn.model.VpnMode
import net.nymtech.vpn.util.InvalidCredentialException
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint
import java.time.Instant

interface VpnClient {

	var entryPoint: EntryPoint
	var exitPoint: ExitPoint
	var mode: VpnMode

	suspend fun validateCredential(credential: String): Result<Instant?>

	@Throws(InvalidCredentialException::class)
	suspend fun start(context: Context, credential: String, foreground: Boolean = false): Result<Unit>

	suspend fun stop(context: Context, foreground: Boolean = false)

	fun prepare(context: Context): Intent?

	val stateFlow: Flow<VpnClientState>

	fun getState(): VpnClientState
}

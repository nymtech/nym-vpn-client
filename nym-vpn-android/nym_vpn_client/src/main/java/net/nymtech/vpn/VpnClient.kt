package net.nymtech.vpn

import android.content.Context
import android.content.Intent
import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.model.ClientState
import net.nymtech.vpn.model.VpnMode
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint

interface VpnClient {

	var entryPoint: EntryPoint
	var exitPoint: ExitPoint
	var mode: VpnMode
	fun start(context: Context, credential: String, foreground: Boolean = false)

	fun stop(context: Context, foreground: Boolean = false)

	fun prepare(context: Context): Intent?

	val stateFlow: Flow<ClientState>

	fun getState(): ClientState
}

package net.nymtech.vpn

import android.content.Context
import android.content.Intent
import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.model.ClientState
import net.nymtech.vpn.model.Country
import net.nymtech.vpn.model.VpnMode
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint

interface VpnClient {
	fun configure(entryPoint: EntryPoint, exitPoint: ExitPoint, mode: VpnMode = VpnMode.TWO_HOP_MIXNET)

	fun prepare(context: Context): Intent?

	fun start(context: Context)

	fun startForeground(context: Context)

	fun disconnect(context: Context)

	val stateFlow: Flow<ClientState>

	fun getState(): ClientState

	suspend fun gateways(exitOnly: Boolean = false): Set<Country>

	suspend fun getLowLatencyEntryCountryCode(): Country
}

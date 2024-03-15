package net.nymtech.vpn

import android.content.Context
import android.content.Intent
import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.model.ClientState
import net.nymtech.vpn.model.EntryPoint
import net.nymtech.vpn.model.ExitPoint
import net.nymtech.vpn.model.VpnMode

interface VpnClient {
    fun prepare(context : Context) : Intent?
    fun connect(context: Context, entryPoint: EntryPoint, exitPoint: ExitPoint, mode: VpnMode)
    fun connectForeground(context: Context, entryPoint: EntryPoint, exitPoint: ExitPoint, mode: VpnMode)
    fun disconnect(context: Context)
    val stateFlow : Flow<ClientState>
    fun getState() : ClientState
    suspend fun gateways(exitOnly: Boolean = false) : List<String>
}
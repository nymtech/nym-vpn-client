package net.nymtech.vpn

import android.content.Context
import android.content.Intent
import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.model.EntryPoint
import net.nymtech.vpn.model.ExitPoint
import net.nymtech.vpn.model.VpnState
import net.nymtech.vpn.model.VpnStatistics

interface VpnClient {
    fun prepare(context : Context) : Intent?
    fun connect(context: Context, entryPoint: EntryPoint, exitPoint: ExitPoint, isTwoHop: Boolean)
    fun disconnect(context: Context)
    val statistics : Flow<VpnStatistics>
    val stateFlow : Flow<VpnState>
}
package net.nymtech.vpn_client

import android.content.Context
import android.content.Intent
import kotlinx.coroutines.flow.Flow

interface VpnClient {
    fun prepare(context : Context) : Intent?
    fun connect(entryIso: String, exitIso: String, vpnService: NymVpnService)
    fun disconnect()
    val statistics : Flow<VpnStatistics>
}
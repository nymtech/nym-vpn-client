package net.nymtech.vpn

import android.content.Context
import android.content.Intent
import kotlinx.coroutines.flow.Flow

interface VpnClient {
    fun prepare(context : Context) : Intent?
    fun connect()
    fun disconnect()
    val statistics : Flow<VpnStatistics>
}
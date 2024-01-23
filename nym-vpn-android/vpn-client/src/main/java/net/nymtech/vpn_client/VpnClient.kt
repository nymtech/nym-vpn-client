package net.nymtech.vpn_client

import kotlinx.coroutines.flow.Flow

interface VpnClient {
    fun connect(entryIso: String, exitIso: String)
    fun disconnect()
    val statistics : Flow<VpnStatistics>
}
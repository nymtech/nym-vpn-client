package net.nymtech.vpn_client

data class VpnStatistics(
    val connectionSeconds: Long? = null,
    val rx: Long = 0,
    val tx: Long = 0)

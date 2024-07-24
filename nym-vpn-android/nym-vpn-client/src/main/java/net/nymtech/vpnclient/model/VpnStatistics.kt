package net.nymtech.vpnclient.model

data class VpnStatistics(
	val connectionSeconds: Long? = null,
	val rx: Long = 0,
	val tx: Long = 0,
)

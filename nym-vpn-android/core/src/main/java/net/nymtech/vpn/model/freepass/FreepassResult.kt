package net.nymtech.vpn.model.freepass

sealed class FreepassResult {
	data object Redeemed : FreepassResult()
	data class Failed(val error: FreepassError) : FreepassResult()
}

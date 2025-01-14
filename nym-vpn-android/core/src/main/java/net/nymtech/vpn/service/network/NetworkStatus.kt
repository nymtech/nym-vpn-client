package net.nymtech.vpn.service.network

sealed class NetworkStatus {
	object Unknown : NetworkStatus()
	object Connected : NetworkStatus()
	object Disconnected : NetworkStatus()
}

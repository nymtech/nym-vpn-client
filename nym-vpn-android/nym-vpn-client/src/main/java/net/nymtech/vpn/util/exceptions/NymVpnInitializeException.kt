package net.nymtech.vpn.util.exceptions

sealed class NymVpnInitializeException : Exception() {
	class VpnAlreadyRunning : NymVpnInitializeException()
	class VpnPermissionDenied : NymVpnInitializeException()
}

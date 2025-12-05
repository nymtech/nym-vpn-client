package net.nymtech.vpn.util.exceptions

sealed class BackendException : Exception() {
	class VpnAlreadyRunning : BackendException()
	class VpnPermissionDenied : BackendException()
}

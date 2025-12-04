package net.nymtech.vpn.util.exceptions

sealed class BackendException : Exception() {
	class InitLoggingFailed : BackendException()
	class VpnAlreadyRunning : BackendException()
	class VpnPermissionDenied : BackendException()
}

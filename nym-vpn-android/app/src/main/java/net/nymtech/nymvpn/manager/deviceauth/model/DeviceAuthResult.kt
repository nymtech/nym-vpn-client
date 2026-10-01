package net.nymtech.nymvpn.manager.deviceauth.model

sealed interface DeviceAuthResult {
	data object Success : DeviceAuthResult
	data object Cancelled : DeviceAuthResult
	data object NotSetUp : DeviceAuthResult
	data class Error(val code: Int, val message: String) : DeviceAuthResult
}

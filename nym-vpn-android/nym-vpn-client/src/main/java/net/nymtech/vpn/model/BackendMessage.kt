package net.nymtech.vpn.model

// TODO map error states and bandwidth states
sealed class BackendMessage {
	data object Error {
		data object StartFailed : BackendMessage()
	}
	data object Message
	data object None : BackendMessage()
}

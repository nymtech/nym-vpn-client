package net.nymtech.vpn.model.pairing

sealed class PairingCommandResult<out T> {
	data class Ok<out T>(val value: T) : PairingCommandResult<T>()
	data class Failed(val error: PairingError) : PairingCommandResult<Nothing>()
}

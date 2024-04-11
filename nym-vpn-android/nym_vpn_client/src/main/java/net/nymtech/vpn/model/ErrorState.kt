package net.nymtech.vpn.model

sealed class ErrorState {
	data object None : ErrorState()

	data class LibraryError(val message: String) : ErrorState()
}

package net.nymtech.vpn.model

sealed class ErrorState {

	data object None : ErrorState()

	data object InvalidCredential : ErrorState()

	data object StartFailed : ErrorState()

	data class CoreLibraryError(val errorMessage: String) : ErrorState()
}

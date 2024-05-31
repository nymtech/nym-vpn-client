package net.nymtech.vpn.model

sealed class ErrorState {

	data object None : ErrorState()

	data object InvalidCredential : ErrorState()

	data object GatewayLookupFailure : ErrorState()

	data object BadGatewayPeerCertificate : ErrorState()

	data object BadGatewayNoHostnameAddress : ErrorState()

	data class VpnHaltedUnexpectedly(val message: String) : ErrorState()
}

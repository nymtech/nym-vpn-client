package net.nymtech.vpn.model.account

sealed class CreateAccountResult {
	data object Created : CreateAccountResult()
	data class Failed(val error: CreateAccountError) : CreateAccountResult()
}

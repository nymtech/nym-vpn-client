package net.nymtech.nymvpn.ui.screens.settings.login

enum class MnemonicError {
	INVALID_RECOVERY_PHRASE,
}

data class LoginUiState(val isLoading: Boolean = false, val mnemonic: String = "", val mnemonicError: MnemonicError? = null, val showMaxDevicesModal: Boolean = false, val deeplink: String? = null)

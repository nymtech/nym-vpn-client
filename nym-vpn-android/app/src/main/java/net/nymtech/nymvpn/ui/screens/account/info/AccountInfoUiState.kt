package net.nymtech.nymvpn.ui.screens.account.info

import net.nymtech.nymvpn.ui.screens.account.info.components.BandwidthUiState
import nym_vpn_lib_types.DeeplinkKind

sealed class AutologinState {
	object Idle : AutologinState()
	object Loading : AutologinState()
	data class PinReady(val url: String, val pinCode: String) : AutologinState()
	data class Error(val kind: DeeplinkKind) : AutologinState()
}

data class AccountInfoUiState(
	val isLoading: Boolean = true,
	val isMnemonicStored: Boolean = false,
	val isLinked: Boolean = false,
	val accountId: String = "",
	val deviceId: String = "",
	val accountLinkUrl: String? = null,
	val manageUrl: String? = null,
	val bandwidth: BandwidthUiState? = null,
)

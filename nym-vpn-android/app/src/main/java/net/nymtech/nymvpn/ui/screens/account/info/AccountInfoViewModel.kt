package net.nymtech.nymvpn.ui.screens.account.info

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.manager.backend.BackendManager
import nym_vpn_lib_types.DeeplinkKind
import nym_vpn_lib_types.StoredAccountMode
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class AccountInfoViewModel @Inject constructor(
	private val backendManager: BackendManager,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-account-vm"
	}

	private val _uiState = MutableStateFlow(AccountInfoUiState())
	val uiState: StateFlow<AccountInfoUiState> = _uiState.asStateFlow()

	init {
		loadAccountData()
	}

	private fun loadAccountData() {
		viewModelScope.launch {
			_uiState.update { it.copy(isLoading = true) }

			val isStored = backendManager.isMnemonicStored()
			val deviceId = backendManager.getDeviceId() ?: ""
			var displayAccountId = backendManager.getAccountId() ?: ""
			val accountMode = backendManager.getAccountMode()
			val isPrivyEnabled = backendManager.getFeatureFlags()?.isPrivyEnabled() ?: false

			val links = backendManager.getAccountLinks()
			var linkUrl = backendManager.getDeeplink(DeeplinkKind.PRIVY_LINK)
			val manageUrl = links?.account

			if (accountMode == StoredAccountMode.PRIVY) {
				try {
					displayAccountId
					linkUrl = links?.account
				} catch (e: Exception) {
					Timber.tag(TAG).e(e, "canonicalRequestFailed")
				}

				_uiState.update {
					it.copy(
						isLoading = false,
						isMnemonicStored = isStored,
						showLinkAccount = false,
						accountId = displayAccountId,
						deviceId = deviceId,
						accountLinkUrl = linkUrl,
						manageUrl = manageUrl,
						isPrivyEnabled = isPrivyEnabled,
					)
				}
			} else {
				_uiState.update {
					it.copy(
						isLoading = false,
						isMnemonicStored = isStored,
						showLinkAccount = true,
						accountId = displayAccountId,
						deviceId = deviceId,
						accountLinkUrl = linkUrl,
						manageUrl = manageUrl,
						isPrivyEnabled = isPrivyEnabled,
					)
				}
			}
		}
	}
}

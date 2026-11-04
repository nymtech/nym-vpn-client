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
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.manager.billing.awaitReady
import net.nymtech.nymvpn.manager.billing.isPlayDistribution
import net.nymtech.nymvpn.util.extensions.toBandwidthUiState
import nym_vpn_lib_types.DeeplinkKind
import nym_vpn_lib_types.StoredAccountMode
import timber.log.Timber
import javax.inject.Inject
import kotlin.time.Duration.Companion.seconds

@HiltViewModel
class AccountInfoViewModel @Inject constructor(private val backendManager: BackendManager, private val billingManager: BillingManager) : ViewModel() {

	companion object {
		private const val TAG = "ui-account-vm"
		private val BILLING_READY_TIMEOUT = 5.seconds
	}

	private val _uiState = MutableStateFlow(AccountInfoUiState())
	val uiState: StateFlow<AccountInfoUiState> = _uiState.asStateFlow()

	init {
		loadAccountData()
		viewModelScope.launch {
			backendManager.accountSummaryFlow.collect { summary ->
				_uiState.update { it.copy(bandwidth = summary?.toBandwidthUiState()) }
			}
		}
	}

	fun loadPlaySubscription() = viewModelScope.launch {
		if (!isPlayDistribution) return@launch
		val hasPlaySubscription = if (billingManager.awaitReady(BILLING_READY_TIMEOUT)) billingManager.hasActiveSubscription() else null
		_uiState.update { it.copy(hasPlaySubscription = hasPlaySubscription) }
	}

	private fun loadAccountData() {
		viewModelScope.launch {
			_uiState.update { it.copy(isLoading = true) }

			val isAccountLinked = runCatching {
				backendManager.getAccountSummary()?.isLinked() ?: false
			}.getOrDefault(false)

			val isStored = backendManager.isMnemonicStored()
			val deviceId = backendManager.getDeviceId() ?: ""
			val displayAccountId = backendManager.getAccountId() ?: ""
			val accountMode = backendManager.getAccountMode()

			val links = backendManager.getAccountLinks()
			var linkUrl = backendManager.getDeeplink(DeeplinkKind.PRIVY_LINK)

			if (accountMode == StoredAccountMode.PRIVY) {
				try {
					linkUrl = links?.account
				} catch (e: Exception) {
					Timber.tag(TAG).e(e, "canonicalRequestFailed")
				}
			}

			_uiState.update {
				it.copy(
					isLoading = false,
					isMnemonicStored = isStored,
					isLinked = isAccountLinked,
					accountId = displayAccountId,
					deviceId = deviceId,
					accountLinkUrl = linkUrl,
					manageUrl = links?.account,
				)
			}
		}
	}
}

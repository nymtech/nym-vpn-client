package net.nymtech.nymvpn.ui.screens.account.create

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import net.nymtech.billing.model.BillingCode
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.util.Constants
import nym_vpn_lib_types.DeeplinkKind
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class CreateAccountViewModel @Inject constructor(private val billingManager: BillingManager, private val backendManager: BackendManager) : ViewModel() {

	companion object {
		private const val TAG = "ui-create-account"
	}

	private val _uiState = MutableStateFlow(CreateAccountUiState())
	val uiState = _uiState.asStateFlow()

	init {
		viewModelScope.launch {
			runCatching {
				val link = backendManager.getDeeplink(DeeplinkKind.PRIVY)
				val signUp = backendManager.getAccountLinks()?.signUp
				_uiState.value = _uiState.value.copy(deeplink = link, signUpLink = signUp)
			}.onFailure { t ->
				Timber.tag(TAG).w(t, "SocialDeeplinkLoadFailed")
			}

			val billingAllowed = BuildConfig.APPLICATION_ID == Constants.APP_ID
			val billingAvailableNow = billingAllowed && billingManager.isAvailable()

			if (!billingAvailableNow) {
				_uiState.value = _uiState.value.copy(isBillingAvailable = false)
				return@launch
			}

			_uiState.value = _uiState.value.copy(isLoading = true)

			try {
				billingManager.initialize()

				val response = withTimeoutOrNull(10_000) {
					billingManager.uiState
						.map { it.billingInfo?.responseCode ?: BillingCode.UNKNOWN }
						.filter { it != BillingCode.UNKNOWN }
						.first()
				} ?: BillingCode.UNKNOWN

				if (response == BillingCode.BILLING_UNAVAILABLE) {
					_uiState.value = _uiState.value.copy(isBillingAvailable = false)
					return@launch
				}

				val subscribed = billingManager.hasActiveSubscription()
				_uiState.value = _uiState.value.copy(
					hasActiveSubscription = subscribed,
					isBillingAvailable = true,
				)
			} catch (t: Throwable) {
				Timber.tag(TAG).w(t, "BillingInitOrCheckFailed")
				_uiState.value = _uiState.value.copy(
					isBillingAvailable = false,
					hasActiveSubscription = false,
				)
			} finally {
				_uiState.value = _uiState.value.copy(isLoading = false)
			}
		}
	}
}

package net.nymtech.nymvpn.ui.screens.account.welcome

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import net.nymtech.billing.model.BillingCode
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.util.Constants
import javax.inject.Inject

@HiltViewModel
class WelcomeAccountViewModel @Inject constructor(
	private val billingManager: BillingManager,
) : ViewModel() {

	private val _loading = MutableStateFlow(false)
	val loading = _loading.asStateFlow()

	private val _activeSubscription = MutableStateFlow(false)
	val activeSubscription = _activeSubscription.asStateFlow()

	init {
		viewModelScope.launch {
			if (billingManager.isAvailable() && BuildConfig.APPLICATION_ID == Constants.APP_ID) {
				_loading.emit(true)
				try {
					if (billingManager.isReady()) {
						checkSubscription()
					} else {
						billingManager.initialize()
						val response = billingManager.uiState
							.map { it.billingInfo?.responseCode ?: BillingCode.UNKNOWN }
							.first()

						if (response == BillingCode.BILLING_UNAVAILABLE || response == BillingCode.UNKNOWN) {
							_loading.emit(false)
							return@launch
						}
						if (response == BillingCode.OK) {
							checkSubscription()
						}
					}
				} finally {
					_loading.emit(false)
				}
			}
		}
	}

	private suspend fun checkSubscription() {
		val subscribed = billingManager.hasActiveSubscription()
		_activeSubscription.value = subscribed
	}
}

package net.nymtech.nymvpn.ui.screens.account.create

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
import net.nymtech.nymvpn.manager.environment.EnvironmentManager
import net.nymtech.nymvpn.util.Constants
import javax.inject.Inject

@HiltViewModel
class CreateAccountViewModel @Inject constructor(
	private val billingManager: BillingManager,
	private val environmentManager: EnvironmentManager,
) : ViewModel() {

	private val _uiState = MutableStateFlow(CreateAccountUiState())
	val uiState = _uiState.asStateFlow()

	init {
		viewModelScope.launch {
			_uiState.value = _uiState.value.copy(
				isPrivyEnabled = environmentManager.isPrivyEnabled(),
			)

			val billingAllowed = BuildConfig.APPLICATION_ID == Constants.APP_ID
			val billingAvailableNow = billingAllowed && billingManager.isAvailable()

			_uiState.value = _uiState.value.copy(
				isBillingAvailable = billingAvailableNow && billingManager.isReady(),
			)

			if (!billingAvailableNow) return@launch

			_uiState.value = _uiState.value.copy(isLoading = true)
			try {
				if (!billingManager.isReady()) {
					billingManager.initialize()

					val response = billingManager.uiState
						.map { it.billingInfo?.responseCode ?: BillingCode.UNKNOWN }
						.first()

					if (response == BillingCode.BILLING_UNAVAILABLE || response == BillingCode.UNKNOWN) {
						_uiState.value = _uiState.value.copy(isBillingAvailable = false)
						return@launch
					}
				}

				val subscribed = billingManager.hasActiveSubscription()
				_uiState.value = _uiState.value.copy(
					hasActiveSubscription = subscribed,
					isBillingAvailable = true,
				)
			} finally {
				_uiState.value = _uiState.value.copy(isLoading = false)
			}
		}
	}

	fun isBillingAvailable(): Boolean {
		val s = _uiState.value
		return s.isBillingAvailable
	}
}

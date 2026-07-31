package net.nymtech.nymvpn.ui.screens.onboarding

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.nymtech.billing.model.BillingCode
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.ui.screens.onboarding.components.OnboardingPlanPricing
import net.nymtech.nymvpn.util.Constants
import javax.inject.Inject

@HiltViewModel
class OnboardingViewModel
@Inject
constructor(private val settingsRepository: SettingsRepository, private val billingManager: BillingManager) : ViewModel() {

	val isBillingAvailable: Boolean = billingManager.isAvailable() && BuildConfig.APPLICATION_ID == Constants.APP_ID

	val planPricing: StateFlow<OnboardingPlanPricing?> = billingManager.products
		.map { OnboardingPlanPricing.from(it) }
		.stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), null)

	init {
		if (isBillingAvailable) {
			billingManager.initialize()
			viewModelScope.launch {
				billingManager.uiState
					.map { it.billingInfo?.responseCode }
					.filter { it == BillingCode.OK }
					.first()
				billingManager.fetchSubscriptions()
			}
		}
	}

	suspend fun onOnboardingCompleted() {
		settingsRepository.setOnboardingCompleted(true)
	}
}

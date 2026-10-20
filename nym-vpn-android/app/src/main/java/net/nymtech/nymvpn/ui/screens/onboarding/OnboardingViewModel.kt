package net.nymtech.nymvpn.ui.screens.onboarding

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.manager.billing.TrialEligibility
import net.nymtech.nymvpn.manager.billing.TrialEligibilityRepository
import net.nymtech.nymvpn.manager.billing.awaitProducts
import net.nymtech.nymvpn.ui.screens.onboarding.components.OnboardingPlanPricing
import javax.inject.Inject
import kotlin.time.Duration.Companion.seconds

@HiltViewModel
class OnboardingViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val billingManager: BillingManager,
	private val trialEligibilityRepository: TrialEligibilityRepository,
) : ViewModel() {

	companion object {
		private val PRODUCTS_TIMEOUT = 5.seconds
	}

	private val _planPricing = MutableStateFlow<OnboardingPlanPricing?>(null)

	/** Null until Play pricing and eligibility resolve (both time out), then set once. */
	val planPricing: StateFlow<OnboardingPlanPricing?> = _planPricing.asStateFlow()

	init {
		trialEligibilityRepository.refresh(viewModelScope)
		viewModelScope.launch {
			val products = async { billingManager.awaitProducts(PRODUCTS_TIMEOUT) }
			val eligibility = trialEligibilityRepository.state.filterIsInstance<TrialEligibility.Resolved>().first()
			_planPricing.value = OnboardingPlanPricing.resolve(products.await(), eligibility.eligible)
		}
	}

	suspend fun onOnboardingCompleted() {
		settingsRepository.setOnboardingCompleted(true)
	}
}

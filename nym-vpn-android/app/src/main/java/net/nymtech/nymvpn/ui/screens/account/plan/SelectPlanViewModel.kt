package net.nymtech.nymvpn.ui.screens.account.plan

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.manager.billing.TrialEligibility
import net.nymtech.nymvpn.manager.billing.TrialEligibilityRepository
import net.nymtech.nymvpn.manager.billing.awaitProducts
import net.nymtech.nymvpn.manager.billing.isPlayBillingSupported
import javax.inject.Inject
import kotlin.time.Duration.Companion.seconds

@HiltViewModel
class SelectPlanViewModel @Inject constructor(private val billingManager: BillingManager, private val trialEligibilityRepository: TrialEligibilityRepository) : ViewModel() {

	companion object {
		private val PRODUCTS_TIMEOUT = 5.seconds
	}

	private val _uiState = MutableStateFlow(SelectPlanUiState())
	val uiState: StateFlow<SelectPlanUiState> = _uiState.asStateFlow()

	init {
		viewModelScope.launch {
			trialEligibilityRepository.state.collect { eligibility ->
				_uiState.update {
					it.copy(
						trialEligibility = eligibility,
						lastResolvedTrialEligible = (eligibility as? TrialEligibility.Resolved)?.eligible ?: it.lastResolvedTrialEligible,
					)
				}
			}
		}
		trialEligibilityRepository.refresh(viewModelScope)
		viewModelScope.launch {
			val products = billingManager.awaitProducts(PRODUCTS_TIMEOUT)
			_uiState.update { it.copy(subscriptions = products, productsResolved = true) }
			billingManager.products.collectLatest { productList ->
				_uiState.update { it.copy(subscriptions = productList) }
			}
		}
	}

	fun isBillingAvailable(): Boolean = billingManager.isReady() && billingManager.isPlayBillingSupported()

	fun fetchSubscriptions() {
		trialEligibilityRepository.refresh(viewModelScope)
		billingManager.fetchSubscriptions()
	}
}

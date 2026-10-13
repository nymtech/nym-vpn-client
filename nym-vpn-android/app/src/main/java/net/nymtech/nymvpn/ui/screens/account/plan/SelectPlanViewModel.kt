package net.nymtech.nymvpn.ui.screens.account.plan

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.util.Constants
import timber.log.Timber
import javax.inject.Inject
import kotlin.time.Duration.Companion.seconds

@HiltViewModel
class SelectPlanViewModel @Inject constructor(private val backendManager: BackendManager, private val billingManager: BillingManager) : ViewModel() {

	companion object {
		private const val TAG = "ui-select-plan-vm"
		private val TRIAL_ELIGIBILITY_TIMEOUT = 5.seconds
	}

	private val _uiState = MutableStateFlow(SelectPlanUiState())
	val uiState: StateFlow<SelectPlanUiState> = _uiState.asStateFlow()

	private var trialEligibilityJob: Job? = null

	init {
		refreshTrialEligibility()
		viewModelScope.launch {
			billingManager.initialize()
			billingManager.products.collectLatest { productList ->
				_uiState.update { it.copy(subscriptions = productList) }
			}
		}
	}

	fun isBillingAvailable(): Boolean = billingManager.isReady() && billingManager.isAvailable() && BuildConfig.APPLICATION_ID == Constants.APP_ID

	fun fetchSubscriptions() {
		refreshTrialEligibility()
		billingManager.fetchSubscriptions()
	}

	// Read on every screen load and sheet open, never cached: eligibility flips as soon as the
	// account buys anything. Anything but an explicit yes from the backend means no trial, and a
	// slow backend must not block the purchase, so a timeout also means no trial.
	private fun refreshTrialEligibility() {
		trialEligibilityJob?.cancel()
		_uiState.update { it.copy(trialEligibilityResolved = false) }
		trialEligibilityJob = viewModelScope.launch {
			val eligible = withTimeoutOrNull(TRIAL_ELIGIBILITY_TIMEOUT) { backendManager.isTrialEligible() }
			// A newer refresh replaced this one; its answer must not be overwritten by a stale one.
			ensureActive()
			if (eligible == null) Timber.tag(TAG).w("TrialEligibilityTimedOut")
			Timber.tag(TAG).i("TrialEligibility eligible=%b", eligible == true)
			_uiState.update { it.copy(trialEligible = eligible == true, trialEligibilityResolved = true) }
		}
	}
}

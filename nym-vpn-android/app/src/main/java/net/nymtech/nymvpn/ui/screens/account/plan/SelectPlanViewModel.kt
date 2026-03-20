package net.nymtech.nymvpn.ui.screens.account.plan

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.ui.screens.account.info.AutologinState
import net.nymtech.nymvpn.util.Constants
import nym_vpn_lib_types.DeeplinkKind
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class SelectPlanViewModel @Inject constructor(private val backendManager: BackendManager, private val billingManager: BillingManager) : ViewModel() {

	private val _uiState = MutableStateFlow(SelectPlanUiState())
	val uiState: StateFlow<SelectPlanUiState> = _uiState.asStateFlow()

	private var autologinJob: Job? = null

	init {
		viewModelScope.launch {
			billingManager.initialize()
			billingManager.products.collectLatest { productList ->
				_uiState.update { it.copy(subscriptions = productList) }
			}
		}
	}

	fun isBillingAvailable(): Boolean = billingManager.isReady() && billingManager.isAvailable() && BuildConfig.APPLICATION_ID == Constants.APP_ID

	fun fetchSubscriptions() {
		billingManager.fetchSubscriptions()
	}

	fun fetchAutologin() {
		autologinJob?.cancel()
		autologinJob = viewModelScope.launch {
			_uiState.update { it.copy(autologin = AutologinState.Loading) }
			runCatching { backendManager.getAutologinDeeplink(DeeplinkKind.AUTOLOGIN_RENEW) }
				.onSuccess { response ->
					if (response != null) {
						_uiState.update { it.copy(autologin = AutologinState.PinReady(response.url, response.pinCode)) }
					} else {
						_uiState.update { it.copy(autologin = AutologinState.Error(DeeplinkKind.AUTOLOGIN_RENEW)) }
					}
				}
				.onFailure {
					Timber.e(it, "autologin failed")
					_uiState.update { it.copy(autologin = AutologinState.Error(DeeplinkKind.AUTOLOGIN_RENEW)) }
				}
		}
	}

	fun cancelAutologin() {
		autologinJob?.cancel()
		_uiState.update { it.copy(autologin = AutologinState.Idle) }
	}

	fun dismissAutologin() {
		_uiState.update { it.copy(autologin = AutologinState.Idle) }
	}
}
